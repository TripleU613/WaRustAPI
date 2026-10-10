# Identity notification key-read phase

APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
Offsets are method-relative bytes. **Confirmed static** describes inspected DEX;
**implemented/offline** describes Rust tests, not a live server observation.

The follow-up [changed-primary application](identity-refresh-application.md) now
implements guarded classic changed-key adoption under the explicit receive policy.
This original read-phase trace still applies; identity ACK remains deferred.

## Normal incoming callback address

**Confirmed static:** [0yO.BFC](apk/2.26.37.71-notification-key-read/notification-entry.txt)
extracts the incoming root (`0032..003a`), calls `CmG.A02` with null error
(`0054..005a`), and runs FpD selector **9** (`005c..0070`). An exact `14z.BFC`
body search found no matching body; this is its base notification path.
[Constructor](apk/2.26.37.71-notification-key-read/notification-dispatch-constructor.txt)
stores argument 3 as handler/A01, argument 1 as incoming/A02 and argument 2 as
descriptor/A03 (`000a..0016`), so the run branch uses the just-built descriptor.

[CmG.A02](apk/2.26.37.71-notification-key-read/notification-descriptor.txt)
passes a null address, `notification` class, null type default and null failure
reason to [CmG.A00](apk/2.26.37.71-notification-key-read/descriptor-address.txt).
Because the address is null, A00 reads root `from` as generic Jid (`0000..0014`)
and writes it to `CSu.A02` (`004a..004e`). Root `participant` is read separately
(`0020..0030`) and written to `CSu.A01` (`0056..005a`). This wrapper does not
read `recipient` to replace A02. Existing
[CSu.A00/Cb1 constructor evidence](identity-notifications.md#callback-address-construction-continuation-2026-10-09)
then carries A02 unchanged to Cb1.A02.

[FpD.run](apk/2.26.37.71-notification-key-read/notification-dispatch.txt)
selects `0768` for 9, reads the descriptor unchanged, and either calls
`0yO.A01` (`07e2..07e8`) or queues Fpc selector **3** (`07f2..0804`).
[Executor constructor](apk/2.26.37.71-notification-key-read/notification-executor-constructor.txt)
keeps handler/incoming/descriptor/string in A01/A02/A03/A04 (`000a..001a`).
[Executor run](apk/2.26.37.71-notification-key-read/notification-executor.txt)
case 3 (`0072..009a`) forwards those same arguments to `0yO.A01`.
The [base handler](apk/2.26.37.71-identity-notifications/identity-handler-ack.txt)
assigns Cb1.A02 to callback A01 at `0146..014e` before A09. Thus the normal
identity callback device is derived from **root from**, not participant or an
assumed alias selected by the separate metadata parser.

[0DK.A00](apk/2.26.37.71-held-bundle-validation/notification-device-conversion.txt)
keeps DeviceJid unchanged or uses UserJid.getPrimaryDevice. Its
[predicate](apk/2.26.37.71-notification-key-read/device-type-predicate.txt)
accepts nonnull Jid type values `0, 11, 18, 22, 26, 7` (`0000..003a`).
**Unknown:** the full native subtype/server correspondence; do not map all those
numbers to Rust namespaces by guess. Rust currently supports nonempty PN/LID
users with agent zero and ordinary plain/AD encodings, preserving the actual
device and canonicalizing only primary encoding. Hosted/interop/other native
subtypes remain unported.

## Other writers and negative results

The exact [34 direct-write matches](notification-address-writes.json) include
outbound receipts, messages and descriptor copies; they are not 34 independent
notification targets. The retained [Cb1.A00 copy](apk/2.26.37.71-notification-key-read/descriptor-copy.txt)
reads/writes A02 at `000a..0012`. The
[0h8.A02 normalization](apk/2.26.37.71-notification-key-read/descriptor-primary-normalization.txt)
can unwrap device-zero A02 to its user (`0000..003a`) and independently normalize
A01 (`003a..006e`). This method is not called in the selected BFC/FpD/Fpc path.
[D34.B38](apk/2.26.37.71-notification-key-read/notification-deferred-descriptor.txt)
also delegates directly to CmG.A02 (`0000..0008`); its complete outer scheduling
remains unknown.

Fresh method inspections eliminated these candidates from the normal constructor
trace: `0zH.A05` builds class `message` (`0026..002a`), `0zH.A06` builds a
`picture` notification (`0034..0040`), `12s.A01` builds a final-location
notification (`0000..0004`, `0058..005c`), `1Qy.A02` is AckStanzaHandler,
`1RZ.A03` is StanzaParsingErrorHandler, and `D54.BEw` handles bot-message
plaintext (its A02 write is `051c..0520`). CmG.A01's selected wrapper is `call`
(`0034..0038`), while CmG.A03 is `receipt` with default `delivery`
(`0030..0038`). These do not establish a participant-selected identity refresh.
The inspected `1RV.A06` parsing-error path calls CmG.A02 (`01d2..01d8`) before
1R9.A05; it does not redefine the normal BFC address. These negative traces
are reproducible with `xtask apk-smali --class ... --method ... --signatures
--exceptions`. Reflection/native writes and other untraced callers remain unknown.

## Durable Rust read phase

**Implemented/offline:** `identity_notifications::queue` recognizes only
`notification type=encrypt` with first child `identity` (14z.A09's selection).
It reads the supported device from root `from`; participant/recipient are separate
and never substitute it. The native DLp worker's existing-identity gate
(`0246..0252`, [scheduler evidence](identity-notifications.md)) is applied to the
exact saved device pin. Missing pins skip; malformed saved keys refuse with
`invalid saved peer identity`. Recognized supported input requires a nonempty
notification ID of at most 128 bytes (local bound), with fixed refusal
`missing or excessive notification id`. Account and saved login PN/LID aliases
protect own primary/companion identities. Unsupported routes stay captured.

One store transaction records hashed peer/event markers and an `identity_refresh`
action with the existing [reason=identity builder](bulk-prekey-refresh.md).
The authenticated actor captures before this transaction. Its ordinary action
pump reserves the query before socket write, including while chat outbox sends
are disabled. Every duplicate event returns its original action. A different
event for the same device cannot reset a queued, uncertain, refused or invalid
notification read. A genuinely new event after a successful signed bundle can
queue another read. Signature validation here permits another **read**, not pin
replacement. This is the local once-only policy; native job retries, complete
in-flight filtering across other query families, conditional owner sync, PQ and
ADV preparation remain unported.

The generic action queue saves correlated replies/refusals. `actions --show`
recognizes this action kind and prints an offline structured snapshot, including
namespace/device, signature validity, current-pin agreement and a numeric refusal
code from the root or exact requested device. It prints no raw nodes, peer
addresses or public keys. Generic `ok` and `verified_bundle` are distinct; even
a verified mismatching identity remains unadopted. Inspect with:

```sh
whatsapp-cli --profile PROFILE --json actions
whatsapp-cli --profile PROFILE actions --show ACTION_ID
```

Use `listen --config PRIVATE_JSON --allow-live-request --seconds 45` for an
explicit bounded receive window; queued actions transmit independently of
`--send-outbox`. `actions` status `ok` proves a generic correlated result only,
not a valid key signature or successful adoption. This phase does not derive or
save a session, change a peer pin, emit an identity notification ACK, merge
metadata/aliases or finish the native refresh lifecycle. Unsupported processing
must not be reported as completed compatibility.

## Offline evidence and remaining work

The [synthetic route fixture](../../crates/whatsapp-api/fixtures/prekeys/2.26.37.71-notification-routes.json)
is manually grounded in the selected constructor path, with no real account
values. Tests cover PN/LID primary/companion routes, AD-zero primary encoding,
participant/recipient separation, missing/invalid pins, first-child dispatch,
unsupported routes and own-account aliases. Generated signatures distinguish
valid versus forged prior responses and a changed key versus its preserved pin; refused, empty and uncertain reads stay
blocked. Reopening the encrypted store preserves IDs, pins, opaque saved session rows and event
head. The read-phase fixtures deliberately do not derive a session. A fake actor with chat sends disabled captures an identity-first/count-
sibling notification, reserves the exact device query, cuts its first write,
and reconnects without repeating it or emitting an identity ACK. That test
establishes local durability, not real account or server behavior.

**Unknown / unported:** signed response admission and atomic session adoption
without a chat send, full native hybrid/companion gates, stale-response pin
version handling, mapping authority, deferred ACK lifecycle and fresh live changed-identity/companion
admission. A primary-LID read was subsequently observed below. The next implementation needs those boundaries;
no new account, identity or request-ID replay was used. The bounded live
read below uses the existing saved account.

The [hash-pinned recipe](apk/recipes/2.26.37.71-notification-key-read.json)
retains 11 topics / 16 files / 102,122 bytes with signatures, register flow and
exception tables. Exact regeneration and cached self-diff validate retention;
they do not compare a new release or prove server behavior.


## Bounded live receive/read validation (2026-10-09)

**Observed live:** one saved-account connection authenticated, completed 45
receive seconds and stopped cleanly (60.52 seconds including connection setup).
The private active configuration was copied with automatic reconnect attempts
zero and automatic reads false; chat outbox sends stayed disabled. Preflight
found no queued chat intent/action; two historical uncertain actions remained
reserved. Originals, account keys and identifiers were preserved.

Saved state added one `identity_refresh` action with a matching generic `ok`
result, one peer marker, one event marker and nine inbound capture rows. Offline
`actions --show` on that same saved action reported `verified_bundle`, namespace
`lid`, device **0**, `signature_valid=true`, `pin_matches=true` and no refusal
code. Thus this is a successful automatic **primary-LID key read**, not an
observed key change or identity adoption. The notification's age/redelivery
history is unknown. The current tracing filter printed no identity-read INFO
line; durable markers/action/reply establish the read, not log absence.

All 20 earlier actions compared exactly equal, including the two uncertain
reservations. Chat outbox and nine receipt outcomes compared exactly equal;
event head stayed equal. Checkpoint count increased by 12, entirely the three
read records and nine inbound captures; other table counts stayed equal.
There were 11 `Unsupported` incoming log entries and no `Decrypted`,
`Undecryptable` or `Duplicate` entries. These are control classifications, not
missing or delivered user messages. Count equality is not byte-for-byte proof
of existing pin/session content; the read-phase code has no adoption write.

[Public redacted aggregates](identity-notification-key-read-live.json) carry no
account/peer/action ID, keys, request body, timestamps from the wire or plaintext.
Full local logs/config/response remain in ignored private validation storage.
The preflight wrapper's initial capitalization check stopped before network
access; exactly one live receive window ran after correction. No live replay
was performed to test the new offline inspector or repeat the notification.
Changed-key response admission, session adoption, companion gates, ACK lifecycle
and human messaging confirmation remain unverified/unimplemented as stated above.

## Offline session preview continuation

An invokable [session preview](session-refresh-preview.md) now stages and discards
a primary classic same-identity session from a saved read without chat encryption
or store writes. The actor read phase above still does not adopt responses or
ACK identity notifications. Native stored attestation state and admission remain
unported; preview success explicitly reports admission unevaluated.

## Original refresh guards continuation

New queries now atomically retain an immutable notification-time identity
snapshot in addition to the original action/event/device markers.
[Guard behavior and historical limits](identity-refresh-guards.md) cover delayed
responses and primary/companion A->B->A transitions. `actions --show` exposes
fixed redacted `refresh_guard` labels; matching is not admission or commit
permission. Older live reads above remain untracked, without backfill or replay.
