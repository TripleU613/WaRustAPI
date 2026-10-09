# Token state restored by chat and history imports

2026-10-09. **Confirmed static**, comparison APK 2.26.37.71, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The [seven-topic recipe](apk/recipes/2.26.37.71-token-import.json) reproduces
[methods, register flow, full descriptors and catch tables](apk/2.26.37.71-token-import/targets.json).
This resolves the two previously untraced `0q2.A0V` callers in
[outbound issue producers](token-issue.md). It implements no history importer
or token request in Rust.

## Chat-transfer input

**Confirmed:** `CnI.A03` logs `p2p/fpm/ChatTransferMessagesImporter`. It loads
the chat through `0Fs.A00`; the token restoration block is within the missing-chat
creation branch, after its latch reaches zero. Existing-chat control jumps from
0026 directly to 02e0. A creation timeout reaches the failure at 02ba.

At 00de..0144 it requires `0DE.A0l` on the supplied chat before casting to
UserJid. Both token-byte and token-timestamp presence bits (0x40000/0x80000)
are needed to call `0q2.A0P(user, bytes, timestamp)` at 011c..0122.
The separate 0x2000000 presence bit calls `0q2.A0V(user, tcTokenSenderTimestamp)`
at 013e..0144. The sender-timestamp branch does not require incoming token
bytes. No current-clock substitution appears in these calls.

## Companion initial-history input

**Confirmed:** `CoO.A02` parses a `BZA` HistorySync protobuf from its byte-array
argument. Its sync-type enum ordinal switch at 0140 sends ordinal 0 to 114e,
where the log says `apply_initial_bootstrap_sync_start`. This is an ordinal
mapping, not a claim about the enum's wire number. The conversation loop at
12c0..12f8 reads each BZK and resolves the chat via `0pq.A01`; null and the
`0DE.A0Q` exclusion skip it. Effective account/LID gates and the full resolver
remain unknown here.

At 1a34 it tests `0DE.A0l` and takes the UserJid branch. It prefixes the imported
contact primary identity bytes with 0x05, parses them through `B7H.A02`, and
queues `DNu` selector 13 through `0kY.A01`. The retained catch table covers
code units 0d23..0d5c (bytes 1a46..1ab8), catching C4M at byte 1ab8. That
invalid-public-key log continues into token restoration at 1ac2; token import
must not be described as conditional on successful identity parsing.

The byte/token timestamp presence pair calls A0P at 1aee..1af4. The independent
sender-timestamp presence bit calls A0V at 1b0a..1b10. Therefore received token
bytes and outbound issue/send state are restored separately, just as in the
chat-transfer path. A0V being invoked does not itself prove a fresh IQ ACK.
Its SQL monotonicity and counterpart effects are in
[the refresh lifecycle](identity-token-refresh.md).

**Confirmed:** DNu.run selector 13 reaches 099a, reads captured user/CoO/B7E/BZK,
and derives the primary device. The full Object/Object/Object/Object/int
constructor used at CoO 1aaa assigns arguments 1/3/2/4 to A00/A01/A02/A03;
the separate typed overload with the same shorty is not substituted. Feature 0x6f29 selects the identity-save path;
one branch logs `saveIdentityWithoutNotification`, with companion fallback to
A14, canonical-address selection and A0H writes. It then reaches the already
traced imported verification metadata branch. These imported keys are a separate
source from authenticated prekey replacement. The previous
[history verification trace](peer-identity.md) applies; this is not authority
for Rust to accept an arbitrary history blob as a replacement pin.

## Field numbers and a decoder limitation

**Confirmed from BZK.dynamicMethod**, descriptor at 0336 and object array ending
0332: the descriptor has 59 fields, two presence words, min/max tags 1/63,
two lists and three initialization checks. Ayl.A1b initializes object index 0
with bitField0. Ayn.A1O writes supplied bitField1 at index 1 and the hidden
`id_` literal at index 2. Direct array writes begin with messages at index 3,
followed by its class object. This accounts for all 59 field names; neither
bookkeeping nor repeated-message class objects count as names.

| Tag | Field | Descriptor type | Presence index |
| --- | --- | --- | --- |
| 1 | id | string | 0 |
| 21 | tcToken | bytes | 18 |
| 22 | tcTokenTimestamp | int64 | 19 |
| 23 | contactPrimaryIdentityKey | bytes | 20 |
| 28 | tcTokenSenderTimestamp | int64 | 25 |
| 63 | identityVerification | message | 56 |

Numbers come from the encoded number/type/presence entries, not the array
indices. Tag 28's presence index 25 explains the 0x2000000 mask. Tag 63's
presence index 56 belongs to bitField1; no field-number-to-bit assumption is valid.

**Negative, confirmed local:** `apk-proto-fields --class 'LX/BZK;'` refuses
with `decoded 59 fields against 58 names; not guessing`. Its method-local literal
collector does not inline the id-array helper. Its successful descriptor decode
is not a complete field-name extraction. Use the committed descriptor and both
helper dumps to reproduce this manual pairing; no generic helper/object-array
analysis was added and no guessed output was enabled.

## Port boundary and validation

**Unknown/unported:** complete HistorySync/chat-transfer admission, authenticated
source authority, effective identity-save gates, import transaction boundaries,
canonical counterpart behavior and cross-device token restoration in Rust.
The import source resolves the A0V caller inventory; it does not close outbound
scheduling, refresh sending, received feature configuration or job recovery.
No account store, keys, request IDs, waits or live verification decisions changed.

Reproduce with `xtask apk-research-bundle` and the linked recipe, using a new
output directory, or `--check` on the committed bundle. Existing `apk-smali`
with `--signatures --exceptions` exposes each method and helper. Cached
`apk-research-diff` works on these seven stable topics and maps affected Rust
paths; a self-comparison only checks plumbing, not release compatibility.

Token-import research validation: seven topics / 12 files / 412,652 bytes
reproduce from the APK. Cached self-diff reports seven topics and four metadata
artifacts unchanged in 0.021659 seconds; this is not a cross-release benchmark.
All 65 xtask tests, CLI-reference regeneration, API/library-symbol check,
formatting and diff hygiene passed. Repository hygiene checked 1,287 local links
and 24 APK bundles / 7,401,275 bytes. Runtime source changes are comments only;
no account operation or live token/history request was made.
