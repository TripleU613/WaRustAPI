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
/// APK 37.71 Cnr.A03 includes B76 recipient/device fields in its key tuple.
/// Keep the actual sender device: flattening companions merges independent
/// authenticated chains. Existing primary keys keep their original spelling.
/// Historical flattened records are retained, never guessed to belong to a
/// companion. Complete APK actor/category variants remain separately unported.
pub fn sender_key_db_key(group: &Jid, sender: &Jid) -> Vec<u8> {
    format!("sk:{group}:{sender}").into_bytes()
}

/// Minimal in-memory SenderKeyStore over one pre-loaded record. libsignal
/// mutates the record here only; the caller serializes the result and writes it
/// with [`commit_sender_key`], which carries the DB key and the pre-decryption
/// value for its own OCC check.
struct SenderKeys {
    /// The record, updated in place by libsignal.
    record: Option<SenderKeyRecord>,
}

impl SenderKeys {
    fn new(original: Option<&[u8]>) -> Result<Self> {
        let record = original
            .map(SenderKeyRecord::deserialize)
            .transpose()
            .map_err(|_| invalid("corrupt sender key record"))?;
        Ok(Self { record })
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
///
/// The caller passes the DB key and the pre-decryption value to
/// [`commit_sender_key`] itself, because one stanza's SKDM and `skmsg` produce
/// two in-memory updates that are written once under a single OCC check.
pub struct ReceivedGroup {
    /// Authenticated group message plaintext (the WhatsApp Message proto bytes).
    pub plaintext: Zeroizing<Vec<u8>>,
    /// Updated sender key bytes, to write in the atomic inbox commit.
    pub(crate) updated_key: Zeroizing<Vec<u8>>,
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
    previous_bytes: Option<&[u8]>,
    ciphertext: &[u8],
) -> Result<Option<ReceivedGroup>> {
    if ciphertext.len() > 65536 {
        return Err(invalid("skmsg ciphertext exceeds bound"));
    }
    if previous_bytes.is_none() {
        // No SKDM received yet — caller keeps the stanza as Unsupported.
        return Ok(None);
    }
    let mut store = SenderKeys::new(previous_bytes)?;
    let name = SenderKeyName::from_parts(&group.to_string(), &sender.to_string());
    let plaintext = signal::group_decrypt(ciphertext, &mut store, &name)
        .await
        .map_err(crypto)?;
    let plaintext = unpad(plaintext)?;
    let updated_key = Zeroizing::new(
        store
            .record
            .as_ref()
            .ok_or_else(|| invalid("group_decrypt did not produce a record"))?
            .serialize()
            .map_err(|_| invalid("serialize updated sender key"))?,
    );
    Ok(Some(ReceivedGroup {
        plaintext,
        updated_key,
    }))
}

/// Strip the APK's E2E plaintext padding (1..=16 bytes, each equal to the pad
/// length). Identical to the direct-message path in `direct::receive`; group
/// plaintexts are padded the same way before sender-key encryption.
fn unpad(plaintext: Vec<u8>) -> Result<Zeroizing<Vec<u8>>> {
    let mut plaintext = Zeroizing::new(plaintext);
    let padding = plaintext.last().copied().unwrap_or(0) as usize;
    if !(1..=16).contains(&padding)
        || padding > plaintext.len()
        || !plaintext[plaintext.len() - padding..]
            .iter()
            .all(|v| *v as usize == padding)
    {
        return Err(invalid("invalid authenticated group message padding"));
    }
    let unpadded_len = plaintext.len() - padding;
    plaintext.truncate(unpadded_len);
    Ok(plaintext)
}

/// Process a sender key distribution message received inside a direct message.
pub async fn process_skdm(
    group: &Jid,
    sender: &Jid,
    db_key: Vec<u8>,
    previous_bytes: Option<Vec<u8>>,
    skdm_bytes: &[u8],
) -> Result<ReceivedSkdm> {
    let mut store = SenderKeys::new(previous_bytes.as_deref())?;
    let name = SenderKeyName::from_parts(&group.to_string(), &sender.to_string());
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

/// Commit the final sender key for one group stanza. A group stanza can carry
/// both an SKDM (in a `pkmsg`/`msg` child) and an `skmsg` encrypted with the key
/// that SKDM installs, so the two updates are applied in memory and written once
/// here: a single OCC check against the value read before decryption, then one
/// write of the final state.
pub fn commit_sender_key(
    tx: &mut WriteTx<'_>,
    db_key: &[u8],
    original: Option<&[u8]>,
    final_key: &[u8],
) -> Result<()> {
    let current: Option<Vec<u8>> = tx.get_json(Table::SenderKeys, db_key, 1)?;
    if current.as_deref() != original {
        return Err(Error::invalid(
            "inbox",
            "sender key changed concurrently; group stanza not committed",
        ));
    }
    tx.put_json(Table::SenderKeys, db_key, 1, &final_key.to_vec())?;
    Ok(())
}

/// Commit an SKDM sender key update. Returns `false` when another writer changed
/// the record first: the newer state wins and the caller's message is unaffected.
/// Storage failures are propagated, never swallowed.
pub fn commit_skdm(tx: &mut WriteTx<'_>, skdm: &ReceivedSkdm) -> Result<bool> {
    let current: Option<Vec<u8>> = tx.get_json(Table::SenderKeys, &skdm.db_key, 1)?;
    if current.as_deref() != skdm.original_key.as_deref().map(|v| v.as_slice()) {
        return Ok(false);
    }
    tx.put_json(Table::SenderKeys, &skdm.db_key, 1, &*skdm.updated_key)?;
    Ok(true)
}

/// Extract the sender key distribution from a decrypted WhatsApp `Message`
/// protobuf. Returns `(group_id_string, axolotl_skdm_bytes)` when present.
///
/// `Message.senderKeyDistributionMessage` is **field 2**, confirmed live on
/// 2026-09-22: the `msg` child of a real incoming group stanza decrypted to a
/// payload whose only field number was 2. (An earlier implementation used 18,
/// which is `liveLocationMessage`; it silently matched nothing, so every group
/// message stayed undecryptable.) The inner message is `groupId` = 1 and
/// `axolotlSenderKeyDistributionMessage` = 2.
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
        #[prost(message, optional, tag = "2")]
        skdm: Option<SkdmMessage>,
    }
    let envelope = MessageWithSkdm::decode(payload).ok()?;
    let skdm = envelope.skdm?;
    let group_id = skdm.group_id.filter(|s| !s.is_empty())?;
    let axolotl = skdm.axolotl.filter(|b| !b.is_empty())?;
    Some((group_id, axolotl))
}

/// Our own sender key for a chat, plus the distribution message that lets a
/// recipient decrypt what we encrypt with it.
pub struct OwnSenderKey {
    /// `SyncActionValue`-free `Message` protobuf carrying
    /// `senderKeyDistributionMessage` (**field 2**), ready to encrypt 1:1 to
    /// each recipient device. Field 2 is the number confirmed live on the
    /// receive side; an earlier implementation used 18 and matched nothing.
    pub distribution_message: Zeroizing<Vec<u8>>,
    /// Updated sender key record. Reserve it atomically with pairwise state and
    /// the original send intent before any socket write. An uncertain send must
    /// retain the exact generation it may already have distributed.
    pub updated_key: Zeroizing<Vec<u8>>,
}
impl std::fmt::Debug for OwnSenderKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("OwnSenderKey(<redacted>)")
    }
}

