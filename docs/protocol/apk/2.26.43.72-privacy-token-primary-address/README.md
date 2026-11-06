# Privacy-token primary-address admission

**Confirmed static**, WhatsApp APK2.26.43.72 SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Checked DEX export, not a REA Evidence envelope. Reproduce: xtask apk-smali APK
--class LX/10G; --method A09 --signatures --exceptions. This complete method
corroborates the existing [notification ACK audit](../../notification-ack-audit.md).

10G.A09 handles trusted_contact token entries, requesting UserJid from and
sender_lid via0Tp.A0A at008e..00b6. Null from is skipped; null sender_lid falls
back to from. The typed getter's class conversion is separately retained in
[ACK address evidence](../2.26.43.72-group-missing-identification/ack-typed-jid-read.txt).
Complete native subtype grammar, every token store side effect and native
sender_lid fallback policy are not reimplemented by this correction.

**Confirmed local defect/fix:** Rust's supported PN/LID-primary filters required
nonempty user and device0 but omitted agent0. Its generic Jid/parser represents
a legacy agent qualifier independently of namespace/device. A qualified from
could therefore create incoming-trusted-contact state and an ACK value even
though that qualifier is outside supported primary authority; the encoder
separately refuses nonzero agent in AD encoding. Both handle/parse from guards
and the explicit sender_lid guard now also require agent0. Invalid from is
skipped (no token/ACK); invalid explicit sender_lid retains the existing local
InvalidInput/no-write policy. That explicit fallback policy remains stricter
than the native null fallback; it was not silently changed here.

Rust's LID AD domain bit is represented by Server::Lid while agent stays0.
Do not equate the native DeviceLid getAgent/domain flag with Rust's legacy
agent field or reject all LID primaries. Existing zero-AD and PN/LID controls
remain valid. This is a bounded supported-primary correction, not a claim that
every native special UserJid subtype is ported.

The existing empty-peer/store-preservation regression now checks typed and text
agent-qualified PN/LID from, plus agent-qualified sender_lid. The old path
failed by storing1 update where0 was required. With the fix it checks skipped
sender/invalid alias outcomes and unchanged checkpoint/pin/session snapshots
across reopen. No new mapping, token synthesis, Signal pin adoption, query or
live account operation occurred.

**Bounded negative:** reach-out restriction admission was rechecked during this
audit. record_reachout_timelock already invokes base_ack immediately before its
store write, and its invalid-envelope restriction-preservation test covers the
case. No new restriction-envelope defect or change is claimed.
