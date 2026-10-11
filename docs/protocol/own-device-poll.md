# Own-device ordinary poll creation

Follow-up: [own-device ordinary vote updates](own-device-vote.md) now project
supported wrapped votes with authenticated own voter and correct parent ownership.
Special poll lifecycles and full native parent policy remain unfinished.

Follow-up: [voting on synchronized own polls](synchronized-poll-secret.md)
connects authenticated parent/secret fallback to incoming peer vote context,
explicit outgoing votes and binding inspection. It supersedes the lookup gap
below; synchronized vote updates and special poll lifecycles remain unfinished.

2026-10-11. **Implemented/offline**, WhatsApp APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
A held, pinned own-device session can now synchronize ordinary poll creation
into an outgoing NewMessage with recipient chat, question, options and selectable
count. No sender, send intent, key adoption or automatic vote is created.

## Source and port

The [recipe](apk/recipes/2.26.37.71-own-device-poll.json) and
[checksummed DEX bundle](apk/2.26.37.71-own-device-poll/targets.json) retain eight
topics: Message, DeviceSentMessage, poll, option and Message-context descriptors,
context-aware unwrap, wrapper validation and own-source gate. These map to the
existing receiver, wrapper, projection and poll decoder. The common
[DeviceSentMessage trace](device-sent-wrapper.md) establishes one-level
unwrapping, destination validation and outer Message-context replacement before
content dispatch. This supports connecting ordinary poll creation to the existing
projector; it does not establish every poll receiver/storage effect.

BZc.pollCreationMessageV3 is field 64. BYa's current descriptor has **11 fields**:
encKey=1, name=2, options=3, selectableOptionsCount=4, contextInfo=5,
pollContentType=6, pollType=7, correctAnswer=8, endTime=9,
hideParticipantName=10 and allowAddOption=11. BSo.optionName is 1;
BQ2.messageSecret is 3. Full [class inventory](rea/2.26.37.71-own-device-poll/poll-class.json)
and [descriptor rendering](rea/2.26.37.71-own-device-poll/poll-descriptor.json)
are retained. The helper-written name slot and enum verifiers remain explicit
in the DEX export; the field numbers come from its encoded descriptor, rather
than equating Java object-array indices with tags. Older eight-field notes in
[messaging transport](messaging-transport.md#polls-comparison-2263771) are incomplete
for this pinned release. Ordinary `type=poll`/`meta polltype=creation` was also
observed historically on the official peer path; that is not a fresh own-device
observation.

The wrapper now accepts `type=poll` and exactly one supported inner field 64
plus optional Message context. A raw poll submessage survives outer-context
replacement, including the vote secret carried by that context. The existing
poll decoder/projector enforces its question/option bounds and publishes the
normal MessageContent::Poll. Unsupported or malformed content returns before
committing the advanced receiving session.

**Local supported-subset guard:** own-device poll bodies with any fields outside
1..5 refuse, including explicit defaults for 6..11. Quiz, timed, anonymous and
add-option semantics are not silently flattened into ordinary polls. Their
native lifecycle and enum/default equivalence need a separate port. Field 50
updates and mixed ordinary content remain refused by this wrapper route.

## Storage and API

The authenticated transaction retains the original wrapper/source/route under
`received-own:<recipient>:<id>` and commits the advanced held session, Inbox
marker and outgoing event together. Original request IDs, pins, server waits,
read policy and outbox state are unchanged. Existing
[echo correlation](own-device-echo.md) applies to a matching normal Poll intent
by saved wire ID and authenticated chat; it does not establish a send outcome.
The existing listen/event APIs and saved-inbox inspection expose the result.
No route, option, dependency or schema variant is added.

The same committed checkpoint permits the existing sender confirmation and
same-source deduplication after reopen. Local clear removes the wrapper's
plaintext/secret and prevents republication; historical journal entries retain
their existing semantics. Secrets are not added to the event journal or exported
as candidate state.

**Remaining:** the original synchronized secret is retained in the wrapper but
is not imported into `poll-secret:` or used as a fallback by vote lookup. Votes
on a newly synchronized own poll therefore remain unsupported until ownership,
parent selection and secret retrieval are traced and connected. Own-device vote
updates, add-option/results/quiz lifecycle, special/group routes, bootstrap,
alternate-recipient mapping and cross-source reconciliation are also unfinished.
A creation event is not complete synchronized polling support.

## Verification

The real Signal regression failed with Unsupported before the route change.
It now verifies outer-context restoration of a 32-byte secret, exact outgoing
poll content/chat/direction, original wrapper retention, sender confirmation,
duplicate after reopen, clear/redelivery refusal and continued ratchet use.
The same test still covers own text/media, outgoing echo and original uncertain
send-state preservation. Unit coverage retains an ordinary body and rejects
special poll fields 6..11, vote updates and mixed content. Existing ordinary
poll decoder/projector checks remain shared.

No saved account or live request was changed. Fresh own-device poll acceptance,
full APK parity and ban immunity remain unverified.

Validation passed 946 workspace/all-feature tests, strict workspace Clippy and
warnings-denied Rustdoc. The real Signal regression also passed with only TCP,
Noise and Signal enabled. The eight-topic bundle reproduced 13 files / 87,604
bytes. All 26 default CLI tests, the experimental CLI build, API-symbol check,
formatting and repository link/evidence hygiene passed.
