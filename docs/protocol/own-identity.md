# Stored own-user authority before group preparation

2026-10-09. **Confirmed static**, APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Eight-topic recipe](apk/recipes/2.26.37.71-own-identity.json),
[full signatures and branches](apk/2.26.37.71-own-identity/targets.json),
[physical definitions and exact-field writes](own-identity-field-evidence.json).
This closes the BO0/BO1 implementation boundary used by
[receipt admission](receipt-admission.md) and
[identity receipt cleanup](identity-receipts.md). Full mutable cleanup and
per-user admission still remain unfinished.

## Native predicate and source provenance

**Confirmed:** a complete DEX class_data scan finds four code-bearing BO0/BO1
definitions: each in 08a and 08i. Both classes implement 08Z. 08a delegates
both methods to its 08i field; they are not two independent predicate policies.
The earlier no-body lookup of 0bA did not identify these implementations.

08i.BO0(null) returns false. For nonnull input it invokes A09 to load stored
Me state, then compares the input with A0I PhoneUserJid. If unmatched, it invokes
A04 and compares with A0H LidUserJid. Either equality returns true; otherwise
false. It does not consult a remote directory, contact alias or arbitrary
recipient-device list. 08i.BO1 invokes DeviceJid.Companion/0DK.A00, then
passes the resulting device's userJid to BO0. Thus it tests membership in the
own user namespaces, not equality with only the current device number.
0DK's supported user/device conversion was already traced in
[group envelope own-source handling](group-envelope-own.md).

A09 coordinates the stored Me load through 08t and 1XY selector 1; full load
origin/scheduling is **untraced** here. A01(Me,08i) stores the Me object and
parses its jabber_id through PhoneUserJid's 0D9.A01, assigning A0I. Missing
Me/jabber ID yields null; parse failure logs and clears A0I/current PN device.
The two exact A0I write sites belong to this method.

A04 reads nullable SharedPreferences `self_lid`, parses it through Jid's
companion, and calls A02 before marking initialization. It later compares that
value with Me.lid and emits fixed warning reasons for missing/unparseable/mismatch
cases. A02 accepts a typed LidUserJid or typed LID DeviceJid, stores the user in
A0H and a device in A0G; other input does not replace those fields in the shown
branches. Its two A0H writes are the complete direct field-write scan here.
Native preference/database/Me import authority and repair/warning side effects
remain **unknown**. No own identity is fabricated from a phone number guess.

## Rust gap and correction

**Confirmed local source gap:** connection::login already saves pn/lid from the
authenticated success stanza into encrypted `login-identity`. The device
primary-change hook and own sender-key invalidation hook instead read only
`own-lid`, the older group-preparation checkpoint. Before group preparation,
a saved authenticated own LID could therefore reach peer bookkeeping if those
hooks were invoked. Existing incoming remote-identity policy separately checks
login identity, so this is not evidence that live own keys were replaced.
The regression exercises the affected hooks directly without changing any
retained account or transmitting anything.

**Implemented:** both hooks now use one shared own-LID lookup, also exposed by
`group_send::own_lid`. It reads authenticated `login-identity.lid` and the
retained `own-lid` checkpoint, accepts a known source alone or agreeing sources,
and normalizes device/wire representation for user equality. Own PN handling
remains through the existing account record. No checkpoint is populated just to
make the guard work. Missing historical own-LID sources remain unknown.

Malformed/root-nonobject login records, nontext LID values, server-only/non-LID
identities, agent-bearing input and disagreeing available sources refuse with
fixed diagnostics naming the affected source fields. Device-bearing LID values
normalize to their user; no remote PN/LID mapping is used. `record_own_lid` now
also rejects agent-bearing input before storing it. Original records are never
repaired, overwritten or migrated during lookup.

**Local policy distinction:** refusal of two disagreeing Rust checkpoints is
our consistency guard. The native initialization trace instead shows warnings
when stored Me/prefs disagree; it is not a server prerequisite or proof of the
same refusal policy. Full native source repair is unimplemented. This correction
preserves the current remote-only bookkeeping design; native own-primary version
increments and own-device lifecycle are still separate unfinished work.

## Offline inspection and validation boundary

```sh
whatsapp-cli --profile l --json group own-lid
```

The output is `{"lid_known":true}` when valid saved own-LID authority exists,
or false when sources are absent/null. It prints no JID, phone, key or token,
and refuses corrupt/conflicting source state without echo or modification.
No action, query, receipt or message is queued. The OpenAPI path is x-live=false;
since 2026-10-13 it is also served read-only by `serve --store-routes`
([store routes](../api/README.md#store-routes-opt-in)), where a refusal of the
saved sources answers 422 with the same fixed reason and no LID.

Offline fixtures cover authenticated login before any group preparation,
read/write lookup agreement, retained checkpoint fallback, normalized companion
LID, missing/null history, conflict/malformed redaction, unchanged checkpoints,
restart, own primary hook exclusion across wire encodings, and unchanged own
sender-key exposure while foreign primary invalidation/version updates still
work. No own peer alias is inferred from a directory record.

**Unknown/unported:** full Me/prefs import/repair, all own-device state/gates,
complete per-user admission, namespace migration, native identity receipt cleanup
and final fetch/resend recovery. This is a reusable source/guard correction,
not proof of complete APK identity lifecycle or an observed live own-key change.

2026-10-09 validation: all 807 workspace/all-feature tests, strict workspace
Clippy, warning-denying Rustdoc, default CLI tests, CLI reference/API symbols,
formatting and experimental CLI build passed. The eight-topic / 13-file /
49,695-byte APK bundle reproduces exactly; cached self-diff verifies eight
unchanged topics and four metadata artifacts. Hygiene checked 1,426 local links
and 35 bundles / 8,858,028 bytes.

**Observed compatibility regression:** one authorized 45-second receive-only
session authenticated and stopped cleanly, with chat sends and automatic
reconnects disabled. Offline before/after inspection found zero eligible chat
intents, unchanged peer inspection, 20 unchanged actions, nine unchanged receipt
records, unchanged outbox and event summary, and known own LID. Store inspection
changed only by eight additional inbound checkpoints (790 to 798; total
checkpoints 1,031 to 1,039). Those captures were not classified here; they are not
counted as user replies or identity changes. No qualifying live own-primary
change was demonstrated, and no chat intent or held uncertain action was sent.

Follow-up: [partial login persistence](login-identity-updates.md) preserves
absent PN/LID checkpoint members and refuses malformed updates atomically.
This corrects local state loss without claiming native partial-login parity.
