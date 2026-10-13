//! Single-recipient primary-device encryption over the upstream Signal backend.
//! Mutations stay in memory until caller atomically commits them with send intent.
use crate::{
    Error, Result,
    binary::{Node, NodeContent},
    model::jid::Jid,
    session::AccountRecord,
};
use async_trait::async_trait;
use prost::Message;
use rand::SeedableRng;
use wacore_libsignal::protocol as signal;
use wacore_libsignal::protocol::{
    CiphertextMessageType, DeviceId, Direction, IdentityChange, IdentityKey, IdentityKeyPair,
    IdentityKeyStore, PreKeyBundle, PrivateKey, ProtocolAddress, PublicKey, SessionRecord,
    SessionStore, SignalProtocolError, UsePQRatchet, message_encrypt, process_prekey_bundle,
};
use zeroize::Zeroizing;
mod incoming;
pub use incoming::{
    IncomingKem, ReceivedPrekey, incoming_prekey_id, incoming_prekey_identity_fingerprints,
    incoming_prekey_identity_matches, incoming_prekey_kem, receive_prekey,
    receive_prekey_with_policy,
};

fn invalid(reason: &'static str) -> Error {
    Error::invalid("direct_message", reason)
}
fn task_rng() -> rand::rngs::StdRng {
    rand::rngs::StdRng::from_rng(&mut rand::rng())
}

/// Read protobuf field numbers without exposing any message values.
pub fn payload_fields(mut bytes: &[u8]) -> Result<Vec<u32>> {
    use prost::encoding::{DecodeContext, decode_key, skip_field};
    if bytes.len() > 65536 {
        return Err(invalid("payload exceeds inspection limit"));
    }
    let mut fields = Vec::new();
    while !bytes.is_empty() {
        let (tag, wire) =
            decode_key(&mut bytes).map_err(|_| invalid("invalid message protobuf"))?;
        skip_field(wire, tag, &mut bytes, DecodeContext::default())
            .map_err(|_| invalid("invalid message protobuf"))?;
        fields.push(tag);
    }
    Ok(fields)
}

/// Redacted inspection of the comparison Message envelope. Unknown fields stay
/// visible as field numbers; this is not a complete message renderer.
pub fn payload_summary(bytes: &[u8]) -> Result<String> {
    payload_summary_depth(bytes, 0)
}
fn payload_summary_depth(bytes: &[u8], depth: usize) -> Result<String> {
    if depth > 4 {
        return Err(invalid("message wrapper nesting exceeds inspection bound"));
    }
    #[derive(prost::Message)]
    struct Envelope {
        #[prost(message, optional, tag = "31")]
        device_sent: Option<DeviceSent>,
        #[prost(message, optional, tag = "37")]
        view_once: Option<Wrapper>,
        #[prost(message, optional, tag = "55")]
        view_once_v2: Option<Wrapper>,
        #[prost(message, optional, tag = "59")]
        view_once_extension: Option<Wrapper>,
        #[prost(message, optional, tag = "91")]
        associated_child: Option<Wrapper>,
        #[prost(bytes = "vec", optional, tag = "3")]
        image: Option<Vec<u8>>,
        #[prost(bytes = "vec", optional, tag = "6")]
        extended: Option<Vec<u8>>,
        #[prost(message, optional, tag = "12")]
        protocol: Option<Protocol>,
        // APK 2.26.37.71 BZc.dynamicMethod: messageHistoryNotice=102;
        // BV6.dynamicMethod: contextInfo=1, messageHistoryMetadata=2,
        // botHistoryShareSyncMetadata=3. Inspect structure, never process history.
        #[prost(bytes = "vec", optional, tag = "102")]
        history_notice: Option<Vec<u8>>,
    }
    #[derive(prost::Message)]
    struct Wrapper {
        #[prost(bytes = "vec", tag = "1")]
        message: Vec<u8>,
    }
    // APK 2.26.37.71 BUs.dynamicMethod: destination=1, Message=2, phash=3.
    #[derive(prost::Message)]
    struct DeviceSent {
        #[prost(string, optional, tag = "1")]
        destination: Option<String>,
        #[prost(bytes = "vec", optional, tag = "2")]
        message: Option<Vec<u8>>,
        #[prost(string, optional, tag = "3")]
        phash: Option<String>,
    }
    #[derive(prost::Message)]
    struct ImageFlags {
        #[prost(bool, tag = "25")]
        view_once: bool,
    }
    #[derive(prost::Message)]
    struct Protocol {
        #[prost(int32, optional, tag = "2")]
        kind: Option<i32>,
        #[prost(bytes = "vec", optional, tag = "7")]
        key_share: Option<Vec<u8>>,
    }
    let fields = payload_fields(bytes)?;
    let envelope = Envelope::decode(bytes).map_err(|_| invalid("invalid message envelope"))?;
    let mut summary = format!("protobuf fields={fields:?}");
    if let Some(wrapper) = envelope.device_sent {
        let destination = wrapper.destination.map(Zeroizing::new);
        let phash = wrapper.phash.map(Zeroizing::new);
        let message = wrapper.message.map(Zeroizing::new);
        summary.push_str(&format!(
            "; device-sent destination_present={} message_present={} phash_present={}; admission not checked",
            destination.is_some(), message.is_some(), phash.is_some()
        ));
        if let Some(message) = message {
            summary.push_str(&format!(
                "; [{}]",
                payload_summary_depth(&message, depth + 1)?
            ));
        }
    }
    for (label, wrapper) in [
        ("view-once", envelope.view_once),
        ("view-once-v2", envelope.view_once_v2),
        ("view-once-extension", envelope.view_once_extension),
        ("associated-child", envelope.associated_child),
    ] {
        if let Some(wrapper) = wrapper {
            let message = Zeroizing::new(wrapper.message);
            summary.push_str(&format!(
                "; {label} [{}]; attachment not opened",
                payload_summary_depth(&message, depth + 1)?
            ));
        }
    }
    if let Some(image) = envelope.image {
        let image = Zeroizing::new(image);
        summary.push_str(&format!("; image fields={:?}", payload_fields(&image)?));
        let flags =
            ImageFlags::decode(image.as_slice()).map_err(|_| invalid("invalid image flags"))?;
        if flags.view_once {
            summary.push_str("; image view_once=true; attachment not opened");
        }
    }
    if let Some(extended) = envelope.extended {
        let extended = Zeroizing::new(extended);
        summary.push_str(&format!(
            "; extended-text fields={:?}",
            payload_fields(&extended)?
        ));
        #[derive(prost::Message)]
        struct ExtendedContext {
            #[prost(bytes = "vec", optional, tag = "17")]
            context: Option<Vec<u8>>,
        }
        if let Some(context) = ExtendedContext::decode(extended.as_slice())
            .map_err(|_| invalid("invalid extended text"))?
            .context
        {
            let context = Zeroizing::new(context);
            summary.push_str(&format!("; context fields={:?}", payload_fields(&context)?));
        }
    }
    if let Some(protocol) = envelope.protocol {
        summary.push_str(&format!("; protocol type={:?}", protocol.kind));
        if protocol.kind == Some(4) {
            // Comparison LX/C1B.<clinit> 0x0028..003c.
            summary.push_str(" (EPHEMERAL_SYNC_RESPONSE)");
        }
        if protocol.kind == Some(6) {
            let bytes = Zeroizing::new(
                protocol
                    .key_share
                    .ok_or_else(|| invalid("key-share body missing"))?,
            );
            let share = super::app_state::KeyShare::decode(&bytes)?;
            summary.push_str(&format!(
                " (APP_STATE_SYNC_KEY_SHARE); {} structurally valid keys; inspection does not import or check saved import status",
                share.len()
            ));
        }
    }
    if let Some(notice) = envelope.history_notice {
        let notice = Zeroizing::new(notice);
        summary.push_str(&format!(
            "; message-history-notice fields={:?}; history not applied",
            payload_fields(&notice)?
        ));
        match crate::transport::history_notice::decode(bytes) {
            Ok(Some(metadata)) => summary.push_str(&format!(
                "; metadata=validated; receivers={} non_receivers={} oldest_bundle_present={} context_present={} bot_metadata_present={}",
                metadata.receiver_count,
                metadata.non_receiver_count,
                metadata.oldest_bundle_timestamp.is_some(),
                metadata.context_present,
                metadata.bot_metadata_present,
            )),
            Ok(None) => summary.push_str("; metadata=unsupported; reason=notice missing"),
            Err(error) => summary.push_str(&format!(
                "; metadata=unsupported; reason={error}"
            )),
        }
    }
    Ok(summary)
}
fn crypto(error: SignalProtocolError) -> Error {
    // libsignal errors can carry peer addresses, application strings and
    // backend details. Classify by variant, never format the underlying error.
    // Preserved live pkmsg failures (2026-10-06) exposed how the former blanket
    // message hid missing keys versus changed identity pins versus a bad MAC.
    let reason = match error {
        SignalProtocolError::UntrustedIdentity(_) => {
            "Signal peer identity untrusted; saved pin preserved"
        }
        SignalProtocolError::InvalidPreKeyId => "Signal one-time prekey unavailable or invalid",
        SignalProtocolError::InvalidSignedPreKeyId => "Signal signed prekey unavailable or invalid",
        SignalProtocolError::SessionNotFound(_) => "Signal session not found",
        SignalProtocolError::DuplicatedMessage(..) => "Signal duplicate or old message counter",
        SignalProtocolError::BadMac(_) => "Signal message MAC verification failed",
        SignalProtocolError::InvalidMessage(..) => "Signal message invalid",
        SignalProtocolError::InvalidSessionStructure(_)
        | SignalProtocolError::InvalidSenderKeySession => "Signal session structure invalid",
        SignalProtocolError::NoSenderKeyState(_) => "Signal sender-key state missing",
        SignalProtocolError::InvalidRegistrationId(..) => "Signal session registration id invalid",
        SignalProtocolError::SignatureValidationFailed => "Signal signature verification failed",
        SignalProtocolError::InvalidProtobufEncoding => "Signal protobuf encoding invalid",
        SignalProtocolError::CiphertextMessageTooShort(_) => "Signal ciphertext too short",
        SignalProtocolError::LegacyCiphertextVersion(_)
        | SignalProtocolError::UnrecognizedCiphertextVersion(_)
        | SignalProtocolError::UnrecognizedMessageVersion(_) => {
            "Signal message version unsupported"
        }
        SignalProtocolError::NoKeyTypeIdentifier
        | SignalProtocolError::BadKeyType(_)
        | SignalProtocolError::BadKeyLength(..)
        | SignalProtocolError::InvalidMacKeyLength(_) => "Signal key encoding invalid",
        SignalProtocolError::KeyAgreementFailed(_) => "Signal key agreement failed",
        SignalProtocolError::BackendError(..)
        | SignalProtocolError::ApplicationCallbackError(..)
        | SignalProtocolError::FfiBindingError(_) => "Signal backend operation failed",
        _ => "Signal key/session operation failed",
    };
    invalid(reason)
}
fn data(node: &Node) -> Result<&[u8]> {
    match &node.content {
        NodeContent::Bytes(v) => Ok(v),
        NodeContent::Text(v) => Ok(v.as_bytes()),
        _ => Err(invalid("expected key bytes")),
    }
}
pub(crate) fn child<'a>(node: &'a Node, name: &str) -> Result<&'a Node> {
    let mut matches = node.child_nodes().iter().filter(|v| v.description == name);
    let value = matches
        .next()
        .ok_or_else(|| invalid("missing key bundle field"))?;
    if matches.next().is_some() {
        return Err(invalid("duplicate key bundle field"));
    }
    Ok(value)
}
pub(crate) fn number(node: &Node, size: usize) -> Result<u32> {
    let bytes = data(node)?;
    if bytes.len() != size {
        return Err(invalid("invalid key identifier width"));
    }
    Ok(bytes
        .iter()
        .fold(0, |value, byte| (value << 8) | *byte as u32))
}
fn public(node: &Node) -> Result<PublicKey> {
    let bytes = data(node)?;
    if bytes.len() != 32 {
        return Err(invalid("invalid public key width"));
    }
    PublicKey::from_djb_public_key_bytes(bytes).map_err(|_| invalid("invalid public key"))
}

