# Group comparison retry encryption count — 2026-11-05

**Confirmed static / corrected offline:** APK2.26.43.72, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
[Complete checked producer/callback/composer evidence](apk/2.26.43.72-group-retry-count/README.md).
No live group/status retry, key fetch or resend occurred.

The explicit comparison group's retry driver used the ordinary first-send builder,
so its root skmsg and pairwise distribution enc children omitted count. The
[receipt→job trace](retry-enc-count.md) established the incoming count is not
incremented. The new checked trace binds the concrete job callback through EoG,
interface Fv3 and the composer, and follows retryCount into both main-result and
fan-out encryption builders. F3f.A09 emits nonzero decimal count after v/type.

`answer_one` now applies the received count1/2 to those encrypted children after
building the resend and before peer/checkpoint reservation. The original root
ID, addressing/phash, skmsg bytes, distribution ciphertext and unrelated node
fields are preserved. Existing count attributes are replaced rather than
accumulated. The original first-send builder still omits count and is unchanged.
Unsupported counts or unexpected retry encryption topology refuse locally.

The correction is internal to the explicit CLI group/status comparison retry
path. It adds no normal-runtime group retry feature or new CLI/HTTP route, and
does not relax its receipt-key/higher-count/recipient/duplicate guards. Applying
count does not establish full current native group routing or key/session parity.
Root normalization conditions remain separate and unresolved.

Regression: two pairwise targets (msg and pkmsg) plus a root media skmsg; counts1
and2; stale-count replacement; one count per enc; native attribute position;
encode/decode round trip. Stripping only count must recover all original node
fields and exact original wire bytes. Original first-send omission, invalid
counts and missing/wrong root encryption are also checked. These are synthetic
Rust builder/codec tests, not native method execution or live delivery evidence.

The APK count chain is static. The new interface parser verifies the pinned APK
and class_def relation; direct/interface caller exports are retained separately.
Complete method exports avoid JADX duplicate-block warnings and are not a claim
of whole-method semantic coverage. Alternative native callback implementations,
all fan-out classifier conditions, status routing, receipt-key/base-key policies,
full ACK behavior and live acceptance remain unverified.

Validation:1317 workspace/all-feature tests passed, along with strict workspace/
all-target/all-feature Clippy, formatting and CLI-reference regeneration. Checked
DEX/interface reproduction, current APK coverage and repository hygiene verified
before push. No live retry acceptance was measured.