/// Create (or reuse) this device's sender key for `chat` and build the
/// distribution message for it.
///
/// `chat` is the group or `status@broadcast` JID; the sender-key name is built
/// from its raw string, matching `LX/Cnr` (**confirmed**), so group and status
/// keys occupy separate namespaces.
///
/// Reserve the returned record atomically with pairwise state and the original
/// send intent before writing. Missing acceptance does not authorize forgetting
/// a generation that recipients may already hold.
pub async fn prepare_own_sender_key(
    chat: &Jid,
    own: &Jid,
    previous_bytes: Option<&[u8]>,
) -> Result<OwnSenderKey> {
    use prost::Message as _;
    use rand::SeedableRng as _;
    let mut store = SenderKeys::new(previous_bytes)?;
    let name = SenderKeyName::from_parts(&chat.to_string(), &own.to_string());
    // ThreadRng cannot cross the normal actor's await boundary. StdRng retains
    // cryptographic OS seeding while making the staged future Send.
    let mut rng = rand::rngs::StdRng::from_rng(&mut rand::rng());
    let skdm = signal::create_sender_key_distribution_message(&name, &mut store, &mut rng)
        .await
        .map_err(crypto)?;

    #[derive(prost::Message)]
    struct SkdmMessage {
        #[prost(string, optional, tag = "1")]
        group_id: Option<String>,
        #[prost(bytes = "vec", optional, tag = "2")]
        axolotl: Option<Vec<u8>>,
    }
    #[derive(prost::Message)]
    struct MessageWithSkdm {
        #[prost(message, optional, tag = "2")]
        skdm: Option<SkdmMessage>,
    }
    let distribution_message = Zeroizing::new(
        MessageWithSkdm {
            skdm: Some(SkdmMessage {
                group_id: Some(chat.to_string()),
                axolotl: Some(skdm.serialized().to_vec()),
            }),
        }
        .encode_to_vec(),
    );
    let updated_key = Zeroizing::new(
        store
            .record
            .ok_or_else(|| invalid("no sender key record was produced"))?
            .serialize()
            .map_err(|_| invalid("serialize own sender key"))?,
    );
    Ok(OwnSenderKey {
        distribution_message,
        updated_key,
    })
}