/// Validate identity/registration/signed-key fields for the exact requested JID.
pub fn prekey_bundle(reply: &Node, recipient: &Jid) -> Result<PreKeyBundle> {
    prekey_bundle_for(reply, recipient)
}

/// Select and parse one recipient's classic bundle from a key reply that may cover
/// several devices, as a group fan-out's batched query does.
///
/// A reply listing other devices is not an error for this recipient; only the
/// absence of *this* device, or a per-user `<error>` on it, is. The JID match
/// compares the complete address, including agent and device, so a reply that echoes
/// the phone JID where we asked with a LID (or the reverse) is refused rather
/// than silently paired with the wrong identity key.
/// Selected `pqkey` input is refused here: this consumable parser is classic-only
/// and must never discard a KEM key to construct a classic session. Callers that
/// support PQ select `signal::hybrid::bundle_for` (see `hybrid::selects_pq_key`).
pub fn prekey_bundle_for(reply: &Node, recipient: &Jid) -> Result<PreKeyBundle> {
    let user = prekey_user_for(reply, recipient)?;
    if user.child("pqkey").is_some() {
        // Hybrid-capable callers use signal::hybrid::bundle_for; this classic parser
        // never drops a selected PQ key to derive classic state.
        return Err(invalid("post-quantum session processing is unsupported"));
    }
    classic_bundle_from_user(user, recipient)
}

// Diagnostics validate only classic metadata. Callers must keep the existing PQ
// deferral before deriving/applying any session; this is not a consumable PQ bundle.
#[cfg(all(feature = "experimental-noise", feature = "experimental-transport"))]
pub(crate) fn classic_bundle_metadata_for(reply: &Node, recipient: &Jid) -> Result<PreKeyBundle> {
    classic_bundle_from_user(prekey_user_for(reply, recipient)?, recipient)
}

pub(crate) fn prekey_user_for<'a>(reply: &'a Node, recipient: &Jid) -> Result<&'a Node> {
    if reply.description != "iq"
        || reply.get_attr("type").and_then(|v| v.as_text()) != Some("result")
    {
        return Err(invalid("key query was not successful"));
    }
    if reply.child("error").is_some() {
        return Err(invalid("server refused recipient key lookup"));
    }
    let list = child(reply, "list")?;
    let same_address = |v: &Jid| {
        v.user == recipient.user
            && v.server == recipient.server
            && v.agent == recipient.agent
            && v.device == recipient.device
    };
    let matches = |user: &&&Node| {
        user.get_attr("jid").is_some_and(|jid| {
            // APK 37.71 B7H.A06 preserves account category and device.
            // Do not match a hosted/agent address to an ordinary device by
            // flattening it. Duplicate-row rejection below is a local guard.
            jid.as_jid().is_some_and(same_address)
                || jid
                    .as_text()
                    .is_some_and(|v| Jid::parse(v).is_ok_and(|v| same_address(&v)))
        })
    };
    let users: Vec<&Node> = list
        .child_nodes()
        .iter()
        .filter(|n| n.description == "user")
        .collect();
    let mut selected = users.iter().filter(matches);
    let user = *selected
        .next()
        .ok_or_else(|| invalid("key reply carries no bundle for this device"))?;
    if selected.next().is_some() {
        return Err(invalid(
            "key reply carries duplicate bundles for this device",
        ));
    }
    if user.child("error").is_some() {
        return Err(invalid("server refused recipient key lookup"));
    }
    Ok(user)
}

pub(crate) fn classic_bundle_from_user(user: &Node, recipient: &Jid) -> Result<PreKeyBundle> {
    // APK 37.71 Bot.A03 00e8..00f0 defaults only an absent type to 5.
    // Present fields still use exact singleton/width/type checks; see
    // bulk-prekey-processing.md. Signature verification remains a separate unchanged step.
    if user.child("type").is_some() && data(child(user, "type")?)? != [5] {
        return Err(invalid("unsupported identity key type"));
    }
    let identity = IdentityKey::new(public(child(user, "identity")?)?);
    let signed = child(user, "skey")?;
    let signature = data(child(signed, "signature")?)?.to_vec();
    if signature.len() != 64 {
        return Err(invalid("invalid signed-key signature width"));
    }
    let prekey = user
        .child("key")
        .map(|_| -> Result<_> {
            let key = child(user, "key")?;
            Ok((
                number(child(key, "id")?, 3)?.into(),
                public(child(key, "value")?)?,
            ))
        })
        .transpose()?;
    PreKeyBundle::new(
        number(child(user, "registration")?, 4)?,
        DeviceId::new(recipient.device.into()),
        prekey,
        number(child(signed, "id")?, 3)?.into(),
        public(child(signed, "value")?)?,
        signature,
        identity,
    )
    .map_err(crypto)
}

struct Identities {
    local: IdentityKeyPair,
    registration: u32,
    peer: Option<IdentityKey>,
    address: ProtocolAddress,
}
#[async_trait]
impl IdentityKeyStore for Identities {
    async fn get_identity_key_pair(&self) -> signal::error::Result<IdentityKeyPair> {
        Ok(self.local.clone())
    }
    async fn get_local_registration_id(&self) -> signal::error::Result<u32> {
        Ok(self.registration)
    }
    async fn save_identity(
        &mut self,
        address: &ProtocolAddress,
        identity: &IdentityKey,
    ) -> signal::error::Result<IdentityChange> {
        if address != &self.address || self.peer.as_ref().is_some_and(|v| v != identity) {
            return Err(SignalProtocolError::UntrustedIdentity(address.clone()));
        }
        self.peer = Some(*identity);
        Ok(IdentityChange::NewOrUnchanged)
    }
    async fn is_trusted_identity(
        &self,
        address: &ProtocolAddress,
        identity: &IdentityKey,
        _: Direction,
    ) -> signal::error::Result<bool> {
        Ok(address == &self.address && self.peer.as_ref().is_none_or(|v| v == identity))
    }
    async fn get_identity(
        &self,
        address: &ProtocolAddress,
    ) -> signal::error::Result<Option<IdentityKey>> {
        if address != &self.address {
            return Err(SignalProtocolError::UntrustedIdentity(address.clone()));
        }
        Ok(self.peer)
    }
}
struct Sessions {
    record: Option<SessionRecord>,
    address: ProtocolAddress,
}
#[async_trait]
impl SessionStore for Sessions {
    async fn load_session(
        &self,
        address: &ProtocolAddress,
    ) -> signal::error::Result<Option<SessionRecord>> {
        if address != &self.address {
            return Err(SignalProtocolError::SessionNotFound(address.clone()));
        }
        Ok(self.record.clone())
    }
    async fn has_session(&self, address: &ProtocolAddress) -> signal::error::Result<bool> {
        Ok(address == &self.address && self.record.is_some())
    }
    async fn store_session(
        &mut self,
        address: &ProtocolAddress,
        record: SessionRecord,
    ) -> signal::error::Result<()> {
        if address != &self.address {
            return Err(SignalProtocolError::SessionNotFound(address.clone()));
        }
        self.record = Some(record);
        Ok(())
    }
}

