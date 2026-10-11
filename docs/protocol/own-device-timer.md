# Legacy disappearing timers and own-device synchronization

2026-10-11. **Implemented/offline**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
Held authenticated own-device sessions now project legacy timer set/off and
sync-response messages into outgoing DisappearingTimer events. Shared timer
parsing also fixes signed duration interpretation and current disappearing-mode
field numbers. Full setting application/expiry/ordering state remains unported.

## Native builders and signed values

The [recipe](apk/recipes/2.26.37.71-own-device-timer.json) and
[DEX bundle](apk/2.26.37.71-own-device-timer/targets.json) retain eleven topics:
D9s/D9i receivers, Bn8 normalization, 1xl zero helper, protocol enum/descriptors,
BW7 mode, after-read adapter/property, context unwrap and wrapper validation.
Complete independent REA envelopes retain the
[type-3 builder](rea/2.26.37.71-own-device-timer/timer-builder.json),
[type-4 builder](rea/2.26.37.71-own-device-timer/sync-builder.json),
[normalizer](rea/2.26.37.71-own-device-timer/sync-normalization.json),
[zero](rea/2.26.37.71-own-device-timer/zero-helper.json),
[enum](rea/2.26.37.71-own-device-timer/protocol-types.json) and
[mode descriptor](rea/2.26.37.71-own-device-timer/mode-descriptor.json).

C4F assigns A0D=EPHEMERAL_SETTING/3 and A0E=EPHEMERAL_SYNC_RESPONSE/4.
The [exact A0E field census](rea/2.26.37.71-own-device-timer/type-four-field-references.txt)
locates D9i.CF4/AD7, constructor and enum lookup; it is not a transitive caller
closure. D9s.CF4 reads ephemeralExpiration_ as a Java int and stores zero for
nonpositive values. D9i.CF4 calls Bn8.A0q; null/nonpositive values become
1xl.A0q(), independently verified as zero. Thus raw uint32 values with the high
bit set become a native disable value, not a huge duration. The shared decoder
now interprets those bits as signed and clamps nonpositive values to zero for
both types, retaining the original wire bytes in receive records.

D9i also normalizes setting timestamps through its historical seconds/milliseconds
heuristic and records origin metadata. Those effects and subsequent chronology,
chat/default state, notifications and expiration application are not reproduced
by a seconds-only event. Java duplicate-block warnings are retained; selected
DEX register flow and branches remain authoritative.

## Disappearing mode and after-read policy

The current BW7 descriptor has initiator=1, trigger=2,
initiatorDeviceJid=3 **string**, initiatedByMe=4 **bool**. The old Rust schema
inferred a bool at 3, causing valid device-string messages to fail protobuf
decoding. The corrected schema retains both fields. Device metadata does not
supply identity, source or alias authority. Normal outbound legacy timers still
omit this optional mode; no new mode option is introduced.

Both native builders retain afterReadDuration_ separately. A positive value
with a false C8kT.A1R clears both durations. The
[adapter](rea/2.26.37.71-own-device-timer/after-read-adapter.json) delegates to
0zA.A03, which reads MobileConfig property **25649**:
[property reader](rea/2.26.37.71-own-device-timer/after-read-feature.json).
An effective property/state snapshot is not available to the current pure
legacy decoder, and the event model does not represent the second duration.
It therefore declines nonzero after-read payloads instead of falsely projecting
only legacy seconds. Negative signed after-read bit patterns are also outside
this supported model; native downstream treatment remains unported. Zero/absent
after-read duration follows the confirmed legacy branch. This guard is not
claimed as the entire native feature policy or complete after-read support.

## Receive, send validation and API

The own wrapper admits one nested field 12 plus optional Message context only
when the shared decoder yields supported type 3/4 legacy timer content. Other
protocol controls and privacy variants remain outside this own-device route.
Original protocol bytes survive outer-context replacement. Existing registered
account, own source/recipient, held pin/session, transaction and deduplication
guards remain effective. The normal projector emits recipient chat, outgoing
direction, no sender and DisappearingTimer seconds. No timer cache, setting
ordering state or sender wait is guessed. Timer-intent echo correlation and
full native setting duplicate policy remain unported.

The advanced receiving session, original wrapper/source/route and event commit
atomically. Existing sender confirmation, duplicate after reopen and local
clearing apply. Timer controls are excluded from the shared normal-own-message
parent selector. This does not send app-state patches or automatically enforce
future message lifetime; original journal history retains its existing semantics.

Normal timer intents now validate **0..2147483647** at enqueue, eligibility and
payload preparation. Larger unsigned requests refuse rather than silently
requesting a native disable. The low-level raw codec remains lossless for
research; normal send paths use the validator. Historical intents/attempts,
facts, IDs, idempotency, pins and server waits are not rewritten or cancelled.
Existing UI values 0/86400/604800/7776000 remain valid. Default-timer IQ policy
is separate and is not newly inferred from these per-chat E2E builders.

Existing disappearing/listen/event API and CLI expose the behavior. No route,
option, dependency or schema variant is added. Descriptions/input bounds now
name the signed per-chat limit and the unsupported after-read/state effects.

## Verification and limits

Two independent regressions failed before correction: the old decoder exposed
2147483648 seconds instead of zero, and a literal mode device string returned
None instead of its timer. Type-3/type-4 fixtures cover zero, one, signed maximum,
high-bit and uint32 maximum values, plus after-read refusal. Independent mode
bytes verify tags 3/4 and retained values. The encrypted outbox accepts signed
maximum, refuses unsigned overflow and preserves all tables, journal and original
idempotent ticket. Wrapper fixtures cover legacy set/sync, unknown revoke/privacy,
after-read and mixed-content refusal.

The real Signal fixture receives timer set, off and sync events, checks their
outgoing chat/direction/content and sender confirmations, reopens/deduplicates,
clears all own rows and continues the held ratchet. Existing text/media/poll/
vote/reaction/echo tests remain active. No live account or server request changed.
Fresh live interoperability, after-read/default/privacy state, full timestamp/
origin/ordering/expiry policy, bootstrap and complete native parity remain
unfinished or unverified. No ban immunity is established.

Validation: 955 workspace/all-feature tests, strict workspace Clippy,
warnings-denied Rustdoc, the real Signal test with only TCP/Noise/Signal
enabled, 26 default CLI tests and experimental CLI build passed. The eleven-topic
pinned recipe reproduced all 16 files / 88,746 bytes. No live account was opened.
API-symbol and generated CLI-reference checks, formatting and repository
link/evidence hygiene passed.
