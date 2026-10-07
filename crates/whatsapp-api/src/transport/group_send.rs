//! Planning for a sender-key fan-out: who an outgoing group or status message
//! has to be addressed to, and in which JID form.
//!
//! The crypto lives in [`crate::signal::group`] and the stanza in
//! [`super::groups::sender_key_stanza`]; this module answers the question that
//! blocked both — which device JIDs go into `<participants>`.
//!
//! ## Where the recipient set comes from (comparison 2.26.37.71)
//!
//! `SendE2EMessageJob` does **not** compute it. Its constructor takes
//! `targetDeviceRawJids` (a `HashSet`), `groupParticipantHash`,
//! `groupParticipantUserHash`, `useParticipantUserHash`, `multiDeviceFanOut`,
//! `useLidForEncryption`, `includeSenderKeysInMessage` and
//! `forceSenderKeyDistribution` as parameters — every one of them is decided by
//! the caller (`LX/DNu->run`, over an `LX/Ceo` parameter object) before the job
//! exists. **Confirmed** by reading the constructor's field assignments and the
//! `invoke-direct/range {v23..v64}` argument set-up at that call site.
//!
//! A **negative result worth keeping**: `messageRecipientDevicesProvider`
//! (`LX/CQl`) is *not* the fresh resolver. Its non-status branch reads
//! `LX/0Z4`→`LX/0Z5`/`LX/0Z6`/`LX/0ZE`, whose own string constants name them
//! `receipt_device` / `message_add_on_receipt_device` and
//! `MESSAGE_DEVICE_RECEIPTS_CACHE` — devices that already hold a receipt row for
//! *this* message, used on retries, and reached only inside the
//! `counterAbuseTokenGate` branch of `A06`. Do not re-trace it. Its one useful
//! branch is `instanceof LX/BuQ` → `E2eStatusJobParams.A02()`, which is where a
//! *status* post's recipient list comes from.
//!
//! So the recipient set is the ordinary one: the chat's participants, expanded
//! to their devices. We build it from the saved group snapshot and the saved
//! `usync devices` lists, which is what a client that has those two things can
//! honestly do.
//!
//! ## Addressing mode
//!
//! **Confirmed** (`A06` `12d0..1322`): `addressing_mode` is `"lid"` when the
//! group is LID-addressed and `"pn"` otherwise. **Observed** live on
//! 2026-09-22: every stanza from the operator's group carried
//! `addressing_mode="lid"` with a LID `participant`. We therefore choose the
//! form from the membership we actually hold and refuse a mixed set rather than
//! guessing — addressing a member in the wrong form would encrypt to an
//! identity the recipient does not treat as the sender.
//!
//! ## The participant hash
//!
//! Settled on 2026-09-22 against the `phash` the server returned in the `ack`
//! for our first fan-out: it covers **every member including the sender**, in
//! the AD JID form, under SHA-256. See
//! [`super::groups::participant_hash_v2`].
//!
//! ## Still unknown
//!
//! Whether our own other devices belong in the fan-out. This account is a
//! primary with no companions, so the question does not arise for it and
//! nothing here pretends to answer it; [`FanoutPlan::own_devices_unknown`]
//! records that honestly.
use crate::{
    Error, Result,
    model::jid::{Jid, Server},
    store::{ProtocolStore, Table, WriteTx},
};

/// Checkpoint holding this account's own LID, learned from `<success>`.
pub const OWN_LID_KEY: &[u8] = b"own-lid";

fn invalid(reason: impl Into<String>) -> Error {
    Error::invalid("group_send", reason)
}

/// Record the LID the server stamped on our login.
///
/// **Confirmed** from `LX/0iS->A12` (2.26.37.71): the `<success>` stanza's
/// `lid` attribute is read, converted with `DeviceJid`/`getUserJid`, and its
/// absence is reported as `reg-lid-chatd-lid-expected-but-null` — so it is the
/// client's own LID and the client expects it to be there. A LID-addressed
/// group names us by it, so a fan-out cannot be built without it.
pub fn record_own_lid(tx: &mut WriteTx<'_>, lid: &Jid) -> Result<()> {
    if lid.server != Server::Lid || lid.user.is_empty() {
        return Err(invalid("own lid must be a LID identity"));
    }
    tx.put_json(Table::Checkpoints, OWN_LID_KEY, 1, &lid.to_non_ad())
}

