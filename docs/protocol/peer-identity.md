# Peer identity changes

## APK evidence: comparison 2.26.37.71

The local comparison APK has SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
Class names below apply to this build, not 2.26.37.70.
Offsets are method-relative byte offsets from `xtask apk-smali`.

**Confirmed:** `LX/0jc.A0K` separates the saved identity and its verification
metadata. It opens a database transaction at `00b4..00c6`, serializes the new
`B7D` identity at `00f6..0100` and calls `LX/0kc.A0H` at `0102..0108`.
It compares old/new identities through `LX/0dj.A00` at `0124..012a`.
At `012c..014a`, it preserves `CUm.A00:J` and retains `CUm.A01:Z` only when
the old verified bit is true **and** the identity is equal; a changed identity
clears that bit. `LX/0kc.A0I` writes this metadata at `014a..0150`, before the
transaction commits at `0164..016a`. A changed previously verified identity
adds the address to a set at `0158..015e`; post-commit observer scheduling
uses that set and a `D3V` runnable at `0246..0278`.

**Confirmed:** `LX/0kc.A0I` names the stored fields: `mark_as_verified`
(`001e..0036`) and `mark_as_verified_action_seq` (`0036..004c`), updated in
the `identities` table with recipient ID/type/device selectors (`0050..006a`).
`CUm.A00` is an **action sequence**, not an observation timestamp. The
coordinator's `A15` delegates to this metadata update; it is not by itself an
incoming-key trust check.

**Confirmed:** `LX/0jc.A14` checks whether the address belongs to the local
account via `08Z.BL6` (`0008..0012`), applies a companion-mode condition and
logs `SignalCoordinator/saveIdentity - Not allowed to save my companion
identity as primary` (`0044..004e`) on the refused branch. The accepted
branch calls `A0K` at `003a..0040`.

**Confirmed:** the other DEX call-site found for `A0K` is `LX/DNu.run`
(`2098..209e`). Its surrounding path converts a `DeviceJid` via `B7H`, then
asserts `08Z.BL6` at `2086..2098`. This is a local-account path, not evidence
that an arbitrary remote prekey envelope should replace a pin.

**Unknown:** the full remote receive caller sequence, the producers/meaning of
all identity-save branch inputs, and observer behavior after replacement.
`A0L` (bundle processing) reads a previous remote identity and can invalidate
a session on differing identity/registration inputs, but its acceptance path
has not been completely traced. `A17` compares saved message base keys;
`A18` stores sessions. Neither establishes the remote identity trust policy.
Native fingerprint-generation wrappers were found (`A0U`/`A0V`, wamsys),
but the safety-number algorithm and display are not established here.
An exact APK string-reference search for `isTrustedIdentity` returned no
references; this does not prove the check is absent (obfuscation/native paths
may implement it without that string).

**Confirmed, subsequent trace:** the obfuscated trust adapter is
`LX/0km.BRS`, implementing `LX/0kl.BRS`. It converts the supplied address
(`0000..0008`), logs `SignalIdentityKeyStore/trusting` and returns constant
true (`0010..0056`). It does not compare a saved pin in this method.
The identity-save adapter `0km.COt` converts the supplied identity and calls
`0jc.A14` (`0028..003a`), connecting to the verification-state transition
above. This refutes the idea that the APK adapter itself has our blanket
unchanged-pin requirement.

**Confirmed:** `LX/CQc.A00`, called by bundle processing in `0jc.A0L` at
`0310..0324`, invokes that trust interface at `0014..001a`; its return value
is not used on this path. It checks the signed device key signature
(`0030..0064`) and, on the Kyber branch, the PQ key signature (`0068..00b4`),
then loads/archives session state and derives a new session. It stores the
session through `0kk.CdI` at `04ea..04f0` and saves the identity through
`0kl.COt` at `04f0..04f6`. This is a **bundle-processing** trace, not yet the
incoming prekey authentication/commit ordering. `A10`/`A11` in the coordinator
query Kyber-key inventory, and `A12` delegates a separate predicate; none of
those methods establishes a remote receive trust check.

Reproduction:

```sh
cargo run -p xtask -- apk-smali ~/Downloads/WhatsApp.apk --class 'LX/0jc;'
cargo run -p xtask -- apk-smali ~/Downloads/WhatsApp.apk --class 'LX/0kc;'
cargo run -p xtask -- apk-callers ~/Downloads/WhatsApp.apk --class 'LX/0jc;' --method A0K
```

The disassembler currently shows field opcodes `0x53`/`0x55` as unknown
mnemonics, while retaining operands. They are DEX `iget-wide`/`iget-boolean`;
the field descriptors and store column names independently identify their
types. Short backwards branch offsets may print in two's-complement hex;
interpret the signed width before deriving a target. Private scoped dumps
remain local under ignored `lab/`; no bulk decompiled tree is committed.

## Current receive behavior

**Observed, 2026-10-06:** saved-account primary login and activation succeeded.
One automatic retry got a matching server ACK, but the redelivery still could
not decrypt. Offline comparison of its prekey public identity to saved PN
and LID pins found differences in both; a group pairwise prekey also differed
from its saved participant pins. The underlying cause is **unknown**. This
does not establish a legitimate remote identity change, nor a server refusal.

**Implemented/offline:** direct and group pairwise prekey processing records
each differing `(peer, saved fingerprint, observed fingerprint)` once in the
encrypted store before attempting decryption. `identity-conflicts` lists these
records without addresses, keys, message IDs or content. Redelivery/restart
preserves the first observation time; malformed envelopes and first-use
identities create no conflict. Local SHA-256 fingerprints are diagnostics,
**not WhatsApp safety numbers**. These records are unauthenticated observations;
they do not emit the journal's `IdentityChanged` event because an actual
replacement has not been established.

Saved pins, sessions, prekeys, queued sends and prior verification remain
intact. Inspection changes nothing. The client still refuses changed pins;
verified replacement and affected-send policy resolution remain unfinished.
No extra chat send or live key replacement was performed for this work.


## Offline verification, 2026-10-07

**Observed:** replay of the original authenticated captures completed with
zero newly decrypted messages, 33 already committed and three still
unsupported. It created two distinct conflict records (two session-address
selectors; this is not a count of people). No network request or receipt was
sent. The cause remains the saved-pin mismatch. Private redacted replay and
inspection outputs remain in the saved account directory.

**Implemented/offline:** pairwise Signal failures in group messages are now
classified `Undecryptable`, like direct failures, rather than `Unsupported`.
The earlier replay used the build before this classification correction; its
counts are historical evidence, not a claim about the corrected classifier.
Regression coverage exercises the actual group-processing path and preserves
the unsupported outcome when no sender key/session is available. The normal
session can now queue the same durable once-only retry for a failing pairwise
group child. No live group retry was sent in this change.

Verification: 498 workspace/all-feature tests passed, including conflict
restart/deduplication and incoming-envelope integration, alongside the existing
Signal identity-refusal tests. Strict Clippy, formatting and repository hygiene
passed. The CLI reference was regenerated. APK trust-policy implementation
remains incomplete; these checks do not validate a replacement on the server.
