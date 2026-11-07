# Optional receipt addresses and fact projection

2026-10-11 follow-up: [typed recipient admission](receipt-user-recipient.md)
distinguishes UserJid recipient from generic participant. Recognized non-user
and device recipients also follow the native null path; original target guards
and generic participant handling remain separate.

2026-10-11, **implemented/offline**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
Supported direct/group receipt facts now use the same nullable namespace
admission as the [receipt ACK builder](receipt-ack-namespace.md). An optional
address that fails this check no longer suppresses a supported ordinary receipt
just because its raw attribute exists.

## Confirmed APK evidence

[CmG.A03](apk/2.26.37.71-direct-receipts/receipt-wrapper.txt), `0010..0020`,
reads `recipient` through `0hs.A0A(UserJid, name)`. The shared
[CmG.A00 route builder](apk/2.26.37.71-direct-receipts/receipt-route-builder.txt),
`0020..0030`, reads `participant` through `0hs.A0A(Jid, name)`. These nullable
values become separate descriptor fields; raw attribute presence is not the
same as a successfully parsed qualifier.

The complete retained [A0A accessor](rea/2.26.37.71-receipt-list-shape/optional-jid.json)
uses the [nullable factory](rea/2.26.37.71-receipt-list-shape/nullable-jid-factory.json)
for text, returning null on failed parsing. The complete
[factory smali fallback](rea/2.26.37.71-receipt-list-shape/jid-parse.json) throws
for unknown namespaces, while the [server-only parser](rea/2.26.37.71-receipt-list-shape/server-jid-parse.json)
recognizes only `s.whatsapp.net`, `g.us` and `call`. Empty user text before `@`
also fails. Java decompiler warnings/stubs are not substituted for the exact
factory trace. This change reuses that evidence; it adds no new APK observation.

## Runtime and retained authority

`receipts::record` and its transactional direct/group handlers now check a
nullable admitted recipient, rather than mere raw presence. An admitted
recipient still selects an unsupported qualified route and leaves it unprojected.
For direct receipts, a participant rejected by the same namespace check is
treated like an absent optional participant. A recognized participant still
requires the original addressed device and authenticated chat association.

For groups, the participant is still required, must be supported and must match
an original addressed device. Ignoring a bad optional recipient does not supply
a missing participant or add a target. Message IDs, original chat correlation,
PN/LID associations, frozen target snapshots, independent delivery/read/played
facts, atomic batch rollback and ambiguous-ID refusals remain effective. No
mapping is adopted from an optional field. Aggregate routing uses the same
recipient admission before its existing inner-ID and per-target parsing.

The original node and raw capture are retained. The shared general Jid parser,
ACK correlation, read policy, retry-message routing, outbox attempts, request
IDs, account keys and server waits are not changed. Existing CLI/API fact fields
show the recovered observation; no new endpoint or configuration is introduced.

## Verification and limits

The direct regression failed before implementation on an unparseable participant.
After actual binary encode/decode it covers PN/LID originals, participant versus
recipient, unknown namespaces, invalid bare lid, empty-user `@` forms and legacy
c.us, independently for delivery/read/played. Original intent, attempts, progress,
prior uncertainty and received actor/time are checked; duplicates across reopen
append no new outbox event or fact. The group regression covers all three facts,
original-device selection, unknown/mismatching participants and reopen recovery.
Existing recognized-qualifier, self-status, mapping, target and rollback tests
continue to verify their refusals.

Validation: 917 workspace/all-feature tests, strict Clippy, warnings-denied
Rustdoc, default CLI/API-symbol tests, CLI-reference check, CLI build and
repository formatting/link/evidence hygiene passed.

The namespace helper does not port every specialized native JID grammar or
UserJid subtype cast. Recognized unsupported qualifiers remain deferred;
read-self/played-self, full native dispatch and fresh live fact projection remain
unported/unverified. No account operation or network request ran for this change.


## Current recipient text-class follow-up (2026-11-06)