/// The recorded own LID, if a login has provided one.
pub fn own_lid(store: &ProtocolStore) -> Result<Option<Jid>> {
    store.read(|tx| tx.get_json(Table::Checkpoints, OWN_LID_KEY, 1))
}

/// A `Message` protobuf carrying one styled text status: `extendedTextMessage`
/// with the colour and font fields a full-screen text status uses.
///
/// Field numbers are the ones the receive side already decodes from the
/// `LX/BZF` descriptor (**confirmed**, 2.26.37.71): `text` 1, `textArgb` 7 and
/// `backgroundArgb` 8 as **fixed32** — not varint — and `font` 9. Emitting the
/// two colours as varints would put the wrong wire type on the field rather
/// than merely mis-sign the alpha channel.
///
/// With no styling this is still an `extendedTextMessage` rather than a bare
/// `conversation`, which is what a status post is.
pub fn status_text_payload(text: &str, style: &super::quotes::TextStyle) -> Result<Vec<u8>> {
    use prost::Message as _;
    if text.is_empty() || text.len() > 4096 {
        return Err(invalid("a status text is 1..=4096 bytes"));
    }
    #[derive(prost::Message)]
    struct Extended {
        #[prost(string, optional, tag = "1")]
        text: Option<String>,
        #[prost(fixed32, optional, tag = "7")]
        text_argb: Option<u32>,
        #[prost(fixed32, optional, tag = "8")]
        background_argb: Option<u32>,
        #[prost(int32, optional, tag = "9")]
        font: Option<i32>,
    }
    #[derive(prost::Message)]
    struct WithExtended {
        // `Message.extendedTextMessage` is **6** (`LX/BVL`), which is what the
        // receive side already decodes. Encoding it anywhere else would repeat
        // the field-18 mistake: self-consistent here, unreadable everywhere else.
        #[prost(message, optional, tag = "6")]
        extended: Option<Extended>,
    }
    Ok(WithExtended {
        extended: Some(Extended {
            text: Some(text.into()),
            text_argb: style.text_argb,
            background_argb: style.background_argb,
            font: style.font.map(|f| f as i32),
        }),
    }
    .encode_to_vec())
}

/// A `Message` protobuf carrying one plain text body, the payload both the
/// direct path and a sender-key send encrypt. Kept here so a group send does
/// not have to reach into the direct path's private encoder.
pub fn text_payload(text: &str) -> Result<Vec<u8>> {
    use prost::Message as _;
    if text.is_empty() || text.len() > 4096 {
        return Err(invalid("text must be 1..=4096 bytes"));
    }
    #[derive(prost::Message)]
    struct TextMessage {
        #[prost(string, tag = "1")]
        conversation: String,
    }
    Ok(TextMessage {
        conversation: text.into(),
    }
    .encode_to_vec())
}

/// Who a sender-key send must address, and what we could not resolve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FanoutPlan {
    /// Group or broadcast chat.
    pub chat: Jid,
    /// `lid` or `pn`, chosen from the membership form.
    pub addressing_mode: &'static str,
    /// Our own identity in this chat's addressing form; the sender-key name and
    /// the `participant` the server will stamp both use it.
    pub own: Jid,
    /// Member identities (non-AD), excluding us.
    pub recipients: Vec<Jid>,
    /// Device JIDs to put in `<participants>`, in a stable order.
    pub devices: Vec<Jid>,
    /// Members with no saved device list; their devices are missing from
    /// `devices` and they will not receive the message.
    pub without_device_list: Vec<Jid>,
    /// Always true: whether our own companion devices belong in the fan-out was
    /// not established, and this account has none to observe it with.
    pub own_devices_unknown: bool,
    /// The version 2 participant hash over the whole membership, us included.
    /// See [`super::groups::participant_hash_v2`] for how its construction was
    /// established against a value the server itself returned.
    pub participant_hash: String,
}

/// Build the fan-out plan for `chat` from the saved group snapshot and the
/// saved device lists.
///
/// `own` is our identity in the chat's addressing form. Refuses rather than
/// guesses when the membership mixes JID forms, when we are not a member, or
/// when no member has a device list — each of those would produce a stanza
/// addressed to the wrong identities.
pub fn plan(store: &ProtocolStore, chat: &Jid, own: &Jid) -> Result<FanoutPlan> {
    if chat.server != Server::Group {
        return Err(invalid("this plan reads a group snapshot"));
    }
    let snapshot = super::groups::get(store, chat)?
        .ok_or_else(|| invalid("no saved group snapshot; fetch group info first"))?;
    let mut members = Vec::new();
    for member in &snapshot.members {
        let jid = Jid::parse(&member.jid).map_err(|_| invalid("unparsable member JID"))?;
        if !jid.is_user_like() {
            return Err(invalid("a member is neither a phone number nor a LID"));
        }
        members.push(jid.to_non_ad());
    }
    plan_over(store, chat, own, members)
}