/// Encrypt one outgoing payload with our sender key for `chat`, producing the
/// `skmsg` ciphertext.
///
/// `plaintext` is the bare `Message` protobuf: **this function applies the APK's
/// E2E padding itself**, exactly as [`crate::signal::direct::prepare_payload`]
/// does for the pairwise path. Padding is the encrypting layer's job on both
/// paths so that a caller cannot pad a second time — a payload padded twice
/// survives the recipient's single strip with 1..=16 junk bytes still attached,
/// which protobuf tolerates or rejects depending on what the random bytes
/// happen to spell. That produced months of intermittent, unexplainable
/// failures before it was found.
///
/// Returns the ciphertext and the advanced record, which must be reserved
/// atomically with the send intent before writing ciphertext to the socket.
pub async fn encrypt_group_payload(
    chat: &Jid,
    own: &Jid,
    key_bytes: &[u8],
    plaintext: &[u8],
) -> Result<(Zeroizing<Vec<u8>>, Zeroizing<Vec<u8>>)> {
    if plaintext.is_empty() || plaintext.len() > 1_048_576 {
        return Err(invalid("invalid group payload size"));
    }
    let padded = pad(plaintext)?;
    let mut store = SenderKeys::new(Some(key_bytes))?;
    let name = SenderKeyName::from_parts(&chat.to_string(), &own.to_string());
    use rand::SeedableRng as _;
    let mut rng = rand::rngs::StdRng::from_rng(&mut rand::rng());
    let message = signal::group_encrypt(&mut store, &name, &padded, &mut rng)
        .await
        .map_err(crypto)?;
    let updated = Zeroizing::new(
        store
            .record
            .ok_or_else(|| invalid("group_encrypt produced no record"))?
            .serialize()
            .map_err(|_| invalid("serialize advanced sender key"))?,
    );
    Ok((Zeroizing::new(message.serialized().to_vec()), updated))
}

