//! Polls, traced from comparison APK **2.26.37.71** protobuf-lite descriptors
//! and a live poll received from the official client.
//!
//! * `LX/BYa.dynamicMethod` (PollCreationMessage): encKey=1 bytes, name=2,
//!   options=3 repeated (`LX/BSo`: optionName=1, optionHash=2),
//!   selectableOptionsCount=4 uint32, contextInfo=5, pollContentType=6,
//!   pollType=7, correctAnswer=8. The live poll arrived as `message type=poll`
//!   with `meta polltype=creation`, payload fields 35 (`messageContextInfo`,
//!   `LX/BQ2`: messageSecret=3) and 64 (`pollCreationMessageV3`).
//! * `LX/BWH.dynamicMethod` (PollUpdateMessage): pollCreationMessageKey=1,
//!   vote=2 (PollEncValue: encPayload=1, encIv=2), metadata=3,
//!   senderTimestampMs=4. `LX/BOq` (PollVoteMessage): selectedOptions=1
//!   repeated bytes. `polltype` values in the APK: creation, vote, add_option,
//!   result_snapshot, quiz_creation. `Message.pollUpdateMessage` is field 50
//!   (public WA6 numbering, consistent with the observed 46 and 64).
//! * Vote crypto: `LX/0ZG.A01` (MessageAddOnPollVoteUtils) requires a 32-byte
//!   secret and 12-byte IV, builds the AAD as `pollMessageId || 0x00 ||
//!   voterJid` and hands secret, ciphertext, IV, poll id, creator JID, voter JID
//!   and the label `"Poll Vote"` to `LX/C9s.A00` → `LX/CmM.A02` → native
//!   `JniBridge`. The derivation inside the native library is not visible in
//!   DEX; this module uses the publicly documented recipe consistent with those
//!   parameters: `key = HKDF-SHA256(secret, salt = none, info = pollId ||
//!   creatorJid || voterJid || "Poll Vote", 32)` and AES-256-GCM. Option hashes
//!   are SHA-256 of the option name (`optionHash`, `PollNameHashMismatch`).
//!   Live confirmation decides whether the recipe is right.
use crate::{
    Error, Result,
    model::{
        ids::MessageId,
        jid::Jid,
        message::{Direction, MessageRef},
    },
};
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, Payload},
};
use prost::Message as _;
use sha2::{Digest, Sha256};

/// Longest poll question or option, in bytes (local bound).
pub const MAX_TEXT_BYTES: usize = 1024;
/// Most options accepted (local bound; the official UI allows twelve).
pub const MAX_OPTIONS: usize = 12;

fn invalid(reason: &'static str) -> Error {
    Error::invalid("poll", reason)
}

#[derive(prost::Message)]
pub(crate) struct Envelope {
    #[prost(message, optional, tag = "35")]
    pub(crate) context: Option<ContextInfo>,
    #[prost(message, optional, tag = "50")]
    pub(crate) update: Option<PollUpdate>,
    #[prost(message, optional, tag = "64")]
    pub(crate) creation_v3: Option<PollCreation>,
}
#[derive(prost::Message)]
pub(crate) struct ContextInfo {
    #[prost(bytes = "vec", optional, tag = "3")]
    pub(crate) message_secret: Option<Vec<u8>>,
}
#[derive(prost::Message)]
pub(crate) struct PollCreation {
    #[prost(string, optional, tag = "2")]
    pub(crate) name: Option<String>,
    #[prost(message, repeated, tag = "3")]
    pub(crate) options: Vec<PollOption>,
    #[prost(uint32, optional, tag = "4")]
    pub(crate) selectable_options_count: Option<u32>,
}
#[derive(prost::Message)]
pub(crate) struct PollOption {
    #[prost(string, optional, tag = "1")]
    pub(crate) option_name: Option<String>,
}
#[derive(prost::Message)]
pub(crate) struct PollUpdate {
    #[prost(message, optional, tag = "1")]
    pub(crate) poll_creation_message_key: Option<super::reactions::Key>,
    #[prost(message, optional, tag = "2")]
    pub(crate) vote: Option<EncValue>,
    #[prost(bytes = "vec", optional, tag = "3")]
    pub(crate) metadata: Option<Vec<u8>>,
    #[prost(int64, optional, tag = "4")]
    pub(crate) sender_timestamp_ms: Option<i64>,
}
impl PollUpdate {
    fn metadata_present_value(&self) -> bool {
        self.metadata.is_some()
    }
}
/// `PollEncValue`: AES-256-GCM ciphertext and its 12-byte nonce.
#[derive(prost::Message, Clone, PartialEq, Eq)]
pub struct EncValue {
    /// Ciphertext with tag.
    #[prost(bytes = "vec", optional, tag = "1")]
    pub enc_payload: Option<Vec<u8>>,
    /// Nonce.
    #[prost(bytes = "vec", optional, tag = "2")]
    pub enc_iv: Option<Vec<u8>>,
}
#[derive(prost::Message)]
pub(crate) struct VoteMessage {
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub(crate) selected_options: Vec<Vec<u8>>,
}