/// Prepared encryption plus updated durable session and peer identity.
pub struct PreparedDirect {
    /// Opaque encrypted Signal message, not plaintext.
    pub ciphertext: Vec<u8>,
    /// `pkmsg` for first use, `msg` for an established session.
    pub kind: &'static str,
    /// Secret session serialization, must commit before transmission.
    pub session: Zeroizing<Vec<u8>>,
    /// Public peer identity, pin on first use and refuse changes.
    pub identity: Vec<u8>,
}
impl std::fmt::Debug for PreparedDirect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreparedDirect")
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}

/// Prepare one text message without mutating durable state or doing network I/O.
/// Supplied bundles are authenticated; only a new session consumes their keys.
/// An existing session is never reset.
pub async fn prepare(
    account: &AccountRecord,
    recipient: &Jid,
    bundle: &PreKeyBundle,
    previous_session: Option<&[u8]>,
    previous_identity: Option<&[u8]>,
    text: &str,
) -> Result<PreparedDirect> {
    if text.is_empty() || text.len() > 4096 || !account.is_registered() {
        return Err(invalid("invalid message/account"));
    }
    let plain = text_payload(text)?;
    prepare_payload(
        account,
        recipient,
        Some(bundle),
        previous_session,
        previous_identity,
        &plain,
    )
    .await
}

pub(crate) fn text_payload(text: &str) -> Result<Zeroizing<Vec<u8>>> {
    if text.is_empty() || text.len() > 4096 {
        return Err(invalid("invalid text payload"));
    }
    #[derive(prost::Message)]
    struct TextMessage {
        #[prost(string, tag = "1")]
        conversation: String,
    }
    Ok(Zeroizing::new(
        TextMessage {
            conversation: text.into(),
        }
        .encode_to_vec(),
    ))
}

/// Verify a fetched signed device key before encryption or identity admission.
pub(crate) fn validate_bundle_signature(bundle: &PreKeyBundle) -> Result<()> {
    // APK 2.26.37.71 CQc.A00 0030..0064 checks this signature before
    // session derivation/save. A fetched identity signing itself is not human
    // verification; the directory response must arrive over authenticated Noise.
    if !bundle
        .identity_key()
        .map_err(crypto)?
        .public_key()
        .verify_signature(
            &bundle.signed_pre_key_public().map_err(crypto)?.serialize(),
            bundle.signed_pre_key_signature().map_err(crypto)?,
        )
    {
        return Err(invalid("invalid signed directory bundle signature"));
    }
    Ok(())
}

/// Refuse a fetched identity mismatch even when a saved session can encrypt.
/// No session, identity or ratchet is changed by this check.
pub(crate) fn validate_bundle_identity(
    bundle: Option<&PreKeyBundle>,
    previous_identity: Option<&[u8]>,
    recipient: &Jid,
) -> Result<()> {
    // APK 2.26.37.71 0jc.A0L compares serialized identity type/key bytes
    // before using a held session. Replacement requires separate trust policy.
    if let (Some(bundle), Some(pinned)) = (bundle, previous_identity) {
        let pinned = IdentityKey::decode(pinned).map_err(crypto)?;
        if bundle.identity_key().map_err(crypto)? != &pinned {
            return Err(Error::IdentityChanged(
                crate::model::ids::DeviceAddress::from_jid(recipient)?,
            ));
        }
    }
    Ok(())
}

/// Stage a classic same-identity bundle without constructing a chat ciphertext.
/// This is a cryptographic primitive, not native CPB admission or a store write.
#[cfg(all(feature = "experimental-noise", feature = "experimental-transport"))]
pub(crate) async fn prepare_session_refresh(
    account: &AccountRecord,
    recipient: &Jid,
    bundle: &PreKeyBundle,
    previous_session: Option<&[u8]>,
    previous_identity: &[u8],
) -> Result<Zeroizing<Vec<u8>>> {
    if !account.is_registered() {
        return Err(invalid("unregistered refresh account"));
    }
    validate_bundle_identity(Some(bundle), Some(previous_identity), recipient)?;
    validate_bundle_signature(bundle)?;
    let key = account
        .signal_identity
        .as_ref()
        .ok_or_else(|| invalid("missing saved Signal identity"))?;
    let public = PublicKey::from_djb_public_key_bytes(&key.0.public)
        .map_err(|_| invalid("invalid local public key"))?;
    let private = PrivateKey::deserialize(&key.0.private)
        .map_err(|_| invalid("invalid local private key"))?;
    let address = ProtocolAddress::new(&recipient.user, DeviceId::new(recipient.device.into()));
    let pin = IdentityKey::decode(previous_identity).map_err(crypto)?;
    let mut identities = Identities {
        local: IdentityKeyPair::new(IdentityKey::new(public), private),
        registration: account
            .signal_registration_id
            .ok_or_else(|| invalid("missing local registration id"))?,
        peer: Some(pin),
        address: address.clone(),
    };
    let record = previous_session
        .map(SessionRecord::deserialize)
        .transpose()
        .map_err(crypto)?;
    if let Some(record) = &record {
        let remote = record
            .session_state()
            .ok_or_else(|| invalid("saved session has no current state"))?
            .remote_identity_key()
            .map_err(|_| invalid("invalid saved session identity"))?;
        if remote.as_ref() != Some(&pin) {
            return Err(invalid("saved session identity does not match pin"));
        }
    }
    let mut sessions = Sessions {
        record,
        address: address.clone(),
    };
    // 37.71 0jc.A0L matching-key branch 0086..009c still reaches CQc.A00
    // 0310..0324. CQc archives a held current state at 0124..014c before
    // deriving another one. Unlike ordinary encryption, always process the
    // validated classic bundle here; the backend retains the previous receiving
    // state. Native admission remains the caller's separate responsibility.
    process_prekey_bundle(
        &address,
        &mut sessions,
        &mut identities,
        bundle,
        &mut task_rng(),
        UsePQRatchet::No,
    )
    .await
    .map_err(crypto)?;
    Ok(Zeroizing::new(
        sessions
            .record
            .ok_or_else(|| invalid("missing staged session"))?
            .serialize()
            .map_err(crypto)?,
    ))
}

/// Encrypt a bounded, already encoded Message protobuf with the saved session.
/// Used for non-text payloads; retains the same identity and transaction rules.
pub async fn prepare_payload(
    account: &AccountRecord,
    recipient: &Jid,
    bundle: Option<&PreKeyBundle>,
    previous_session: Option<&[u8]>,
    previous_identity: Option<&[u8]>,
    payload: &[u8],
) -> Result<PreparedDirect> {
    if payload.is_empty() || payload.len() > 32768 || !account.is_registered() {
        return Err(invalid("invalid payload/account"));
    }
    let key = account
        .signal_identity
        .as_ref()
        .ok_or_else(|| invalid("missing saved Signal identity"))?;
    let public = PublicKey::from_djb_public_key_bytes(&key.0.public)
        .map_err(|_| invalid("invalid local public key"))?;
    let private = PrivateKey::deserialize(&key.0.private)
        .map_err(|_| invalid("invalid local private key"))?;
    let local = IdentityKeyPair::new(IdentityKey::new(public), private);
    let address = ProtocolAddress::new(&recipient.user, DeviceId::new(recipient.device.into()));
    validate_bundle_identity(bundle, previous_identity, recipient)?;
    // CQc.A00 (APK 2.26.37.71, 0030..0064) verifies signed keys before
    // session construction. Do not silently accept a supplied unverified bundle
    // just because this local path can encrypt with an existing ratchet.
    if let Some(bundle) = bundle {
        validate_bundle_signature(bundle)?;
    }
    let pinned = previous_identity
        .map(IdentityKey::decode)
        .transpose()
        .map_err(crypto)?;
    let mut identities = Identities {
        local,
        registration: account
            .signal_registration_id
            .ok_or_else(|| invalid("missing local registration id"))?,
        peer: pinned,
        address: address.clone(),
    };
    let mut sessions = Sessions {
        record: previous_session
            .map(SessionRecord::deserialize)
            .transpose()
            .map_err(crypto)?,
        address: address.clone(),
    };
    if let Some(record) = sessions.record.as_ref()
        && let Some(state) = record.session_state()
        && state.session_version().map_err(|e| crypto(e.into()))? == 4
    {
        let pinned = identities
            .peer
            .as_ref()
            .ok_or_else(|| invalid("missing pinned hybrid peer identity"))?;
        if state
            .remote_identity_key()
            .map_err(|e| crypto(e.into()))?
            .as_ref()
            != Some(pinned)
        {
            return Err(Error::IdentityChanged(
                crate::model::ids::DeviceAddress::from_jid(recipient)?,
            ));
        }
        if state.local_identity_key().map_err(|e| crypto(e.into()))?
            != *identities.local.identity_key()
            || state.local_registration_id() != identities.registration
            || identities
                .local
                .private_key()
                .public_key()
                .map_err(|e| crypto(e.into()))?
                != *identities.local.public_key()
        {
            return Err(invalid("held hybrid session does not match saved account"));
        }
        if state
            .sender_ratchet_private_key()
            .map_err(|e| crypto(e.into()))?
            .public_key()
            .map_err(|e| crypto(e.into()))?
            != state.sender_ratchet_key().map_err(|e| crypto(e.into()))?
        {
            return Err(invalid("invalid held hybrid ratchet key pair"));
        }
        return super::hybrid::HybridSession::from_record(
            sessions.record.take().expect("held record"),
        )?
        .prepare(payload);
    }
    if sessions.record.is_none() {
        // A bundle is needed only to establish a session. A fan-out asks for
        // bundles for the devices it has no session with, so requiring one for
        // every device would force a pointless query per established device.
        let bundle = bundle.ok_or_else(|| invalid("no session and no prekey bundle"))?;
        process_prekey_bundle(
            &address,
            &mut sessions,
            &mut identities,
            bundle,
            &mut task_rng(),
            UsePQRatchet::No,
        )
        .await
        .map_err(crypto)?;
    }
    let mut plain = Zeroizing::new(payload.to_vec());
    // LX/Cb1.A00: random count 1..16, repeated count byte appended.
    let mut random = [0; 1];
    getrandom::fill(&mut random).map_err(|_| invalid("padding entropy unavailable"))?;
    let padding = (random[0] & 15) + 1;
    let padded_len = plain.len() + padding as usize;
    plain.resize(padded_len, padding);
    let encrypted = message_encrypt(&plain, &address, &mut sessions, &mut identities)
        .await
        .map_err(crypto)?;
    let kind = match encrypted.message_type() {
        CiphertextMessageType::PreKey => "pkmsg",
        CiphertextMessageType::Whisper => "msg",
        _ => return Err(invalid("unsupported ciphertext type")),
    };
    Ok(PreparedDirect {
        ciphertext: encrypted.serialize().to_vec(),
        kind,
        session: Zeroizing::new(
            sessions
                .record
                .ok_or_else(|| invalid("missing updated session"))?
                .serialize()
                .map_err(crypto)?,
        ),
        identity: identities
            .peer
            .ok_or_else(|| invalid("missing pinned peer identity"))?
            .serialize()
            .to_vec(),
    })
}

