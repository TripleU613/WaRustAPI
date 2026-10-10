# Bulk prekey preparation and processing

APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
**Confirmed static:** this follows the
[bulk request/callback](bulk-prekey-refresh.md) into preparation, queued result
processing and native session construction. It does not establish live key
rotation or full native transaction/retry parity.

## Preparation is conditional user sync

The job projects its concatenated device lists to each DeviceJid.userJid through
[Ayo.A1N](apk/2.26.37.71-bulk-prekey-processing/device-user-projection.txt), then
deduplicates before Cio.A02. Thus this prerequisite concerns owner users, not
one sync request per companion device.

[Cio.A02](apk/2.26.37.71-bulk-prekey-processing/preparation-entry.txt) delegates
to A00 with a null timeout and false input. Its
[A03 selector](apk/2.26.37.71-bulk-prekey-processing/preparation-peers.txt):

- keeps PhoneUserJid values (`001c..0032`);
- excludes values selected by 0hd.A02 and 0DE.A0h (`005e..00b2`);
- removes queried 0kV.A00 rows whose 0DV.A0A is false (`00c8..0126`);
- asks 0kT.A05 with labels `requestSyncDevicesAndLidIfMissingLids` and
  `PreKeyFetch`, then removes keys present in that returned map (`0134..0162`).

**Unknown:** complete subtype predicates, missing/cache-row semantics and
0kT mapping authority. The labels are evidence of intent, not proof that every
selected user lacked a LID or that all existing mappings are trustworthy.

[A00](apk/2.26.37.71-bulk-prekey-processing/preparation-core.txt) returns a
1hI result with discriminator 1 for an empty selected set. A separate Boolean
branch can return discriminator 0, but the job's false input does not take it.
For remaining users it calls
[0ss.A0A](apk/2.26.37.71-bulk-prekey-processing/preparation-sync-request.txt)
with context 0xX.A0M, then waits on its future; null timeout selects untimed get
(`006a..0082`). The
[enum source](apk/2.26.37.71-bulk-prekey-processing/preparation-sync-context.txt)
names A0M **MESSAGE_MULTI_PROTOCOL_QUERY** (`0222..0244`). 0ss.A0A forwards
0xh.A0g, 0xo.A0L and two true inputs to A06. **Unknown:** full A06 wire/defaults,
sync response/storage semantics and host/Android scheduling. No pin/session
reset is demonstrated by these wrappers. Native untimed waiting does not
replace Rust's bounded transport timeouts or durable uncertain reservations.

## Result forwarding and processing gates

The three 1Dz wrappers forward to 1Dw.CDr. Their message constructors are distinct:

| Path | Message fields | Consumer |
| --- | --- | --- |
| [A1O/A0Z](apk/2.26.37.71-bulk-prekey-processing/success-message.txt) | what=0, arg1=74, arg2=0; obj=list; pqUpgradeDevicesRequested | 14I.BFC case 74 |
| [A15/A0M](apk/2.26.37.71-bulk-prekey-processing/none-message.txt) | what=0, arg1=75, arg2=0; obj=B76; isPqSessionUpgrade | case 75: **none**, not successful key adoption |
| [A1T/A0e](apk/2.26.37.71-bulk-prekey-processing/error-message.txt) | what=0, arg1=76, arg2=0; obj=B76 array; errorCode/PQ fields | case 76 |

[14I.BFC](apk/2.26.37.71-bulk-prekey-processing/incoming-prekey-dispatch.txt)
case 74 -> `0602` filters PQ-upgrade candidates with no PQ key (`065a..0680`).
It schedules DNe selector 9 for skipped peers, and only retained candidates
proceed. At `0732..074c`, fewer than 128 retained items select DLx **23**;
128 or more select **22**. The
[existing DLx run](apk/2.26.37.71-status-observers/post-runnable-run.txt)
case 23 -> `119c` calls 14J.A04; case 22 -> `00b2` implements the separate batch
path. This is local native scheduling, not a server batch limit.

The [existing 14J.A04 source](apk/2.26.37.71-sync-resend/bundle-processing.txt)
per candidate invokes
[CPB.A00](apk/2.26.37.71-bulk-prekey-processing/candidate-admission.txt)
(`0098..00c0`). Bjp skips the candidate; Bjo continues with null C0F; Bjn carries
C0F into processing (`0280..0294`). This is an admission boundary before session
construction; callback parsing alone does not prove acceptance.