/// A decoded poll creation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Poll {
    /// Question.
    pub name: String,
    /// Options in wire order.
    pub options: Vec<String>,
    /// Selectable options; `0` means unlimited.
    pub selectable_count: u32,
    /// The message secret used to encrypt votes.
    pub secret: Option<[u8; 32]>,
}
/// A decoded (still encrypted) vote update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptedVote {
    /// Poll id from the key.
    pub poll_id: MessageId,
    /// Sender-relative `fromMe` from the key.
    pub poll_from_sender: bool,
    /// Ciphertext.
    pub enc_payload: Vec<u8>,
    /// Nonce.
    pub enc_iv: Vec<u8>,
}

fn validate_text(text: &str, what: &'static str) -> Result<()> {
    if text.is_empty() || text.len() > MAX_TEXT_BYTES || text.chars().any(char::is_control) {
        return Err(invalid(what));
    }
    Ok(())
}
/// Validate a poll intent.
pub fn validate_poll(name: &str, options: &[String], selectable_count: u32) -> Result<()> {
    validate_text(
        name,
        "poll name must be 1..=1024 bytes without control characters",
    )?;
    if options.len() < 2 || options.len() > MAX_OPTIONS {
        return Err(invalid("polls need 2..=12 options"));
    }
    for option in options {
        validate_text(
            option,
            "option must be 1..=1024 bytes without control characters",
        )?;
    }
    let mut unique: Vec<&str> = options.iter().map(String::as_str).collect();
    unique.sort_unstable();
    unique.dedup();
    if unique.len() != options.len() {
        return Err(invalid("options must be distinct"));
    }
    if selectable_count as usize > options.len() {
        return Err(invalid("selectable count exceeds the option count"));
    }
    Ok(())
}

/// Decode a poll creation payload (fields 64 and optionally 35).
pub fn decode_poll(payload: &[u8]) -> Option<Poll> {
    let envelope = Envelope::decode(payload).ok()?;
    let creation = envelope.creation_v3?;
    let name = creation.name.unwrap_or_default();
    let options: Vec<String> = creation
        .options
        .into_iter()
        .filter_map(|o| o.option_name)
        .collect();
    if options.is_empty() || options.len() > 64 {
        return None;
    }
    let secret = envelope
        .context
        .and_then(|c| c.message_secret)
        .and_then(|s| <[u8; 32]>::try_from(s).ok());
    Some(Poll {
        name,
        options,
        selectable_count: creation.selectable_options_count.unwrap_or(0),
        secret,
    })
}
// APK 37.71 Ban.A01 first validates BZa using C7G.A00 (0066..007e).
// Both feature-selected paths require a sender timestamp; the deferred branch
// explicitly checks a 12-byte IV (0544..060c). Metadata presence is not a
// common prerequisite. Native ChatJid subtype parity remains incomplete.
fn vote_failure(update: &PollUpdate) -> Option<&'static str> {
    let Some(key) = update.poll_creation_message_key.as_ref() else {
        return Some("missing_key");
    };
    if let Some(reason) = super::reactions::key_failure(key) {
        return Some(reason);
    }
    if update.sender_timestamp_ms.is_none() {
        return Some("missing_sender_timestamp");
    }
    let Some(vote) = update.vote.as_ref() else {
        return Some("missing_vote");
    };
    let Some(iv) = vote.enc_iv.as_ref() else {
        return Some("missing_enc_iv");
    };
    if iv.len() != 12 {
        return Some("invalid_enc_iv");
    }
    if vote.enc_payload.is_none() {
        return Some("missing_enc_payload");
    }
    None
}

/// Fixed, redacted structural diagnosis of a decodable poll update.
/// `None` means no decodable field-50 update, not a valid vote. `valid` proves
/// only the supported key, timestamp and encrypted-value structure, not AEAD
/// authentication, target attribution, option validity or native JID parity.
pub fn vote_validation(payload: &[u8]) -> Option<&'static str> {
    let update = Envelope::decode(payload).ok()?.update?;
    Some(vote_failure(&update).unwrap_or("valid"))
}