/// Apply the APK's E2E plaintext padding: 1..=16 bytes, each equal to the pad
/// length. The exact inverse of [`unpad`], and identical to what the direct
/// path applies before encrypting.
///
/// Deliberately **not public**. Both encrypting paths apply it themselves, and a
/// caller that padded as well left junk on the recipient's protobuf after their
/// single strip — an intermittent failure that cost two lab accounts to find.
/// Keeping it crate-private makes that impossible rather than merely documented.
pub(crate) fn pad(plaintext: &[u8]) -> Result<Zeroizing<Vec<u8>>> {
    let mut random = [0; 1];
    getrandom::fill(&mut random).map_err(|_| invalid("padding entropy unavailable"))?;
    let padding = (random[0] & 15) + 1;
    let mut padded = Zeroizing::new(plaintext.to_vec());
    padded.resize(plaintext.len() + padding as usize, padding);
    Ok(padded)
}

#[cfg(test)]
mod tests {
    /// Our own send path must produce something our own receive path accepts:
    /// mint a sender key, publish its distribution message, install that on a
    /// notional recipient, then encrypt and decrypt a padded payload. This
    /// exercises prepare_own_sender_key, pad, encrypt_group_payload and
    /// receive_group together.
    #[tokio::test]
    async fn own_sender_key_round_trips_through_the_receive_path() {
        use prost::Message as _;
        let chat = group_jid();
        let own = sender_jid();

        // 1. Create our sender key and its distribution message.
        let mine = prepare_own_sender_key(&chat, &own, None).await.unwrap();
        // The distribution message is a Message carrying field 2, which is what
        // the receive side extracts.
        let (group_id, axolotl) =
            extract_skdm_from_payload(&mine.distribution_message).expect("field 2 must be present");
        assert_eq!(group_id, chat.to_string());

        // 2. A recipient installs it.
        let db_key = sender_key_db_key(&chat, &own);
        let installed = process_skdm(&chat, &own, db_key, None, &axolotl)
            .await
            .unwrap();

        // 3. We encrypt a padded payload with our key.
        #[derive(prost::Message)]
        struct TextMessage {
            #[prost(string, tag = "1")]
            conversation: String,
        }
        let body = TextMessage {
            conversation: "fan-out".into(),
        }
        .encode_to_vec();
        let (ciphertext, advanced) = encrypt_group_payload(&chat, &own, &mine.updated_key, &body)
            .await
            .unwrap();
        assert_ne!(&*advanced, &*mine.updated_key, "our chain must advance");

        // 4. The recipient decrypts it, and the padding strips cleanly.
        let received = receive_group(&chat, &own, Some(&installed.updated_key), &ciphertext)
            .await
            .unwrap()
            .expect("the installed key must decrypt our message");
        assert_eq!(&*received.plaintext, body.as_slice());
        assert_eq!(
            TextMessage::decode(&**received.plaintext)
                .unwrap()
                .conversation,
            "fan-out"
        );
    }