/// Decrypted padded protobuf plus the advanced session. Commit both atomically.
pub struct ReceivedDirect {
    /// Authenticated message protobuf, with APK padding removed.
    pub plaintext: Zeroizing<Vec<u8>>,
    /// Updated receiving ratchet; never discard after committing plaintext.
    pub session: Zeroizing<Vec<u8>>,
}
impl std::fmt::Debug for ReceivedDirect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ReceivedDirect(<redacted>)")
    }
}

/// Decrypt a `msg` reply using an existing pinned primary-device session.
/// No prekey fallback, identity replacement, or durable mutation is performed.
pub async fn receive(
    account: &AccountRecord,
    sender: &Jid,
    previous_session: &[u8],
    previous_identity: &[u8],
    ciphertext: &[u8],
) -> Result<ReceivedDirect> {
    if !account.is_registered() || ciphertext.len() > 65536 {
        return Err(invalid("invalid receive account or size"));
    }
    let key = account
        .signal_identity
        .as_ref()
        .ok_or_else(|| invalid("missing saved Signal identity"))?;
    let public = PublicKey::from_djb_public_key_bytes(&key.0.public)
        .map_err(|_| invalid("invalid local public key"))?;
    let private = PrivateKey::deserialize(&key.0.private)
        .map_err(|_| invalid("invalid local private key"))?;
    let address = ProtocolAddress::new(&sender.user, DeviceId::new(sender.device.into()));
    let mut identities = Identities {
        local: IdentityKeyPair::new(IdentityKey::new(public), private),
        registration: account
            .signal_registration_id
            .ok_or_else(|| invalid("missing registration id"))?,
        peer: Some(IdentityKey::decode(previous_identity).map_err(crypto)?),
        address: address.clone(),
    };
    let mut sessions = Sessions {
        record: Some(SessionRecord::deserialize(previous_session).map_err(crypto)?),
        address: address.clone(),
    };
    let message = signal::SignalMessage::try_from(ciphertext).map_err(crypto)?;
    let decrypted = signal::message_decrypt_signal(
        &message,
        &address,
        &mut sessions,
        &mut identities,
        &mut task_rng(),
    )
    .await
    .map_err(crypto)?;
    let mut plaintext = Zeroizing::new(decrypted.plaintext);
    let padding = plaintext.last().copied().unwrap_or(0) as usize;
    if !(1..=16).contains(&padding)
        || padding > plaintext.len()
        || !plaintext[plaintext.len() - padding..]
            .iter()
            .all(|v| *v as usize == padding)
    {
        return Err(invalid("invalid authenticated message padding"));
    }
    let unpadded_len = plaintext.len() - padding;
    plaintext.truncate(unpadded_len);
    Ok(ReceivedDirect {
        plaintext,
        session: Zeroizing::new(
            sessions
                .record
                .ok_or_else(|| invalid("missing receiving session"))?
                .serialize()
                .map_err(crypto)?,
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn crypto_diagnostics_classify_without_address_or_backend_details() {
        let address = ProtocolAddress::new("private-peer-marker", DeviceId::new(0));
        for (error, expected) in [
            (
                SignalProtocolError::UntrustedIdentity(address.clone()),
                "identity untrusted",
            ),
            (
                SignalProtocolError::SessionNotFound(address),
                "session not found",
            ),
            (SignalProtocolError::InvalidPreKeyId, "one-time prekey"),
            (SignalProtocolError::InvalidSignedPreKeyId, "signed prekey"),
            (
                SignalProtocolError::DuplicatedMessage(123, 456),
                "old message counter",
            ),
            (
                SignalProtocolError::InvalidArgument("private-peer-marker".into()),
                "operation failed",
            ),
            (
                SignalProtocolError::BackendError(
                    "private-peer-marker",
                    Box::new(std::io::Error::other("private-peer-marker")),
                ),
                "backend operation failed",
            ),
        ] {
            let error = crypto(error);
            let Error::InvalidInput { field, reason } = error else {
                panic!("same typed decryption classification")
            };
            assert_eq!(field, "direct_message");
            assert!(reason.contains(expected));
            assert!(!reason.contains("private-peer-marker"));
            assert!(!reason.contains("123"));
            assert!(!reason.contains("456"));
        }
    }
    #[cfg(all(feature = "experimental-noise", feature = "experimental-transport"))]
    #[tokio::test]
    async fn receive_process_exit_fixture() {
        use crate::{
            model::ids::AccountId,
            store::{MasterKey, OpenOptions, ProtocolStore, StaticKeySource, Table},
            transport::inbox::{self, IncomingResult},
        };
        let Some(path) = std::env::var_os("TELEWHATSAPP_RECEIVE_CRASH_FIXTURE") else {
            return;
        };
        let store = ProtocolStore::open(OpenOptions {
            path: path.into(),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([2; 32]))),
            create: false,
        })
        .unwrap();
        let account: AccountRecord = store
            .read(|tx| tx.get_json(Table::Account, b"crash-fixture-account", 1))
            .unwrap()
            .unwrap();
        let capture: Vec<u8> = store
            .read(|tx| tx.get_json(Table::Checkpoints, b"crash-fixture-capture", 1))
            .unwrap()
            .unwrap();
        let wire: Vec<u8> = store
            .read(|tx| tx.get_json(Table::Checkpoints, &capture, 1))
            .unwrap()
            .unwrap();
        let limits = crate::profile::Limits::default();
        let raw = crate::binary::unpack_frame_payload(&wire, &limits).unwrap();
        let node = crate::binary::Decoder::new(
            &raw,
            crate::binary::TokenDictionary::comparison_2_26_37_71(),
            &limits,
        )
        .decode_node()
        .unwrap();
        let stage = std::env::var("TELEWHATSAPP_RECEIVE_CRASH_STAGE").unwrap();
        if stage == "captured" {
            std::process::exit(80);
        }
        assert_eq!(stage, "committed");
        assert_eq!(
            inbox::process(&store, &account, &node, &capture)
                .await
                .unwrap(),
            IncomingResult::Decrypted
        );
        // No socket/receipt is written; destructors do not run.
        std::process::exit(81);
    }
    #[test]
    fn payload_inspection_distinguishes_text_and_control_without_values() {
        // Comparison BXF: extendedTextMessage=6, protocolMessage=12;
        // BXE: type=2. Hand-written protobuf fixtures, no live message data.
        let text = b"\x32\x08\x0a\x06secret";
        let summary = payload_summary(text).unwrap();
        assert!(summary.contains("extended-text fields=[1]"));
        assert!(!summary.contains("secret"));
        let image = payload_summary(b"\x1a\x08\x0a\x06secret").unwrap();
        assert!(image.contains("image fields=[1]"));
        assert!(!image.contains("secret"));
        let control = payload_summary(b"\x62\x02\x10\x04").unwrap();
        assert!(control.contains("protocol type=Some(4)"));
        assert!(!control.contains("extended-text"));
        assert!(payload_summary(b"\x62\x05\x10").is_err());
        assert!(payload_fields(&vec![0; 65537]).is_err());
        assert_eq!(payload_fields(b"\x98\x06\x01").unwrap(), vec![99]);
    }
    #[test]
    fn device_sent_inspection_retains_presence_without_destination_or_payload_values() {
        // Independent wire encoding: Message.31 -> BUs{1 destination, 2 Message, 3 phash}.
        let full = b"\xfa\x01\x1a\x0a\x051@lid\x12\x08\x0a\x06secret\x1a\x07private";
        let summary = payload_summary(full).unwrap();
        assert!(summary.contains(
            "device-sent destination_present=true message_present=true phash_present=true"
        ));
        assert!(summary.contains("[protobuf fields=[1]]"));
        assert!(summary.contains("admission not checked"));
        for value in ["1@lid", "secret", "private"] {
            assert!(!summary.contains(value));
        }
        let absent = payload_summary(b"\xfa\x01\x00").unwrap();
        assert!(
            absent.contains("destination_present=false message_present=false phash_present=false")
        );
        let empty = payload_summary(b"\xfa\x01\x06\x0a\x00\x12\x00\x1a\x00").unwrap();
        assert!(empty.contains("destination_present=true message_present=true phash_present=true"));
        assert!(payload_summary(b"\xfa\x01\x03\x0a\x01\xff").is_err());
        assert!(payload_summary(b"\xfa\x01\x03\x12\x01\x0a").is_err());
        let mut nested = Vec::new();
        for _ in 0..6 {
            let mut wrapper = vec![0x12, nested.len() as u8];
            wrapper.extend(nested);
            nested = vec![0xfa, 0x01, wrapper.len() as u8];
            nested.extend(wrapper);
        }
        assert!(payload_summary(&nested).is_err());
    }

    #[test]
    fn history_notice_inspection_is_structural_and_redacted() {
        // Independently encoded BZc field 102 (wire key b2 06), containing
        // BV6 field 2. The metadata values are intentionally opaque here.
        let notice = b"\xb2\x06\x0a\x12\x08\x0a\x06secret";
        let summary = payload_summary(notice).unwrap();
        assert!(summary.contains("protobuf fields=[102]"));
        assert!(summary.contains("message-history-notice fields=[2]"));
        assert!(summary.contains("history not applied"));
        assert!(!summary.contains("secret"));
        assert!(summary.contains("metadata=unsupported"));
        let valid = payload_summary(b"\xb2\x06\x0d\x12\x0b\x0a\x051@lid\x10\x0a\x18\x02").unwrap();
        assert!(valid.contains("metadata=validated; receivers=1 non_receivers=0"));
        assert!(!valid.contains("1@lid"));
        let mut mixed = b"\xb2\x06\x0d\x12\x0b\x0a\x051@lid\x10\x0a\x18\x02".to_vec();
        mixed.extend_from_slice(b"\x0a\x06secret");
        let summary = payload_summary(&mixed).unwrap();
        assert!(summary.contains("metadata=unsupported"));
        assert!(!summary.contains("secret"));
        assert!(payload_summary(b"\xb2\x06\x01\x12").is_err());
        assert!(
            payload_summary(b"\xb2\x06\x04\xfa\x07\x01\x00")
                .unwrap()
                .contains("message-history-notice fields=[127]")
        );
    }
    use crate::session::{KeyMaterial, RegistrationStatus, SignalIdentityKey};
    use signal::{
        GenericSignedPreKey, KeyPair, PreKeyId, PreKeyRecord, PreKeySignalMessage, PreKeyStore,
        SignedPreKeyId, SignedPreKeyRecord, SignedPreKeyStore, message_decrypt_prekey,
    };

    struct Keys {
        one: PreKeyRecord,
        signed: SignedPreKeyRecord,
    }
    #[async_trait]
    impl PreKeyStore for Keys {
        async fn get_pre_key(&self, id: PreKeyId) -> signal::error::Result<PreKeyRecord> {
            assert_eq!(u32::from(id), 8);
            Ok(self.one.clone())
        }
        async fn save_pre_key(
            &mut self,
            _: PreKeyId,
            record: &PreKeyRecord,
        ) -> signal::error::Result<()> {
            self.one = record.clone();
            Ok(())
        }
        async fn remove_pre_key(&mut self, _: PreKeyId) -> signal::error::Result<()> {
            Ok(())
        }
    }
    #[async_trait]
    impl SignedPreKeyStore for Keys {
        async fn get_signed_pre_key(
            &self,
            id: SignedPreKeyId,
        ) -> signal::error::Result<SignedPreKeyRecord> {
            assert_eq!(u32::from(id), 7);
            Ok(self.signed.clone())
        }
        async fn save_signed_pre_key(
            &mut self,
            _: SignedPreKeyId,
            record: &SignedPreKeyRecord,
        ) -> signal::error::Result<()> {
            self.signed = record.clone();
            Ok(())
        }
    }

    #[test]
    fn key_bundle_absent_type_defaults_without_relaxing_present_type_or_signature() {
        let identity = IdentityKeyPair::generate(&mut rand::rng());
        let signed = KeyPair::generate(&mut rand::rng());
        let signature = identity
            .private_key()
            .calculate_signature(&signed.public_key.serialize(), &mut rand::rng())
            .unwrap();
        let peer = Jid::parse("15555550100@s.whatsapp.net").unwrap();
        let fields = vec![
            Node::new("registration").bytes(vec![0, 0, 0, 9]),
            Node::new("identity").bytes(identity.public_key().public_key_bytes().to_vec()),
            Node::new("skey").children(vec![
                Node::new("id").bytes(vec![0, 0, 7]),
                Node::new("value").bytes(signed.public_key.public_key_bytes().to_vec()),
                Node::new("signature").bytes(signature.to_vec()),
            ]),
        ];
        let reply = |extra: Vec<Node>| {
            let mut all = fields.clone();
            all.extend(extra);
            Node::new("iq")
                .attr("type", "result")
                .children(vec![Node::new("list").children(vec![
                    Node::new("user").attr("jid", peer.clone()).children(all),
                ])])
        };
        // APK 37.71 Bot.A03 00e8..00f0 defaults only an absent type to 5.
        let bundle = prekey_bundle_for(&reply(vec![]), &peer).unwrap();
        assert_eq!(bundle.identity_key().unwrap(), identity.identity_key());
        validate_bundle_signature(&bundle).unwrap();
        for pq in [
            Node::new("pqkey"),
            Node::new("pqkey").children(vec![
                Node::new("id").bytes(vec![0, 0, 8]),
                Node::new("value").bytes(vec![8; 1569]),
                Node::new("signature").bytes(vec![0; 64]),
            ]),
        ] {
            let pq_reply = reply(vec![pq]);
            let error = prekey_bundle_for(&pq_reply, &peer)
                .err()
                .expect("PQ input must not silently become a classic session bundle");
            assert!(
                error
                    .to_string()
                    .contains("post-quantum session processing is unsupported")
            );
            let metadata = classic_bundle_metadata_for(&pq_reply, &peer).unwrap();
            validate_bundle_signature(&metadata).unwrap();
            assert_eq!(
                metadata.identity_key().unwrap(),
                bundle.identity_key().unwrap()
            );
            let mut unselected = reply(vec![]);
            let NodeContent::Children(lists) = &mut unselected.content else {
                panic!("fixture")
            };
            let NodeContent::Children(users) = &mut lists[0].content else {
                panic!("fixture")
            };
            let NodeContent::Children(pq_lists) = &pq_reply.content else {
                panic!("fixture")
            };
            let NodeContent::Children(pq_users) = &pq_lists[0].content else {
                panic!("fixture")
            };
            users.push(pq_users[0].clone().attr("jid", peer.with_device(3)));
            validate_bundle_signature(&prekey_bundle_for(&unselected, &peer).unwrap()).unwrap();
            assert!(prekey_bundle_for(&unselected, &peer.with_device(3)).is_err());
        }
        let explicit =
            prekey_bundle_for(&reply(vec![Node::new("type").bytes(vec![5])]), &peer).unwrap();
        assert_eq!(
            explicit.identity_key().unwrap(),
            bundle.identity_key().unwrap()
        );
        validate_bundle_signature(&explicit).unwrap();
        for extra in [
            vec![Node::new("type").bytes(vec![])],
            vec![Node::new("type").bytes(vec![5, 5])],
            vec![Node::new("type").bytes(vec![6])],
            vec![Node::new("type").children(vec![Node::new("private-type-canary")])],
            vec![
                Node::new("type").bytes(vec![5]),
                Node::new("type").bytes(vec![5]),
            ],
        ] {
            let error = prekey_bundle_for(&reply(extra), &peer).err().unwrap();
            assert!(!error.to_string().contains(&peer.user));
            assert!(!error.to_string().contains("private-type-canary"));
        }
        let mut forged = reply(vec![]);
        let NodeContent::Children(lists) = &mut forged.content else {
            panic!("fixture")
        };
        let NodeContent::Children(users) = &mut lists[0].content else {
            panic!("fixture")
        };
        let NodeContent::Children(fields) = &mut users[0].content else {
            panic!("fixture")
        };
        let skey = fields.iter_mut().find(|n| n.description == "skey").unwrap();
        let NodeContent::Children(signed_fields) = &mut skey.content else {
            panic!("fixture")
        };
        signed_fields
            .iter_mut()
            .find(|n| n.description == "signature")
            .unwrap()
            .content = NodeContent::Bytes(vec![0; 64].into());
        let bundle = prekey_bundle_for(&forged, &peer).unwrap();
        assert!(validate_bundle_signature(&bundle).is_err());
    }

    /// A fan-out asks for several devices in one query, so the reply carries
    /// several `<user>` elements and the parser has to pick the right one.
    /// Picking the first would pair a device with another device's identity
    /// key, which encrypts to the wrong peer without failing.
    #[test]
    fn prekey_bundle_for_selects_the_requested_device_among_many() {
        use wacore_libsignal::protocol::KeyPair;
        let mut rng = rand::rng();
        let user = |jid: &str, identity: &IdentityKeyPair, errored: bool| {
            let signed = KeyPair::generate(&mut rand::rng());
            let signature = identity
                .private_key()
                .calculate_signature(&signed.public_key.serialize(), &mut rand::rng())
                .unwrap();
            let mut children = vec![
                Node::new("registration").bytes(vec![0, 0, 0, 9]),
                Node::new("type").bytes(vec![5]),
                Node::new("identity").bytes(identity.public_key().public_key_bytes().to_vec()),
                Node::new("skey").children(vec![
                    Node::new("id").bytes(vec![0, 0, 7]),
                    Node::new("value").bytes(signed.public_key.public_key_bytes().to_vec()),
                    Node::new("signature").bytes(signature.to_vec()),
                ]),
            ];
            if errored {
                children.push(Node::new("error").attr("code", "480"));
            }
            Node::new("user")
                .attr("jid", Jid::parse(jid).unwrap())
                .children(children)
        };
        let first = IdentityKeyPair::generate(&mut rng);
        let second = IdentityKeyPair::generate(&mut rng);
        let reply =
            Node::new("iq")
                .attr("type", "result")
                .children(vec![Node::new("list").children(vec![
                    user("222222222222222@lid", &first, false),
                    user("222222222222222:3@lid", &second, false),
                ])]);
        let primary =
            prekey_bundle_for(&reply, &Jid::parse("222222222222222@lid").unwrap()).unwrap();
        let companion =
            prekey_bundle_for(&reply, &Jid::parse("222222222222222:3@lid").unwrap()).unwrap();
        assert_eq!(primary.identity_key().unwrap(), first.identity_key());
        assert_eq!(companion.identity_key().unwrap(), second.identity_key());
        assert_ne!(primary.identity_key().unwrap(), second.identity_key());
        // A device the reply does not cover, and a per-user refusal, are both
        // failures rather than a silently wrong bundle.
        assert!(prekey_bundle_for(&reply, &Jid::parse("222222222222222:9@lid").unwrap()).is_err());
        let refused = Node::new("iq").attr("type", "result").children(vec![
            Node::new("list").children(vec![user("222222222222222@lid", &first, true)]),
        ]);
        assert!(prekey_bundle_for(&refused, &Jid::parse("222222222222222@lid").unwrap()).is_err());

        let peer = Jid::parse("222222222222222:3@lid").unwrap();
        let exact = user("222222222222222:3@lid", &second, false);
        // Wire encoding kind is not a different account/device identity.
        let mut different_encoding = peer.clone();
        different_encoding.kind = crate::model::jid::JidKind::Plain;
        let equivalent = Node::new("iq").attr("type", "result").children(vec![
            Node::new("list").children(vec![exact.clone().attr("jid", different_encoding)]),
        ]);
        assert_eq!(
            prekey_bundle_for(&equivalent, &peer)
                .unwrap()
                .identity_key()
                .unwrap(),
            second.identity_key()
        );
        let mut foreign = peer.clone();
        foreign.agent = 1;
        for text in [false, true] {
            let agent = exact.clone().attr(
                "jid",
                if text {
                    crate::binary::AttributeValue::Text(foreign.to_string())
                } else {
                    crate::binary::AttributeValue::Jid(foreign.clone())
                },
            );
            let agent_only = Node::new("iq")
                .attr("type", "result")
                .children(vec![Node::new("list").children(vec![agent.clone()])]);
            assert!(prekey_bundle_for(&agent_only, &peer).is_err());
            let mixed = Node::new("iq")
                .attr("type", "result")
                .children(vec![Node::new("list").children(vec![agent, exact.clone()])]);
            assert_eq!(
                prekey_bundle_for(&mixed, &peer)
                    .unwrap()
                    .identity_key()
                    .unwrap(),
                second.identity_key()
            );
        }
        for duplicate in [
            exact.clone(),
            user("222222222222222:3@lid", &first, false),
            exact.clone().attr("jid", peer.to_string()),
        ] {
            let ambiguous = Node::new("iq").attr("type", "result").children(vec![
                Node::new("list").children(vec![exact.clone(), duplicate]),
            ]);
            let error = prekey_bundle_for(&ambiguous, &peer)
                .err()
                .unwrap()
                .to_string();
            assert!(error.contains("duplicate bundles"));
            assert!(!error.contains(&peer.user));
        }
        let one = Node::new("key").children(vec![
            Node::new("id").bytes(vec![0, 0, 8]),
            Node::new("value").bytes(first.public_key().public_key_bytes().to_vec()),
        ]);
        let mut children = exact.child_nodes().to_vec();
        children.extend([one.clone(), one]);
        let duplicate_prekey = Node::new("iq").attr("type", "result").children(vec![
            Node::new("list").children(vec![exact.clone().children(children)]),
        ]);
        assert!(
            prekey_bundle_for(&duplicate_prekey, &peer)
                .err()
                .unwrap()
                .to_string()
                .contains("duplicate key bundle field")
        );
        let conflicting_outer_error = Node::new("iq").attr("type", "result").children(vec![
            Node::new("list").children(vec![exact]),
            Node::new("error").attr("code", "503"),
        ]);
        assert!(prekey_bundle_for(&conflicting_outer_error, &peer).is_err());
    }

    #[tokio::test]
    async fn held_session_refuses_forged_same_identity_bundle() {
        let mut rng = rand::rng();
        let alice = IdentityKeyPair::generate(&mut rng);
        let bob = IdentityKeyPair::generate(&mut rng);
        let signed = KeyPair::generate(&mut rng);
        let signature = bob
            .private_key()
            .calculate_signature(&signed.public_key.serialize(), &mut rng)
            .unwrap();
        let bundle = PreKeyBundle::new(
            42,
            DeviceId::new(0),
            None,
            7.into(),
            signed.public_key,
            signature.to_vec(),
            *bob.identity_key(),
        )
        .unwrap();
        let forged = PreKeyBundle::new(
            42,
            DeviceId::new(0),
            None,
            7.into(),
            signed.public_key,
            vec![0; 64],
            *bob.identity_key(),
        )
        .unwrap();
        let account = AccountRecord {
            registration: RegistrationStatus::Registered,
            signal_registration_id: Some(31),
            signal_identity: Some(SignalIdentityKey(KeyMaterial {
                public: alice.public_key().public_key_bytes().try_into().unwrap(),
                private: *alice.private_key().serialize(),
            })),
            ..Default::default()
        };
        let recipient = Jid::parse("15551234567@s.whatsapp.net").unwrap();
        let saved = prepare(&account, &recipient, &bundle, None, None, "first")
            .await
            .unwrap();
        let session_before = saved.session.to_vec();
        let identity_before = saved.identity.clone();
        let staged = prepare_session_refresh(
            &account,
            &recipient,
            &bundle,
            Some(&saved.session),
            &saved.identity,
        )
        .await
        .unwrap();
        let old = SessionRecord::deserialize(&saved.session).unwrap();
        let new = SessionRecord::deserialize(&staged).unwrap();
        assert_eq!(
            new.previous_session_count(),
            old.previous_session_count() + 1
        );
        let original = old.session_state().unwrap();
        let archived = new.previous_session_states().next().unwrap().unwrap();
        assert_eq!(archived.alice_base_key(), original.alice_base_key());
        assert_eq!(
            archived.get_sender_chain_key().unwrap().index(),
            original.get_sender_chain_key().unwrap().index()
        );
        assert_ne!(
            new.session_state().unwrap().alice_base_key(),
            original.alice_base_key()
        );
        assert!(
            prepare_session_refresh(
                &account,
                &recipient,
                &forged,
                Some(&saved.session),
                &saved.identity
            )
            .await
            .is_err()
        );

        let error = prepare(
            &account,
            &recipient,
            &forged,
            Some(&saved.session),
            Some(&saved.identity),
            "second",
        )
        .await
        .expect_err("forged fetched bundle must refuse even with a held session");
        assert!(
            error
                .to_string()
                .contains("invalid signed directory bundle signature")
        );
        assert_eq!(saved.session.as_slice(), session_before);
        assert_eq!(saved.identity, identity_before);
        let valid = prepare(
            &account,
            &recipient,
            &bundle,
            Some(&saved.session),
            Some(&saved.identity),
            "second",
        )
        .await
        .unwrap();
        assert_eq!(valid.identity, saved.identity);
        let without_bundle = prepare_payload(
            &account,
            &recipient,
            None,
            Some(&saved.session),
            Some(&saved.identity),
            &[0x0a, 1, b'x'],
        )
        .await
        .unwrap();
        assert_eq!(without_bundle.identity, saved.identity);
        assert!(
            prepare(&account, &recipient, &forged, None, None, "first")
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn generated_bundle_encrypts_decrypts_and_refuses_changed_identity() {
        let mut rng = rand::rng();
        let alice = IdentityKeyPair::generate(&mut rng);
        let bob = IdentityKeyPair::generate(&mut rng);
        let signed = KeyPair::generate(&mut rng);
        let one = KeyPair::generate(&mut rng);
        let signature = bob
            .private_key()
            .calculate_signature(&signed.public_key.serialize(), &mut rng)
            .unwrap();
        let bundle = PreKeyBundle::new(
            42,
            DeviceId::new(0),
            Some((8.into(), one.public_key)),
            7.into(),
            signed.public_key,
            signature.to_vec(),
            *bob.identity_key(),
        )
        .unwrap();
        let account = AccountRecord {
            registration: RegistrationStatus::Registered,
            signal_registration_id: Some(31),
            signal_identity: Some(SignalIdentityKey(KeyMaterial {
                public: alice.public_key().public_key_bytes().try_into().unwrap(),
                private: *alice.private_key().serialize(),
            })),
            ..Default::default()
        };
        let recipient = Jid::parse("15551234567@s.whatsapp.net").unwrap();
        let encrypted = prepare(&account, &recipient, &bundle, None, None, "test")
            .await
            .unwrap();
        assert_eq!(encrypted.kind, "pkmsg");
        use sha2::Digest;
        let (old, observed) = incoming_prekey_identity_fingerprints(
            &encrypted.ciphertext,
            &bob.identity_key().serialize(),
        )
        .unwrap();
        assert_eq!(
            observed,
            hex::encode(sha2::Sha256::digest(alice.identity_key().serialize()))
        );
        assert_eq!(
            old,
            hex::encode(sha2::Sha256::digest(bob.identity_key().serialize()))
        );
        assert_ne!(old, observed);
        assert!(
            incoming_prekey_identity_matches(
                &encrypted.ciphertext,
                &alice.identity_key().serialize()
            )
            .unwrap()
        );
        assert!(
            !incoming_prekey_identity_matches(
                &encrypted.ciphertext,
                &bob.identity_key().serialize()
            )
            .unwrap()
        );
        assert!(
            incoming_prekey_identity_matches(&vec![0; 65537], &alice.identity_key().serialize())
                .is_err()
        );
        let bad_bundle = PreKeyBundle::new(
            42,
            DeviceId::new(0),
            Some((8.into(), one.public_key)),
            7.into(),
            signed.public_key,
            vec![0; 64],
            *bob.identity_key(),
        )
        .unwrap();
        assert!(
            prepare(&account, &recipient, &bad_bundle, None, None, "test")
                .await
                .is_err()
        );
        let address = ProtocolAddress::new("offline-alice", DeviceId::new(0));
        let mut identities = Identities {
            local: bob,
            registration: 42,
            peer: None,
            address: address.clone(),
        };
        let mut sessions = Sessions {
            record: None,
            address: address.clone(),
        };
        let signed_record = SignedPreKeyRecord::new(
            7.into(),
            signal::Timestamp::from_epoch_millis(0),
            &signed,
            &signature,
        );
        let mut keys = Keys {
            one: PreKeyRecord::new(8.into(), &one),
            signed: signed_record.clone(),
        };
        let signed_keys = Keys {
            one: keys.one.clone(),
            signed: signed_record,
        };
        let message = PreKeySignalMessage::try_from(encrypted.ciphertext.as_slice()).unwrap();
        let bob_account = AccountRecord {
            registration: RegistrationStatus::Registered,
            signal_registration_id: Some(42),
            signal_identity: Some(SignalIdentityKey(KeyMaterial {
                public: identities
                    .local
                    .public_key()
                    .public_key_bytes()
                    .try_into()
                    .unwrap(),
                private: *identities.local.private_key().serialize(),
            })),
            ..Default::default()
        };
        let sender = Jid::parse("15557654321@s.whatsapp.net").unwrap();
        let signed_bytes = signed_keys.signed.serialize().unwrap();
        let one_bytes = keys.one.serialize().unwrap();
        // A valid classic nested MAC must not make ignored KEM fields consumable.
        let mut with_kem = encrypted.ciphertext.clone();
        with_kem.extend_from_slice(&[0x38, 0, 0x42, 0]);
        let metadata = incoming_prekey_kem(&with_kem).unwrap().unwrap();
        assert_eq!(metadata.prekey_id, 0);
        assert!(metadata.ciphertext.is_empty());
        assert_eq!(format!("{metadata:?}"), "IncomingKem(<redacted>)");
        for partial in [&[0x38, 0][..], &[0x42, 0][..]] {
            let mut incomplete = encrypted.ciphertext.clone();
            incomplete.extend_from_slice(partial);
            assert!(incoming_prekey_kem(&incomplete).is_err());
            assert!(incoming_prekey_id(&incomplete).is_err());
            assert!(
                incoming_prekey_identity_matches(&incomplete, &alice.identity_key().serialize())
                    .is_err()
            );
            assert!(
                incoming_prekey_identity_fingerprints(
                    &incomplete,
                    &alice.identity_key().serialize()
                )
                .is_err()
            );
        }
        let mut repeated = with_kem.clone();
        repeated.extend_from_slice(&[0x38, 0x81, 0x02, 0x42, 2, 4, 5]);
        repeated[0] = 0x4f; // Native parsing admits high nibble 4 and ignores the low nibble.
        let metadata = incoming_prekey_kem(&repeated).unwrap().unwrap();
        assert_eq!(metadata.prekey_id, 257);
        assert_eq!(metadata.ciphertext, [4, 5]);
        assert!(
            receive_prekey(
                &bob_account,
                &sender,
                (None, None),
                &signed_bytes,
                Some(&one_bytes),
                &with_kem,
            )
            .await
            .is_err()
        );
        let incoming = receive_prekey(
            &bob_account,
            &sender,
            (None, None),
            &signed_bytes,
            Some(&one_bytes),
            &encrypted.ciphertext,
        )
        .await
        .unwrap();
        assert_eq!(incoming.consumed_prekey, Some(8));
        assert_eq!(&*incoming.received.plaintext, b"\x0a\x04test");
        // Registration can publish a signed prekey without a one-time prekey.
        // This path must work without inventing or consuming a missing OPK.
        let signed_only = PreKeyBundle::new(
            42,
            DeviceId::new(0),
            None,
            7.into(),
            signed.public_key,
            signature.to_vec(),
            *identities.local.identity_key(),
        )
        .unwrap();
        let signed_only_message = prepare(&account, &recipient, &signed_only, None, None, "test")
            .await
            .unwrap();
        let signed_only_incoming = receive_prekey(
            &bob_account,
            &sender,
            (None, None),
            &signed_bytes,
            None,
            &signed_only_message.ciphertext,
        )
        .await
        .unwrap();
        assert_eq!(signed_only_incoming.consumed_prekey, None);
        assert_eq!(&*signed_only_incoming.received.plaintext, b"\x0a\x04test");
        assert!(
            receive_prekey(
                &bob_account,
                &sender,
                (None, None),
                &signed_bytes,
                None,
                &encrypted.ciphertext
            )
            .await
            .is_err()
        );
        let changed = IdentityKeyPair::generate(&mut rng)
            .identity_key()
            .serialize();
        assert!(
            receive_prekey(
                &bob_account,
                &sender,
                (None, Some(&changed)),
                &signed_bytes,
                Some(&one_bytes),
                &encrypted.ciphertext
            )
            .await
            .is_err()
        );
        let accepted_change = receive_prekey_with_policy(
            &bob_account,
            &sender,
            (
                None,
                Some(&changed),
                crate::registration::request::IdentityChangePolicy::ApkAuthenticated,
            ),
            &signed_bytes,
            Some(&one_bytes),
            &encrypted.ciphertext,
        )
        .await
        .unwrap();
        assert_eq!(&*accepted_change.received.plaintext, b"\x0a\x04test");
        assert_eq!(
            accepted_change.identity,
            alice.identity_key().serialize().to_vec()
        );
        assert_eq!(accepted_change.consumed_prekey, Some(8));
        let mut damaged = encrypted.ciphertext.clone();
        // Break the authenticated nested Signal message, preserving its
        // public identity. Permitting a key change must never skip its MAC.
        let envelope = PreKeySignalMessage::try_from(encrypted.ciphertext.as_slice()).unwrap();
        let nested = envelope.message().serialized();
        let start = damaged
            .windows(nested.len())
            .position(|w| w == nested)
            .unwrap();
        damaged[start + nested.len() - 1] ^= 1;
        assert!(
            receive_prekey_with_policy(
                &bob_account,
                &sender,
                (
                    None,
                    Some(&changed),
                    crate::registration::request::IdentityChangePolicy::ApkAuthenticated
                ),
                &signed_bytes,
                Some(&one_bytes),
                &damaged,
            )
            .await
            .is_err()
        );
        let decrypted = message_decrypt_prekey(
            &message,
            &address,
            &mut sessions,
            &mut identities,
            &mut keys,
            &signed_keys,
            &mut rng,
            UsePQRatchet::No,
        )
        .await
        .unwrap();
        assert_eq!(&decrypted.plaintext[..6], b"\x0a\x04test");
        let padding = *decrypted.plaintext.last().unwrap() as usize;
        assert!((1..=16).contains(&padding));
        assert_eq!(decrypted.plaintext.len(), 6 + padding);
        assert!(
            decrypted.plaintext[6..]
                .iter()
                .all(|v| *v as usize == padding)
        );
        let wrong = IdentityKeyPair::generate(&mut rng)
            .identity_key()
            .serialize();
        assert!(
            prepare(&account, &recipient, &bundle, None, Some(&wrong), "test")
                .await
                .is_err()
        );
        let changed_peer = IdentityKeyPair::generate(&mut rng);
        let changed_signature = changed_peer
            .private_key()
            .calculate_signature(&signed.public_key.serialize(), &mut rng)
            .unwrap();
        let changed_bundle = PreKeyBundle::new(
            42,
            DeviceId::new(0),
            Some((8.into(), one.public_key)),
            7.into(),
            signed.public_key,
            changed_signature.to_vec(),
            *changed_peer.identity_key(),
        )
        .unwrap();
        let error = prepare(
            &account,
            &recipient,
            &changed_bundle,
            Some(&encrypted.session),
            Some(&encrypted.identity),
            "stale session",
        )
        .await
        .err()
        .unwrap();
        assert!(matches!(error, Error::IdentityChanged(_)));
        assert!(!error.to_string().contains(&recipient.user));
        // The same original ratchet remains usable with its matching bundle.
        let resumed = prepare(
            &account,
            &recipient,
            &bundle,
            Some(&encrypted.session),
            Some(&encrypted.identity),
            "second",
        )
        .await
        .unwrap();
        assert_ne!(encrypted.ciphertext, resumed.ciphertext);
        assert_ne!(*encrypted.session, *resumed.session);
        let reply = message_encrypt(
            b"\x0a\x05reply\x01",
            &address,
            &mut sessions,
            &mut identities,
        )
        .await
        .unwrap();
        assert_eq!(reply.message_type(), CiphertextMessageType::Whisper);
        #[cfg(all(feature = "experimental-noise", feature = "experimental-transport"))]
        {
            use crate::{
                binary::Node,
                model::ids::{AccountId, EventCursor},
                store::{
                    ConsumerId, MasterKey, OpenOptions, ProtocolStore, StaticKeySource, Table,
                },
                transport::inbox::{self, IncomingResult},
            };
            let dir = tempfile::tempdir().unwrap();
            let options = || OpenOptions {
                path: dir.path().join("ratchet.redb"),
                account_id: AccountId::new("test").unwrap(),
                key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes(
                    [2; 32],
                ))),
                create: true,
            };
            let store = ProtocolStore::open(options()).unwrap();
            let peer_key = format!("comparison-direct:{recipient}");
            // The PN ratchet fallback does not change the actual LID chat key.
            let inbox_key = "received:12345@lid:reply-fixture";
            let mapping = b"authenticated-lid:12345@lid";
            store
                .write(|tx| {
                    tx.put_json(
                        Table::SignalSessions,
                        peer_key.as_bytes(),
                        1,
                        &*resumed.session,
                    )?;
                    tx.put_json(
                        Table::PeerIdentities,
                        peer_key.as_bytes(),
                        1,
                        &encrypted.identity,
                    )?;
                    tx.put_json(
                        Table::Checkpoints,
                        mapping,
                        1,
                        &Jid::parse("15555550101@s.whatsapp.net")?,
                    )
                })
                .unwrap();
            let node = Node::new("message")
                .attr("id", "reply-fixture")
                .attr("from", Jid::parse("12345@lid").unwrap())
                .attr("sender_pn", recipient.clone())
                .children(vec![
                    Node::new("enc")
                        .attr("v", "2")
                        .attr("type", "msg")
                        .bytes(reply.serialize().to_vec()),
                ]);
            let capture = inbox::preserve(&store, &node).unwrap();
            // A late transactional failure after successful decryption must
            // roll back both the advanced session and all inbox/event writes.
            assert!(
                inbox::process(&store, &account, &node, &capture)
                    .await
                    .is_err()
            );
            let unchanged: Vec<u8> = store
                .read(|tx| tx.get_json(Table::SignalSessions, peer_key.as_bytes(), 1))
                .unwrap()
                .unwrap();
            assert_eq!(unchanged, *resumed.session);
            let absent: Option<serde_json::Value> = store
                .read(|tx| tx.get_json(Table::Checkpoints, inbox_key.as_bytes(), 1))
                .unwrap();
            assert!(absent.is_none());
            store
                .write(|tx| tx.delete(Table::Checkpoints, mapping).map(|_| ()))
                .unwrap();
            store
                .write(|tx| {
                    tx.put_json(Table::Account, b"crash-fixture-account", 1, &account)?;
                    tx.put_json(Table::Checkpoints, b"crash-fixture-capture", 1, &capture)
                })
                .unwrap();
            drop(store);
            let crash = |stage: &str, code| {
                let output = std::process::Command::new(std::env::current_exe().unwrap())
                    .args([
                        "--exact",
                        "signal::direct::tests::receive_process_exit_fixture",
                        "--test-threads=1",
                    ])
                    .env(
                        "TELEWHATSAPP_RECEIVE_CRASH_FIXTURE",
                        dir.path().join("ratchet.redb"),
                    )
                    .env("TELEWHATSAPP_RECEIVE_CRASH_STAGE", stage)
                    .output()
                    .unwrap();
                assert_eq!(
                    output.status.code(),
                    Some(code),
                    "receive crash fixture failed"
                );
            };
            crash("captured", 80);
            let store = ProtocolStore::open(options()).unwrap();
            let unadvanced: Vec<u8> = store
                .read(|tx| tx.get_json(Table::SignalSessions, peer_key.as_bytes(), 1))
                .unwrap()
                .unwrap();
            assert_eq!(unadvanced, *resumed.session);
            assert!(
                store
                    .read(|tx| tx.get(Table::Checkpoints, inbox_key.as_bytes(), 1))
                    .unwrap()
                    .is_none()
            );
            assert_eq!(
                store.read(|tx| tx.journal_head()).unwrap(),
                EventCursor::START
            );
            drop(store);
            crash("committed", 81);
            let store = ProtocolStore::open(options()).unwrap();
            let committed: Vec<u8> = store
                .read(|tx| tx.get_json(Table::SignalSessions, peer_key.as_bytes(), 1))
                .unwrap()
                .unwrap();
            assert_ne!(committed, *resumed.session);
            let saved: serde_json::Value = store
                .read(|tx| tx.get_json(Table::Checkpoints, inbox_key.as_bytes(), 1))
                .unwrap()
                .unwrap();
            assert_eq!(saved["status"], "decrypted");
            assert_eq!(
                saved["protobuf"],
                serde_json::json!(b"\x0a\x05reply".to_vec())
            );
            drop(store);
            let store = ProtocolStore::open(options()).unwrap();
            assert_eq!(
                inbox::process(&store, &account, &node, &capture)
                    .await
                    .unwrap(),
                IncomingResult::Duplicate
            );
            let restored: Vec<u8> = store
                .read(|tx| tx.get_json(Table::SignalSessions, peer_key.as_bytes(), 1))
                .unwrap()
                .unwrap();
            assert_eq!(restored, committed);
            let confirmation = crate::transport::messaging::received_confirmation(&node).unwrap();
            assert_eq!(confirmation.description, "receipt");
            assert_eq!(
                confirmation.get_attr("id").and_then(|v| v.as_text()),
                Some("reply-fixture")
            );
            assert!(confirmation.get_attr("type").is_none());
            assert_eq!(
                confirmation.get_attr("to").and_then(|v| v.as_jid()),
                Some(&Jid::new_ad("12345", 0, 0, crate::model::jid::Server::Lid).unwrap())
            );
            let events = store
                .read_events(&ConsumerId::new("test").unwrap(), EventCursor::START, 10)
                .unwrap();
            assert_eq!(events.events.len(), 1);
            assert!(matches!(
                events.events[0].event,
                crate::sync::SyncEvent::NewMessage(_)
            ));

            // Independent key-share wire fixture encrypted by the generated
            // remote peer. A regular contact's protocol message must never
            // import keys; only the pinned own-device route may do so.
            let mut share_wire = b"\x62\x24\x10\x06\x3a\x20\x0a\x1e\x0a\x08\x0a\x06\x00\x01\x00\x00\x00\x02\x12\x12\x0a\x04test\x12\x08\x08\x07\x10\x02\x1a\x02\x01\x02\x18\x09".to_vec();
            share_wire.push(1); // WhatsApp padding
            let share_cipher =
                message_encrypt(&share_wire, &address, &mut sessions, &mut identities)
                    .await
                    .unwrap();
            let contact_node = Node::new("message")
                .attr("id", "contact-key-share")
                .attr("from", recipient.clone())
                .children(vec![
                    Node::new("enc")
                        .attr("v", "2")
                        .attr("type", "msg")
                        .bytes(share_cipher.serialize().to_vec()),
                ]);
            let capture = inbox::preserve(&store, &contact_node).unwrap();
            assert_eq!(
                inbox::process(&store, &account, &contact_node, &capture)
                    .await
                    .unwrap(),
                IncomingResult::Decrypted
            );
            let key_name = b"app-state-key:000100000002";
            assert!(
                store
                    .read(|tx| tx.get(Table::Checkpoints, key_name, 1))
                    .unwrap()
                    .is_none()
            );

            let mut peer = recipient.clone();
            peer.device = 1;
            let mut own_account = account.clone();
            own_account.own_jid = Some(recipient.clone());
            let own_session_key = format!("comparison-direct:{peer}");
            store
                .write(|tx| {
                    // Seed an explicitly pinned fixture session, not production
                    // device bootstrapping or a TOFU route.
                    tx.put_json(
                        Table::SignalSessions,
                        own_session_key.as_bytes(),
                        1,
                        &committed,
                    )?;
                    tx.put_json(
                        Table::PeerIdentities,
                        own_session_key.as_bytes(),
                        1,
                        &encrypted.identity,
                    )?;
                    account.save(tx) // absent own JID exercises commit-time guard
                })
                .unwrap();
            let peer_node = Node::new("message")
                .attr("id", "own-key-share")
                .attr("category", "peer")
                .attr("from", peer)
                .children(vec![
                    Node::new("enc")
                        .attr("v", "2")
                        .attr("type", "msg")
                        .bytes(share_cipher.serialize().to_vec()),
                ]);
            let capture = inbox::preserve(&store, &peer_node).unwrap();
            assert!(
                inbox::process(&store, &own_account, &peer_node, &capture)
                    .await
                    .is_err()
            );
            let unchanged: Vec<u8> = store
                .read(|tx| tx.get_json(Table::SignalSessions, own_session_key.as_bytes(), 1))
                .unwrap()
                .unwrap();
            assert_eq!(unchanged, committed);
            assert!(
                store
                    .read(|tx| tx.get(Table::Checkpoints, key_name, 1))
                    .unwrap()
                    .is_none()
            );
            store.write(|tx| own_account.save(tx)).unwrap();
            assert_eq!(
                inbox::process(&store, &own_account, &peer_node, &capture)
                    .await
                    .unwrap(),
                IncomingResult::Decrypted
            );
            let imported = store
                .read(|tx| tx.get(Table::Checkpoints, key_name, 1))
                .unwrap()
                .unwrap();
            drop(store);
            let store = ProtocolStore::open(options()).unwrap();
            assert_eq!(
                inbox::process(&store, &own_account, &peer_node, &capture)
                    .await
                    .unwrap(),
                IncomingResult::Duplicate
            );
            assert_eq!(
                store
                    .read(|tx| tx.get(Table::Checkpoints, key_name, 1))
                    .unwrap()
                    .unwrap(),
                imported
            );
        }
        let received = receive(
            &account,
            &recipient,
            &resumed.session,
            &encrypted.identity,
            reply.serialize(),
        )
        .await
        .unwrap();
        assert_eq!(&*received.plaintext, b"\x0a\x05reply");
        assert!(
            receive(
                &account,
                &recipient,
                &received.session,
                &encrypted.identity,
                reply.serialize()
            )
            .await
            .is_err()
        );
        assert!(
            receive(
                &account,
                &recipient,
                &resumed.session,
                &wrong,
                reply.serialize()
            )
            .await
            .is_err()
        );
        let mut tampered = reply.serialize().to_vec();
        *tampered.last_mut().unwrap() ^= 1;
        assert!(
            receive(
                &account,
                &recipient,
                &resumed.session,
                &encrypted.identity,
                &tampered
            )
            .await
            .is_err()
        );
        let malformed = message_encrypt(
            b"\x0a\x05reply\x00",
            &address,
            &mut sessions,
            &mut identities,
        )
        .await
        .unwrap();
        assert!(
            receive(
                &account,
                &recipient,
                &received.session,
                &encrypted.identity,
                malformed.serialize()
            )
            .await
            .is_err()
        );
    }
}