/// Decode a structurally supported poll update (field 50) without decrypting it.
pub fn decode_vote(payload: &[u8]) -> Option<EncryptedVote> {
    let update = Envelope::decode(payload).ok()?.update?;
    if vote_failure(&update).is_some() {
        return None;
    }
    let key = update.poll_creation_message_key?;
    let vote = update.vote?;
    Some(EncryptedVote {
        poll_id: MessageId::new(key.id?).ok()?,
        poll_from_sender: key.from_me.unwrap_or(false),
        enc_payload: vote.enc_payload?,
        enc_iv: vote.enc_iv?,
    })
}

/// SHA-256 of an option name, the poll option hash.
pub fn option_hash(name: &str) -> [u8; 32] {
    Sha256::digest(name.as_bytes()).into()
}
fn derive_key(secret: &[u8; 32], poll_id: &str, creator: &Jid, voter: &Jid) -> [u8; 32] {
    let mut info = Vec::new();
    info.extend_from_slice(poll_id.as_bytes());
    info.extend_from_slice(creator.to_non_ad().to_string().as_bytes());
    info.extend_from_slice(voter.to_non_ad().to_string().as_bytes());
    info.extend_from_slice(b"Poll Vote");
    let hk = hkdf::Hkdf::<Sha256>::new(None, secret);
    let mut key = [0u8; 32];
    hk.expand(&info, &mut key)
        .expect("32 bytes is a valid HKDF length");
    key
}
fn aad(poll_id: &str, voter: &Jid) -> Vec<u8> {
    let mut aad = Vec::new();
    aad.extend_from_slice(poll_id.as_bytes());
    aad.push(0);
    aad.extend_from_slice(voter.to_non_ad().to_string().as_bytes());
    aad
}

/// Encrypt a vote for `poll_id` created by `creator`, cast by `voter`.
pub fn encrypt_vote(
    secret: &[u8; 32],
    poll_id: &str,
    creator: &Jid,
    voter: &Jid,
    selected_names: &[String],
    iv: [u8; 12],
) -> Result<EncValue> {
    let plain = VoteMessage {
        selected_options: selected_names
            .iter()
            .map(|n| option_hash(n).to_vec())
            .collect(),
    }
    .encode_to_vec();
    let cipher = Aes256Gcm::new(derive_key(secret, poll_id, creator, voter).as_ref().into());
    let enc_payload = cipher
        .encrypt(
            Nonce::from_slice(&iv),
            Payload {
                msg: &plain,
                aad: &aad(poll_id, voter),
            },
        )
        .map_err(|_| invalid("vote encryption failed"))?;
    Ok(EncValue {
        enc_payload: Some(enc_payload),
        enc_iv: Some(iv.to_vec()),
    })
}
/// Decrypt a vote; returns the selected option hashes.
pub fn decrypt_vote(
    secret: &[u8; 32],
    poll_id: &str,
    creator: &Jid,
    voter: &Jid,
    vote: &EncryptedVote,
) -> Result<Vec<Vec<u8>>> {
    if vote.enc_iv.len() != 12 {
        return Err(invalid("vote IV must be 12 bytes"));
    }
    let cipher = Aes256Gcm::new(derive_key(secret, poll_id, creator, voter).as_ref().into());
    let plain = cipher
        .decrypt(
            Nonce::from_slice(&vote.enc_iv),
            Payload {
                msg: &vote.enc_payload,
                aad: &aad(poll_id, voter),
            },
        )
        .map_err(|_| invalid("vote decryption failed"))?;
    Ok(VoteMessage::decode(plain.as_slice())
        .map_err(|_| invalid("vote plaintext is not a PollVoteMessage"))?
        .selected_options)
}
/// Diagnostic view of a vote payload's key and metadata fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoteKeyFields {
    /// `key.remoteJid` as sent.
    pub remote_jid: Option<String>,
    /// `key.fromMe` as sent.
    pub from_me: Option<bool>,
    /// Whether the (empty) `metadata` field was present.
    pub metadata_present: bool,
    /// `senderTimestampMs` as sent.
    pub sender_timestamp_ms: Option<i64>,
}

/// Diagnostic: raw key fields and metadata presence of a vote payload.
pub fn vote_key_fields(payload: &[u8]) -> Option<VoteKeyFields> {
    let update = Envelope::decode(payload).ok()?.update?;
    let metadata_present = update.metadata_present_value();
    let sender_timestamp_ms = update.sender_timestamp_ms;
    let key = update.poll_creation_message_key?;
    Some(VoteKeyFields {
        remote_jid: key.remote_jid,
        from_me: key.from_me,
        metadata_present,
        sender_timestamp_ms,
    })
}

