# Poll vote keys and structural validation

2026-10-11 follow-up: [synchronized own-poll secret lookup](synchronized-poll-secret.md)
connects already authenticated creations to incoming peer and explicit outgoing
votes. Current Clc.A04/1hN.A02 parent-relative storage is traced independently;
full native parent hydration, special poll policy and own-device updates remain
unported. Historical local reservations retain priority and are not repaired.

## Receiver and parent-key trace (2026-10-09)

**Confirmed, static APK 2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Recipe](apk/recipes/2.26.37.71-poll-vote-key.json) and
[checksummed source topics](apk/2.26.37.71-poll-vote-key/targets.json) retain
full selected register operands and exception tables. These are build-specific
names, not the names from 2.26.37.70 or the pinned registration profile.

The exact DEX field-reference scan for `BWH.pollCreationMessageKey_:BZa`
found six references, all in `Ban.A01(LL)` and `Ban.AD7(VLL)`:
[field inventory](poll-vote-key-field-evidence.txt). This is a direct field
inventory, not a complete virtual/interface call graph. Only `A01` was traced
here; the `AD7` writer remains untraced in this chunk.

`BWH.dynamicMethod` declares pollCreationMessageKey=1, vote=2, metadata=3 and
senderTimestampMs=4. `BSp.dynamicMethod` declares encPayload=1 and encIv=2.
`BZa.dynamicMethod` supplies remoteJid=1, fromMe=2, id=3 and participant=4.
Top-level field 50 is the existing poll update envelope; vote creation and
cryptography evidence remain in [messaging transport](messaging-transport.md).

`Ban.A01` checks vote and encrypted-value presence at `0020..0066`, then
calls `C7G.A00` at `0066..007e`. The latter requires present/nonempty id and
remoteJid and a successful native ChatJid parse. Missing `fromMe` presence is
not rejected. The Rust shared key check retains local MessageId bounds and a
supported JID subset; complete native factory/subtype parity is **unknown**.
A recognized structural key is not proof of sender authority or poll ownership.

The receiver constructs a vote model from the envelope key/time and calls
`Clc.A02(sender, envelopeKey, pollCreationMessageKey)` at `00b8..00d0`.
`Clc.A02` delegates to `BoQ.A00` (`0010..0024`), passing the sender only for a
non-own envelope with a non-null sender. Thus the shared mapper is confirmed
on the poll path, independently of the earlier reaction trace. Its ordinary
branch uses the envelope chat and preserves parent fromMe; hosted, special
chat and own-envelope branches remain outside our direct-message port.
See [reaction mapper evidence](reactions.md). This call does not itself prove
that any contact-directory alias authorizes changing a saved target chat.

## Rust decoder and inspection

**Confirmed, implementation and independent wire fixture.** `decode_vote`
now applies the shared supported key check and requires timestamp, vote,
ciphertext and IV presence, with an exactly twelve-byte IV. This matches the
common supported receiver prerequisites. Both feature-selected APK branches
require senderTimestampMs: `0104..0118` and `055e..056e`. The deferred branch
checks the IV length at `0592..05a8`; the existing crypto helper also requires
it. We do not invent a positive timestamp requirement: present zero/negative
values remain structurally decodable. Present empty ciphertext is also
structurally decodable and subsequently fails AEAD authentication. Metadata
presence is not required by this common check.

`lab-inbox --inspect` calls the public `polls::vote_validation` independently
of successful decoding, so refused saved updates explain themselves without
printing target keys, option names, IVs, ciphertext or timestamp values. Labels:

- `valid`, `missing_key`, `missing_id`, `invalid_id`;
- `missing_remote_jid`, `invalid_remote_jid`, `unsupported_remote_jid`;
- `missing_sender_timestamp`, `missing_vote`, `missing_enc_iv`, `invalid_enc_iv`,
  `missing_enc_payload`.

