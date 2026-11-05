# Application-padding admission and retry exclusion

**Confirmed static**, APK2.26.43.72 code264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reuses complete checked-DEX [EHq.A00 result factory and EtY retry predicate](../2.26.43.72-outgoing-retry-error-map/README.md)
and [Eyj.A01/0Yl.A00 group decrypt](../2.26.43.72-group-counter-error-paths/README.md).
No new APK extraction, REA Evidence envelope, target execution or live traffic.

## Application count is separate from crypto block padding

EHq.A00 at0014..001e rejects null/empty successful-decrypt output as status-1000.
Otherwise0024..002c reads the final byte and masks0xff, so count is unsigned.
Zero count selects status-10000 at003a..0048. At004a, count>=plaintext length
also selects-10000. Valid count is therefore1..length-1. At0054..0060 it subtracts
that count and copies the prefix; there is no count<=16 gate and no repeated-byte
validation. EtY's selected retry predicate excludes both-1000 and-10000.

Rust previously required application count1..16 and equal trailing bytes in
three receive adapters. It also admitted padding that consumed the entire buffer,
returning empty plaintext. Rejected padding was an InvalidInput direct/group
crypto-adjacent error, which the inbox merged into generic Undecryptable and
normal receive could retry. Those admission and retry classifications differed
from the selected native result factory.

**Implemented / offline tested:** signal::padding is a private shared helper
for classic direct/prekey and group receive. Existing hybrid receive delegates
to direct receive and inherits its application trimming; no KEM primitive policy
is changed. The helper reads the final count, accepts only0<count<length and
truncates the prefix in Zeroizing storage. It accepts nonrepeating tails and
counts above16, including255 with enough plaintext. Sender-side padding creation
is unchanged. Existing Signal MAC/signature/AES block-padding verification still
precedes this helper; no crypto padding/check was relaxed.

Empty/invalid application framing produces fixed redacted
SignalInvalidApplicationPadding. Direct/group inbox boundaries preserve capture,
continue as Unsupported and queue neither automatic retry nor delivery confirmation.
They retain Rust's existing atomic ratchet/inbox commit boundary, so these refused
payloads commit no message event/ratchet or prekey consumption here. Native Eyj
calls its Signal store before the coordinator's EHq trimming; full native outer
transaction/durability semantics around this ordering remain untraced. This
change does not claim that every native persistence/ACK consequence is ported.

## Crypto-failure exploration retained

Eyj.A01 uses AES/CBC/PKCS5Padding at02f8..031a. Its protected-region catch table
routes IllegalBlockSizeException/BadPaddingException to032a, wrapping the cause
as EDY(Throwable). NoSuchAlgorithm/NoSuchPadding/InvalidKey/InvalidAlgorithmParameter
instead go to0338 and AssertionError. These block/provider failures precede
EHq's application trimming and must not be conflated with invalid outer count.
Backend source identity is retained: its CBC failure shares InvalidMessage with
a future-counter producer, while key/IV failures use InvalidSenderKeySession.
No provider-string parser, blanket CBC retry code or outer-padding retry is added.
Complete provider-failure and native persistence/ACK mapping remains open.

## Offline reproduction

cargo test --locked -p whatsapp-api --all-features application_padding
covers unsigned counts1/16/17/20/255, nonrepeating tails, empty/zero/oversized/
entire-buffer rejection, and actual signed/AES-encrypted group fixtures. The
production receive-loop test sends authenticated zero-count and entire-padding
messages, then a subsequent receipt as a processing barrier. It asserts only the
subsequent receipt ACK is written, no retry outcome or message event exists, sender
keys remain unchanged, and receive continues. Existing direct/prekey/group/hybrid
round trips remain covered by the workspace suite. Tests do not execute Android.

Recheck EHq via xtask apk-smali APK --class 'LX/EHq;' --method A00
--signatures --exceptions, and Eyj/A01 for separate crypto failure paths. Complete
route/key/count progression, other typed failures, native commit/ACK policy and
server acceptance remain open. No zero-ban guarantee or full audit completion.

Test-harness follow-up: a concurrent workspace run reported an unexpected write
in the inventory policy test. That test now uses Tokio's paused clock, keeping
its no-extra-upload assertions independent of host CPU/wall-clock scheduling.
Fake-link failures now report only redacted tag/type/namespace/child-tag shape;
production heartbeat/transport behavior is unchanged.

Coverage inventory was regenerated for the new private source module. Its APK
evidence is the linked local checked-DEX export; no new REA-provider record was
manufactured. The existing report correctly lists the module as uncited within
that report's narrower retained-record mapping scope.

Validation:1332 workspace/all-feature tests passed on the final run. Formatting,
strict workspace/all-target/all-feature Clippy, CLI reference regeneration,
regenerated APK coverage, staged whitespace and research checksum/link hygiene
passed. The invalid table fixture and inventory-test clock were corrected before
that final run. Offline only.
