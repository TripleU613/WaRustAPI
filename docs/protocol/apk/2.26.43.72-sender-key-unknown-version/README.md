# Newer unknown sender-key message version retry

**Confirmed static**, APK2.26.43.72 code264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reuses full checked-DEX [Fj2 constructor and version-path evidence](../2.26.43.72-message-version-error-paths/README.md),
[Eyj/0Yl](../2.26.43.72-group-counter-error-paths/README.md) and
[EtY/F2q/DxB](../2.26.43.72-outgoing-retry-error-map/README.md).
No new APK extraction, REA Evidence envelope, target execution or live account traffic.

Fj2([B) takes its version from the high nibble, admits exactly3 and branches
above3 at003e..0042 to00b8..00cc, constructing EDY(String) Unknown version: n.
This precedes payload-field admission and signature checking. Eyj.A01 callsFj2
only after establishing a nonempty sender-key record. 0Yl.A00 maps EDY -> -1005;
EtY's refinements include Unknown type: but not Unknown version:. The status
therefore remains -1005 -> internal4 -> DxB wire4. This is distinct from lower
versions: ED6 -> -1007, excluded from the selected retry eligibility predicate.

The installed backend SenderKeyMessage parser also admits exactly3 after minimum
byte-length admission, and above3 yields typed UnrecognizedCiphertextVersion.
Its exact source identity is retained. **Inferred compatibility mapping / implemented
offline:** only receive_group/skmsg maps that typed error when version>3 to fixed
redacted SignalUnknownSenderKeyVersion. Inbox preserves UnknownSenderKeyVersion;
normal receive chooses RetryCause::UnknownSenderKeyVersion -> error4. No version
or raw provider payload is included in diagnostics.

The empty-record check remains ahead of backend parsing and still selects
MissingSession/error1. Version rejection precedes signature checking, so a forged
or stale signature on such an unrecognized envelope does not select error5.
A retry cause is not authentication and does not authorize delivery confirmation,
inbox/event commit, identity changes or ratchet advancement. Existing count1,
original timestamp, routing and durable once-only/uncertain-write reservation
are unchanged. Offline CLI replay recognizes the outcome and sends nothing.

Offline inbox regression changes real synthetic skmsg header high nibbles to4
and15 under fresh IDs and asserts UnknownSenderKeyVersion with unchanged journal
and sender-key rows. Version0/1/2 retain Unsupported;64-byte inputs with version2
or4 stay generic Undecryptable because backend length admission wins. The production
uncertain-write loop changes a generated skmsg header to0x43, checks error4,
original timestamp and reservation before failed write, then no repeat retry or
delivery ACK after reconnect/redelivery. Existing other causes remain covered.

Reproduce: cargo test --locked -p whatsapp-api --all-features
consumed_group_record_is_a_duplicate_on_redelivery and
undecryptable_message_requests_once_before_write_cut_and_never_replays.
APK reproduction uses xtask apk-smali APK --class 'LX/Fj2;' --method '<init>'
--signatures --exceptions. These are offline/backend observations and static APK
inferences, not Android runtime packet acceptance. Short-input native result
parity, distribution/direct/prekey version policies and complete retry key/count/
routing/ACK behavior remain open. No blanket unsupported-version rule added.

Validation:1329 workspace/all-feature tests passed. Formatting, strict workspace
all-target/all-feature Clippy, CLI reference regeneration, APK coverage, staged
whitespace and research checksum/link hygiene passed. Offline only.