    /// A device that joins the group later, or whose first stanza was lost,
    /// only ever sees the **second** distribution message. If re-publishing our
    /// key from its advanced record emitted iteration 0, that device would
    /// install a chain behind ours and silently fail to decrypt everything —
    /// a failure that shows up only on the recipient's handset. So: advance the
    /// chain, re-publish from the advanced record, install that on a receiver
    /// that has never seen the first one, and decrypt.
    #[tokio::test]
    async fn a_republished_sender_key_still_decrypts_later_messages() {
        let chat = group_jid();
        let own = sender_jid();

        let first = prepare_own_sender_key(&chat, &own, None).await.unwrap();
        let (_, first_axolotl) =
            extract_skdm_from_payload(&first.distribution_message).expect("field 2");
        let (_, advanced) = encrypt_group_payload(&chat, &own, &first.updated_key, b"one")
            .await
            .unwrap();

        // Re-publish from the advanced record, exactly as a second send does.
        let second = prepare_own_sender_key(&chat, &own, Some(&advanced))
            .await
            .unwrap();
        let (_, second_axolotl) =
            extract_skdm_from_payload(&second.distribution_message).expect("field 2");
        let (ciphertext, _) = encrypt_group_payload(&chat, &own, &second.updated_key, b"two")
            .await
            .unwrap();

        // A receiver that only ever saw the second distribution message.
        let db_key = sender_key_db_key(&chat, &own);
        let late = process_skdm(&chat, &own, db_key.clone(), None, &second_axolotl)
            .await
            .unwrap();
        let received = receive_group(&chat, &own, Some(&late.updated_key), &ciphertext)
            .await
            .unwrap()
            .expect("a late joiner must decrypt the message that followed its SKDM");
        assert_eq!(&**received.plaintext, b"two");

        // And a receiver that saw both still works, which is the ordinary case.
        let early = process_skdm(&chat, &own, db_key.clone(), None, &first_axolotl)
            .await
            .unwrap();
        let both = process_skdm(
            &chat,
            &own,
            db_key,
            Some(early.updated_key.to_vec()),
            &second_axolotl,
        )
        .await
        .unwrap();
        let received = receive_group(&chat, &own, Some(&both.updated_key), &ciphertext)
            .await
            .unwrap()
            .expect("a member that saw both distributions must still decrypt");
        assert_eq!(&**received.plaintext, b"two");
    }

    /// The bug that cost weeks: both encrypting paths pad, so a caller that
    /// pads as well leaves 1..=16 junk bytes on the recipient's protobuf after
    /// their single strip. Protobuf sometimes tolerates that and sometimes does
    /// not, depending on what the random bytes spell — which is why it looked
    /// like an intermittent, device-specific fault rather than a bug.
    ///
    /// This drives a *bare* protobuf through the group path and requires it back
    /// byte-for-byte. A caller that pre-pads would see its padding survive here,
    /// and a payload that is never padded at all would fail `unpad`.
    #[tokio::test]
    async fn the_group_path_pads_exactly_once() {
        let chat = group_jid();
        let own = sender_jid();
        let mine = prepare_own_sender_key(&chat, &own, None).await.unwrap();
        let (_, axolotl) = extract_skdm_from_payload(&mine.distribution_message).unwrap();
        let installed = process_skdm(&chat, &own, sender_key_db_key(&chat, &own), None, &axolotl)
            .await
            .unwrap();

        let bare = b"\x0a\x02hi".to_vec();
        let (ciphertext, _) = encrypt_group_payload(&chat, &own, &mine.updated_key, &bare)
            .await
            .unwrap();
        let received = receive_group(&chat, &own, Some(&installed.updated_key), &ciphertext)
            .await
            .unwrap()
            .expect("our own message must decrypt");
        assert_eq!(
            &*received.plaintext, &bare,
            "exactly one layer of padding: anything else leaves or strips bytes"
        );

        // And the distribution message itself is handed over bare, so the
        // pairwise path can apply the one layer it is responsible for.
        assert!(
            unpad(mine.distribution_message.to_vec()).is_err()
                || extract_skdm_from_payload(&mine.distribution_message).is_some(),
            "the distribution message must not arrive pre-padded"
        );
    }

    #[test]
    fn padding_is_one_to_sixteen_bytes_and_strips_back_to_the_original() {
        for _ in 0..64 {
            let padded = pad(b"payload").unwrap();
            let added = padded.len() - b"payload".len();
            assert!((1..=16).contains(&added), "pad added {added} bytes");
            assert_eq!(&*unpad(padded.to_vec()).unwrap(), b"payload");
        }
    }

    use super::*;

    fn group_jid() -> Jid {
        Jid::parse("120363000000000001@g.us").unwrap()
    }

    fn sender_jid() -> Jid {
        Jid::parse("14155550100@s.whatsapp.net").unwrap()
    }