/// Plan a status post: the same sender-key fan-out, aimed at `status@broadcast`
/// with an **explicit** audience rather than a group's membership.
///
/// A status has no membership list to read. The APK's own
/// `E2eStatusJobParams.A02()` is the *retry* list — an explicit target set, else
/// the devices holding a receipt row for this status, else the parent status's
/// for a repost — so it does not answer "who should see a new one" either. That
/// audience is the account's status privacy list, which this client does not
/// sync, so the caller names it. Naming it is also the honest thing for a lab
/// account: it is WhatsApp's "share with only…" and nothing is inferred about
/// who else might be entitled to see it.
///
/// `own` is included in the participant hash, exactly as it is for a group, but
/// never addressed.
pub fn plan_status(store: &ProtocolStore, own: &Jid, audience: &[Jid]) -> Result<FanoutPlan> {
    let chat = Jid::parse("status@broadcast")?;
    if audience.is_empty() || audience.len() > 256 {
        return Err(invalid("a status audience is 1..=256 identities"));
    }
    let mut members = vec![own.to_non_ad()];
    for jid in audience {
        if !jid.is_user_like() {
            return Err(invalid(
                "a status recipient is a phone-number or LID identity",
            ));
        }
        members.push(jid.to_non_ad());
    }
    plan_over(store, &chat, own, members)
}