**Unknown/unsupported:** undecodable protobuf or absent field 50 returns no
label. `valid` means supported structure only, not a valid decrypted vote,
resolved poll, acceptance, delivery or complete APK behavior. The diagnostic
never changes saved ratchets, projection markers, keys, request IDs or waits.
The raw `vote_key_fields` helper remains a raw diagnostic API and does not
validate or authorize those values.

**Confirmed, static branch; not ported:** property `0x1a74` chooses immediate
versus deferred vote decryption in `Ban.A01`. The immediate branch resolves a
parent via `1hQ.A02`/`Ayn.A0T`, checks a poll model, handles end-time rejection,
edit metadata via `CfV`, and resolves option hashes via `Cjh.A00`. The effective
property, transitive parent lookup, expiry/edit policy, quiz handling and full
option validation are **unknown/incomplete**. Absence of metadata in one
supported branch must not be promoted to complete edit/quiz support.

## Remaining chat binding

**Confirmed, local code audit; not corrected by structural validation:**
`inbox_events::poll_context` loads our own `poll-secret:<id>` without using
its saved `chat`; it projects outgoing-poll updates under the envelope sender.
`outgoing::vote_payload_for` checks the caller-supplied target chat but likewise
does not compare the saved own-poll chat before using its secret. Normal send
reservation saves `chat` alongside secret/options/creator identities. Some
historical fixtures lack chat; their binding is **unknown**, not permission to
invent one. Incoming historical PN polls already use original authenticated
capture proof through `direct_record_in` and retain their stored target key.

Next work must bind outgoing poll targets to saved evidence, retain actual
voter identity and AEAD identity candidates, and refuse foreign stored poll
references without replaying sends or guessing aliases. Existing projection
markers and uncertain sends must remain unchanged. No fresh live poll or vote
was sent for this structural correction.

## Verification

Exact APK/recipe re-export passed `apk-research-bundle --check`. A checksum-
verified cached self-comparison took 0.020667 seconds for seven source topics
and four metadata artifacts. This exercises fast cached comparison, not a
new-release equivalence claim; no APK is read by the diff command.

The shared Rust change passed 732 workspace/all-feature tests, strict workspace
Clippy across all targets/features, warning-free Rustdoc, 24 default CLI tests,
CLI reference regeneration, API/library-symbol validation and formatting.
The CLI subprocess regression inspects a saved malformed poll key and reports
the missing remoteJid even though decoding refuses; the private target is not
printed. Repository hygiene checks 1,075 local links and 5,208,567 bytes across
11 checksummed APK bundles. Offline historical projection upgraded zero events;
existing durable markers were preserved.

The recovered active profile's peer keys, action outcomes, receipts, pending
outbox and event summary were unchanged after offline projection/inspection.
Normalized capture diagnostics were also unchanged. It contains no decodable
saved poll updates, so this inspection does not establish retained/live poll
interoperability; the existing two saved reaction keys still validate. New poll
validation is covered by the independent wire fixture and CLI subprocess test.

## Saved chat attribution (2026-10-09)

**Confirmed, local implementation correction.** This supersedes the initial
chat-binding gap above. Normal `outgoing::prepare` already atomically saves
`poll-secret:<wire-id>.chat` with the creation secret and its reserved send.
The correction uses that existing private field; it introduces no new request
format or protocol attribute.

`poll_context` now resolves an own poll only when its saved chat matches the
actual primary envelope sender, either exact address components or one
existing `authenticated-lid:<primary-lid>` PN association in either direction.
PN and LID remain distinct namespaces. Wire Plain/zero-component AD encodings
may represent the same primary address. Raw `sender_pn`, a directory/contact
`lid:` alias, unrelated addresses, nonzero agent/device, groups and peer routes
do not supply a binding. Successful vote AEAD projects the original saved poll
chat with outgoing direction/no target author while retaining the voter actor.
The original chat is also an authenticated voter-identity candidate, allowing
PN-encrypted votes arriving through an already associated LID. Existing incoming
historical poll lookup and original target-author attribution remain intact.
Previously committed event markers are never rewritten to change attribution.