    #[test]
    fn sender_key_db_key_preserves_device_and_primary_history() {
        let g = group_jid();
        let s = Jid::parse("14155550100:3@s.whatsapp.net").unwrap();
        let s_non_ad = sender_jid();
        assert_ne!(sender_key_db_key(&g, &s), sender_key_db_key(&g, &s_non_ad));
        assert_eq!(
            sender_key_db_key(&g, &s_non_ad),
            format!("sk:{g}:{s_non_ad}").into_bytes()
        );
        assert_ne!(
            sender_key_db_key(&g, &s),
            sender_key_db_key(&g, &s.with_device(4))
        );
        let key = sender_key_db_key(&g, &s);
        assert!(key.starts_with(b"sk:"));
        assert!(
            key.windows(g.to_string().len())
                .any(|w| w == g.to_string().as_bytes())
        );
    }

    #[tokio::test]
    async fn primary_and_companion_sender_chains_stay_independent_across_store_reopen() {
        use crate::{
            model::ids::AccountId,
            store::{MasterKey, OpenOptions, ProtocolStore, StaticKeySource},
        };
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("device-chains.redb");
        let open = || {
            ProtocolStore::open(OpenOptions {
                path: path.clone(),
                account_id: AccountId::new("device-chains").unwrap(),
                key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                    [9; 32],
                ))),
                create: true,
            })
            .unwrap()
        };
        for base in [sender_jid(), Jid::parse("111111111111111@lid").unwrap()] {
            let store = open();
            let chat = group_jid();
            let senders = [base.clone(), base.with_device(3), base.with_device(4)];
            let primary_key = sender_key_db_key(&chat, &base);
            // Preserve historical primary storage without inventing companion ownership.
            store
                .write(|tx| tx.put_json(Table::SenderKeys, &primary_key, 1, &vec![27u8; 8]))
                .unwrap();
            let (_, missing) = store
                .read(|tx| load_sender_key_bytes(tx, &chat, &senders[1]))
                .unwrap();
            assert!(missing.is_none());
            let mut ciphertexts = Vec::new();
            for (index, sender) in senders.iter().enumerate() {
                let mine = prepare_own_sender_key(&chat, sender, None).await.unwrap();
                let (_, axolotl) = extract_skdm_from_payload(&mine.distribution_message).unwrap();
                let db_key = sender_key_db_key(&chat, sender);
                let original = store
                    .read(|tx| load_sender_key_bytes(tx, &chat, sender))
                    .unwrap()
                    .1;
                // Synthetic primary history is not decoded as a valid chain.
                let installed = process_skdm(&chat, sender, db_key.clone(), None, &axolotl)
                    .await
                    .unwrap();
                store
                    .write(|tx| {
                        commit_sender_key(tx, &db_key, original.as_deref(), &installed.updated_key)
                    })
                    .unwrap();
                let payload = vec![0x0a, 1, b'a' + index as u8];
                let (ciphertext, _) =
                    encrypt_group_payload(&chat, sender, &mine.updated_key, &payload)
                        .await
                        .unwrap();
                ciphertexts.push((ciphertext, payload));
            }
            drop(store);
            let store = open();
            for (sender, (ciphertext, payload)) in senders.iter().zip(&ciphertexts) {
                let (db_key, original) = store
                    .read(|tx| load_sender_key_bytes(tx, &chat, sender))
                    .unwrap();
                let received = receive_group(&chat, sender, original.as_deref(), ciphertext)
                    .await
                    .unwrap()
                    .unwrap();
                assert_eq!(&*received.plaintext, payload);
                store
                    .write(|tx| {
                        commit_sender_key(tx, &db_key, original.as_deref(), &received.updated_key)
                    })
                    .unwrap();
            }
            let (_, primary) = store
                .read(|tx| load_sender_key_bytes(tx, &chat, &base))
                .unwrap();
            assert!(
                receive_group(&chat, &senders[1], primary.as_deref(), &ciphertexts[1].0)
                    .await
                    .is_err()
            );
        }
    }

    /// Full offline round trip: a "sender" side creates an SKDM and encrypts a
    /// padded text Message; the receiver processes the SKDM through
    /// `process_skdm`, decrypts through `receive_group`, and the unpadded
    /// plaintext parses as the original protobuf. This exercises the libsignal
    /// plumbing, the sender-key serialization and the padding strip together.
    #[tokio::test]
    async fn skdm_then_skmsg_round_trips_with_padding() {
        use prost::Message as _;
        use wacore_libsignal::protocol::{
            SenderKeyName, create_sender_key_distribution_message, group_encrypt,
        };
        let group = group_jid();
        let sender = sender_jid();
        let name = SenderKeyName::from_parts(&group.to_string(), &sender.to_string());
        let mut rng = rand::rng();

        // Sender side: fresh record + SKDM.
        let mut sender_store = SenderKeys::new(None).unwrap();
        let skdm = create_sender_key_distribution_message(&name, &mut sender_store, &mut rng)
            .await
            .unwrap();
        let skdm_bytes = skdm.serialized().to_vec();

        // Sender side: pad a text Message exactly as the APK does, then encrypt.
        #[derive(prost::Message)]
        struct TextMessage {
            #[prost(string, tag = "1")]
            conversation: String,
        }
        let mut plain = TextMessage {
            conversation: "group hello".into(),
        }
        .encode_to_vec();
        let unpadded_len = plain.len();
        let pad = 7u8;
        plain.resize(unpadded_len + pad as usize, pad);
        let skmsg = group_encrypt(&mut sender_store, &name, &plain, &mut rng)
            .await
            .unwrap();
        let ciphertext = skmsg.serialized().to_vec();

        // Receiver side: no key yet, so the skmsg alone must not decrypt.
        let db_key = sender_key_db_key(&group, &sender);
        assert!(
            receive_group(&group, &sender, None, &ciphertext)
                .await
                .unwrap()
                .is_none()
        );

        // Receiver side: process the SKDM, then decrypt with the resulting key.
        let after_skdm = process_skdm(&group, &sender, db_key.clone(), None, &skdm_bytes)
            .await
            .unwrap();
        let received = receive_group(&group, &sender, Some(&after_skdm.updated_key), &ciphertext)
            .await
            .unwrap()
            .expect("sender key present, so the skmsg must decrypt");

        // Padding stripped, and the plaintext is the original protobuf.
        assert_eq!(received.plaintext.len(), unpadded_len);
        let decoded = TextMessage::decode(&**received.plaintext).unwrap();
        assert_eq!(decoded.conversation, "group hello");
        // And it survives the field scan the projection uses.
        assert_eq!(
            crate::signal::direct::payload_fields(&received.plaintext).unwrap(),
            vec![1]
        );
    }

    #[tokio::test]
    async fn group_plaintext_with_bad_padding_is_refused() {
        // A plaintext whose trailing bytes are not a valid pad must not be
        // silently accepted as message content.
        assert!(unpad(vec![1, 2, 3, 99]).is_err());
        assert!(unpad(vec![]).is_err());
        // Valid pad of 3.
        assert_eq!(&*unpad(vec![1, 2, 3, 3, 3]).unwrap(), &[1, 2][..]);
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
    fn extract_skdm_parses_field_2() {
        use prost::Message as _;
        // Build a minimal Message proto with field 2 = senderKeyDistributionMessage.
        #[derive(prost::Message)]
        struct SkdmMsg {
            #[prost(string, optional, tag = "1")]
            group_id: Option<String>,
            #[prost(bytes = "vec", optional, tag = "2")]
            axolotl: Option<Vec<u8>>,
        }
        #[derive(prost::Message)]
        struct MsgWithSkdm {
            #[prost(message, optional, tag = "2")]
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
            #[prost(message, optional, tag = "2")]
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
        let result = receive_group(&g, &s, None, &[1, 2, 3, 4]).await.unwrap();
        assert!(result.is_none());
    }
}