/// Map selected hashes back to option names; unknown hashes come back as hex.
pub fn resolve_options(hashes: &[Vec<u8>], options: &[String]) -> (Vec<String>, Vec<String>) {
    let mut selected = Vec::new();
    let mut unresolved = Vec::new();
    for hash in hashes {
        match options
            .iter()
            .find(|o| option_hash(o).as_slice() == hash.as_slice())
        {
            Some(name) => selected.push(name.clone()),
            None => unresolved.push(hex::encode(hash)),
        }
    }
    (selected, unresolved)
}

/// Encode a poll creation payload with a fresh message secret.
pub fn creation_payload(
    name: &str,
    options: &[String],
    selectable_count: u32,
    secret: [u8; 32],
) -> Result<Vec<u8>> {
    validate_poll(name, options, selectable_count)?;
    Ok(Envelope {
        context: Some(ContextInfo {
            message_secret: Some(secret.to_vec()),
        }),
        update: None,
        creation_v3: Some(PollCreation {
            name: Some(name.to_owned()),
            options: options
                .iter()
                .map(|o| PollOption {
                    option_name: Some(o.clone()),
                })
                .collect(),
            selectable_options_count: Some(selectable_count),
        }),
    }
    .encode_to_vec())
}
/// Encode a vote payload. `remote` is the key's `remoteJid` identity (the
/// chat partner, LID when the chat is LID-addressed, as for reactions).
pub fn vote_payload(
    remote: &Jid,
    target: &MessageRef,
    vote: EncValue,
    sender_timestamp_ms: i64,
) -> Vec<u8> {
    Envelope {
        context: None,
        update: Some(PollUpdate {
            poll_creation_message_key: Some(super::reactions::Key {
                remote_jid: Some(remote.to_non_ad().to_string()),
                from_me: Some(target.key.direction == Direction::Outgoing),
                id: Some(target.key.id.as_str().to_owned()),
                participant: None,
            }),
            vote: Some(vote),
            // The official client sends an empty PollUpdateMessageMetadata (observed live).
            metadata: Some(Vec::new()),
            sender_timestamp_ms: Some(sender_timestamp_ms),
        }),
        creation_v3: None,
    }
    .encode_to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn creation_roundtrips_and_validates() {
        let options = vec!["Yes".to_owned(), "No".to_owned(), "Maybe".to_owned()];
        let bytes = creation_payload("Lunch?", &options, 0, [7; 32]).unwrap();
        // Field 35, wire type 2: (35 << 3) | 2 = 282 = varint 9A 02.
        assert_eq!(&bytes[..2], &[0x9A, 0x02]);
        let poll = decode_poll(&bytes).unwrap();
        assert_eq!(poll.name, "Lunch?");
        assert_eq!(poll.options, options);
        assert_eq!(poll.selectable_count, 0);
        assert_eq!(poll.secret, Some([7; 32]));
        assert!(
            validate_poll("q", &["a".into()], 0).is_err(),
            "two options minimum"
        );
        assert!(
            validate_poll("q", &["a".into(), "a".into()], 0).is_err(),
            "distinct"
        );
        assert!(
            validate_poll("q", &["a".into(), "b".into()], 3).is_err(),
            "selectable bound"
        );
        assert!(validate_poll("", &["a".into(), "b".into()], 1).is_err());
        assert!(decode_poll(b"\x0a\x04test").is_none());
    }
    #[test]
    fn votes_encrypt_decrypt_bind_aad_and_resolve_hashes() {
        let secret = [3u8; 32];
        let creator = Jid::parse("15555550101@s.whatsapp.net").unwrap();
        let voter = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let options = vec!["Yes".to_owned(), "No".to_owned()];
        let enc = encrypt_vote(&secret, "POLL1", &creator, &voter, &options[..1], [9; 12]).unwrap();
        let target = MessageRef {
            key: crate::model::message::MessageKey {
                chat: crate::model::ids::ChatId::from_jid(&creator).unwrap(),
                sender: None,
                direction: Direction::Incoming,
                id: MessageId::new("POLL1").unwrap(),
            },
        };
        let bytes = vote_payload(&creator, &target, enc, 5);
        // Field 50, wire type 2: (50 << 3) | 2 = 402 = varint 92 03.
        assert_eq!(&bytes[..2], &[0x92, 0x03]);
        let vote = decode_vote(&bytes).unwrap();
        assert_eq!(vote.poll_id.as_str(), "POLL1");
        assert!(!vote.poll_from_sender);
        let hashes = decrypt_vote(&secret, "POLL1", &creator, &voter, &vote).unwrap();
        assert_eq!(hashes, vec![option_hash("Yes").to_vec()]);
        let (selected, unresolved) = resolve_options(&hashes, &options);
        assert_eq!(selected, vec!["Yes".to_owned()]);
        assert!(unresolved.is_empty());
        // Wrong voter, creator, secret or poll id fails authentication.
        let other = Jid::parse("15555550102@s.whatsapp.net").unwrap();
        assert!(decrypt_vote(&secret, "POLL1", &creator, &other, &vote).is_err());
        assert!(decrypt_vote(&secret, "POLL1", &other, &voter, &vote).is_err());
        assert!(decrypt_vote(&[4; 32], "POLL1", &creator, &voter, &vote).is_err());
        assert!(decrypt_vote(&secret, "POLL2", &creator, &voter, &vote).is_err());
        let (selected, unresolved) = resolve_options(&[vec![1, 2, 3]], &options);
        assert!(selected.is_empty());
        assert_eq!(unresolved, vec!["010203".to_owned()]);
    }
}