fn plan_over(
    store: &ProtocolStore,
    chat: &Jid,
    own: &Jid,
    members: Vec<Jid>,
) -> Result<FanoutPlan> {
    if !matches!(chat.server, Server::Group | Server::Broadcast) {
        return Err(invalid("a sender-key send targets a group or broadcast"));
    }
    if members.is_empty() {
        return Err(invalid("no members to address"));
    }
    let lid = members.iter().filter(|j| j.server == Server::Lid).count();
    let addressing_mode = if lid == members.len() {
        "lid"
    } else if lid == 0 {
        "pn"
    } else {
        return Err(invalid(
            "membership mixes LID and phone-number identities; cannot choose an addressing mode",
        ));
    };
    let own = own.to_non_ad();
    if (own.server == Server::Lid) != (addressing_mode == "lid") {
        return Err(invalid(
            "our own identity is not in the group's addressing form",
        ));
    }
    if !members.contains(&own) {
        return Err(invalid("we are not a member of this group"));
    }
    // The hash covers every member, the sender included, before we drop
    // ourselves from the recipient list.
    // This planner still hashes primary-member addresses. APK 37.71
    // 1jL.A0N/A0O -> AzR.A03 instead hash complete cached device sets.
    // Companion/own-device inventory integration remains unported; see
    // docs/protocol/groups.md participant cache/versioned hash trace.
    let participant_hash = super::groups::participant_hash_v2(&members);
    let mut recipients: Vec<Jid> = members.into_iter().filter(|j| *j != own).collect();
    recipients.sort_by_key(Jid::to_string);
    recipients.dedup();
    if recipients.is_empty() {
        return Err(invalid("the group has no other member to address"));
    }
    let mut devices = Vec::new();
    let mut without_device_list = Vec::new();
    for recipient in &recipients {
        match super::devices::device_list(store, recipient)? {
            Some(list) if !list.devices.is_empty() => {
                devices.extend(list.devices.iter().map(|d| d.jid.clone()));
            }
            _ => without_device_list.push(recipient.clone()),
        }
    }
    // Group by identity first, then by device index, so the primary of each
    // member leads: a raw string sort would put `:3@lid` before `@lid`.
    devices.sort_by_key(|d| (d.to_non_ad().to_string(), d.device));
    devices.dedup();
    if devices.is_empty() {
        return Err(invalid(
            "no member has a saved device list; run a usync devices query first",
        ));
    }
    Ok(FanoutPlan {
        chat: chat.clone(),
        addressing_mode,
        own,
        recipients,
        devices,
        without_device_list,
        own_devices_unknown: true,
        participant_hash,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::time::Timestamp;
    use crate::{
        binary::Node,
        model::ids::AccountId,
        store::{MasterKey, OpenOptions, StaticKeySource},
        transport::devices::{DeviceEntry, DeviceList},
    };

    fn open(path: &std::path::Path) -> ProtocolStore {
        ProtocolStore::open(OpenOptions {
            path: path.to_owned(),
            account_id: AccountId::new("test").unwrap(),
            key_source: std::sync::Arc::new(StaticKeySource::new(MasterKey::from_bytes([7; 32]))),
            create: true,
        })
        .unwrap()
    }

    fn jid(raw: &str) -> Jid {
        Jid::parse(raw).unwrap()
    }

    /// Save a group snapshot the same way an info reply would.
    fn save_group(store: &ProtocolStore, chat: &str, members: &[&str]) {
        let node = Node::new("iq").attr("type", "result").children(vec![
            Node::new("group").attr("id", chat).children(
                members
                    .iter()
                    .map(|m| Node::new("participant").attr("jid", jid(m)))
                    .collect(),
            ),
        ]);
        assert!(super::super::groups::record_info_reply(store, &node).unwrap());
    }

    fn save_devices(store: &ProtocolStore, owner: &str, devices: &[&str]) {
        let owner = jid(owner);
        store
            .write(|tx| {
                tx.put_json(
                    Table::Checkpoints,
                    format!("devices:{owner}").as_bytes(),
                    1,
                    &DeviceList {
                        owner: owner.clone(),
                        devices: devices
                            .iter()
                            .map(|d| DeviceEntry {
                                jid: jid(d),
                                key_index: 0,
                            })
                            .collect(),
                        raw_id: None,
                        list_timestamp: None,
                        expected_timestamp: None,
                        companions_verified: true,
                        unverified_reason: None,
                        notified_hash: None,
                        hash_matches: None,
                        hosted_skipped: 0,
                        fetched_at: Timestamp::now(),
                    },
                )
            })
            .unwrap();
    }

    /// The whole point of the plan: a LID-addressed group expands to its
    /// members' device JIDs, excluding us, and says so in the LID form the
    /// group actually uses.
    #[test]
    fn plan_expands_lid_members_to_their_devices() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("p.redb"));
        let chat = jid("120363000000000001@g.us");
        let own = jid("111111111111111@lid");
        save_group(
            &store,
            "120363000000000001@g.us",
            &["111111111111111@lid", "222222222222222@lid"],
        );
        save_devices(
            &store,
            "222222222222222@lid",
            &["222222222222222@lid", "222222222222222:3@lid"],
        );
        let plan = plan(&store, &chat, &own).unwrap();
        assert_eq!(plan.addressing_mode, "lid");
        assert_eq!(plan.recipients, vec![jid("222222222222222@lid")]);
        assert_eq!(
            plan.devices,
            vec![jid("222222222222222@lid"), jid("222222222222222:3@lid")]
        );
        assert!(plan.without_device_list.is_empty());
        assert!(plan.own_devices_unknown);
        // The hash covers the whole membership, us included — that is the part
        // the server's own value settled, and dropping ourselves would silently
        // produce a different one.
        assert_eq!(
            plan.participant_hash,
            super::super::groups::participant_hash_v2(&[
                jid("111111111111111@lid"),
                jid("222222222222222@lid"),
            ])
        );
        assert_ne!(
            plan.participant_hash,
            super::super::groups::participant_hash_v2(&[jid("222222222222222@lid")])
        );
        // We never address ourselves.
        assert!(!plan.devices.iter().any(|d| d.to_non_ad() == own));
    }

    /// A member whose devices we never fetched is reported, not silently
    /// dropped: the caller decides, and the CLI refuses rather than sending a
    /// message that reaches only part of the group.
    #[test]
    fn plan_reports_members_without_device_lists() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("p.redb"));
        let chat = jid("120363000000000002@g.us");
        let own = jid("15550000001@s.whatsapp.net");
        save_group(
            &store,
            "120363000000000002@g.us",
            &[
                "15550000001@s.whatsapp.net",
                "15550000002@s.whatsapp.net",
                "15550000003@s.whatsapp.net",
            ],
        );
        save_devices(
            &store,
            "15550000002@s.whatsapp.net",
            &["15550000002@s.whatsapp.net"],
        );
        let plan = plan(&store, &chat, &own).unwrap();
        assert_eq!(plan.addressing_mode, "pn");
        assert_eq!(
            plan.without_device_list,
            vec![jid("15550000003@s.whatsapp.net")]
        );
        assert_eq!(plan.devices, vec![jid("15550000002@s.whatsapp.net")]);
    }

    /// Each refusal is a stanza we would otherwise have addressed to the wrong
    /// identities.
    #[test]
    fn plan_refuses_what_it_cannot_address_honestly() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("p.redb"));
        let own = jid("111111111111111@lid");
        // No snapshot at all.
        assert!(plan(&store, &jid("120363000000000009@g.us"), &own).is_err());
        // A mixed membership has no single addressing mode.
        save_group(
            &store,
            "120363000000000003@g.us",
            &["111111111111111@lid", "15550000002@s.whatsapp.net"],
        );
        assert!(plan(&store, &jid("120363000000000003@g.us"), &own).is_err());
        // We are not a member.
        save_group(
            &store,
            "120363000000000004@g.us",
            &["222222222222222@lid", "333333333333333@lid"],
        );
        assert!(plan(&store, &jid("120363000000000004@g.us"), &own).is_err());
        // Our own identity is in the wrong form for the group.
        save_group(
            &store,
            "120363000000000005@g.us",
            &["15550000001@s.whatsapp.net", "15550000002@s.whatsapp.net"],
        );
        assert!(plan(&store, &jid("120363000000000005@g.us"), &own).is_err());
        // Right form, right membership, but nothing addressable yet.
        save_group(
            &store,
            "120363000000000006@g.us",
            &["111111111111111@lid", "222222222222222@lid"],
        );
        assert!(plan(&store, &jid("120363000000000006@g.us"), &own).is_err());
        // Not a group chat at all.
        assert!(plan(&store, &jid("15550000002@s.whatsapp.net"), &own).is_err());
    }

    /// The LID is what a LID-addressed group names us by; a phone JID there
    /// would name the wrong sender key.
    #[test]
    fn own_lid_round_trips_and_refuses_other_identities() {
        let dir = tempfile::tempdir().unwrap();
        let store = open(&dir.path().join("p.redb"));
        assert!(own_lid(&store).unwrap().is_none());
        store
            .write(|tx| record_own_lid(tx, &jid("111111111111111:2@lid")))
            .unwrap();
        // Stored without the device suffix: the sender-key name is per identity.
        assert_eq!(own_lid(&store).unwrap(), Some(jid("111111111111111@lid")));
        assert!(
            store
                .write(|tx| record_own_lid(tx, &jid("15550000001@s.whatsapp.net")))
                .is_err()
        );
    }

    /// A styled status must come back out of the receive path exactly as it
    /// went in — same text, same colours, same font. That path was written and
    /// live-verified against a real status from the operator's handset long
    /// before this encoder existed, so it is an independent check rather than
    /// this function agreeing with itself.
    #[test]
    fn a_styled_status_round_trips_through_the_receive_path() {
        use super::super::quotes::{StatusFont, TextStyle, decode};
        let style = TextStyle {
            background_argb: Some(0xFF26_2626),
            text_argb: Some(0xFFFF_FFFF),
            font: Some(StatusFont::BebasNeueRegular),
        };
        let bytes = status_text_payload("hello status", &style).unwrap();
        let decoded = decode(&bytes).expect("the receive path must read it");
        assert_eq!(decoded.text, "hello status");
        assert_eq!(decoded.style, style);
        // An unstyled status is still an extendedTextMessage, not a bare
        // `conversation`: that is what distinguishes a status from a chat line.
        let plain = status_text_payload("bare", &TextStyle::default()).unwrap();
        assert_eq!(plain[0], 0x32, "field 6, length-delimited");
        assert_eq!(decode(&plain).unwrap().text, "bare");
        assert!(!decode(&plain).unwrap().style.is_styled());
        assert!(status_text_payload("", &style).is_err());
    }

    /// Field 1 `conversation`, the same shape the receive path decodes.
    #[test]
    fn text_payload_is_a_conversation_message() {
        let bytes = text_payload("hi").unwrap();
        assert_eq!(bytes, vec![0x0a, 0x02, b'h', b'i']);
        assert!(text_payload("").is_err());
        assert!(text_payload(&"x".repeat(4097)).is_err());
    }
}
