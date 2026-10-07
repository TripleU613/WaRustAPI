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

The disassembler currently shows field opcodes `0x53`/`0x55` as unknown
mnemonics, while retaining operands. They are DEX `iget-wide`/`iget-boolean`;
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
