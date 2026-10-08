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
a session on differing serialized identity type/key bytes, but its acceptance path
has not been completely traced. `A17` compares saved message base keys;
`A18` stores sessions. Neither establishes the remote identity trust policy.
Subsequent safety-number research refutes the earlier classification of both
`A0U` and `A0V` as native wrappers: `A0V` uses the traced Java legacy algorithm;
`A0U` and `A0T` use native dispatch. Current scheme selection, native algorithms
and verification remain unestablished. See [safety-number evidence](safety-number.md).
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

**Confirmed, receive adapter:** the two callers of interface `0kl.BRS` are
`CQc.A00` above and `LX/Cnp.A04`. The latter handles an incoming `DRw`
prekey envelope: it loads the session at `0000..0014`, invokes `BRS` at
`0034..003a` without using the return value, and selects/promotes a matching
session by version and Alice base key (`0052..0196`). If none matches, it
loads the actual signed/optional one-time keys and constructs a new session.
Both the existing-session branch and new-session branch converge on identity
save `0kl.COt` at `0098..009e`, then nested-message decrypt `Cnp.A01` at
`00a4..00aa`, callback `Dc6.BEw` at `00b0..00b6`, session store `0kk.CdI`
at `00c6..00cc`, and optional prekey deletion (`00da..00f8`).

**Unknown:** whether the outer caller encloses the identity save and nested
message authentication in a rollback-capable transaction; this method alone
does not establish that. Do not claim the APK saves an identity only after MAC
validation: its inner ordering is the opposite. A client implementation should
still hold candidate changes in memory and commit the identity, session,
consumed prekey and event only after successful authentication, preserving the
previous identity/session history. This is an implementation invariant, not a
claim about a traced APK transaction. Trust-policy configuration and atomic
authenticated replacement remain the next implementation boundary.

Reproduction:

```sh
cargo run -p xtask -- apk-smali ~/Downloads/WhatsApp.apk --class 'LX/0jc;'
cargo run -p xtask -- apk-smali ~/Downloads/WhatsApp.apk --class 'LX/0kc;'
cargo run -p xtask -- apk-callers ~/Downloads/WhatsApp.apk --class 'LX/0jc;' --method A0K
```

The disassembler formerly showed field opcodes `0x53`/`0x55` as unknown
mnemonics, while retaining operands. It now names them `iget-wide`/`iget-boolean`;
the field descriptors and store column names independently identify their
types. Short backwards branch offsets may print in two's-complement hex;
interpret the signed width before deriving a target. Private scoped dumps
remain local under ignored `lab/`; no bulk decompiled tree is committed.

## Receive checkpoint, 2026-10-06

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


## Explicit authenticated-change policy, 2026-10-07

**Implemented/offline:** private request JSON `transport.identity_changes`
accepts `pinned` (default) or `apk_authenticated`. Old JSON defaults to
`pinned`; unknown spellings are rejected. Both `listen` and lab receive apply
this policy. Offline recovery explicitly selects it with:

```sh
whatsapp-cli --profile PROFILE lab-inbox --replay-captures --accept-authenticated-identity-changes
```

The flag requires replay and conflicts with inspection. This selects normal
APK-compatible remote peer changes; it is not operator verification of a person
or WhatsApp safety-number verification. The local account identity, own-device
messages, outgoing bundle trust, queued send IDs and uncertain reservations
are unaffected. No private live profile was changed to enable it in this step.

The new prekey identity is an in-memory candidate. Nested Signal decryption,
MAC and padding checks must succeed before a durable update. Existing sessions
are supplied to the Signal backend; our transaction independently archives
previous pin/session bytes in encrypted storage. Inbox/session/pin/prekey OCC
checks precede or share the commit; failure rolls everything back. A successful
replacement appends `IdentityChanged` and `identity-history:<event-id>` in the
same transaction as the authenticated inbox/session/prekey update. Unique event
IDs preserve repeated A→B→A transitions without overwriting prior ratchets.
History records explicitly mark the new identity **unverified**. The identity
change is inspectable through the normal durable `events` journal; unauthenticated
conflict observations remain separately inspectable through `identity-conflicts`.

First-use identities and identical pins do not emit a replacement event. Failed
MACs and default-policy refusals retain the pin/session and emit no replacement.
Direct and group pairwise replacement tests use generated registered signed
keys and actual Signal envelopes, test damaged nested MACs, inspect archived
prior bytes and verify duplicate suppression after reopening the store.
This implements the configured policy resolution described in
[the messaging design](../design/messaging.md), rather than replacing pins on
every bundle fetch. Human safety-number verification and full cross-device
verification-state synchronization remain unfinished.


**Observed offline, isolated real-state validation:** the new policy replayed
original server captures against a private copy of the saved encrypted account
store. It recovered one direct message, counted 34 already committed captures
(including its repeated envelope) and left one group capture unprocessed. The
journal contains one `identity_changed` entry. The group failure is now
`Signal one-time prekey unavailable or invalid`, replacing the old identity
refusal. This is not proof that the key can never be recovered: the group path
currently selects the PN session even when `participant` names a LID, unlike
the corrected direct path. That alias/session selection is the next diagnosis.
The original database SHA-256 stayed identical before/after every isolated
command. No network request, read receipt, delivery confirmation or plaintext
export occurred. The recovered content stays encrypted in the private copy.

Verification: 500 workspace/all-feature tests passed; strict Clippy,
warning-free rustdoc and CLI-reference regeneration passed. Regression coverage
also confirms transaction rollback removes an attempted archive/journal append
and the remote policy cannot bootstrap own-device sessions or replace saved
local PN/LID aliases. This verifies the implementation and original-capture
recovery offline; current live account policy remains `pinned`.