Missing/null legacy chat provenance remains **unknown**: receive stores an
opaque supported placeholder rather than guessing a parent or decrypting with
an unrelated own-poll secret. A foreign/unattributed/companion target similarly
remains opaque. A malformed/non-string/unsupported saved chat or malformed
saved authenticated association produces a fixed local error atomically.
No secret, address, option or raw JSON is included in those errors.

`outgoing::vote_payload_for` additionally compares the actual saved own-poll
chat against the requested peer before identity lookup and vote entropy.
Refusals distinguish `saved poll has no chat binding`,
`invalid saved poll chat binding`, `saved poll belongs to another chat`, and
`invalid authenticated LID mapping`. They do not erase state or rotate IDs.
This checks preparation, not the offline `vote` enqueue operation: an invalid
queued intent may receive a local preparation refusal. Incoming poll voting
still requires the authenticated poll under the requested chat's exact key.

`polls::own_chat_binding(store, poll_id, peer)` is a read-only inspection of
that saved binding for the supplied peer. `lab-inbox --inspect` uses it for
own-poll votes and prints `own poll stored-selector binding` with `matched`,
`unattributed`, `missing_chat_binding`, `unknown_poll`, `inspection_failed` or
`invalid_peer`. Its peer is the inspected record's selector, which can be a
retained historical PN selector; this label does not independently recover an
envelope or prove that event projection would succeed. Vote cryptographic
inspection remains separately labelled. No query, receipt or send is queued.

**Confirmed APK context, not full native parent-store parity:** the prior
seven-topic 2.26.37.71 bundle traces `Ban.A01 -> Clc.A02 -> BoQ.A00` on poll
keys. It confirms envelope-chat reconciliation and common key validation.
Our own secret's saved chat and authenticated association are local durable
provenance guards, not claimed native database schemas. Transitive native
parent lookup/storage, key subtype parity, effective property, expiry/edit/
quiz/companion/group policy and fresh live PN/LID poll interoperability remain
**unknown/incomplete**. No parent-key or sender metadata alone authorizes pin
adoption, directory repair or retrying uncertain sends.

Offline tests exercise actual AEAD votes for exact PN/LID, both authenticated
namespace directions and zero-component AD encoding; the original target chat,
voter, selected option and direction survive durable projection/reopen without
a duplicate event. Unattributed, alias-only, foreign, missing-chat, corrupt-map,
corrupt-chat and companion cases verify opaque/error boundaries and unchanged
checkpoint state. Preparation fixtures verify encrypted valid votes, all fixed
refusal classes and read-only inspection without creating an outbox entry.

The new saved-binding helpers are available with the comparison receive feature
set (`experimental-signal-crypto`, `experimental-noise`,
`experimental-transport`). The initial default CLI build exposed an accidental
reference to gated receive modules; matching helper gates corrected that build
boundary without enabling experimental transport for default users.

This chat-binding chunk passed 734 workspace/all-feature tests, strict workspace
Clippy, warning-free Rustdoc, 24 default CLI tests, CLI reference regeneration,
API/library-symbol checks and formatting. The expanded CLI subprocess fixture
reports missing legacy binding without printing the private option. Hygiene
validated 1,078 local links and all eleven APK checksum bundles. Offline active-
profile projection upgraded zero events; peer/action/receipt/outbox/event
summaries and normalized capture diagnostics were unchanged. It has no saved
poll updates, so this does not establish retained-poll interoperability.
New canonical records retain actual voter namespaces; historical record/event
selectors and already committed markers retain their earlier representation.

**Observed, live runtime only:** a 45-second receive-only window authenticated,
entered active state and stopped cleanly with exit 0, chat sends disabled and
reconnect attempts zero. Peer keys, 20 action outcomes (including two retained
uncertain `sent` actions), nine acknowledged receipts, empty outbox and event
summary were unchanged. Fifteen decrypted payloads remained fifteen; capture
shape lines increased 432 to 437, not five user messages. No fresh DM/poll/vote
established new attribution behavior. No uncertain send/action was replayed.
