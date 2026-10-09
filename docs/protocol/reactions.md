# Reaction keys and historical target stability

## APK evidence, 2026-10-09

**Confirmed static:** comparison APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The [hash-pinned recipe](apk/recipes/2.26.37.71-reaction-key.json) retains
21 topics / 26 methods with register operands and exception tables in the
[bounded bundle](apk/2.26.37.71-reaction-key/targets.json). The exact
[BWI.key_ field inventory](reaction-key-field-evidence.txt) found 16 references
across BNx, Bal, DGD and DHR; this is DEX inventory, not Java virtual-dispatch
closure or live coverage.

- [BWI.dynamicMethod](apk/2.26.37.71-reaction-key/reaction-schema.txt) retains
  key=1, text=2, groupingKey=3, senderTimestampMs=4;
  [BZa.dynamicMethod](apk/2.26.37.71-reaction-key/key-schema.txt) retains
  remoteJid=1, fromMe=2, id=3, participant=4.
- [C7G.A00](apk/2.26.37.71-reaction-key/key-validator.txt), 0004..004e,
  requires the ID presence bit 4, nonempty ID, remoteJid presence bit 1,
  nonempty remoteJid and a nonnull ChatJid parse. It does **not** require a
  fromMe presence bit. [1xf.A0k](apk/2.26.37.71-reaction-key/chat-jid-parse-adapter.txt)
  delegates to [0D7.A02](apk/2.26.37.71-reaction-key/chat-jid-nullable-parser.txt),
  whose nullable parse converts exceptions into null. Its
  [A01](apk/2.26.37.71-reaction-key/chat-jid-required-parser.txt) requires
  the parsed JID to be the 0Cy subtype. [0D1.A00](apk/2.26.37.71-reaction-key/jid-parser.txt)
  dispatches recognized namespaces and throws on unknown ones; its native
  factory/cache/subtype and special server-only behavior are not fully ported.
- [Bal.A01](apk/2.26.37.71-reaction-key/reaction-incoming-builder.txt)
  validates an ordinary reaction through C7G at 01b2..01ca, constructs its
  model with A00 at 01fa..0200 and, for the non-own envelope with a chat,
  resolves its target through BoQ.A00 at 0206..0256. Other encrypted-reaction
  and edit branches remain outside this implementation.
  [Bal.A00](apk/2.26.37.71-reaction-key/reaction-model-key.txt), 0012..0050,
  first constructs a key from remoteJid/fromMe/id and participant.
  [Ayo.A0S](apk/2.26.37.71-reaction-key/message-key-builder.txt) reads the ID
  and builds 1gc with the supplied chat and boolean.
- [BoQ.A00](apk/2.26.37.71-reaction-key/ordinary-target-mapper.txt) treats
  a parsed 8oR specially at 0016..006a. Otherwise 00fe..0106 selects the
  envelope's 1gc.A00 chat; the 00f6..00fe branch keeps the supplied fromMe
  flag. Thus **inference from the traced ordinary branch:** substituting an
  arbitrary payload remoteJid as our ordinary update chat is not justified.
  Complete participant/own/hosted branches and Android parent storage are
  still **unknown/unported**.
- [8oR](apk/2.26.37.71-reaction-key/status-special-remote.txt) constructs
  user `status`, returns server `broadcast`, and has type 5. This is the
  status singleton, not every broadcast address. 0D1.A00 separately constructs
  location and ordinary broadcast variants at 00a4..00f4.

**Negative findings:** DGD.CF7 is explicitly FStatusReactionProtobuf, not the
ordinary reaction receiver. Its [builder](apk/2.26.37.71-reaction-key/status-reaction-builder.txt)
uses CYF's [status target helper](apk/2.26.37.71-reaction-key/status-target-mapper.txt).
The status serializer and DHR key writer are retained as separate field-inventory
paths; their full caller/runtime conditions were not established. Bal.AD7 is
an outgoing serializer and reaches CSZ.A01; complete outgoing normalization,
feature gates and encrypted reactions were not established by this trace.
No identity-only adoption, identity-change cleanup or full APK chat merge is
proved by any of these methods.

## Implemented local receive correction