[Current receipt-class correction](receipt-user-recipient.md#current-text-class-admission-including-dot-qualifiers-2026-11-06)
now applies the existing native PN/LID/hosted classifier before generic parsing.
Dot-qualified zero/ignored suffixes and constructor-invalid users cannot gain
UserJid recipient authority. They omit the ACK recipient and allow ordinary
facts only under the unchanged original-chat/device/attempt correlation.
Generic participant handling and strict outbound-receipt ACK settlement remain
separate. No global parser, alias normalization, retry or live operation changed.


## Non-user text constructor failures in receipts (2026-11-06)

**Confirmed selected static / implemented**, pinned APK2.26.43.72 build264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
The nullable receipt reader is generic Jid for source/participant, not only
PN/LID. Its constructor failures must remain null even when Rust's wider generic
Jid parser accepts the text. Previously the receipt helper checked only the
PN/LID/hosted classifier. Invalid group/broadcast/newsletter/temp/call strings
could consequently become selected ACK addresses or suppress direct facts as
if they were nonnull participants.

The shared text-null check now also reuses the exact existing
ack_non_user_text_valid and ack_call_text_valid predicates. A null source grants
no response through the client's required-source capability guard; a null optional
participant is omitted from ordinary, copied487 and fresh487 ACK contexts. Direct
fact projection treats it as absent and retains its original chat/device guards.
Successfully constructed objects keep their own protocol-compliance/writer rules;
valid temp/type2 still refuses the whole ACK rather than being omitted as null.
Typed/binary objects are not reclassified through text constructors. No cache,
identity association or generic JID-model rewrite is introduced.

Evidence is reused, not newly observed execution:

- [Group/newsletter constructors and complete selected factory/accessor chain](apk/2.26.43.72-group-newsletter-ack-text/README.md): group1bc/0nW accepts selected numeric or phone-time shapes; newsletter uses Java-trimmed nonempty/nonzero-leading text, preserving the original user. Newsletter is not generally digit-only.
- [Broadcast constructor and cache boundaries](apk/2.26.43.72-broadcast-ack-text/README.md): selected2U8 numeric/status/location construction, nullable factory, generic cache and selected call handling remain separately evidenced.
- [Call and noncompliant temp constructor boundary](apk/2.26.43.72-call-temp-ack-text/README.md): retained factory coverage and selected predicate are reused; valid object/writer refusal differs from construction failure.
- [Receipt routing/context and copied error policy](apk/2.26.43.72-unhandled-receipt-root/README.md): original field selection precedes ACK construction; this correction prevents resurrecting known-null fields there.

Regression cases cover invalid group numeric/phone-time forms, broadcast leading
zero, newsletter zero-leading letters, temp without hyphen, and call user shorter
than the selected18..32 ASCII-hex grammar. Valid group and existing valid-but-
unwritable controls remain distinct. The direct-fact matrix covers PN/LID peers,
all three fact kinds, and invalid optional participant/recipient values with
existing idempotence/target/ratchet/identity guards. The copied487 receive-loop
case now includes a constructor-null group participant, explicit redelivery,
capture-before-write and cut/reopen without uncertain replay. The older generic-
participant codec test incorrectly expected 123@call and 123@temp to remain nonnull; those expectations are corrected to their retained
Dv5/7Kr constructor evidence. Codec tests encode the resulting ACKs. These are
local offline guarantees, not server acceptance.
Complete binary/string/cache/native scheduling parity and other sender grammars
remain open. No live account operation or account-restriction guarantee.


## Shared definite-null text factory/cache boundary (2026-11-06)

**Confirmed selected static / implemented**, pinned APK2.26.43.72 build264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
The earlier receipt null helper reused individual recognized-family constructor
predicates, but not the selected unknown-bare/domain/empty-user factory checks.
Consequently generic ordinary receipt output omitted unparseable optional text,
while retry's context guard could refuse that same null field and suppress its
otherwise supported schema/version response or completion confirmation.

The existing call-ACK definite-null text predicate is now factored into a shared
pure helper and reused by receipt construction/facts. Call sender handling retains
its separate missing-attribute behavior; typed/binary values still do not pass
through text factories. Empty text, empty user, unknown qualified domain and
unknown bare name OUTSIDE the recognized parser/factory/cache names remain null.
Recognized family names without a fully classified bare route remain unclassified;
false is not a claim of native validity. Native bare s.whatsapp.net/g.us/call and
status_me/lid_me must not be erased as parser failure.

The [complete factory, cache-field/caller census and limitations](apk/2.26.43.72-call-sender-text/README.md)
are reused. Its0Dd.A00 domain switch/default throw,0De.A00 bare factory,
0De.A03/A04 selected cache producers and0Dd.A02 optional catch boundary support
this selected null classifier. No new APK runtime observation is claimed. Existing
PN/LID/hosted/group/broadcast/newsletter/temp/call constructor predicates and
valid-but-unwritable distinctions remain unchanged. Full reflection/native cache
mutation, unclassified recognized-family and typed-JID behavior remain open.

Retry's context guard now sees these text failures as omitted optional
participants. The direct root still selects the requesting device, the explicit
recipient capability guard remains unchanged, and pure success ACKs remain
unexposed until actual durable retry work commits. Error487 schema/version
responses may now omit a null participant instead of being incorrectly blocked.
Unknown typed objects remain local unsupported context and cannot gain the same
omission merely by matching a text spelling.

Tests cover empty/bare/domain/empty-user participant text, fresh error487,
handled candidates, codec encoding, and no generic valid-retry success ACK. Bare
nonnull controls retain their distinct classification. An older version-error
test incorrectly expected participant=bad to suppress the response; it now
expects copied487 with no participant, while root-source/explicit-recipient/id
local refusals remain. The saved-session count2 query/ACK/refetch workflow now
uses unknown bare/domain optional text and succeeds; preflight refusal fixtures
use a typed unsupported namespace instead. Existing overflow-null tests and
unencodable/valid-but-unwritable preflight checks remain. No identity mapping,
new query policy, native error cause, HTTP route or live operation is introduced.