A04 then obtains a device lock. Non-primary devices call 11T.A0D for ADV
verification; false follows the failure path and skips session construction
(`0100..015a`). The primary path skips this companion-only check. Both still
pass the admitted candidate into 0jc.A0L (`016a..01b4`). Return status zero
continues success bookkeeping; nonzero logs and dispatches failure. The same
[post-session helper](apk/2.26.37.71-bulk-prekey-processing/post-session-pq.txt)
executes before that status branch and has its own B77/PQ guard; do not reorder
it based solely on the word success.

[0jc.A0L](apk/2.26.37.71-bulk-prekey-processing/session-creation.txt) normalizes
its session address and locks it. Property-dependent paths compare saved remote
identity bytes/type and invoke archive/reset helpers when changed. It prepends
the supplied key type to raw key bytes, decodes native public keys, builds C6P
and invokes CQc.A00 (`0310..0324`). **Unknown:** complete native primitives,
archive/reset authority, all exception/status semantics and full transaction
boundaries. These paths do not authorize rotating our account identity or
silently resetting an outcome-unknown action.

## Primary hybrid admission is separate

CPB.A00 has a primary-device/property 0x6f6f gate (`0008..0034`). Its longer
branch classifies CWS encryption metadata, checks saved identity/state and
performs attestation parsing/signature checks. For example it requires a
64-byte signature, parses BP2 account type and verifies prefixed attestation
bytes with B7H.A09 (`0112..0192`). It distinguishes changed-key and same-key
failures, sticky non-E2EE state and an additional property 0x7497.

**Unknown/unported:** actual property values, C0F enum/state authority, the full
CPB transition matrix, B7H attestation semantics and the batch equivalent. Do
not take a logged fallback as general permission to skip attestation, fabricate
an account mode or bypass a challenge. Existing Rust legacy Signal/ADV guards
remain; this research is not a claim that hybrid handling is implemented.

## Missing key type: implemented correction

**Confirmed local bug:** prekey_bundle_for required `<type>`, whereas the retained
[Bot.A03 source](apk/2.26.37.71-bulk-prekey-refresh/bulk-callback-result.txt)
`00dc..00f0` defaults only an absent node to key type **5**. Present input must
contain exactly one byte (failure branch names that width). Rust already rejected
present unsupported types; the missing-node mismatch was the defect.

**Implemented/offline tested:** absence now takes the existing classic type-5
path. Present empty/multibyte/unsupported/child-tree/duplicate inputs continue to
refuse with fixed diagnostics. Rejection of types other than 5 is our supported
curve boundary; complete native support for other type bytes is **unknown**.
Existing key widths, exact recipient selection, signature verification and pin
policy remain separate and unchanged.

A failing-before test with generated, correctly signed synthetic keys stopped
at `missing key bundle field`. After the correction it parses absent and explicit
5, then successfully verifies both signatures. A forged signature still fails
verification; malformed present types still refuse without exposing the peer or
a private-value canary. No store,
account, pin, session or live request was used by that parser regression.

Operators reproduce ordinary saved evidence through `peer-keys --show`; the
request shape remains inspectable with `--preview-identity-change`. Neither
command adopts a key. Previously captured valid signed replies without type can
now be inspected under the corrected parser; their raw records are not rewritten.
Uncertain/refused actions are not replayed or reset. No new CLI flag or HTTP route
accompanies this parser fix.
Automatic notification refresh and native preparation/admission/ACK still remain
unported. The next implementation step is a durable notification-refresh flow
with the traced prerequisite and explicit authenticated identity-change policy;
full native primitive/state parity is still incomplete.

## Retention and reproduction

The [recipe](apk/recipes/2.26.37.71-bulk-prekey-processing.json) retains 16 topics /
21 files / 137,128 bytes with signatures, register flow and exception tables. DLx.run, 14J.A04 and Bot.A03
are reused from existing bundles rather than copied. The xref anchor
`pqUpgradeDevicesRequested` found four references: two in 14I.BFC and one each
in 1Dz.A0Z/A0e, identifying the consumer before tracing it.

```sh
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-bulk-prekey-processing.json \
  --output docs/protocol/apk/2.26.37.71-bulk-prekey-processing --check
cargo test -p whatsapp-api --all-features key_bundle_absent_type
```

Cached self-diff checks retention, not equivalence with another APK release.
No fresh live absent-type reply, identity transition or attestation case was
observed. Complete automated refresh remains unfinished.