**Subsequent observed offline correction:** selecting the held LID pairwise
session recovered the remaining group capture in the same isolated store
(one newly decrypted, 35 duplicates, zero unsupported). See
[the group receive correction](groups.md#group-pairwise-pnlid-session-correction-2026-10-07).
The earlier missing-prekey result was a wrong-session symptom, not proof that
the account lacked every usable key. The original database remained unchanged.


## Canonical saved-account recovery, 2026-10-07

**Observed offline:** after the isolated test recovered both captures, the
saved profile selected `transport.identity_changes=apk_authenticated` in
`lab/accounts/l/request-authenticated-receive.json`. Its original request,
profile and pre-recovery encrypted database were preserved privately before
this data change. No local account key, account generation, send ID, reservation
or refusal state was reset. The original configuration remains intact; the
active profile points to the corrected request file.

Canonical offline replay recovered **two** pending payloads, counted **34**
duplicates, and left **zero** unsupported captures. The first authenticated
peer replacement archives its old pin/session and emits `IdentityChanged`;
the group reuses the corrected LID session. No receipt or network request was
sent during replay. This count refers to authenticated payloads, not two new
human replies. A subsequent bounded receive-only live check is recorded below
when complete; do not infer automatic read-receipt success from offline replay.


**Observed live follow-up:** the canonical account's 45-second receive-only
session delivery-confirmed both recovered captures on duplicate redelivery.
A 20-second follow-up session acknowledged both selected saved-message read
receipts. The receive/config correction did not require local identity rotation
or replaying any chat send. Handset confirmation and fresh-message auto-read
remain unknown; see [the read recovery record](messaging-transport.md#authenticated-duplicate-auto-read-correction-2026-10-07).

## Outgoing held-session identity guard (2026-10-07)

**Confirmed (APK 2.26.37.71):** `LX/0jc.A0L`, offsets `0044..008a`,
reads the held session remote public identity, separates its first type byte
from the key bytes, compares the type to `v33` and key bytes to `v31`, then
invalidates the session through `A0E`/`A0s` (`0096..00a8`) when changed.
**Refuted:** the earlier description of this comparison as identity/registration
inputs. `v33` is the serialized curve-key type byte, not registration ID;
registration is decoded separately later (`01a2..01ac`). Full replacement
acceptance and branch producers remain **unknown**.

**Confirmed (implementation inspection):** `signal/direct.rs::prepare_payload`
previously ignored a supplied bundle whenever a session already existed. This
could encrypt using a stale peer pin/session even after a key query returned
a changed identity. The common preparation path now refuses mismatched pins
before using or advancing any ratchet. The normal outbox stores the exact reply
in encrypted checkpoints (saving does not authenticate its key/signature),
leaves the original intent queued with no send attempt,
and marks its existing query reservation `identity_changed`. Receive continues;
restart does not repeat the query or send. `outbox-suspended` exposes only the
local intent ID and hashed peer selector. Cancelled intents disappear from this
inspection. The error display contains no contact address.

**Offline:** generated, independently signed changed bundles are refused with
an existing real Signal session; matching bundles still advance that original
ratchet. A normal-outbox fixture verifies unchanged pins/sessions, preserved
reply, no wire/send reservation, restart protection and cancellation. This is
an implementation guard, not a WhatsApp server refusal. **Unknown:** canonical
PN/LID outgoing pin agreement, policy-qualified outgoing replacement, and live
outgoing validation after the incoming identity recovery. No new chat was sent.

**Observed locally:** `outbox-suspended --profile l --json` returned an empty
list from the canonical saved account on 2026-10-07; no key query or send was
issued. Private evidence: `lab/accounts/l/outbox-suspended-2026-10-07.json`.
**Confirmed (implementation):** this inspector uses only `ProtocolStore::read`,
but `ProtocolStore::open` itself opens/commits a write transaction to validate
metadata and ensure tables exist (`store/mod.rs::open`). The pre/post file hash
changed during this inspection, so raw file-byte equality is not a valid
read-only protocol-state assertion for this API. That observation does not
establish a peer-key, account-identity or send-state change. The offline fixture
asserts those logical records directly; byte-immutable opening remains absent.

## Primary PN/LID identity inspection (2026-10-07)

**Confirmed (implementation):** incoming processing stores
`authenticated-lid:<LID>` only in the authenticated inbox commit, and refuses
changing an existing mapping (`inbox.rs::process_direct`). Replacement history
is written in that same transaction by `identity_conflicts::archive_replacement`;
its `previous_identity` and `current_fingerprint` remain encrypted. These are
our local provenance records, not an APK safety-number format or user approval.

`peer-identity --chat <primary-PN-JID>` now reads one transaction's view of the
PN pin/session and every mapped primary LID. It reports diagnostic SHA-256
fingerprints, whether each pin agrees with its current session remote identity,
whether each LID pin agrees with the PN pin, and whether saved authenticated
history connects the PN pin to that current LID pin. Missing evidence is null
or false; multiple mappings are shown without selecting one. Companion-device
inputs are refused. No pin, ratchet, journal or send record is written; store
opening still has the documented metadata commit. This command is CLI-only.

**Offline validation:** generated signed bundles create independent real Signal
sessions for old PN and updated LID identities. The inspector distinguishes
cross-alias mismatch from within-session mismatch, requires history to match
the current LID pin, preserves all mappings across pagination/restart, marks
missing pins unknown and exposes no peer JID. These tests establish inspection
behavior, not an actual remote identity change or permission to replace keys.

**Confirmed (implementation correction):** the previous `outbox-suspended`
command omitted the CLI transport feature gate. Both its enum variant and
dispatch are now gated, matching the transport-only module; default-feature
verification is included alongside the all-feature suite.

**Confirmed (APK 2.26.37.71, outgoing trace revisited):** `LX/0jc.A0L`
reconstructs the fetched serialized identity using the supplied type byte at
`00b4..00d6`, reconstructs the supplied signed device key at `0152..017a`,
and passes them with its signature into `C6P` (`023e..0256`) and then
`CQc.A00` (`0310..0324`). The signature verification/session derivation/save
ordering is documented above. **Inferred:** valid authenticated directory
bundle processing can refresh a remote session without an earlier incoming
PN/LID transition; the traced trust adapter accepts the supplied remote key.
The new inspector's history flag must therefore not be presented as an APK
requirement for outgoing replacement. It is evidence about our held aliases.
**Unknown:** complete caller/own-device policy boundaries and the non-classic
PQ branch. Outgoing implementation must keep an explicit policy, verify the
signed bundle, preserve prior state and refuse local-account replacement.

**Observed locally (canonical saved account):** `peer-identity` inspected the
contact of the recovered direct text. Redacted result: `{"phone_session_pin_matches": true, "linked_alias_count": 1, "linked": [{"session_pin_matches": true, "pins_agree": false, "authenticated_transition_from_phone_pin": true}]}`.
Private evidence: `lab/accounts/l/peer-identity-2026-10-07.json`.
No query, send, replacement or protocol-state write was performed.

## Explicit signed outgoing bundle policy (2026-10-07)

**Confirmed (APK 2.26.37.71):** `LX/0jc.A0L` compares the remote identity
before invalidating its held session, reconstructs the fetched identity and
signed device key, and calls `LX/CQc.A00`. The latter checks the signed device
key signature (`0030..0064`) before deriving/storing the new session and saving
its identity (`04ea..04f6`). `0km.BRS` returns true and does not enforce our
unchanged-pin default. This is normal peer key processing, not a registration
reset, server-refusal bypass or account identity rotation.

**Implementation:** `transport.outgoing_identity_changes` independently selects
`pinned` (default) or `apk_authenticated`. Incoming recovery does not enable it.
Under the outgoing opt-in, a changed primary remote directory bundle must pass
its signed device key signature check; its response is supplied by the actor's
authenticated Noise connection. The candidate ratchet is derived in memory.
The original pin/session are archived with `authentication=signed_directory_bundle`
and `verification=unverified`; `IdentityChanged`, new pin/session and the same
intent's one send reservation commit atomically after original-state checks.
Old incoming history is still supported and new incoming archives explicitly
say `authentication=incoming_message`. The PN/LID inspector's incoming history
flag excludes directory archives. Own PN and saved login PN/LID aliases remain
pinned under both policies.

Invalid signed bundles leave the original intent queued, preserve the reply and
old pin/session and expose reason `invalid_signed_bundle` through
`outbox-suspended`; receive continues. Explicitly selecting the original
`outbox-send` JSON under the outgoing opt-in can resolve a previous
`identity_changed` suspension from its saved reply, without a second key query,
new local ID or reset. General `listen --send-outbox` retains its existing
eligible-list behavior and does not select suspended intents. Unknown/refused
queries, invalid signatures and in-flight/uncertain sends never resume. Suspensions
without an observed pin snapshot cannot resume. A saved reply also cannot roll
a newer peer pin back: a changed current pin must already match that reply. The
returned message is reserved before socket write, so a crash before writing
still leaves uncertainty and never causes automatic retransmission.

**Offline:** generated signed bundle tests decrypt the refreshed outgoing
prekey message with the new peer's real Signal keys, check old-key/session
history, one identity event and send reservation, account-key preservation,
restart protection, pinned/default refusal, invalid-signature refusal and own
PN/login-alias guards. The same original suspended intent can resume under
explicit policy with its saved reply; unanswered queries cannot. **Unknown:**
live outgoing directory key agreement and sending after the recovered peer's
identity change. The canonical profile has not enabled the new outgoing policy;
no chat or peer key replacement was performed on that account.

**Diagnosability:** `outbox-send` can inspect a suspended intent through the live
client's store handle, rather than reopening a locked database or waiting for an
ACK that cannot arrive. It returns the static pinned, invalid-signed-bundle or
own-identity-pinned reason while preserving the original intent. This diagnostic
makes no network request. Own-policy refusals are `own_identity_pinned`; a valid
identity-suspended reply under an enabled remote policy gets time to resolve.

**Negative-path correction:** a pinned mismatch can contain an invalid signed
bundle, since pinned policy refuses before attempting signature admission.
Later explicit resume now changes that same saved suspension to
`invalid_signed_bundle` without losing its original reply/observation time,
reserving a send or ending receive. The signed-policy fixture covers this
pinned-to-enabled transition, as well as stale-current-pin refusal.

The directory reply is also retained in encrypted query checkpoints after a
successful send reservation (`answered`), preserving the fetched signed-key
material for later rechecking. Retention does not make an in-flight/uncertain
send replayable. Selected resume preserves the original observation time and
reply across refusal classification; no secret-bearing reply is printed.

## Directory key inspection without chat sending (2026-10-07)

**Confirmed (inherited comparison evidence, APK 2.26.37.70):**
`LX/0y0.A00` `1786..193a` constructs `iq/get`, `xmlns=encrypt`,
`key/user/jid`; an earlier live result supplied identity, signed key and an
optional one-time key. Source/offsets are recorded in
[messaging transport](messaging-transport.md).
This builder attribution has not been independently re-anchored to 37.71.
**Confirmed (APK 2.26.37.71):** `LX/CQc.A00` `0030..0064` checks the signed
device key before session derivation; the build separation is intentional.

`peer-keys --chat <primary-PN-JID>` queues one directory IQ offline through the
existing durable action queue. The action and per-peer selector commit together;
concurrent/restarted repeats return the same ID. A session reserves it before
socket write, and unknown writes do not repeat. `--show` reads the saved response
and current pins in one transaction: queued/outcome_unknown/server_refused/
invalid_reply/invalid_signature/verified_bundle, numeric refusal code, signature
validity, diagnostic fingerprint and every PN/LID pin match. No pin, session,
receipt, event or chat send is written during inspection. Raw key bytes,
contacts and server error text stay in encrypted storage. This is CLI-only.

An explicit `--refresh` requires an earlier completed valid signed result;
missing, queued, unknown, refused or invalid evidence cannot be reset. Prior
actions/replies remain encrypted. General action reply handling now recognizes
`list/user/error` even without `type=error` on the user, matching the already
recognized recipient-refusal shape in `signal/direct.rs::prekey_bundle_for`.
The fixture numeric codes 480/503 test round-trip preservation, not inferred
server-internal meanings. Unrecognized outer replies remain invalid_reply.

**Offline:** generated independently signed bundles prove signature and pin
comparison behavior. Four simultaneous queue calls produce one action;
restart/write-cut reservations never repeat; a verified refresh preserves old
reply evidence; invalid signatures, outer and per-user refusals block refresh.
Inspection leaves logical pins, sessions, outbox and checkpoints unchanged and
omits peer identifiers/error text. Store opening still has its documented
metadata commit. **Unknown:** the recovered peer's current directory identity;
no live query has yet been issued by this new command.

**Observed live (2026-10-07):** preflight showed no eligible chats or existing
queued actions. One peer directory action was queued; repeating the command
returned its original ID. A 25-second receive-only session logged in/activated
successfully and settled the action. The response supplied a valid signed bundle:
`verified_bundle`, `signature_valid=true`, `phone_pin_matches=false`, and the
one authenticated mapped LID `pin_matches=true`. Pre/post `peer-identity`
summaries were identical. This establishes that the current directory identity
matches the recovered LID and differs from the held PN pin. It is not human
verification or proof of sending with that identity. No chat was sent and no
outgoing pin was adopted. Private evidence: `lab/accounts/l/peer-keys-live-2026-10-07.log`,
`peer-keys-result-2026-10-07.json` and the paired pre/post pin summaries.

One new private outgoing test JSON is prepared with a stable ID and text
"TeleWhatsApp identity recovery test.", plus an independent config enabling the
outgoing policy. It is not queued or sent. All previously saved test sends are
acknowledged and cannot be replayed. The new chat send awaits explicit operator
authorization, while other protocol work may continue.

**Attribution correction:** `MessageRequest::recipient` still repeated the old
unqualified `CPQ.A00` address-producer citation. Its comment now points to the
original authenticated LID captures and the recorded 37.71 negative trace in
[groups](groups.md#group-pairwise-pnlid-session-correction-2026-10-07), without
claiming a producer re-anchor. Behavior is unchanged.

**Confirmed (implementation review correction):** the new transaction-scoped action helper
initially moved the existing public queue builder callback inside the store
write lock. That would break callbacks that update local state. The public
builder keeps its original outside-lock behavior; only the explicit transaction
helper builds under its caller's transaction. A bounded callback-reentrancy
regression and concurrent once-only directory test cover both paths.


## Verification metadata writers and rotation guards (37.71)

**Confirmed/static**, APK 2.26.37.71, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
All offsets are method-relative bytes from the existing `xtask apk-smali`.
This follows [scan-result tracing](safety-number.md#java-comparison-selection-and-scan-result-action).
No saved identity, verification bit or sequence was changed by this research.

Exact caller searches now resolve the metadata writers:

| Target | Callers |
| --- | --- |
| 0kc.A0I | 0jc.A0K, 0jc.A15, Cg3.A00 |
| 0jc.A15 | CSj.A00, DNu.run |
| CSj.A00 | Two sites in 1gz.A00 |
| Cg3.A00 | Two sites in 6Am.invokeSuspend |
| Cg3.A01 | 5XZ.onClick, 5Xc.onClick |

The fresh `0kc.A0I` disassembly resolves the earlier opaque reads: CUm.A01
is read with iget-boolean at 0022..0026 and A00 with iget-wide at 003a..003e.
It updates existing `identities` rows selected by recipient ID/type/device,
writes both `mark_as_verified` and `mark_as_verified_action_seq`, and returns
whether the database update changed any row (006a..007a). This leaf has no
key-equality or sequence-order guard: callers must supply those conditions.
`0jc.A15 0000..002e` normalizes the address, locks it and delegates to this
leaf. It is not a universal verification/trust decision.

**Confirmed/static, local sender flow `Cg3.A00`:** its log names identify
MarkAsVerifiedFlowController and MarkAsVerifiedSender. It refuses a null
rendered identity key (0000..001c), the companion-device branch (0046..006c)
and a target identified as self (006c..008a). It addresses the target's primary
device through B7H.A04 (008a..009a), loads the current identity via A0W, and
refuses absent identity or inequality with the supplied rendered B7D
(00aa..00e6). Thus a stale UI identity must not authorize a different key.

At 00ee..0104 it reads the prior CUm action sequence, defaults to zero if
metadata is absent (012a..0130), and adds one. It constructs CUm with this
sequence and the requested mark boolean, then locks the address at
0116..0128. **Under that lock**, 0130..0164 reads the current serialized
identity using 0kc.A0J and compares it with the supplied key's B7E.A00 bytes.
Missing row and rotation produce distinct refusal branches; only equality
reaches 0kc.A0I at 0164..016a. After closing the lock, the success branch
inserts verification events through CSj.A01 (019a..01b0) and builds companion
fan-out. A second key check is required because the identity could change
between rendering, the first check and acquisition of the write lock.
**Unknown:** sequence-overflow policy beyond the traced add-long/2addr and
complete concurrency semantics of the surrounding sender monitor; no global
atomicity claim is made for the subsequent event/fan-out operations.

The fan-out list is built from HxZ.A0A DeviceJids at 01c4..01f0. An empty list
logs `no companions — skipping fanout` and returns at 01fc..020e: lack of
companions does not undo the prior local metadata update. With companions,
020e..02b4 builds BWE with target JID, requested boolean, raw B7E.A01 key and
sequence. It creates per-device BnY peer messages at 0302..031e, attempts
0ob.A02 insertion, and logs an insertion failure per device at 0336..034e.
The value 0x8c is the domain message type passed to Bn4, not an established
ProtocolMessage enum. Complete peer serialization/transmission remains untraced.

`6Am.invokeSuspend` passes true to Cg3.A00 at 0490..0496 and false at
04ec..04f2, then schedules result handling. `Cg3.A01` constructs an explicit
confirmation dialog with an AES positive-button callback (0034..0058) and a
negative button (0058..0066). Exact callers locate 5XZ.onClick and 5Xc.onClick,
but those click handlers, the dialog callback and complete connection to scan
success remain untraced. This evidence establishes the guarded writer, not
permission to mark our live peer verified without human comparison.

**Confirmed/static, inbound own-account action `1gz.A00`:** its peer branch
checks the source DeviceJid's UserJid against the local account through BO0 at
017c..01be. BnY is dispatched at 425a..4272. The action handler's second mode
check at 42b4..42c8 logs `dropping inbound MarkAsVerifiedAction on primary`.
The accepted companion branch requires payload and all four BWE presence bits
(42c8..42fc), a parsed LID (42fc..4318), a non-self target (4318..4338),
exactly 32 key bytes (4338..436e) and actionSeq >=1 (436e..43a8).
It loads primary-device metadata and rejects sequence <= saved sequence
(43ac..4424). For mark=true, a differing/missing current key can be saved
through A14 before marking and device invalidation (4456..44c6). For
mark=false, only a matching key reaches the applier (44ca..44fc); a mismatch
is a no-op (4500..4508). This trusted own-account companion action is distinct
from accepting an arbitrary remote prekey identity or a remote verification bit.
**Unknown:** complete transport authentication/linked-device provenance and
atomic application boundaries; no new receive trust policy is implemented.

`CSj.A00 0000..004a` builds CUm, calls A15 and inserts events only if an
identity row was updated; failure logs that event insertion is skipped.
`CSj.A01` chooses event constructors P9R/type 0xe5 for mark and P9S/type 0xe6
for unmark (004a..0068, 0136..014c), with additional group event branches.
Full group selection and event rendering remain untraced.

**Confirmed/static, history path `DNu.run`:** the branch beginning
099a..09c8 takes a UserJid and derives its primary address. The surrounding
identity-save mode depends on feature 0x6f29 and other inputs. At 0a14..0a52,
BZK.identityVerification presence is required; BT5 must contain both verified
and actionSeq, with sequence >=1. At 32d0..3302, verified=true additionally
requires contactPrimaryIdentityKey presence and length 32; false bypasses that
particular key-presence/length guard. The branch writes CUm through A15 at
3302..331c. This does not establish a universal stale-sequence guard for history:
that guard is confirmed above specifically for the live peer action handler.
History authenticity, key-save branch inputs and complete ordering remain unknown.

**Confirmed/static, descriptor schemas:** `apk-proto-fields` decodes
BWE.dynamicMethod's descriptor at 005c..006a and BT5.dynamicMethod's at
0048..0056 (field-name strings start at 0036 in both classes). BZb.dynamicMethod identifies
ProtocolMessage field 32 as markAsVerifiedAction; BZc identifies Message field
12 as protocolMessage. The nested layouts are:

| Class | Tag | Field | Type |
| --- | --- | --- | --- |
| BWE | 1 | userJidString | string |
| BWE | 2 | verified | bool |
| BWE | 3 | verifiedIdentityKey | bytes |
| BWE | 4 | actionSeq | int64 |
| BT5 | 1 | verified | bool |
| BT5 | 2 | actionSeq | int64 |

Schema optionality does not remove the handler's presence/size/mode guards.
**Negative/static:** BZc has no direct markAsVerified field; it is nested under
ProtocolMessage. An exact BnY class-definition dump finds no class in the local
APK despite the references above. Its absence from the DEX definitions is not
absence of the referenced protocol capability; BnY's conversion remains unknown.

The delayed UI boundary is now resolved: activity constructor 0216..0224
assigns A0q to L52 selector 1, whose run branch 09d8..0a9a removes focus,
hides and fades the result view. JoG.onAnimationEnd selector 0 at 0024..004a
hides the scanner and invokes the supplied Runnable, already mapped to
L54 selector 4 -> A18. These callbacks perform presentation, not verification
metadata writes. This closes those previous unknowns without inferring that
human verification is absent elsewhere in the APK.

Private reproduction artifacts retain writer/caller lists, CSj/DNu/1gz/Cg3
method dumps, refreshed database A0I, 6Am coroutine, BWE/BT5 descriptor dumps
and field outputs, BZb/BZc field outputs, L52/JoG callbacks and the negative
BnY dump under `lab/apk-2.26.37.71/`. Reproduce with existing `apk-callers`,
`apk-smali --class 'LX/Cg3;' --method A00`, and
`apk-proto-fields --class 'LX/BWE;'` against the pinned comparison APK.

Validation: independent BWE schema extraction reproduced the exact saved
field output; CLI reference regeneration/test and repository hygiene passed
(294 local links). These are static research checks, not a live verification
action or implemented account-state transition.


## Local verification control and captured identity (37.71)

**Confirmed/static**, comparison APK 2.26.37.71, same APK SHA-256 as above.
Offsets are method-relative bytes. This resolves the local human-action
boundary without claiming that the client now implements it or the current
account has been verified.

`6Ac.invokeSuspend 0a36..0a8a` receives a 4AH state containing the LID,
rendered B7D identity and boolean. It renders one of two labels and installs
`5XZ(activity,lid,key,boolean)` on the visible text control. The exact constructor
search found only this invocation (classes4.dex offset 0x6d0f20). Its state
producer is `DUm.invokeSuspend` selector 15 (0638..073c): feature 0x729f is
required at 0652..0662; the companion-device branch, non-0ZY target and self
target are excluded at 0662..0696. It also runs 1lL.A00 and checks predicates
1lL.A03/A02 (0696..06e0); the follow-up below resolves their static inputs. It derives
the primary address, loads saved CUm.A01 through A0X (06f8..071a), loads B7D
through A0W (071a..072a), then returns 4AH with both. Missing metadata defaults
the boolean to false. `1xm.A1Q 0000..000c` is exactly the integer ==1 test,
so the control's boolean comes from the saved verified bit, not scan success.
No complete snapshot atomicity is proven for these separate reads.

`5XZ.onClick` reads that captured boolean at 0000..0004. True takes
0028..005a: it schedules 6Am selector 6, whose branch at 04cc..04f2 calls
Cg3.A00 with **false** (unmark). False takes 005a..00aa: it invokes Cg3.A01
with the captured LID/key to open the mark confirmation dialog. Cg3.A01's
positive listener is AES selector 3; `AES.onClick` reaches its default branch
000a..0040, which schedules 6Am selector 5. That branch calls Cg3.A00 with
**true** at 0478..0496. The controller's earlier key-rotation checks therefore
use the identity rendered on the control, rather than silently reselecting a
new identity after the operator confirms.

`5Xc.onClick` has a parallel CompareNumberBottomSheet branch at 000c..007a:
its captured true boolean schedules selector 6 (unmark); its false branch
at 0172..019c invokes Cg3.A01 (mark confirmation). Its other selector branch
belongs to 48w and is not verification evidence. The complete bottom-sheet
state producer remains untraced. Neither verified-control branch requires the
A18 scan-result flag. This is an explicit local mark/unmark workflow; it does
not establish an automatic scan-to-verified state transition.

**Implementation finding:** the static trace exposed missing names for DEX
opcodes 0x5c and 0x5a in the development disassembler. Their operands/widths
were already decoded as format 22c; they appeared as opaque instructions.
The decoder now names **iput-boolean** and **iput-wide**, as specified by the
[Android instruction table](https://source.android.com/docs/core/runtime/dalvik-bytecode).
An independent literal regression fixture uses APK classes7.dex bytes
`5c 16 a8 29` at 0x5fc138 (Cg3.A00 025c) and `5a 13 a4 29` at 0x5fc180
(Cg3.A00 02a4). It checks mnemonics, registers v6/v1 and v3/v1, two-code-unit
width and byte extents, with field indices intentionally unresolved in the
minimal test DEX. The actual APK field references are verified separately by
re-rendering the method with the rebuilt utility. These bytes are public APK
code, not account identifiers or verification values.

**Unknown:** feature 0x729f runtime state, the contact-state value's meaning
and actual exclusion-predicate inputs for this account,
complete device-key/fingerprint inputs, companion fan-out protocol serialization,
and UI result callback behavior after writer failure. Our account client still
has no human verification commit API. Existing replacement history labels the
new identity unverified; it does not prove an implemented editable verification
metadata record. No live mark/unmark or identity replacement was performed.

Private artifacts retain `safety-number-local-click-5XZ.smali`,
`safety-number-local-click-5Xc.smali`, `safety-number-confirmation-positive.smali`,
`safety-number-local-toggle-producers.txt`, `safety-number-verification-ui-render.smali`,
`safety-number-local-verification-state.smali`,
`safety-number-verification-render-state-callers.txt` and
`safety-number-verified-bool-test.smali`. Exact 4AH constructor search found
one site in DUm.invokeSuspend. Existing APK instruction dumps remain preserved;
the corrected writer is saved separately rather than overwriting its source trace.

Validation: rebuilt disassembler rendered the actual Cg3.A00 verified and
actionSeq stores with their resolved fields/registers. Formatting, 530
workspace/all-feature tests (zero failed/ignored), strict workspace Clippy,
CLI reference regeneration/test and repository hygiene passed. This changes
research tooling only; no live verification claim or account mutation occurred.

## Verification contact exclusions (37.71)

**Confirmed/static:** the same comparison APK 2.26.37.71 (SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`)
resolves DUm selector 15's remaining 1lL predicates. These are local UI gates,
not server refusals or permission to synthesize contact/attestation state.

| Predicate | Exact static behavior | Evidence |
| --- | --- | --- |
| 1lL.A00 | Resolves A07's provider and discards the result; no contact mutation is visible in this method | 0000..0010 |
| 1lL.A03 | Returns false for null, otherwise the truth value of A04, which calls 0bo.A02 with its third argument true | A03 0000..0016; A04 0000..0028 |
| 1lL.A02 | Requires non-self UserJid; loads contact through 0YA.A0C; returns true only for non-null 0DV.A0D.A0J whose 0lr.A01 returns true | 0000..0054 |
| 0lr.A01 | Tests its integer A01 for equality to 2 | 0000..0010 |

The contact load in A02 is dereferenced without a visible null guard before
reading 0DV.A0D. The semantic name and producer of 0lr's value 2 remain
**unknown**; this trace does not identify it as a hosted flag, verified bit,
ban, or server challenge. Provider initialization side effects in 1lL.A00
also remain unknown. The separate 1lL.A01 has an optional company-name branch
and is not called by this DUm path; its existence does not classify the target.

**Confirmed/static, hosted predicate:** 0bo.A02 first excludes self
(0000..0022). Its PN branch (0022..00fa) may map a PhoneUserJid through
0Xw.A0E depending on a lazy Boolean and other predicates. On a usable target,
it locks A0B, initializes the sets when needed, then returns true if A01
contains the target; alternatively, with the passed third argument true,
A00 membership requires feature 0x6f6f (0100..0180). Thus 1lL.A03 checks both
sources, not just a field on a JID. PN-mapping predicates and current feature
values remain unknown; DUm already requires the LID class 0ZY before this call.

0bo.A01 initializes A01 from 0bo.A00 and A00 from saved Signal identities
(0000..0160). The first loader's SQL joins user_device_info to jid by
user_jid_row_id and selects raw_string for **account_encryption_type = 1**;
when its supplied Boolean is true it additionally requires **type != 0**.
The query labels are GET_HOSTED_ACCOUNT_JIDS and GET_HOSTED_LID_ACCOUNT_JIDS
(0bo.A00 005c..0096). The branch includes main-thread diagnostics, not a
network fetch. The complete row parser below 0bp.A01 is untraced.

The second set is empty without feature 0x6f6f. With that feature it calls
0kc.A0D(C0F.A03), converts saved B76 addresses through B7H.A00 to DeviceJid
userJid values, omits null conversions, and, when the lazy Boolean is true,
filters out the PN type through 0DE.A0e (0bo.A01 0098..0152). The two sets
are installed under A0B; an observer is registered once while the feature is
active (0152..019e). Its AU2 callback/invalidation behavior is **unknown**.

0kc.A0D 001c..0040 queries existing identities with:

```sql
SELECT recipient_id, recipient_type, device_id FROM identities
WHERE account_encryption_attestation_type = ?
```

The bound value is C0F.intValue. C0F.<clinit> 0028..003c identifies A03 as
**HYBRID_E2EE = 2**, and its constructor 0000..000c stores the third argument
in intValue. Other values are NOT_SET=0, E2EE=1, NON_E2EE=3. These database
attestation values are distinct from the native fingerprint QR HostedState
enum (E2EE=0, HOSTED=1); reusing one numeric enum for the other is refuted.
This is a stored-type query, not proof that an attestation was authenticated
or that our current client has the necessary hosted-device inventory.

Reproduce with the existing offline CLI:

```sh
target/debug/xtask apk-smali ~/Downloads/WhatsApp.apk --class 'LX/1lL;'
target/debug/xtask apk-smali ~/Downloads/WhatsApp.apk --class 'LX/0lr;' --method A01
target/debug/xtask apk-smali ~/Downloads/WhatsApp.apk --class 'LX/0bo;'
target/debug/xtask apk-smali ~/Downloads/WhatsApp.apk --class 'LX/0kc;' --method A0D
target/debug/xtask apk-smali ~/Downloads/WhatsApp.apk --class 'LX/C0F;'
```

Private artifacts under lab/apk-2.26.37.71 retain the scoped
verification-contact-gates, verification-contact-kind,
verification-hosted-predicate, verification-hosted-set-producer,
verification-hosted-device-set, verification-hosted-identity-query,
verification-attestation-enum and verification-attestation-enum-constructor
smali outputs. No account records, verification metadata, key inventories or
feature flags were changed, and no live request was made.

Validation of this research checkpoint: generated CLI-reference check,
OpenAPI library-symbol check, staged repository hygiene (299 local links)
and diff whitespace checks passed. No Rust runtime behavior changed, so the
workspace suite was not repeated; this is static evidence, not live validation.

## Retained device identity-key coverage

**Implemented/offline, motivated by confirmed 37.71 evidence:**
`peer-identity --chat <primary-PN-JID>` now includes `device_key_coverage` for
the PN pin and each authenticated primary LID mapping. Its nullable summary
counts saved E2EE retained devices and valid saved identity pins, lists missing
pins by SHA-256 session selector, and carries saved `companions_verified`,
`hosted_devices_skipped` and `notified_hash_matches`. Null means no saved list;
it does not manufacture a primary-only list from a held session.

APK JhJ.A0Z 003a..006a requires an identity map covering every selected peer
device before fingerprint generation, and KXj.A02 conditionally excludes
hosted device classes. [Inventory evidence](safety-number.md#peer-device-inventory-and-scheme-inputs-3771)
records those gates. Our E2EE device-list implementation already excludes
hosted devices and can retain only the primary when companions cannot be
verified. Therefore even `pinned_devices == retained_devices` is **not** an
assertion of complete APK inventory, fresh directory state, selected scheme,
authenticated attestation, matching safety numbers, or human verification.
A saved notification mismatch remains visible even with every retained pin.
No new keys are fetched, no pin or ratchet is advanced, and no number is
calculated from this diagnostic snapshot.

The list and all its pins are read within the same logical transaction as
PN/LID/session/history inspection. Malformed pins, duplicate/wrong-owner
entries or a list lacking its primary are diagnosed as corrupt local state;
these are local consistency checks, not WhatsApp server requirements. Counts
refer to retained device entries, not unique key values. Public API consumers
can inspect the same `PeerPinState.device_key_coverage` output. Old serialized
inspection values without this optional field still decode.

Offline tests cover no list, a primary-only fallback with all retained keys,
a signed companion list with a missing pin and notification mismatch, adding
the missing pin, restart stability, redaction, byte-identical logical tables
before/after inspection, malformed pins, duplicate entries and absent primary.
No runtime scheme or current account completeness is guessed.

**Observed/offline saved-account check (2026-10-07):** the rebuilt CLI was
run on an isolated private database copy with the original account/master key
and existing contact; it exited zero. PN coverage was null. Its one mapped LID
had retained_devices=1, pinned_devices=1, zero missing pins,
companions_verified=false, hosted_devices_skipped=0 and
notified_hash_matches=null. This is an all-pinned **primary-only fallback**, not
proof that the contact has no companions or that fingerprint inventory is
complete. The original encrypted database's SHA-256 matched before/after the
check. No network operation, original database open, identity replacement or
verification commit occurred. The isolated copy and redacted stdout/stderr are
retained privately in lab/accounts/l/identity-device-coverage-check.

Validation: 536 workspace/all-feature tests, strict all-target/all-feature
Clippy, generated CLI reference and formatting passed. Tests include the
new coverage/regression matrix and existing PN/LID transition inspection.

The additional old-JSON compatibility assertion passed in a targeted rerun:
missing device_key_coverage decodes to null. Final strict Clippy,
warning-free all-feature library rustdoc, whitespace and repository hygiene
(301 local links) also passed. No default-feature workspace rerun was needed
for these transport-feature-only runtime changes.

## Local manual verification metadata (2026-10-07)

**Confirmed (static APK 2.26.37.71), implemented and offline tested:**
`Cg3.A00` rechecks the displayed serialized identity under the address lock
(`0130..016a`) before writing `CUm(sequence, requested_bit)`; its explicit
mark/unmark advances the local sequence (`00fa..0116`). The mark control is
independent of scan success. `0jc.A0K` (`012c..0150`) preserves `action_seq` and
clears an existing verified bit on identity change. These are the same build
and source artifacts attributed in the writer/control sections above.

`transport::peer_verification::set_local` implements a primary remote-LID local
human decision in one encrypted-store transaction. Its input is an expected
SHA-256 of the serialized pin (a diagnostic selector, **not a WhatsApp safety
number**), expected action sequence, and requested boolean. It requires saved
local account/Signal identity and a known login LID, excludes own LID/account
and own Signal key, and refuses missing/corrupt pins. The stored metadata binds
the actual serialized identity. `peer-identity` reports nullable `verification`
with `verified`, `action_sequence` and `identity_matches_pin`; null means no
local decision. A stored mark whose identity no longer matches reports false.

Reproduction: put `peer`, `expected_fingerprint`, `expected_action_sequence`
(zero for a null inspection), and `verified` in private JSON, chmod 600, then
run `whatsapp-cli --store PRIVATE_DB --account ACCOUNT --key-env KEY_ENV
peer-verification --config PRIVATE_JSON`. The peer is a canonical plain primary
`@lid`; obtain its pin selector and current metadata from `peer-identity` for
its saved PN mapping. This command records an explicit human decision: it does
not compute or compare a safety number. Output contains only metadata; no raw
peer, key, input JSON or parse-error value is printed. It opens an existing
store, never creates an account and does not transmit anything.

**Local policy, not an APK/server requirement:** the expected action sequence
is compare-and-set checked with the pin inside the transaction; stale repeats
refuse. Signed-int64 exhaustion refuses rather than wrapping. APK overflow and
a universal sequence CAS guarantee remain **unknown**. Unknown local LID is a
fail-closed own-account guard in our tooling. Config bounds (64 KiB), permissions
and canonical JID restrictions are our operator policies.

Both authenticated incoming and signed-directory replacement archival paths
clear existing metadata in the same transaction as identity history/event and
pin/session installation, without advancing the decision sequence. Rotation
A→B→A does not restore A's old mark. Synthetic tests cover both archive entry
points, rollback, restart persistence, mark/unmark, stale decisions, missing/own
inputs, exhaustion, and unchanged pin/session records after a manual decision.
These tests are local, not a live identity-change or human-comparison result.

**Unknown / unimplemented:** automated QR comparison, complete current APK
fingerprint selection, local verification observer events, and companion
verification-action serialization/fanout remain unfinished. No real account has
been marked/unmarked by this implementation and no server verification is
claimed. A human mark is never authority to accept an unauthenticated new key.


Saved device coverage now distinguishes historical verification from current
primary-key authority and authenticated changes queue a transactional refresh.
[Authority binding and limits](cross-device-sync.md#signed-device-authority-and-authenticated-refresh-2026-10-07).


## Exact directory bundle selection (2026-10-07)

**Confirmed static context**, APK 2.26.37.71:
[B7H.A06](apk/2.26.37.71-core/sender-key-address-build.txt) preserves recipient
category and DeviceJid.getDevice when constructing a session address. Existing
CQc.A00 signed-key checks (`0030..0064`) still apply before new session derivation.
These sources do not establish the APK's malformed/duplicate reply policy or a
mapping between an arbitrary hosted/agent identity and an ordinary user device.

**Observed source bug / implemented local correction:** shared
`signal::direct::prekey_bundle_for` selected the first matching user row using
non-AD identity plus device index. That erased the agent distinction, and two
rows for the requested address silently selected the first. The optional
one-time `<key>` also used the first child, although required fields already
used duplicate-refusing `child`. Body decoding already supports Text and Bytes;
there was no missing UTF-8 body fix to make. Required-field/list singleton checks
were already present and have not been rediscovered as new behavior.

Selection now compares **user, server, agent and device**, ignoring only the
wire-encoding kind. Parsed Text JIDs and Jid attributes use the same comparison.
Other devices in a legitimate batch are still allowed. Exactly one matching
user is required; both identical and conflicting duplicates fail with the fixed,
redacted `key reply carries duplicate bundles for this device` diagnostic.
Duplicate optional keys fail through the existing singleton guard, and an outer
`error` contradicting `type=result` is refused. These are **local ambiguity
guards**, not claims about mandatory server response shapes or newly confirmed
APK refusal policy. No identity is adopted, no session is reserved and no retry
or reset is introduced by parsing failure.

**Offline evidence:** independently generated identity/signed-key rows exercise
primary and companion selection, agent-only refusal, mixed-agent batch selection
for both attribute forms, encoding-kind equivalence, identical/conflicting/Text
alias duplicates, duplicate optional prekeys and contradictory outer errors.
A durable directory-action regression saves an ambiguous reply, reopens the
store and reports `invalid_reply` with fingerprint/signature null. Refresh is
refused, ordinary repeat reuses the original action ID and reserves nothing;
pins, sessions and checkpoints remain unchanged. The first new harness compile
failed because PreKeyBundle lacks Debug for unwrap_err; the harness now extracts
the error explicitly. That failure did not change protocol behavior.

**CLI/API:** existing `peer-keys --show` exposes its existing `invalid_reply`
classification. Comparison group/status sending and normal direct outbox
preparation share the corrected parser. No new command/flag/HTTP route exists.
Malformed saved evidence stays private and once-only outcomes remain preserved.
**Unknown:** live occurrence of ambiguous/agent-conflicting directory responses;
no such response is manufactured or queried for this change.


Validation: 667 workspace/all-feature tests, 24 default CLI tests, strict
workspace/all-target Clippy, warning-free Rustdoc and regenerated CLI reference
passed. **Observed offline using saved live evidence:** the rebuilt parser still
reports verified_bundle/signature_valid=true for the existing primary directory
response; pins, actions, outbox, receipts and device inspection remain unchanged.
Private evidence index: `lab/releases/exact-bundle-inspection/active.json`. No
network session or new query was issued. This validates compatibility with the
saved successful response, not fresh live malformed-response handling.

## Outer prekey dispatch and companion admission (2026-10-07)

**Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
This extends the inner Cnp.A04 trace above; it does not establish full companion
bootstrap parity or change any private account. The scoped source is exported
in the core bundle; [caller and exception evidence](receive-prekey-caller-evidence.txt)
retains exact scans and typed catches outside that bundle.

**Confirmed:** both direct DEX invocations of Cnp.A04 are in
[0jc.A0O](apk/2.26.37.71-core/receive-prekey-outer-coordinator.txt).
It obtains a `sessions` context through 0kT.A02 (`000c..0022`), takes the
0jp.A00 lock (`0022..0034`), parses DRw (`0036..0044`) and closes B75 on
success/error/exception paths. This establishes serialization and cleanup,
**not** a rollback-capable database transaction. Its identity comparison uses
serialized `remoteIdentityPublic_` from the current session when A0J is true
(`0050..0088`), or A0M when false (`00ee..00f8`); differing bytes select A0E
or A0s (`008e..00a0`, `00e6..00ec`). **Unknown:** full A0J/A02/A0M/A0E
semantics, whether the feature can change during the operation, and transaction
coverage. Do not convert these calls into an unconditional Rust reset.

**Confirmed:** the special `B76.A03 == B77.A04` branch (`00a4..00e4`) is
**STATELESS**, not the companion-device predicate. The
[B77 initializer](apk/2.26.37.71-core/signal-actor-bucket-enum.txt)
names A03=REGULAR, A02=PQ and A04=STATELESS. Enum ordinals and the separately
stored `intValue` are both 0/1/2 in this initializer. This establishes their
local values, not an on-wire encoding or the actor-selection producers.
On STATELESS, A0O constructs a fresh
[DS0](apk/2.26.37.71-core/receive-prekey-stateless-session.txt): synchronized
map lookup returns a new Cgc if absent, and CdI serializes into that map.
Its [DRz identity adapter](apk/2.26.37.71-core/receive-prekey-stateless-identity.txt)
delegates AjY/Amk/BRS to 0km, but COt immediately returns without saving an
identity. This refutes treating every Cnp.A04 invocation as a durable identity
or session write. It does **not** prove that no signed/one-time prekey changes
occur: the other constructor stores still come from 0km/0kn/0kp.

**Confirmed:** the ordinary
[0ko.A04 factory](apk/2.26.37.71-core/receive-prekey-ordinary-factory.txt)
converts the supplied address through CoY.A05 and uses 0km for the identity
and other key interfaces, with its 0kn and the coordinator's 0kp session
interface (`0000..0024`). A0O wraps the optional callback in DRs, otherwise
DRr (`0100..011e`). Feature 0x60e5 and 0kY.A02 choose whether the actual
Cnp.A04 call holds Cnp.A06's monitor (`011e..0166`); both branches converge
on the same decrypt method. No explicit ADV check appears in this factory; admission is upstream.

**Confirmed:** checked exception metadata connects local result codes to
obfuscated exception types. For the decrypt ranges, C4T maps to -1001, C4p to
-1002, C4q to -1003, C4x to -1005, C4V to -1007 and 2iA to -1010.
The DRw parsing range separately maps C4U to -1006 and C4x to -1005 with
its message. Generic Exception maps to -1000 except explicit rethrows for
DS2, SQLiteFullException and SQLiteDatabaseCorruptException. The appendix
preserves ordered catch ranges/targets in **code units**, whereas instruction
positions above are **bytes**. **Unknown:** the human meanings/hierarchies of
these obfuscated exception classes. These are APK local results, not newly
recognized WhatsApp server responses, ban reasons or Rust retry authority.

**Confirmed:** an exact A0O caller scan has five invocation sites: CQn.A00,
DM8.run, DN2.run, DOK.call and DOL.call. Only the selected contexts are traced
here. [CQn.A00](apk/2.26.37.71-core/receive-message-cipher-dispatch.txt)
reads ClY.A00; kind 0 calls A0N (`0276..029c`), kind 1 calls A0O with the
supplied address (`02a6..02dc`). It sets/removes A0U around that prekey call;
a catch-all also removes it before rethrowing. Its kind-2 path constructs Cnr
and handles sender-key decryption separately. An exact CQn.A00 scan found
two sites, both in CoU.A04.

**Confirmed:** the selected messaging producer
[CoU.A04](apk/2.26.37.71-core/receive-message-companion-admission.txt)
obtains a sender DeviceJid through CjN.A00 (`0008..0010`) and examines stored
device membership via 0bN.A0B (`002c..0044`). Its companion-prekey branch
requires non-primary sender, a nonnull CjN.A06 ciphertext and kind 1
(`00c0..00e8`), extracts the candidate public key with C8C.A00 and calls
**11T.A0D**, type 5, source 1 (`00e8..00fa`). ADV bytes come from CaD.A0H
for Bp1 (`00aa..00c0`), or Bp2.A0L (`097c..098a`). Failed remote ADV
validation requests a user update through 101.A03 and calls CoU.A0A with
local values 5/12, then returns (`0100..0172`); the self failure path differs.
The branch logs unknown-device messages without prekeys and requests user
refresh (`0c0a..0c66`). These local values are not server-response codes.
The two eventual CQn calls are `085c..0862` and `08c6..08cc`; the latter is a
conditional second-ciphertext path, not proof of an automatic network retry.
CoU.A00 only constructs CQn from dependencies and supplied message context;
it does not itself produce the Signal address.

**Unknown:** CjN sender/ciphertext/ADV producers, full feature and dead-device
predicates, the final A0D Boolean's exact source/meaning, message-to-B76 address
production and all admission/cleanup side effects. The existing traced A0D
signature checks and raw-ID/account-type policy still require implementation
work; see [companion identity validation](cross-device-sync.md#primary-identity-only-query-2026-10-07).
Do not admit or replace a companion solely on a valid nested Signal MAC, the
presence of an ADV node, or the ordinary factory's permissive trust adapter.

**Confirmed negative boundary:** DOL.call's non-primary kind-1 branch invokes
A0D with type 5/source 2/false before A0O; its failure diagnostic explicitly
names `voip/encryption/decryptCallPayload/invalid device identity`. That is
voice-call evidence, not the messaging admission source. DOK.call discriminator
3 decrypts PdR/P6L on SignalExecutor and restores A0V; discriminator 4 instead
saves a *primary* identity through A14. The worker's other outgoing cases must
not be mistaken for incoming companion bootstrap. DM8/DN2 and the other A14
callers remain untraced by this chunk.

**Implementation boundary:** the current group selector still supports only
already-held exact companion `msg` sessions with a pin; `pkmsg` companion
bootstrap/replacement remains unsupported. The APK itself has a companion
admission path, so this is our present limitation, not a WhatsApp prohibition.
Primary authenticated replacement remains buffered until authentication and
atomic commit. No new capability, account mutation or live claim follows from
this static trace.

**Validation:** final recipe regeneration verified all 141 core files, 1,975,976
bytes, covering 136 topics / 318 selected methods. Cached comparison against the
prior snapshot took 0.072842 seconds: seven added topics, all 129 previous
topics unchanged; inventory, manifest and tokens unchanged. This measures cached
scoped source comparison, not whole-APK equivalence. All 58 xtask tests, Rust
formatting, regenerated CLI reference and five API checks passed. The caller/
typed-catch appendix stays outside the core bundle. No new live test was run.
Repository hygiene passed 850 local links, licenses and all four APK checksum
bundles (3,650,447 bytes). Only research and an evidence comment changed.

## Companion ADV cryptographic binding and inspection (2026-10-07)

**Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The [producer evidence](companion-identity-producer-evidence.txt) preserves
scoped protobuf/field/caller scans and selected source outside the core bundle.

- [CjN.A00](apk/2.26.37.71-core/receive-sender-device-extract.txt) passes
  A04's route to DeviceJid.Companion.A00. [A04](apk/2.26.37.71-core/receive-sender-route-extract.txt)
  selects A04 when present for Bp1, otherwise A05; Bp2 uses A04 for group/status
  routes and A05 otherwise (`0000..0046`). The device conversion's full grammar
  remains **unknown**. [A06](apk/2.26.37.71-core/receive-pairwise-cipher-extract.txt)
  returns CaD.A00 for Bp1 or Bp2.A09. [A0H](apk/2.26.37.71-core/receive-message-status-extract.txt)
  returns Bp1.A01/Bp2.A0B, an Integer; its complete semantics remain **unknown**.
- [C8C.A00](apk/2.26.37.71-core/receive-companion-key-extract.txt) skips
  the ciphertext's first byte and parses BY9 (`0000..0022`), extracts identityKey,
  validates through B7H.A02 and returns raw B7E.A01 public bytes (`0022..0044`).
  Typed catches return null for protobuf, C4p/C4M or array-bounds failure. It
  does not authenticate the nested ciphertext. The new Rust `verify_prekey`
  uses the existing Signal parser for that identity, with a local 65,536-byte
  envelope bound; no session or prekey is loaded/consumed by inspection.
- [DFt.CFA](apk/2.26.37.71-core/receive-status-adv-child.txt) copies the
  `device-identity` child's byte body into CRF.A0U (`0006..001e`).
  [CRF.A00](apk/2.26.37.71-core/receive-status-message-build.txt) transfers it
  through constructor argument 18 (`009c..00a4`, `00e8..00ec`) to
  [CaD.A0H](apk/2.26.37.71-core/receive-status-adv-container.txt) (`001e..0026`),
  which CoU.A04 passes to 11T.A0D. This is the **status/Bp1 producer**; normal
  Bp2.A0L has one direct writer, its constructor. The Bp2 producer was still **unknown** at this signature-component
  checkpoint; the ordinary producer is traced in the follow-up below. No
  whole-message parser parity is claimed.
  The other CRF.A0U writer, DPV.invoke, copies Ca9.A0E; its upstream producer
  remains **unknown**. CaD has one direct constructor call, CRF.A00.
- Grz's recovered fields are details=1, accountSignatureKey=2,
  accountSignature=3, deviceSignature=4; GsB's are rawId=1, timestamp=2,
  keyIndex=3, accountType=4, deviceType=5. They match the existing generated
  Rust ADV messages. [08F.<clinit>](apk/2.26.37.71-core/adv-device-signature-prefixes.txt)
  supplies A0E=`06 00`, A0F=`06 05`, A0G=`06 01`, A0H=`06 06`
  (`0104..013e`). The selected 11T.A0D source chooses A0F/A0H for
  **HOSTED device type**, otherwise A0E/A0G, independently of account type.
  Azc.<clinit> names E2EE=0, HOSTED=1, NON_E2EE=2. HOSTED device/E2EE account
  fails the compatibility branch (`01e0..0226`).

**Implemented, offline:** `signal::adv::VerifiedDeviceIdentity::verify` checks
both signatures using the upstream XEdDSA adapter. The account signature covers
its prefix + exact serialized details + raw companion public key; the device
signature covers its prefix + details + companion key + raw account key.
`verify_prekey` takes the companion candidate from the incoming envelope.
Bundled account key bytes must match an **independent held primary pin**;
absent/empty bundled bytes use that pin. The APK also has supplied-key adoption
and account reconciliation branches; those are **unported**, not silently
implemented by trusting a candidate's own signature. Signed raw-ID, timestamp
and key-index getters retain authenticated metadata for future admission work.
Debug output is redacted.

**Local supported subset/parser policy:** outer/detail bounds 16,384/8,192
bytes, singleton-field uniqueness, exact raw/signature lengths, required raw ID/
timestamp/key index, and rejection of NON_E2EE/unknown encryption types. Missing
encryption types default E2EE. These are not invented server requirements;
the APK's enum fallback and protobuf duplicate behavior are broader. Unknown
singleton fields may remain covered by the exact-byte signatures; duplicate
unknown fields also refuse locally. No freshness floor, raw-ID reconciliation,
teardown/member check, human trust or nested Signal MAC is established by this
cryptographic verifier.

**CLI:** `lab-inbox --inspect` now displays `companion_adv=signatures_verified`
for the verified candidate against its same-namespace held primary, or bounded
missing/invalid/ambiguous diagnostics. It accepts typed/text sender attributes
and group participant routes, and does not substitute an alias/companion pin.
The display emits no key/ADV/ciphertext/raw metadata values. It reads captures
and existing pins only; no query, membership/pin adoption, session advance,
prekey consumption, journal write or send follows from verification. The
existing runtime companion `pkmsg` guard remains until the admission policy
and normal-message producer are traced and implemented.

**Validation:** four synthetic real-signature tests cover both signatures,
wrong independent keys, ciphertext candidate mismatch, exact-byte details,
E2EE/HOSTED prefix selection, account/device compatibility, malformed/duplicate
metadata and bounded input. These are adapter fixtures, not live Android
interoperability captures. The CLI fixture verifies PN/LID and typed/direct vs
text/group routing, missing primary authority despite a saved companion pin,
redacted refusal and unchanged stored rows. No real companion bootstrap or
live identity replacement is established.

**Observed locally:** the rebuilt CLI inspected the saved account offline. No
qualifying companion ADV diagnostic was produced from its captures, so this
is not fresh companion interoperability evidence. Outbox/actions/receipts/
device/pin inspections were identical before and after. Private inspection
records remain in ignored lab storage. Workspace/all-feature tests, strict
Clippy, warning-free Rustdoc, default CLI tests and regenerated CLI reference
passed. The final APK export contains 145 topics / 327 methods / 150 files,
2,001,638 bytes; cached checksum comparison took 0.075721 seconds with nine
added topics and all 136 prior topics unchanged. Four bundles total 3,676,109
bytes. Inventory/manifest/tokens unchanged; selected appendix outside core.
Repository hygiene passed 861 local links, licenses and all four checksum bundles.


## Ordinary companion ADV producer and baseline timestamps (2026-10-07)

**Confirmed, static APK 2.26.37.71** (SHA-256 `1755111a…7d5c`): the
[full-register producer/baseline appendix](companion-identity-producer-evidence.txt)
closes the ordinary Bp2 producer gap. Exact field-write scanning finds one
Cih.A0V writer, DEs.CF2, and exact constructor-call scanning finds one Bp2
constructor caller, Cih.A01. These are scoped exact scans, not proof that
reflection or every parser callback path is understood.

- DEs.CF2 reads the node byte body (`0006..000a`). A `registration` child
  stores Cih.A0W only when its body is nonnull and exactly four bytes
  (`000a..002a`). A `device-identity` child stores the body in Cih.A0V
  (`002c..0040`), without that four-byte requirement.
- Cih.A01 moves A0V through v17 into v42 (`0104..010c`, `0168..016c`).
  Its Bp2 constructor call starts at v16 (`01ac..01b2`): v42 is argument
  26, the last object argument. Bp2's constructor stores argument 26
  (v39 in that constructor) into A0L (`00a8..00b0`). A0W separately reaches
  A0M through argument 25. This is the ordinary producer used by the already
  traced CjN.A06/CoU.A04 companion-prekey check; it is distinct from the
  status/Bp1 producer. Parser callback selection and malformed/duplicate
  child handling upstream remain **unknown**.

**Confirmed, matched-baseline remote branch:** after signatures, 11T.A0D
requires matching held primary key, stored raw ID and account encryption type
before this branch (`03ce..04a0`). 0bN.A04 reads the remote baseline via
0bp.A02; feature/alias branches can select another stored user. 0bN.A0C copies
1IW.A09's device-index map and inserts the primary device with index zero.
11T.A0D compares the actual candidate device's saved index with its signed
keyIndex (`04c8..04fa`). If the stored list timestamp is at least the signed
candidate timestamp and the index is absent or different, it refuses with the
local log “device has been deleted already” (`04fa..05fa`). An equal index
continues even for an older candidate timestamp. A newer candidate continues
through expected-timestamp update and refresh scheduling; an absent/different
index additionally enters device-map mutation. These are local admission
branches, not server ban/refusal meanings. Own-companion, primary-key/raw-ID/
account-type reconciliation and pending teardown remain separate.

**Confirmed storage mapping:** 0bp.A00 names the SQL columns and passes them
through LbQ.<init>; 0bp.A05 independently writes the same mapping:

| LbQ field | `user_device_info` column |
| --- | --- |
| A01 | raw_id |
| A00 | account_encryption_type |
| A05 | timestamp |
| A02 | expected_timestamp |
| A04 | expected_ts_last_device_job_ts |
| A03 | expected_timestamp_update_ts |

0bN.A05 updates only when both A05 and A02 are below the candidate timestamp.
The new LbQ retains raw ID, account type and **A05**, sets A02 to the candidate,
sets A04 from `adv_last_device_job_ts`, and retains A03 unless the former A05
is at least the former A02; that case obtains the update clock (with wall-clock
fallback). 11T.A0D then calls 0bN.A0E and 101.A03 with local reason 3
(`05fa..0628`). For remote users A0E iterates 1IW.A02 aliases inside a
transaction, calls 1IW.A06, then 0bp.A05 on success. A06 bypasses its lengthy
account-type side effects when A07 detects no encryption-type transition.
0bp.A05 performs update-or-insert and registers a post-transaction callback;
its cache callback body and full alias fan-out are **not traced here**. The
own branch instead writes companion-mode preferences.

**Refuted:** advancing our verified `DeviceList.list_timestamp` directly to a
newer message ADV timestamp does not match 0bN.A05. The expected/pending value
is separate. Cryptographic verification alone must not erase a deletion or
turn a pending refresh into a verified device list.

**Remaining implementation:** atomic companion pkmsg admission/bootstrap,
expected timestamp and refresh behavior, new-device map application, alias
fan-out, raw-ID/type transitions and own-account reconciliation. The runtime
still supports exact held-companion `msg`, while companion `pkmsg` is our
unimplemented receive path. This trace changes no account, prekey, session,
request, API/CLI behavior or live claim. Existing `apk-smali`, `apk-field-refs`
and `apk-callers` commands reproduce the evidence; the added checksum topics
make the producer and baseline readers visible in cached release comparisons.


**Validation:** thirteen full-register method sources were independently
re-exported byte-for-byte from the hash-pinned APK, including SQL mapping and
account-type transition helpers. The compact export now has 155 topics / 337
methods / 160 files / 2,036,151 bytes (61,001 bytes below its enforced limit).
Cached checksum comparison took 0.071644 seconds: ten added topics, all 145
prior topics unchanged, and inventory/manifest/tokens unchanged. The thirteen
raw source outputs above include typed exception metadata; the appendix and
compact source serve different purposes. xtask's 58 tests, regenerated CLI
reference, OpenAPI library-symbol check, formatting and whitespace checks passed.
No new runtime or live validation is claimed for this source-only change.


## Known companion prekey bootstrap (2026-10-07)

**Implemented/offline, APK 2.26.37.71 matched-baseline branch:** group/status
version-2 companion `pkmsg` can now establish the actual device's pairwise
session. The ordinary producer and pending timestamp trace above supplies the
source: DEs.CF2 -> Cih.A0V -> Bp2.A0L -> CoU.A04 -> 11T.A0D. Admission uses
both ADV signatures against an independent held primary and checks a current
signed device list, raw ID, E2EE account/device types, actual device index and
candidate timestamp. 11T.A0D `04c8..05fa` accepts an equal index when the stored
list is at least as recent. This implements that branch; it does not change
the list to manufacture membership. No primary or PN/LID alias ratchet substitutes.

[companion_admission.rs](../../crates/whatsapp-api/src/transport/companion_admission.rs)
prepares authority from one read transaction. Our local subset requires a unique
pairwise candidate, unique byte-body ADV and a known own identity in the
candidate's namespace; malformed/duplicate inputs and unknown own namespaces
defer. Own PN/LID candidates always defer, independently of the configured
policy: pinned policy alone allows first-seen identities and is not an own-device
guard. The saved account generation, registration status, own route, Signal
public identity and registration ID must match the caller. These are our
transaction/unsupported-branch guards, not newly inferred server requirements.

`inbox::process_with_policy` authenticates the nested Signal MAC before
committing the exact companion session/pin, one-time prekey consumption, SKDM
and control/message together. Its write transaction rechecks the complete
saved device list, independent primary pin, login aliases and account identity,
plus existing session/pin/prekey compare-and-swap checks. Changed authority
aborts the entire transaction. `apk_authenticated` replacements retain the
prior exact companion pin/session in encrypted history and emit the existing
IdentityChanged event; `pinned` still refuses a changed companion key.
SKDM-only controls earn delivery eligibility but no user message/read receipt,
and dedupe after reopening. A refused candidate does not block a separately
usable held `skmsg`.

**CLI/API:** `lab-inbox --inspect` adds a fixed `companion_admission` after
successful signature verification. `ready` means matched-baseline eligibility,
not nested authentication or completed receive. Other fixed statuses explain
missing/stale authority, own-device/unknown-own namespace, raw-ID/type mismatch,
old missing/different device index (`deleted_device`) or newer candidate
(`refresh_required`). No address, key, signed metadata or ciphertext is printed.
The same read-only library inspection is documented in OpenAPI; `listen`,
`lab-transport --receive` and offline capture replay share the group receive
implementation. Existing `x-cli`, `x-transport` and `x-live` retain their actual
behavior; no HTTP handler was added.

**Remaining:** direct companion prekeys, newer-candidate expected-timestamp/
refresh application (0bN.A05 preserves the verified list timestamp), new-device
map mutation, supplied-primary/raw-ID/type transitions, alias fan-out and own
account reconciliation. These are implementation gaps; the APK's broader
admission must not be inferred from this matched-baseline subset. No fresh
companion interoperability sample or live companion replacement is established.

**Validation:** five real-crypto fixture tests cover PN/LID exact-device
bootstrap, equal/older timestamps, one-time consumption, SKDM control markers,
no read receipt, unchanged primary/list, reopen dedupe, nested MAC tamper,
pinned refusal versus authenticated replacement, nine deferred authority/input
cases, four authority changes with transaction rollback, and held-sender-key
fallback without pairwise/prekey/pin adoption. These are offline adapter
fixtures, not APK-generated live captures.


**Observed locally:** final experimental CLI build and saved-account read-only
inspection completed. Captures yielded no qualifying companion ADV/admission
sample, so no live bootstrap is claimed. Before/after outbox, actions, receipts,
device and pin summaries were identical; no login/query/chat was sent. Private
inspection records remain in ignored lab storage. Default CLI tests (24),
regenerated CLI reference, OpenAPI library-symbol check and final build passed.

The persisted list format has no account-type field: the existing
`VerifiedIndexList::verify` rejects non-E2EE account type before the device list
is marked verified (`signal/adv.rs`). Candidate `is_e2ee` therefore checks
compatibility with that supported baseline, rather than assuming arbitrary
stored ADV accounts are E2EE. Fixture membership rows are synthetic saved state;
existing device-list tests separately cover signed-list verification. Repository
hygiene passed 869 local links, licenses and four checksum bundles.


## Direct companion receive and receipts (2026-10-08)

**Implemented/offline, APK 37.71 evidence:** direct non-hosted companion pkmsg
now uses the same matched-baseline ADV admission and atomic authority checks as
group/status. B7H.A06 carries the actual getDevice into B76; CoU.A04 checks
11T.A0D for ordinary Bp2 prekeys. Both ADV signatures, current held-primary-bound
membership/raw-ID/index/non-newer timestamp and nested Signal MAC are required.
Held companion msg selects the exact actual-device ratchet, with no primary or
alias fallback. Incoming pinned/apk_authenticated policy and archived replacement
remain effective. Own companion prekeys remain excluded by the shared admission.

**Local identity projection:** a direct companion's primary namespace identifies
its chat/dedupe marker, while its actual DeviceAddress identifies the event sender
and its pairwise/SKDM sessions. Optional sender_pn does not select the ratchet or
change that chat marker. Authenticated explicit PN aliases are saved under the
primary LID; no companion-wrapped alias key is created. This is our stable local
projection, not a claim to have recovered every APK chat model producer.
Primary direct routing retains its prior behavior. Direct SKDM sender-key storage
now also preserves the actual companion; the prior primary-only selector was
refuted by the real-crypto fixture. Existing direct SKDM projection remains its
legacy decrypted representation; this change does not claim complete control-event
parity with the separate group-only sender_key_distributed marker.

**Confirmed receipt branch, static APK 2.26.37.71:**
[full-register receipt evidence](direct-companion-receipt-evidence.txt) retains
Cp2 and CmC.A05/A06, plus the 0DE.A0Q classifier. Cp2.A00 passes its supplied
DeviceJid/chat/alternate route to CmC.A06 (`0000..0008`) and writes the resulting
pair's first member as `to` (`0064..0086`). For a nongroup chat with neither
chat nor alternate matching A0Q, CmC.A06 `003a..0042` uses the supplied DeviceJid
as destination when present. A0Q recognizes types 3/5/6, not a generic companion
predicate. Cp2.A04 passes message A0p:DeviceJid (`00d6..00ea`); A07 passes
CM8.A05:DeviceJid (`0034..0044`). The complete A0p/CM8 producer upstream remains
**unknown** here. Cp2.A06 is specifically the Bp1 status builder and is **not**
evidence of the ordinary Bp2 receipt producer; that initial trace was refuted.

The Rust receipt builder's device-zero restriction was our limitation. It now
retains a normal direct non-hosted PN/LID destination device, and the read queue
accepts primary LID chats. Existing edit/recipient/unsupported-agent restrictions
remain local bounds. No read receipt is queued merely for a key distribution.
No new HTTP route or flag exists; lab-inbox inspection/replay and both live receive
entry points share this receive implementation. API descriptions retain honest
transport/live metadata and describe the expanded direct capability.

**Validation:** real-crypto PN/LID direct fixtures cover bootstrap with one-time
consumption, reply/ordinary-msg ratchet continuation, exact device event sender,
primary chat, delivery destination, automatic read queuing, alias disappearance
without duplicate projection, primary/list preservation and reopened duplicate
handling. Nested-MAC/pinned replacement tests now run through direct as well as
group paths. Another direct SKDM fixture preserves the same capture/prekey while
membership authority is absent, then commits to the actual-device sender-key
namespace after restoring the synthetic verified fixture row. These are offline
adapter fixtures, not APK-generated live samples. Newer candidates, missing/new
devices, pending refresh, raw-ID/type transitions, own-account reconciliation
and fresh companion interoperability remain unfinished.


The full suite caught an own category=peer control regression during this
change: session_sender's new primary projection made restore compare an own
control's actual-device marker with a primary chat marker. Explicit peer
controls are again excluded from ordinary chat projection; their existing
held-session/import/duplicate path and record selector remain unchanged. The
receive/backfill alias helper is shared so reopening a companion LID capture
cannot create a device-wrapped alias or replace a PN mapping with the LID itself.


**Validation completion:** all 689 workspace/all-feature tests, strict Clippy,
warning-free Rustdoc, default CLI tests (24), regenerated CLI reference, API
library-symbol check and final experimental CLI build passed. Four scoped APK
receipt source outputs reproduced exactly. Saved-account offline inspection
found no qualifying companion sample; before/after outbox/actions/receipts/device/
pin summaries were identical. No login, query or chat send occurred. Fresh live
companion interoperability remains unverified; private inspection records stay
in ignored lab storage.


## Known companion newer-ADV refresh (2026-10-08)

**Confirmed, static APK 2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
`11T.A0D`'s matched-primary/raw-ID/E2EE branch accepts the actual companion
when its key index matches, including a candidate timestamp newer than the
verified list. `0bN.A05` raises the expected timestamp without advancing the
verified timestamp; its SQL field mapping is retained in the
[producer evidence](companion-identity-producer-evidence.txt).
This supersedes our earlier `refresh_required` refusal for a known matching
index. Absent/different indices with newer timestamps still require the
unported new-device branch; old/equal mismatches remain `deleted_device`.

**Confirmed:** `101.A03(VLI)` validates the user list, deduplicates users under
`AzX.A03`'s monitor and records the clock in `AzX.A01`. Only newly scheduled
users enter `DLx` discriminator 14. `DLx.run` at `16f6..173e` constructs
`SyncDevicesJob` with those users and the unchanged caller reason, then enqueues
it through `0ls.A01`. The candidate caller's reason is 3; this is a local job
reason, not a server ban/refusal code. These names apply to 37.71 and do not
replace differently obfuscated evidence attributed to other builds.

**Confirmed:** `Lcr.A00(LLLIJZ)` carries a supplied response timestamp into
`LbQ.A05`. With an existing baseline and its Boolean argument false, a response
below prior `A02` retains expected timestamp `A02`, job timestamp `A04` and
update clock `A03`. A response reaching that expectation resets the trio.
The Boolean true and missing-baseline branches also reset it. **Unknown:** the
Boolean's full caller semantics and own-account/type/raw-ID reconciliation;
we have not inferred permission to clear expectations from an arbitrary flag.
**Negative finding:** `0bN.A0H(ZLLLZ)` accepts an already constructed `LbQ`
and persists it through `0bp.A05`; it does not itself produce the expected
timestamp. Broader membership transactions remain unported.
Reproduction commands and register context are in
[refresh evidence](companion-refresh-evidence.txt).

**Implemented locally:** a known exact-index candidate reports
`companion_admission=ready_refresh_pending` during read-only `lab-inbox
--inspect`. Both ADV signatures and the independent held primary are required;
nested Signal MAC verification must then succeed. The receive transaction
rechecks authority and atomically saves the message/ratchet/prekey changes,
raises `DeviceList.expected_timestamp` and queues the existing device sync.
The verified timestamp is preserved. Inspect alone changes nothing; invalid
MACs and aborted transactions change neither authority nor the queue. Delaying
the pending update until nested authentication is our atomicity guard, not a
claim about APK job scheduling order.

Existing pending work retains its notification hash. Equal/lower expectations
with a pending entry do not create another revision; a raised expectation uses
the existing durable revision mechanism. A correctly signed response below the
expectation may advance the verified floor, but retains both the expectation
and pending revision. A response reaching it clears the old expectation and
pending entry; separately supplied server `expected_ts` remains represented.
`devices` exposes `expected_timestamp` and `pending_users`. Existing session
planner attempt/defer limits and refusal/uncertain-outcome handling still apply.

**Verified offline:** real signed ADV plus encrypted direct/group fixtures cover
successful admission, nested-MAC refusal, inspection without mutation,
transaction rollback and duplicate processing. Real signed device-list fixtures
cover a lower response retaining pending work, a reaching response clearing it,
and deduplication of lower expectations. **Unknown live:** no saved qualifying
newer companion capture establishes this branch on the server. New-device
addition, own-account reconciliation and full primary identity recovery remain
unfinished.


## Companion new-device membership application (2026-10-08)

**Confirmed static APK 2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
Full scoped methods and reproduction commands are preserved in
[new-device evidence](companion-new-device-evidence.txt). This is further
reverse engineering; it does not claim a new Rust admission capability.

After both signature checks, `11T.A0D(ZLLLBIZ)` tests its final Boolean
argument at `0374..039e`: true logs pending teardown and exits without adding
membership. **Unknown:** the complete ordinary caller producer of that argument;
the string alone does not define a server refusal or authorize a local reset.
The existing remote matched-primary/raw-ID/account-type branch remains the
scope here; own-account, hosted and primary-replacement branches are separate.

**Confirmed:** `04c8..050e` compares the actual device's cached index with the
candidate. An absent/different index is deleted when verified list time is at
least candidate time. A newer candidate raises the pending expectation and
schedules reason 3 as previously traced. A matching index reaches success
without membership mutation. An absent/different newer index reaches the
remote add path at `0690`, while `0628..0690` is the separate own branch.

The remote path constructs a singleton actual DeviceJid -> signed key-index
map. `1IW.A09` obtains cached membership (direct `B7j.A00`, or gated canonical
lookup/rebinding through `Chv`). `06f2..071e` removes **every already present
DeviceJid** from the proposed additions before any insert. Thus the branch does
not overwrite an existing address's different index merely because the ADV is
newer. This refutes treating this path as a generic device-index replacement.
`0bN.A0G` removes entries whose embedded user differs from the supplied owner;
it is an ownership filter, not another signature verifier. `0724..0774` adds
the primary with index zero only when missing. An empty additions map returns
success. Hosted additions with an E2EE account are rejected at `0788..07aa`.

**Confirmed:** nonempty remote additions assert `08Z.BO0(user)` false and pass
through `1IW.A02`. When both `0bq.A06` and `A07` gates are enabled, the method
returns a singleton `Chv.A01` canonical user or an empty set. Otherwise a
qualifying PhoneUserJid is returned directly, or `0Xw.A0U` supplies the selected
user set. **Unknown:** actual runtime gate values and complete canonical/alias
lookup behavior; this is not evidence that arbitrary PN/LID aliases may be
created from a signed device's address.

For each selected user, `AzR.A00` rebuilds DeviceJids with the original device
number under that user and preserves the index. `11T.A0D 08f8..0994` validates
primary index zero / companion index positive, then calls `B7j.A03`. That method
writes only `user_jid_row_id`, `device_jid_row_id`, `key_index` into `user_device`
using `INSERT_DEVICE_JID_SQL` and database conflict argument 4. `B7j.A00` reads
that same table, validates ownership/index shape and caches the map. The
membership insert does not update the signed `LbQ.A05` time floor. This
separation is why a signed new member must not be described as a newly received
full signed device list.

**Confirmed call boundaries, incomplete side-effect trace:** nested and outer
transactions commit; a `DNd` discriminator-19 callback is registered after the
nested writes. `1IW.A05` receives old membership, added membership, an empty
removed set and two false Booleans. Its nonempty removed-set branch is therefore
skipped in this call. `1IW.A04` receives old membership, added membership and an
empty removed set after the outer commit. Its removed-device branch is skipped;
its addition branch reaches `0bI` participant-user operations, including
`A0J`, `A0G`, `A04`, `A07` and `A06`. Finally `JhE.A0L` receives the original
owner, addition keys and an empty removed set. **Unknown:** complete effects of
those helpers, the registered callback and observer, including group fan-out
cache invalidation. The raw helpers are preserved so these branches can be
continued without repeating this trace. No deletion/identity rotation is
justified by this add-only call.

**Implementation still required:** authenticated new-member provenance and
transactional application separate from the verified time floor; canonical
PN/LID selection/rebinding; corresponding participant/observer updates; complete
teardown producer; fresh live coverage. Existing Rust still reports
`refresh_required` for absent/different newer indices and `deleted_device` for
old/equal mismatches. Known matching-index newer candidates already support
pending refresh. No account or transport state was touched for this research.


## Companion teardown producer and registry (2026-10-08)

**Confirmed static APK 2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Caller, registry and field evidence](companion-teardown-evidence.txt) preserves
full scoped methods plus explicitly selected multi-case callback context.
This resolves the ordinary final-Boolean producer left unknown by the preceding
new-device trace; complete runtime teardown application is still unported.

`CoU.A04` first checks current membership through `0bN.A0B`; an already
present actual address jumps from `0044` to `0b60` and sets the final Boolean
to zero. For an absent address it sets `v13=1` at `0098` only after the `0bq.A04` gate,
`1Uf.A08(actual sender)` and nonnull `CjN.A0H` all pass. Ordinary companion
prekey dispatch reaches `11T.A0D` at `00f4..00fa`. A separate normal prekey
branch sets `v13=0` at `0b60` and jumps into that same dispatch. `CjN.A0H`
returns `Bp1.A01` or `Bp2.A0B`, not the retry counter (`CjN.A02` is separately
logged as retryCount). The exact ordinary writer chain is `DEs.CEv` reading
message `offline`, parsing it through `0CK.A06`, saving `Cih.A0D`, then
`Cih.A01 -> Bp2.<init> -> Bp2.A0B` as already preserved in producer evidence.
`dhash` is read separately into `Cih.A0L`; it does not produce this Boolean.
An `offline` field's mere existence is not enough: the parsed Integer must be
nonnull. No specific positive value is tested at this caller.

**Confirmed:** `0CK.A06` uses radix 10 through `A07`, which accumulates a
negative integer with overflow checks, optional leading plus/minus and
`Character.digit`. Empty/sign-only/invalid/overflow inputs return null; zero
and negative parsed values are not excluded by `CoU`'s nonnull test.
Full Java Unicode digit parity has not been ported. The disassembler previously
rendered its multiplication as unknown opcode b2; this change labels it
`mul-int/2addr` with the existing 12x width/register decoding. An independent
literal DEX window at classes.dex `0x495af6` covers multiply, overflow branch
and subtraction. Opcode meaning is corroborated by the
[AOSP bytecode table](https://source.android.com/docs/core/runtime/dalvik-bytecode).
No committed checksum-bundle topic contains op=b2, so this correction does not
change those four bundles or their cached comparison results.

**Confirmed:** `1Uf.A08` compares `getRawStringWithNoAgent()` against an
AtomicReference-held set. `A00` initializes that set from SharedPreferences
`dead_devices`, defaulting empty; `A04` commits the updated string set and
updates the reference. `A06` synchronizes on `A08`, adds a previously absent
address, persists it and requests idle/reap scheduling through `A01`.
It does not synthesize server waits or account refusal state. The constructor
is the sole field assignment to the reference, but pointee contents mutate;
that negative field scan must not be interpreted as immutability. Searches for
1Uf methods A09/A0A/A0B found none; the full class has 13 method bodies.

The two `A06` callers are `DMh.run` (selected discriminator 1 removal callback)
and `DNk.run` case 4. `DMh` only takes its mark-dead branch when `1Uf.A07`
returns true; that predicate depends on worker latch/count or `19M.A0O` plus
the feature gate. It also writes a separate in-memory `1Uf.A09` set, which is
not the persisted registry. `DNk` case 4 removes each selected address from
that transient set and chooses deferred marking versus immediate key-store
calls based on the old membership set, registry membership and `A07`.
**Unknown:** actual gate/worker state and full immediate-removal helper effects;
these are local branches, not new server response codes.

**Confirmed:** `1Uf.A02` avoids reaping while the traced workers are in flight.
Its in-flight branch polls after 2000 ms and parks after at least 15 polls;
these are local scheduling bounds, not WhatsApp cooldowns. Before teardown it
rechecks current membership through `0bN.A0B`: a returned device drops the
registry entry and is left untouched. Otherwise it reaches `0jc.A0s/A0r` and
sender-key removal, including literal `sender_keys` / group-scoped SQL and
batch iteration. Full session/identity deletion semantics remain untraced;
we have not implemented cleanup or inferred permission to erase history.
**Negative finding:** the scoped `A02` has no post-cleanup `A04` call to clear
that dead-device entry; its explicit registry removal is in the returned-device
branch. Broader callback lifecycle remains unknown. Do not infer that a cleanup
completion automatically authorizes a new offline ADV.
The alternate `DNk` identity-response worker supplies final Boolean zero to
11T.A0D (`02de`, `032a..0330`), so the ordinary offline-message guard must not
be imposed indiscriminately on every identity response.

**Current Rust limitation:** no persisted dead-device registry or deferred reap
lifecycle exists. Current old/equal missing-index rejection and independent
primary/MAC/transaction safeguards still apply; new-device addition remains
unfinished. The port must preserve explicit pending removal and recheck current
membership without resetting account identities or replaying requests.
This static research made no saved-account or live-transport changes.


## Remote newer-ADV device application in Rust (2026-10-08)

**Implemented; verified offline**, grounded in APK 37.71 `11T.A0D
04c8..0a96`, `B7j.A03` and the preceding
[new-device trace](#companion-new-device-membership-application-2026-10-08).
With a current independently bound account baseline and matching raw ID/E2EE
signatures, a newer candidate may enter receive even when the device address
is absent or its index differs. Nested Signal authentication and incoming
identity policy still apply. An absent actual namespace/device is added with
its signed positive Java-int index (1 through 2147483647) in the same receive
transaction. APK int-to-long conversion makes higher unsigned values negative;
those candidates are invalid_companion_index. An already present
address retains its old membership index; the candidate's newer expectation
queues the existing device sync. Old/equal mismatches remain deleted_device.
Account keys, IDs and the verified list timestamp are preserved.

`DeviceList.adv_added_devices` records which retained addresses came from
individual authenticated ADV, rather than full signed-list confirmation. It is
local provenance, not a server wire field; historical rows default empty.
The baseline's companions_verified and verifier fingerprint retain their
account-authority meaning. A new addition invalidates hash_matches while keeping
notification metadata. During signed reconciliation an ADV-added member keeps
its provenance when retained as a newer local allocation; provenance clears
only when the signed indexes and incoming address/index actually confirm it,
or when reconciliation removes that address. Unsigned-primary reconciliation
has no retained companions/provenance. `VerifiedIndexList::contains_index`
exposes this already authenticated index membership check.

Inspection remains read-only and uses ready_refresh_pending for newer eligible
candidates. `devices` JSON exposes adv_added_devices; human output shows its
count and expected_ts beside list_ts. Fixed local refusal diagnostics now also
include ambiguous_device_membership, invalid_companion_index,
conflicting_device_membership and offline_device_update_deferred. Duplicate
addresses, index collisions and 1024-entry overflow are our map guards, not
server errors. The obsolete newer-index refresh_required refusal no longer
represents this supported branch.

**Explicit remaining limitation:** absent senders carrying any offline
attribute are deferred because the dead-device registry/reap lifecycle is not
yet implemented. This conservative local presence check also defers malformed
offline metadata; it does not claim APK nullable-Integer/gate parity. Known
addresses bypass that new-device guard as in the traced caller. Actual signed
PN/LID namespace authority is used; no alias is invented or second namespace
updated. Canonical alias fan-out, participant/observer invalidation, own-account
and raw-ID/type reconciliation remain incomplete. Full APK compatibility and
live new-device coverage are not established by this bounded branch.

Real encrypted direct/group fixtures exercise known, absent and different-index
newer candidates, inspection without mutation, valid MAC, invalid MAC,
transaction abort and Duplicate. Add-only membership, original verified floor,
refresh queue and provenance are asserted together. Signed response fixtures
exercise preservation above the response's current_index and later confirmation
of the added address/index. This does not stand in for fresh live validation.


## Namespace-local pending removal registry (2026-10-08)

**Implemented locally; APK 37.71 static evidence:** `1Uf.A06` persists
agent-free dead-device addresses; `CoU.A04` supplies pending teardown for an
absent offline sender; `1Uf.A02` rechecks current membership and drops a
returned-device entry without tearing down its keys. Full register context is
in [teardown evidence](companion-teardown-evidence.txt). Our registry is scoped
to the independently verified PN or LID namespace; canonical rebinding and
worker/gate parity remain incomplete.

`DeviceList.pending_removed_devices` is a nullable local array. Null is unknown
legacy state, not an empty registry. A successful signed reconciliation starts
known state, records cached companions absent from the retained set, merges
existing markers and clears only addresses now present in that authenticated
retained set. Address comparisons ignore wire encoding kind while retaining
user, namespace, agent and device. Refused signed responses do not update it;
unsigned primary-only ingestion neither initializes it nor clears markers.
The registry and device baseline are written in the existing ingest transaction.
It is local metadata, never a WhatsApp response field or account cooldown.

An absent offline ADV candidate with unknown registry still reports
`offline_device_update_deferred`. A matching marker reports the new fixed
local reason `pending_device_teardown`. With known state and no marker for that
actual device, offline admission can use the same independent-primary/ADV,
positive index, newer timestamp, nested MAC, identity-policy and atomic commit
checks as the non-offline branch. Known addresses still bypass the absent-device
teardown branch. Authority snapshots now include the registry through the
complete DeviceList recheck, so a concurrent pending removal aborts commit.
Read-only `devices` JSON shows the nullable array; human output shows its
optional count, preserving unknown versus known-empty.

**Historical local policy, superseded below:** the initial guard used any
offline attribute, even malformed text. Nullable Integer parsing is now ported;
runtime feature gates remain unported. Initializing from a signed response establishes this
client's current removal state, not imported Android preferences or historical
cleanup. No deferred reap scheduler, ratchet/identity/sender-key deletion,
archive erasure, automatic marker reset or request replay is implemented.
Keys/history remain intact. Namespace alias fan-out, participant/observer
side effects, own-device lifecycle and fresh live removal/offline admission
validation remain unfinished.

**Verified offline:** real signed lists remove a cached device, persist its
marker across reopen and clear it on authenticated return while preserving the
primary identity pin. Real encrypted PN/LID direct/group fixtures cover known
clear offline admission, MAC failure/rollback/Duplicate, pending-removal refusal
and unknown-state deferral. A registry race rolls back the receive authority
commit; old JSON defaults to unknown. Fresh live coverage remains unknown.


## Nullable offline companion metadata (2026-10-08)

**Confirmed static, APK 2.26.37.71:** DEs.CEv -> 0CK.A06/A07
produces a nullable signed radix-10 Integer for Cih.A0D -> Bp2.A0B ->
CjN.A0H. CoU.A04 checks nonnull, so zero and negative integers qualify;
empty, sign-only, whitespace, malformed and overflowing values do not.
0hs.A0M delegates to A0C, whose forward array scan returns the first
matching attribute; later duplicates do not replace it. The scoped methods
are preserved in [teardown evidence](companion-teardown-evidence.txt).

**Implemented/offline:** this supersedes the earlier presence-only local
predicate. Receive and read-only admission inspection use the first text
attribute, UTF-16 decimal parsing and signed-i32 bounds. Invalid UTF-8 byte
and JID attributes do not become integers. The existing private decimal
helper is reused without MEX's default-zero or interop length/leading-zero
rules. Only a parsed integer invokes the absent-device removal guard;
unknown registry and pending markers retain their existing fixed refusals.
Independent primary authority, both ADV signatures, MAC authentication,
identity policy and atomic baseline rechecks remain required.

**Reference limits:** the reused 283-case integrator corpus records host Java 21
Integer.parseInt results independently. Tests use java_integer, not the
separate interop accepted flag. The BMP digit table is host-Java-derived,
not extracted Android Unicode data; Android-version Unicode differences
remain unknown. Runtime feature gates, reap lifecycle and fresh live
companion coverage remain unfinished. No cleanup or identity reset occurs.
Encrypted PN/LID direct/group tests cover malformed metadata with unknown
and marked removal state, corruption rollback, reopen and duplicate receive.
Valid zero, signed minimum and positive BMP-digit maximum remain guarded.


**Local verification environment:** host packages changed during this work;
the final CLI build initially failed to load libLLVM.so.22.1 with installed
LLVM 23. The official Arch llvm-libs 22.1.8-2 package was unpacked only into
`/tmp/telewhatsapp-llvm22`; LD_LIBRARY_PATH supplied its library for the
remaining checks. No host package/configuration was changed by this work.
The repeated checks use the resulting system Rust 1.99.0; they do not establish
the repository-pinned 1.98.1 toolchain proof. First-pass workspace/lint/docs/CLI
checks passed before that final build failure. This is a local build-environment
finding, not APK evidence or a WhatsApp refusal.


## Companion removal property and effective value (2026-10-08)

**Confirmed static, APK 2.26.37.71** (SHA-256 1755111a…7d5c): the
ordinary receive predicate references the **field** `0bq.A04:09P`, not a
method or a received offline count. `0bq.<clinit>` at 0050..0064 constructs
`09P(0x8b4d, false, false)`; its integer property ID is **35661**.
`09P.<init>` assigns ID/A01/A02, and `toString` labels these fields
`ABPropBoolean`, `prodDefaultValue`, `debugDefaultValue`. These labels establish
both baked defaults; neither establishes the effective account value.
Full methods and field references are preserved in
[feature gate evidence](companion-feature-gate-evidence.txt).

`CoU.A04` calls `00D.A0G`, which delegates to `A13`. `A13` passes the
property ID and **A01 production default** through `A0E`; it does not read
A02 debug default. `A0E` passes these into `A07` with a Boolean fallback
`1Y1` discriminator **2**, and the Boolean callbacks A0R/A0W. Constructor
`00D.<init>` binds those callbacks to `1Y4(4)` and `1Y5(0)` respectively.
`1Uf.A07` independently reads this same property through `00D.A13` when
its worker predicates qualify. The exact field scan has three references:
constructor write, ordinary receive read and removal-worker read.

**Confirmed resolver behavior:** `00D.A07` selects cached/resolved values
when its context/mapping/state guards qualify, otherwise invokes its fallback;
these guards are not a constant false-return branch. `1Y4.invoke` discriminator
4 resolves `09M.B2y(id)` and, for nonzero mapping, calls
`MobileConfigUnsafeContext.AVn`. Its missing-map result carries false and
`INVALID_SPECIFIER_MISSING_IN_WA_LOOKUP_MAP`. `1Y5.invoke` discriminator 0
reaches `0E6.A04`, which likewise reads mapped Boolean configuration or returns
false for a zero mapping. `1Y1.invoke` discriminator 2 calls `00D.A0D`.
That fallback first checks the relevant cache (A0h), then A15's Boolean map,
then the supplied production default, and reads `SharedPreferences.getBoolean`
using the decimal property ID as key. A map value or stored preference can
therefore supersede the supplied baked false default. Cache/exposure bookkeeping
is separate from the pending dead-device registry.

**Negative / unknown:** `1Y1` has no method named get; its functional entry is
invoke. No runtime property value, property-specific MobileConfig mapping,
saved Android preference, A15-map value for 35661 or server feature assignment
was obtained. Generic resolver branches establish possible sources, not which
source wins for this account. Native MobileConfig semantics and resolver
initialization/exposure side effects are not fully ported. No Android execution
or live property query was performed. Do not infer either always-enabled or
always-disabled behavior from this trace.

**Rust limitation:** our parsed-offline/known-removal guard remains a local
conservative policy with no effective APK property value. It must not be
presented as an unconditional WhatsApp requirement. This finding does not
justify setting the gate false, deleting markers/keys or adding a bypass flag.
Porting received feature configuration and the teardown worker remains work.
The nine new cached core topics preserve the property/default/resolver/fallback
source for release comparison; class names and numeric property IDs remain
build-specific and require review in another APK. Most helper topics have no
unique string anchor; an automatic relocation is not established.


## Companion key-removal scope and callback boundaries (2026-10-08)

**Confirmed static**, APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
This extends the earlier 1Uf.A02 reap trace: 0jc.A0s is removeSession and
0jc.A0r is deleteIdentity. It resolves their SQL scope without enabling cleanup.
The earlier statement that session/identity removal was untraced is superseded
for these selected methods; full canonical mapping and callback lifecycle still
remain incomplete.

### Session deletion and cache variants

0jc.A0s obtains a translated address through 0kT.A02 with table sessions,
then executes A0D inside a database transaction. A0J chooses an address lock
versus a synchronized Cap cache path; the runtime predicate remains unknown.
A0D translates again through the session store and deletes only rows matching
recipient_account_id, recipient_account_type and device_id (005c..0072).
B76.A00 constructs its three SQL arguments from A04, A01, A00 in that order.
This is a device-specific predicate, not an entire contact or account reset.

A0D constructs Cfc and registers DNt discriminator 32 with 0ZN.A04
(00aa..00d4), carrying the caller's translated address. A04 asserts an active
SQLite transaction, wraps the runnable and delegates to 0Gy.A00; complete
commit/rollback callback delivery is still **unknown**, so no after-commit
execution guarantee is inferred just from the call site.

DNt.run's switch maps 32 to 047a. At 047a..04a8 it writes Cfc through
Cap.A03. It then enumerates B77/B78 variants and, except the original variant,
removes matching address/type/device entries from Cap's set and map
(04b4..0534). Thus an SQL single-device delete also has cache-variant effects.
The enum meanings and complete Cfc session semantics are not established here.
Cap.A03 is synchronized, prunes marked entries when its map exceeds 1000,
marks an existing key, and skips map.put for account type 3. That local cache
bound is not a server wait or limit and is not copied into Rust policy.

### Identity deletion and feature-dependent target mapping

0jc.A0r tests CDO.A00 through 00D.A13. CDO defines property 0x79da (31194)
via 1xf.A0t; that helper constructs 09P with false/true defaults. The already
traced 09P constructor/toString identify these as prod/debug defaults. The
**effective** property remains unknown because cached/server/MobileConfig
resolution is separate. This is a different property from removal admission
35661; do not infer its value from that property's default or saved query.

When this gate is true, A0A translates the selected address for identities;
a still-zero account type yields no target, otherwise a singleton. When false,
A0k obtains the device's user counterpart set through 0Xw.A0U, rebinds each to
the same device number, then converts the list to Signal addresses. Failure to
convert the original device uses the original singleton. This branch can select
multiple mapped namespaces; it is not permission to guess PN/LID aliases.
0kT.A02 also has an independent 0x536a (21354) predicate before one mapping
branch. Its current value and full 0Xw/0DE/0hd mapping behavior remain unknown.

A0r locks the selected target set, reads old identity public keys with 0kc.A0E,
then removes with A0C and releases locks. A0E/A0C independently test numeric
31194 and can apply 0kc.A03 -> 0kT.A04 translation. A04 filters translated
account type zero; A03 preserves nonzero targets, translates zero targets via
mapping helpers, and falls back to original inputs for unresolved entries.
Those branches do not establish how an arbitrary namespace should be mapped.

A0C deletes identities rows matching recipient_id, recipient_type and device_id
(013c..0156) in a transaction. A0E's SELECT uses the same three-part tuple.
Deletion results map back to requested targets and are true only when the SQL
affected-row count is positive. A0r invokes 0k4.A0L only when an old key existed,
a deletion succeeded and B7H.A00 yields a device. Its full observer/invalidation
side effects are untraced. Missing old identities or failed deletion must not
be treated as successful identity change merely because the method returned.

### Sender-key scope and remaining port requirements

**Confirmed existing 1Uf.A02 source:** removeAllSenderKeysForSender translates
the address for sender_keys and selects sender_account_id, sender_account_type,
device_id plus group_id IN (...). It invokes the list-grouping helper with 0x3cc (972) twice
at 03fa..0404 before constructing the predicate at 0454..047a; the complete
helper algorithm is not newly established here. Group-list
production, all cache invalidation and broader callback effects remain
incomplete. This is scoped sender/device/group cleanup, not deletion of all
sender keys or all message history.

**Current Rust limitation:** signed reconciliation persists nullable
pending_removed_devices, but no deferred teardown worker or key deletion is
implemented. The port still needs effective gates, authenticated canonical
mapping, membership recheck, in-flight worker scheduling, transaction/callback
parity and precise sender-key group selection. The removal marker's post-cleanup
lifecycle remains unknown. No saved identity, session, key, event or history was
deleted or migrated during this static work; no live request was made.

The reviewed [cleanup recipe](apk/recipes/2.26.37.71-companion-cleanup.json)
retains complete selected methods with exception tables. The full DNt method is
retained to preserve its selector mapping; only discriminator 32 is interpreted
here. All other callback cases remain outside this finding. Exact shorties are
recipe constraints; apk-smali itself accepts only a method substring filter,
not --method-shorty. A failed use of that unsupported flag produced no evidence.


**Observed export/validation:** the 17-topic recipe exported 17 methods /
22 files / 181102 bytes, within the 2 MiB bound, with ordered exception tables.
Direct individual method dumps were checked against corresponding bundle files
for instruction offsets/mnemonics and ordered catch ranges; the renderers
differ in operand detail and full operand equality is not claimed. The
[complete direct dumps](companion-cleanup-evidence.txt) retain those operands;
only the CDO class-wide topic intentionally includes its full initializer.
All 59 xtask tests passed. This is media-free static evidence, not runtime
teardown, automatic release relocation or live peer-removal verification.

Regenerated CLI reference, experimental API-symbol check and formatting also
passed on system Rust 1.99.0. Runtime Rust behavior is unchanged; full workspace
tests/Clippy were not repeated for this static evidence chunk. The scoped direct
dumps retain operands that the compact exporter may omit; offset/mnemonic/catch
coverage equality is not proof of complete bytecode semantic equivalence.

Repository hygiene passed: 935 local links, tracked-file budgets, package
licenses and 4,177,269 bytes across six checksum-verified APK bundles.


## Cleanup callback commit and rollback closure (2026-10-08)

**Confirmed static 2.26.37.71:** this resolves the prior 0ZN.A04 -> 0Gy.A00
callback-delivery boundary for the selected DNt discriminator 32. Full source
is appended to [direct evidence](companion-cleanup-evidence.txt).

0ZN.A04 wraps its Runnable in Cpf discriminator 1. Cpf is a Ddg callback object,
not a Runnable itself: no run method exists. Its onBegin/onCommit/onRollback
only delegate to a SQLiteTransactionListener for discriminator 0. For 1 these
methods do nothing; Bb1 invokes 8kU.A1P, which casts to Runnable and calls run.
Bax invokes the Runnable only for discriminator 2, so the selected 1 callback
is not delivered through the failure/discard path.

0Gy.A00 registers the callback in a thread-local map and invokes onBegin.
SQLite onCommit/onRollback first notify a snapshot of callback objects;
onCommit clears the active flag, while rollback also sets the rollback flag.
Those immediate SQLite notifications do not run discriminator 1. Normal late
delivery is 0Gy.A02: when active=false and rollback=false, it copies values,
clears the map, then invokes Bb1 on that snapshot. With rollback=true it uses
A01, which clears the map and invokes Bax. The callback map's iteration order
is not newly established here; no FIFO ordering or execution during commit is
inferred. A callback exception can stop later callbacks; exactly-once successful
completion is not established merely because the map was cleared.

1gB.A00 marks the transaction successful and calls 0IY.A0G, whose wrapper invokes
SQLite setTransactionSuccessful. close calls A0F -> SQLite endTransaction first.
Only after endTransaction returns, when the local success bit is set and the
manager rollback flag is false, does it reach A02. Other paths—including a
caught endTransaction failure—use A01 and retain/rethrow the failure as traced.
A02 also requires the manager active flag to be false, so it does not deliver
while an outer transaction is still active. This establishes successful-close
late delivery of DNt32 and its cache updates, rather than updating cache merely
when a statement succeeds. Complete concurrent/nested application behavior,
callback ordering and all other discriminator bodies remain unported.

**Implemented research correction:** compact APK exports omitted decoded move,
move-result/move-exception and array-length register detail when their ordinary
annotation was empty. Interactive dumps retained that information. New optional
recipe root include_register_flow (boolean, default false) adds that supported
fallback annotation and marks opted-in targets with include_register_flow=true.
Existing nonempty annotations retain their original precedence; no whole-program
dataflow, lossless DEX encoding or new unsupported-opcode decoder is claimed.
Historical default exports remain stable. The cleanup recipe opts in; future
register-only differences in supported instructions become file differences.

A literal regression uses CDO.<clinit> 000c..000e (0c00, move-result-object v0)
and changes only the destination byte to v1. Legacy annotations remain identical;
opted-in annotations differ. Existing operand annotations and return-void are
also checked, as are rejected nonboolean recipe values/default false behavior.
No server input, identity policy or live account behavior is changed.

Actual source validation: the expanded cleanup bundle preserves 23 topics, 38
methods, 28 files and 214535 bytes. All seventeen previous offset/opcode/mnemonic
lists and ordered catch ranges remain equal. Six new direct dumps also match
those lists; this does not prove full semantic equivalence. CDO.<clinit> now
retains move-result-object v0 at 000c..000e. The [checksum-verified cached report](apk/companion-callback-flow-coverage-diff.json)
against Git 257f186 took 0.011257 seconds: six added, sixteen annotation-changed
and one unchanged topic, with manifest/inventory/tokens unchanged. This is a
same-APK source/rendering expansion, not a vendor release difference.
The earlier apk-smali query for Cpf.run returned no method bodies; full-class
inspection established its callback interface and Bb1/Bax dispatch instead.

Validation on system Rust 1.99.0: all 60 xtask tests, strict xtask/all-target
Clippy, xtask build, regenerated CLI reference, experimental OpenAPI library-symbol
check and formatting passed. Repository hygiene verified 941 local links, package
licenses, tracked-file budgets and 4210702 bytes across six checksum bundles.
Runtime Rust code is unchanged; workspace/runtime tests were not repeated for
this exporter-only Rust change.
Exact hash-pinned APK re-export with --check verified all 28 files/214535 bytes.
