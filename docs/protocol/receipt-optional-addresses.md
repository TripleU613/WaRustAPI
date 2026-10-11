# Optional receipt addresses and fact projection

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
