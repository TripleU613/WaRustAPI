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

**Follow-up local inspection (2026-10-09):** `handle` calls `record` before
checking that every child/token kind is supported. Mixed notifications may store
supported trusted-contact entries while remaining without ACK; malformed supported
entries are validated before the write transaction. This review did not establish
that partial supported-kind processing is a bug or authorize unknown-kind ACKs.
No extra notification behavior was changed in the clock-producer chunk.


## Timestamp parser correction (2026-10-09)

**Confirmed static:** same APK 2.26.37.71/hash as above. `11P.A09`
`00c6..00e4` requires parent `t` through `0hs.A09`, then selects token `t`
with `0hs.A08` and that parent value as the missing-attribute fallback.
A09 delegates to Java Long.parseLong and wraps NumberFormatException in B7l;
A08 defaults only when the attribute getter returns null, otherwise it calls
A09. A present malformed token time is not a request to use the parent time.
The [two-topic recipe](apk/recipes/2.26.37.71-token-notification-time.json) and
[snapshot](apk/2.26.37.71-token-notification-time/targets.json) preserve both
helpers with signatures/register flow/catches. The existing peer-guard bundle
preserves their 11P caller. No new raw APK or bulk class tree is committed.

**Confirmed local correction:** notifications previously used parse::<u64>,
rejecting BMP decimal digit strings accepted by Java's UTF-16 long parser.
The failing-before integration regression stopped at `+２０` with the fixed
`invalid notification timestamp` error. Parent and token time now reuse
mex::java_decimal_long before conversion to nonnegative u64. That preserves
our existing 0..i64::MAX storage range; native acceptance of signed input does
not prove that every negative time is valid token history. Negative nonzero
values still refuse locally, while `-0` parses as zero. No state format changes
or history rewrite occur. This is parser parity inside the supported range,
not full native token storage/admission parity.

**Independent numeric evidence:** local Temurin Java 21.0.12 Long.parseLong
returned `+２０ -> 20`, `٢٠ -> 20`, `+٣٠ -> 30`, `-0 -> 0`, `-1 -> -1`
and maximum i64 unchanged. One-above-maximum, supplementary digits `𝟜𝟘`,
and leading/trailing whitespace threw NumberFormatException. This checks the
selected known digit/sign cases independently of Rust; it is not Android
runtime or all-Unicode-version equivalence. The existing mex parser has its
own independent Java fixtures; no second digit table was introduced.

**Regression scope:** the ACK-producing handler accepts a BMP-digit parent
fallback and BMP-digit token override, preserving timestamp 30 and token bytes
from the override. Older equal-source repeats do not replace it. Malformed,
negative, overflow, whitespace and supplementary-digit later tokens refuse
before committing an earlier valid newer token in the same notification;
checkpoint snapshots are unchanged and diagnostics omit the rejected value.
Negative zero is valid but older and leaves the token unchanged. Reopening
preserves the checkpoint snapshot. These are synthetic encrypted-store tests;
no saved real account, identities, mappings, keys, waits or live session changed.

**Completed checks:** 821 workspace/all-feature tests passed (685 API unit,
four codec, nine crash matrix, 52 CLI unit, one API-symbol and 70 xtask).
Strict workspace/all-target/all-feature Clippy, warnings-denied Rustdoc,
default CLI tests, CLI reference regeneration, formatting and diff whitespace
passed. Exact re-export verifies seven files / 31,039 bytes; self-diff reports
two topics and four artifacts unchanged. Hygiene validates 1,818 local links
and 75 checksum bundles. The new regression failed on the unmodified parser
and passed after the correction. Fresh live Unicode timestamp handling and
full Android numeric/token admission parity remain unverified.


## Explicit zero-qualified text is not primary-user authority (2026-11-06)

**Confirmed static / implemented offline:** the existing current
[10G.A09 export](apk/2.26.43.72-privacy-token-primary-address/privacy-token-handler.txt)
requests UserJid for from and sender_lid at008e..00b6. The
[typed getter](apk/2.26.43.72-group-missing-identification/ack-typed-jid-read.txt)
casts to the requested class, returning null on ClassCastException. The
[parser](apk/2.26.43.72-message-ack-chat/jid-parser.txt)
checks dot/colon *presence* at0196..01b8 before parsing numbers. Qualified
PN/LID forms construct0EE/0EM at021a..0236 and02b2..02bc, even for zero.
The retained [device override ancestry](apk/2.26.43.72-group-missing-identification/current-device-overrides.txt)
shows0EE and0EM inherit DeviceJid, which extends Jid rather than UserJid.
These are existing complete static exports, not new REA Evidence IDs.

Rust had accepted `user:0`, `user.0` and `user.0:0` text as a zero-valued
primary token peer. The store-preservation regression failed with one update
where zero was expected when supplied a newer timestamp. A separate ACK
assertion also failed. A shared token_peer reader now checks raw text qualifier
presence before generic parsing in both handle and parse. Invalid from yields
no supported update/ACK; invalid explicitly consumed sender_lid keeps the
existing InvalidInput/no-write policy. No token or primary identity is inferred
from a qualified zero. PN/LID sender and LID alias controls cover all three
spellings, preserving checkpoint/pin/session snapshots across reopen.

**Distinct binary behavior:** binary zero-device PN/LID decoding in native
123.A02/A04 reconstructs a plain user string; the existing
[binary reader proof](apk/2.26.43.72-group-missing-identification/current-ad-string-reader.txt)
and [joiner](apk/2.26.43.72-group-missing-identification/current-ad-string-join.txt)
remain applicable. Rust typed zero-device input remains supported, including
the existing binary-LID privacy-token ACK/storage regression. No global JID
parser/decoder, stored identity, alias trust or token timestamp policy changed.
Caller-supplied typed values that already lost text provenance remain a limit.

**Native policy versus local support:** the native handler skips a null from
for that entry and falls back to from for a null sender_lid; its base ACK is
separate. Rust retains its stricter supported-envelope/explicit-alias policy,
not full native malformed-envelope ACK parity. Unknown/empty token lists still
do not consume sender_lid; regression now includes unused zero-qualified aliases
that must preserve state and still ACK. Special UserJid families, full parser
parity and live acceptance remain unverified. No account traffic or query added.
