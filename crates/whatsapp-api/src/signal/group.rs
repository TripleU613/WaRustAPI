//! Sender-key group message receive and SKDM processing.
//! All Signal mutations stay in memory until the caller commits them atomically
//! with the inbox/checkpoint record, matching the direct-message pattern.
use crate::{
    Error, Result,
    model::jid::Jid,
    store::{ReadTx, Table, WriteTx},
};
use async_trait::async_trait;
use wacore_libsignal::protocol as signal;
use wacore_libsignal::protocol::{
    SenderKeyDistributionMessage, SenderKeyName, SenderKeyRecord, SenderKeyStore,
    SignalProtocolError,
};
use zeroize::Zeroizing;

fn invalid(reason: &'static str) -> Error {
    Error::invalid("group_message", reason)
}

fn crypto(e: SignalProtocolError) -> Error {
    Error::invalid("group_crypto", e.to_string())
}

/// Storage key for a (group, sender) sender key record.
/// Sender is stored as its non-AD (device-stripped) JID so that device rotation
/// does not create orphaned records; the actual signing key inside the record
/// authenticates the chain, not the device suffix.
pub fn sender_key_db_key(group: &Jid, sender: &Jid) -> Vec<u8> {
    format!("sk:{}:{}", group, sender.to_non_ad()).into_bytes()
}

/// Minimal in-memory SenderKeyStore backed by a pre-loaded record.
/// `group_decrypt` or `process_sender_key_distribution_message` updates the
/// record in memory; `into_commit` extracts the serialized bytes for the caller
/// to write in the same atomic write transaction as the inbox row.
#[allow(dead_code)]
struct SenderKeys {
    db_key: Vec<u8>,
    /// The bytes originally read from DB (used for OCC conflict detection).
    original: Option<Zeroizing<Vec<u8>>>,
    /// The record (updated in-place by libsignal).
    record: Option<SenderKeyRecord>,
}

impl SenderKeys {
    fn new(db_key: Vec<u8>, original: Option<Vec<u8>>) -> Result<Self> {
        let record = original
            .as_deref()
            .map(SenderKeyRecord::deserialize)
            .transpose()
            .map_err(|_| invalid("corrupt sender key record"))?;
        Ok(Self {
            db_key,
            original: original.map(Zeroizing::new),
            record,
        })
    }
}

#[async_trait]
impl SenderKeyStore for SenderKeys {
    async fn load_sender_key(
        &self,
        _name: &SenderKeyName,
    ) -> signal::error::Result<Option<SenderKeyRecord>> {
        Ok(self.record.clone())
    }

    async fn store_sender_key(
        &mut self,
        _name: &SenderKeyName,
        record: SenderKeyRecord,
    ) -> signal::error::Result<()> {
        self.record = Some(record);
        Ok(())
    }
}

/// Result of `receive_group`: plaintext + serialized updated key for atomic commit.
pub struct ReceivedGroup {
    /// Authenticated group message plaintext (the WhatsApp Message proto bytes).
    pub plaintext: Zeroizing<Vec<u8>>,
    /// Updated sender key bytes, to write in the atomic inbox commit.
    pub(crate) updated_key: Zeroizing<Vec<u8>>,
    /// Original bytes for OCC conflict detection in the commit.
    pub(crate) original_key: Option<Zeroizing<Vec<u8>>>,
    /// DB key where the sender key lives.
    pub(crate) db_key: Vec<u8>,
}

impl std::fmt::Debug for ReceivedGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ReceivedGroup(<redacted>)")
    }
}

/// Result of `process_skdm`: serialized updated key for atomic commit.
pub struct ReceivedSkdm {
    /// Updated sender key bytes to write.
    pub(crate) updated_key: Zeroizing<Vec<u8>>,
    /// Original bytes for OCC conflict detection.
    pub(crate) original_key: Option<Zeroizing<Vec<u8>>>,
    /// DB key.
    pub(crate) db_key: Vec<u8>,
}

impl std::fmt::Debug for ReceivedSkdm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ReceivedSkdm(<redacted>)")
    }
}

/// Load previous sender key bytes from the store (sync read, before async decrypt).
pub fn load_sender_key_bytes(
    tx: &ReadTx<'_>,
    group: &Jid,
    sender: &Jid,
) -> Result<(Vec<u8>, Option<Vec<u8>>)> {
    let db_key = sender_key_db_key(group, sender);
    let bytes: Option<Vec<u8>> = tx.get_json(Table::SenderKeys, &db_key, 1)?;
    Ok((db_key, bytes))
}