The former reaction decoder required an ID but ignored remoteJid entirely.
It now requires both fields and a supported JID parse. Existing local MessageId
(128-byte), JID and reaction text bounds remain local. Unknown namespaces,
empty-user/server-only values and native special classes remain unsupported;
this is **not complete 0Cy/native parser parity**. Malformed reactions remain
opaque rather than becoming MessageUpdated events. The existing
`lab-inbox --inspect` reaction shape adds fixed `key_validation` labels:
valid, missing_key, missing_id, invalid_id, missing_remote_jid,
invalid_remote_jid, unsupported_remote_jid, malformed_reaction. No key,
remote address or target ID is echoed by this diagnostic.

For a primary direct reaction on the reacting sender's own incoming message,
projection resolves the target in the authenticated envelope's chat context.
The [existing bounded direct-record proof](peer-identity.md#primary-lid-receive-and-legacy-record-stability-2026-10-09)
can retain the original PN target chat/author while the reaction's `by` remains
its actual LID. A consumed target retains its key without restoring plaintext.
Absent or foreign legacy targets stay in the actual envelope namespace.
Ambiguous qualifying rows or missing original legacy route evidence refuse
inside the same write transaction; neither marker nor event can commit alone.

This is a **local historical-record compatibility rule**, not evidence of an
Android database migration. It neither renames rows, guesses PN/LID identity,
copies sessions/pins, deletes projection markers, nor creates retry authority.
Existing MessageUpdated markers are not rewritten: previously projected
misattributed updates remain historical evidence. Own/outgoing targets, group,
companion and status target reconciliation remain separate unfinished work.
The status-like recognizer now requires zero-component `status@broadcast`;
location/other broadcast targets and companion-suffixed status targets are not
recorded as status likes.

## Evidence and reproduction

An independent literal protobuf-key fixture covers absent/empty ID and remote,
invalid syntax, unsupported/server-only categories, both primary namespaces,
group/status keys, default fromMe and empty-text removal, with redacted labels.
A trusted historical-store projection fixture covers legacy/consumed/canonical/
absent/foreign/ambiguous/missing-route targets, actual reactor attribution,
restart/deduplication, unchanged original records and atomic refusal. This is
local projection evidence, **not a new cryptographic receive or live reaction**.
The existing receive authentication/ratchet tests remain required.

```sh
cargo test --locked -p whatsapp-api --all-features reaction
cargo run --locked -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-reaction-key.json \
  --output docs/protocol/apk/2.26.37.71-reaction-key --check
cargo run --locked -p xtask -- apk-research-diff \
  docs/protocol/apk/2.26.37.71-reaction-key docs/protocol/apk/2.26.37.71-reaction-key
```

The exact APK re-export/check passed for 26 files / 127,657 bytes. A checksum-
verified cached same-bundle diff took **0.028679 seconds**, with 21 unchanged
source topics and four unchanged metadata artifacts; [measurement](apk/reaction-key-cache-check.json).
That is selected-topic cached comparison, not a different-release or whole-APK
equivalence claim. Fresh malformed-key, reaction-on-legacy-target and status-like
live validation remain **unverified**; no new reaction is sent merely to test this.

Inspection now emits the shape diagnostic even when reaction decoding refuses
the key; the former placement inside the successful-decode branch hid exactly
those failures. A subprocess CLI regression reads an isolated malformed stored
reaction and asserts the missing_remote_jid label without the target ID. It
also inspects an extreme captured timestamp without a panic. Capture ages use
one local timestamp per inspection, with checked subtraction. Per-capture now()
previously made repeated shapes differ during a long scan; this was a diagnostic
artifact, not evidence that four captures had been removed.

Final shared-code validation passed: **727 workspace/all-feature tests**, strict
workspace/all-target/all-feature Clippy, warning-free Rustdoc, 24 default CLI
checks, CLI reference generation, OpenAPI library-symbol validation and formatting.
The CLI subprocess test exercises the actual inspection command's refusal path,
not merely the reaction decoder helper.

**Observed offline on retained state:** reprojection upgraded zero events without
network requests. Peer-key/action/receipt/outbox/event inspection stayed unchanged;
two saved reaction shapes reported `key_validation=valid`. Noncapture inspection
was unchanged apart from the new validation suffixes. Capture shape sets matched
after normalizing elapsed age; the former scan's per-capture time produced four
extra duplicate diagnostic lines, not proof of deleted records. This comparison
does not claim raw encrypted-database byte equality or fresh live behavior.
No saved identity/request ID was reset, no uncertain action was replayed and no
new chat or live request was sent in this chunk.