#[cfg(test)]
mod vote_validation_tests {
    use super::*;

    // Independently assembled protobuf wire fixture from APK 37.71 BWH/BSp
    // and BZa descriptors: field 50; key remote=555@lid/id=POLL; explicitly
    // present empty ciphertext, twelve-byte IV and zero sender timestamp.
    // fromMe and metadata are absent: neither is a common Ban.A01 prerequisite.
    const FIXTURE: &str =
        "9203250a0f0a07353535406c69641a04504f4c4c12100a00120c0000000000000000000000002000";

    #[test]
    fn apk_presence_rules_do_not_invent_positive_timestamp_or_metadata_checks() {
        let bytes = hex::decode(FIXTURE).unwrap();
        assert_eq!(vote_validation(&bytes), Some("valid"));
        let vote = decode_vote(&bytes).unwrap();
        assert!(!vote.poll_from_sender);
        assert_eq!(vote.poll_id.as_str(), "POLL");
        assert!(vote.enc_payload.is_empty());
        // Structural validation never establishes authenticated vote contents.
        let creator = Jid::parse("555@lid").unwrap();
        assert!(decrypt_vote(&[7; 32], "POLL", &creator, &creator, &vote).is_err());
        let mut envelope = Envelope::decode(bytes.as_slice()).unwrap();
        envelope.update.as_mut().unwrap().sender_timestamp_ms = Some(-1);
        assert_eq!(vote_validation(&envelope.encode_to_vec()), Some("valid"));
        assert_eq!(vote_validation(b"\x0a\x01x"), None);
        assert_eq!(vote_validation(b"\x92\x03\xff"), None);
    }

    #[test]
    fn malformed_parent_and_encrypted_value_remain_opaque_with_fixed_diagnoses() {
        type Change = fn(&mut PollUpdate);
        let cases: &[(&str, Change)] = &[
            ("missing_key", |u| u.poll_creation_message_key = None),
            ("missing_id", |u| {
                u.poll_creation_message_key.as_mut().unwrap().id = None
            }),
            ("invalid_id", |u| {
                u.poll_creation_message_key.as_mut().unwrap().id = Some(String::new())
            }),
            ("missing_remote_jid", |u| {
                u.poll_creation_message_key.as_mut().unwrap().remote_jid = None
            }),
            ("invalid_remote_jid", |u| {
                u.poll_creation_message_key.as_mut().unwrap().remote_jid =
                    Some("PRIVATE BAD REMOTE".into())
            }),
            ("unsupported_remote_jid", |u| {
                u.poll_creation_message_key.as_mut().unwrap().remote_jid =
                    Some("555@private.invalid".into())
            }),
            ("missing_sender_timestamp", |u| u.sender_timestamp_ms = None),
            ("missing_vote", |u| u.vote = None),
            ("missing_enc_iv", |u| u.vote.as_mut().unwrap().enc_iv = None),
            ("invalid_enc_iv", |u| {
                u.vote.as_mut().unwrap().enc_iv = Some(vec![0; 11])
            }),
            ("invalid_enc_iv", |u| {
                u.vote.as_mut().unwrap().enc_iv = Some(vec![0; 13])
            }),
            ("missing_enc_payload", |u| {
                u.vote.as_mut().unwrap().enc_payload = None
            }),
        ];
        for (expected, change) in cases {
            let mut envelope = Envelope::decode(hex::decode(FIXTURE).unwrap().as_slice()).unwrap();
            change(envelope.update.as_mut().unwrap());
            let payload = envelope.encode_to_vec();
            assert!(decode_vote(&payload).is_none(), "{expected}");
            assert_eq!(vote_validation(&payload), Some(*expected));
        }
    }
}