/// Decrypt one `skmsg` ciphertext using the pre-loaded sender key bytes.
/// Returns `None` when no sender key exists (SKDM not yet received → Unsupported).
pub async fn receive_group(
    group: &Jid,
    sender: &Jid,
    db_key: Vec<u8>,
    previous_bytes: Option<Vec<u8>>,
    ciphertext: &[u8],
) -> Result<Option<ReceivedGroup>> {
    if ciphertext.len() > 65536 {
        return Err(invalid("skmsg ciphertext exceeds bound"));
    }
    if previous_bytes.is_none() {
        // No SKDM received yet — caller keeps the stanza as Unsupported.
        return Ok(None);
    }
    let mut store = SenderKeys::new(db_key.clone(), previous_bytes.clone())?;
    let name = SenderKeyName::from_parts(&group.to_string(), &sender.to_non_ad().to_string());
    let plaintext = signal::group_decrypt(ciphertext, &mut store, &name)
        .await
        .map_err(crypto)?;
    let updated_key = Zeroizing::new(
        store
            .record
            .as_ref()
            .ok_or_else(|| invalid("group_decrypt did not produce a record"))?
            .serialize()
            .map_err(|_| invalid("serialize updated sender key"))?,
    );
    Ok(Some(ReceivedGroup {
        plaintext: Zeroizing::new(plaintext),
        updated_key,
        original_key: previous_bytes.map(Zeroizing::new),
        db_key,
    }))
}

/// Process a sender key distribution message received inside a direct message.
pub async fn process_skdm(
    group: &Jid,
    sender: &Jid,
    db_key: Vec<u8>,
    previous_bytes: Option<Vec<u8>>,
    skdm_bytes: &[u8],
) -> Result<ReceivedSkdm> {
    let mut store = SenderKeys::new(db_key.clone(), previous_bytes.clone())?;
    let name = SenderKeyName::from_parts(&group.to_string(), &sender.to_non_ad().to_string());
    let skdm = SenderKeyDistributionMessage::try_from(skdm_bytes).map_err(crypto)?;
    signal::process_sender_key_distribution_message(&name, &skdm, &mut store)
        .await
        .map_err(crypto)?;
    let updated_key = Zeroizing::new(
        store
            .record
            .ok_or_else(|| invalid("SKDM processing did not produce a record"))?
            .serialize()
            .map_err(|_| invalid("serialize updated sender key"))?,
    );
    Ok(ReceivedSkdm {
        updated_key,
        original_key: previous_bytes.map(Zeroizing::new),
        db_key,
    })
}

/// Commit a received group message's sender key update, with OCC conflict detection.
/// Must be called inside the same write transaction as the inbox row write.
pub fn commit_group(tx: &mut WriteTx<'_>, received: &ReceivedGroup) -> Result<()> {
    let current: Option<Vec<u8>> = tx.get_json(Table::SenderKeys, &received.db_key, 1)?;
    if current.as_deref() != received.original_key.as_deref().map(|v| v.as_slice()) {
        return Err(Error::invalid(
            "inbox",
            "sender key changed concurrently; skmsg not committed",
        ));
    }
    tx.put_json(
        Table::SenderKeys,
        &received.db_key,
        1,
        &*received.updated_key,
    )?;
    Ok(())
}

/// Commit an SKDM sender key update atomically.
pub fn commit_skdm(tx: &mut WriteTx<'_>, skdm: &ReceivedSkdm) -> Result<()> {
    let current: Option<Vec<u8>> = tx.get_json(Table::SenderKeys, &skdm.db_key, 1)?;
    if current.as_deref() != skdm.original_key.as_deref().map(|v| v.as_slice()) {
        return Err(Error::invalid(
            "inbox",
            "sender key changed concurrently; SKDM not committed",
        ));
    }
    tx.put_json(Table::SenderKeys, &skdm.db_key, 1, &*skdm.updated_key)?;
    Ok(())
}

