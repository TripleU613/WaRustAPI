# Concrete peer guard for trusted-contact notifications

**Confirmed local correction, 2026-10-09:** a `privacy_token` notification with
server-only `from` in the supported PN/LID namespace was accepted by our token
recorder, creating an `incoming-trusted-contact:<server>` checkpoint. The generic
`Jid` type deliberately accepts server-only addresses; checking server/device
alone did not establish a concrete peer. The failing regression returned one
stored update where zero was required. This is our guard bug, not an observed
WhatsApp refusal or proof of a server sending malformed addresses.

**Confirmed static evidence:** APK 2.26.37.71, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`,
`11P.A09` at `008e..00bc` reads `from` and optional `sender_lid` through the
class-constrained `UserJid` getter. A missing typed sender skips the item;
a missing typed sender_lid falls back to the sender. At `00e8..00ee` it passes
that user address, bytes and timestamp to `0q2.A0P`. The
[one-topic recipe](apk/recipes/2.26.37.71-token-notification-peer.json) and
[snapshot](apk/2.26.37.71-token-notification-peer/targets.json) retain the full
handler with signatures/register flow/catches: six files / 32,905 bytes.

**Local supported boundary:** both `handle` (ACK-producing entry point) and
`record` now require a nonempty user for their supported primary PN/LID sender.
A supplied sender_lid likewise requires a nonempty LID user. Invalid sender
returns no supported update/ACK; supplied invalid sender_lid retains the fixed
`invalid sender LID` diagnostic and fails before the write transaction. No raw
address or token is included in that diagnostic. This preserves our previous
strict supplied-LID policy; it does not port the APK's typed-getter fallback for
all invalid attributes, extra UserJid subtypes or device routes. Complete native
UserJid parser acceptance remains unknown here.

Valid notifications keep their strictly newer timestamp replacement rule. Tokens
still carry incoming bytes only; they create no authenticated PN/LID alias,
identity pin, outgoing token issue timestamp or refresh authority. See
[messaging transport](messaging-transport.md) and
[identity token refresh](identity-token-refresh.md).

**Regression scope:** server-only text senders and empty structured Plain/AD
PN/LID senders are refused by both entry points. Empty structured Plain/AD
sender_lid values fail with the fixed diagnostic. The test first stores a valid
notification and sentinel peer-pin/session rows, then checks decrypted checkpoints,
pins and sessions remain identical after all refusals and after reopening.
Existing valid-token tests also cover equal/older timestamp handling and ACKs.
These are synthetic offline stores, not operations on saved real accounts.

**Unknown/live:** no qualifying malformed notification was observed live, and no
new receive session, account reset or transmission was used for this correction.

**Verification:** all 818 workspace/all-feature tests passed (682 API unit,
four codec, nine crash matrix, 52 CLI unit, one API-symbol, 70 xtask). Strict
workspace/all-target/all-feature Clippy, warning-free Rustdoc, regenerated CLI
reference, API-symbol integration, formatting and diff whitespace passed.
Exact APK snapshot regeneration passed; self-diff reports its topic and four
artifacts unchanged. Repository hygiene passed: 1,792 links and 71 checksum-
verified bundles. Initial test-authoring compile errors were corrected before
the recorded failing assertion; compile errors are not counted as regression proof.