/// Extract SKDM fields from a decrypted WhatsApp Message protobuf payload.
/// Returns `(group_id_string, axolotl_skdm_bytes)` if field 18 is set.
pub fn extract_skdm_from_payload(payload: &[u8]) -> Option<(String, Vec<u8>)> {
    use prost::Message as _;
    #[derive(prost::Message)]
    struct SkdmMessage {
        #[prost(string, optional, tag = "1")]
        group_id: Option<String>,
        #[prost(bytes = "vec", optional, tag = "2")]
        axolotl: Option<Vec<u8>>,
    }
    #[derive(prost::Message)]
    struct MessageWithSkdm {
        #[prost(message, optional, tag = "18")]
        skdm: Option<SkdmMessage>,
    }
    let envelope = MessageWithSkdm::decode(payload).ok()?;
    let skdm = envelope.skdm?;
    let group_id = skdm.group_id.filter(|s| !s.is_empty())?;
    let axolotl = skdm.axolotl.filter(|b| !b.is_empty())?;
    Some((group_id, axolotl))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn group_jid() -> Jid {
        Jid::parse("120363000000000001@g.us").unwrap()
    }

    fn sender_jid() -> Jid {
        Jid::parse("14155550100@s.whatsapp.net").unwrap()
    }

    #[test]
    fn sender_key_db_key_strips_device() {
        let g = group_jid();
        let s = Jid::parse("14155550100:3@s.whatsapp.net").unwrap();
        let s_non_ad = sender_jid();
        // Device :3 variant and non-AD variant should produce the same key.
        assert_eq!(sender_key_db_key(&g, &s), sender_key_db_key(&g, &s_non_ad));
        let key = sender_key_db_key(&g, &s);
        assert!(key.starts_with(b"sk:"));
        assert!(
            key.windows(g.to_string().len())
                .any(|w| w == g.to_string().as_bytes())
        );
    }

    #[test]
    fn extract_skdm_returns_none_for_plain_text_message() {
        // A plain text Message proto (field 1 = conversation) has no SKDM.
        use prost::Message as _;
        #[derive(prost::Message)]
        struct TextOnly {
            #[prost(string, optional, tag = "1")]
            conversation: Option<String>,
        }
        let payload = TextOnly {
            conversation: Some("hello".into()),
        }
        .encode_to_vec();
        assert!(extract_skdm_from_payload(&payload).is_none());
    }

    #[test]
    fn extract_skdm_parses_field_18() {
        use prost::Message as _;
        // Build a minimal Message proto with field 18 = SenderKeyDistributionMessage.
        #[derive(prost::Message)]
        struct SkdmMsg {
            #[prost(string, optional, tag = "1")]
            group_id: Option<String>,
            #[prost(bytes = "vec", optional, tag = "2")]
            axolotl: Option<Vec<u8>>,
        }
        #[derive(prost::Message)]
        struct MsgWithSkdm {
            #[prost(message, optional, tag = "18")]
            skdm: Option<SkdmMsg>,
        }
        let payload = MsgWithSkdm {
            skdm: Some(SkdmMsg {
                group_id: Some("120363000000000001@g.us".into()),
                axolotl: Some(vec![1, 2, 3, 4]),
            }),
        }
        .encode_to_vec();
        let result = extract_skdm_from_payload(&payload);
        assert!(result.is_some());
        let (group, axolotl) = result.unwrap();
        assert_eq!(group, "120363000000000001@g.us");
        assert_eq!(axolotl, vec![1, 2, 3, 4]);
    }

    #[test]
    fn extract_skdm_rejects_empty_group_or_axolotl() {
        use prost::Message as _;
        #[derive(prost::Message)]
        struct SkdmMsg {
            #[prost(string, optional, tag = "1")]
            group_id: Option<String>,
            #[prost(bytes = "vec", optional, tag = "2")]
            axolotl: Option<Vec<u8>>,
        }
        #[derive(prost::Message)]
        struct MsgWithSkdm {
            #[prost(message, optional, tag = "18")]
            skdm: Option<SkdmMsg>,
        }
        // Empty group_id → None.
        let no_group = MsgWithSkdm {
            skdm: Some(SkdmMsg {
                group_id: Some(String::new()),
                axolotl: Some(vec![1]),
            }),
        }
        .encode_to_vec();
        assert!(extract_skdm_from_payload(&no_group).is_none());
        // Empty axolotl → None.
        let no_axolotl = MsgWithSkdm {
            skdm: Some(SkdmMsg {
                group_id: Some("123@g.us".into()),
                axolotl: Some(vec![]),
            }),
        }
        .encode_to_vec();
        assert!(extract_skdm_from_payload(&no_axolotl).is_none());
    }

    #[tokio::test]
    async fn receive_group_returns_none_without_sender_key() {
        // When no SKDM has been received, receive_group returns None (not an error).
        let g = group_jid();
        let s = sender_jid();
        let db_key = sender_key_db_key(&g, &s);
        let result = receive_group(&g, &s, db_key, None, &[1, 2, 3, 4])
            .await
            .unwrap();
        assert!(result.is_none());
    }
}
