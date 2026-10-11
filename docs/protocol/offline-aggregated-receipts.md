# Offline device receipt aggregates

2026-10-11 follow-up: [receipt ACK namespace/type checks](receipt-ack-namespace.md)
now share the existing strict kind getter and aggregate-target namespace mask.
Invalid server-only lid/broadcast targets skip before optional time parsing;
unsupported root/type receipts remain captured without a control ACK. Native
special JID grammar and complete ACK dispatch remain separate boundaries.

2026-10-10. **Confirmed static**, comparison APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
**Implemented/offline tested:** direct and group `participants message_id`
and `participants key` receipts enter the normal durable fact path. The key
form follow-up below supersedes the earlier unported branch. Fresh live aggregate projection
remains **unknown**, and this is not complete native receipt/status parity.

## Evidence

The complete REA/JADX Evidence envelopes are committed here:

- [Package declarations](rea/2.26.37.71-offline-receipts/package.json),
  `ev_9cce07151adf547c223d1f5647c19918bb11f8016445c9ff246f84b115db5eab`.
  Provider reports 94,738 classes and 14,967 resources, not understood or ported
  classes/resources. Manifest decoding does not verify APK signatures.
- [Cmk.A03 parser](rea/2.26.37.71-offline-receipts/parser.json),
  `ev_3d004b8aa9037da8f88c428311d145603eea064e9274e41e76eb4b10d53d9860`.
  Java text is complete but has two JADX duplicated-block warnings. Source
  positions and exact DEX descriptors are not supplied by this provider.
- [Ayk.A09 pair constructor](rea/2.26.37.71-offline-receipts/pair.json),
  `ev_cc34bc6b75f4d893e96d3d1769080c1303541e13706b663805f2bc6513348a63`.
  It returns `new Pair(obj,obj2)`. The receipt list is not deduplicated by this
  helper; the parallel set does not replace the list.
- [DDF exact DEX methods](rea/2.26.37.71-offline-receipts/aggregate-model.txt).
  The existing [Cmk bytecode](apk/2.26.37.71-direct-receipts/receipt-status-parser.txt)
  independently anchors offsets and the DDF construction.

**Confirmed:** Cmk.A03 `002a..015c` examines only the first child. A
`participants` child with a present readable `message_id` selects the offline
aggregate before ordinary root status validation. It builds one sent-message key
with that inner ID and `fromMe=true`, and retains the remote chat separately.
Each entry supplies a DeviceJid through `0hs.A0B`, type through
`A0M("type","delivery")`, and required time through `A07("t")`, followed by
signed wrapping multiplication by 1,000. A07 calls required A0L and strict A09;
the [clock trace](receipt-clock.md) preserves this different parsing policy.
There is no root timestamp or current-clock fallback in this branch.

DDF.BSJ always returns its one message key. CFC(index), Ccf(index) and
CeD(index) return that indexed CXQ's actual device, status and timestamp.
Its `AT2` value is `AGGREGATE_BY_ID`; CPB retains the remaining sublist when
creating `ReceiptAggregatedByIdProcessingJob`. Existing Cmn interface reads
are documented in [receipt application](receipt-apply.md). That confirms
per-entry attribution, not complete native scheduler, user-row or privacy policy.

**Correction to an earlier negative:** the previous direct-receipt note could
not establish the native ordinary `list/item` array assignment from its bounded
bytecode reading. REA shows `strArr[i] = strA0L3` at each loop iteration,
then reads the next item. The exact bytecode assignment at `060a` is reached
again by the loop back-edge. This resolves that particular static array question;
it does not prove full native list admission or live batch behavior.

## Runtime change

Before this change, the shared handler only used root `id`. An aggregate could
be ignored, fail as missing root ID, or be misattributed to a root transport ID.
The first-child `message_id` branch now validates all entries and applies them
within one encrypted-store transaction. Direct and group entry points share
the transaction-aware existing handlers; no separate store or new request
format was introduced.

Supported delivery/read/played entries become ordinary device observations using
the inner message ID and their own supplied time. Missing type alone defaults
to delivery. Known sender/inactive/self kinds remain unprojected, and unknown
or malformed types refuse with fixed redacted diagnoses. Malformed required
JID/time, missing list or invalid inner ID refuses without partial facts.
Nonpositive/wrapped time can remain in the first public fact; only positive
values update the earliest-positive device row. A duplicate kind/device adds
no second fact or event but can lower that mutable row's positive time.

**Existing authority remains required:** direct participant observations need
an immutable original target snapshot and matching saved chat/user/device;
group observations use the frozen full plan or existing known legacy wire
recipients. Authenticated PN/LID association can join the user namespaces at
the same device ID, retaining the actual fact actor. The mutable timestamp row
still uses its exact saved namespace. No current membership, numeric similarity,
new pin or guessed target is accepted. Direct legacy Boolean summaries remain
unchanged; existing group legacy device summaries retain their own semantics.

Any late mapping/corruption failure rolls back earlier facts, row times and
journal events in the aggregate. Original intent, attempts, uncertain outcomes,
keys and progress are preserved. The local list bound is the existing default
Node limit (4,096 children), and the inner ID uses the existing 128-byte bound;
neither is claimed as a WhatsApp limit. These facts do not authorize resending.

**Historical boundary:** the initial message_id implementation left DDG key
aggregates unprojected. The follow-up below implements their supported fact path.
Full native status/settings admission, primary identity cleanup, orphan storage,
privacy metadata, callback deferral and own-device synchronization remain unported.

## Reproduce and inspect

REA 6.3.0 needs a separately installed full JDK and `jadx-headless-mcp` 0.7.1
JAR. The installed engine matches REA's pinned SHA-256
`6e5eacf500b64292bfb73c49797c1958f6ee44646e43e868039ae7feb573ff75`.
The first MCP package inspection refused because that prerequisite was missing;
that result proved a local setup gap, not an APK failure. The JAR is now installed
outside the repository and configured under the Codex REA server environment,
with a private configuration backup. At the time of this extraction the MCP connection predated that
environment update, so these APK calls used the existing REA CLI. The subsequent
[single-list follow-up](direct-receipts.md#single-list-branch-correction--2026-10-10)
records successful method calls through the connected MCP provider.
No APK, emulator or app process was executed.

```sh
export REA_JADX_MCP_JAR=~/.local/share/rea/providers/jadx/jadx-headless-mcp-0.7.1-all.jar
rea inspect-android-package ~/Downloads/WhatsApp.apk --json
rea inspect-android-method ~/Downloads/WhatsApp.apk X.Cmk A03 --json
rea inspect-android-method ~/Downloads/WhatsApp.apk X.Ayk A09 --json
cargo test -p whatsapp-api --all-features aggregat
whatsapp-cli --profile l --json status --local-id ORIGINAL_LOCAL_ID
whatsapp-cli --profile l --json outbox-versions --local-id ORIGINAL_LOCAL_ID
```

Use the existing versioned APK recipes and `apk-research-diff` for cached
cross-release comparisons. REA decompiled text supplies additional evidence,
not stable obfuscated selectors across releases or automatic Rust patches.

## Validation

893 workspace/all-feature tests passed. Five new encrypted-store tests cover
binary decoding, per-device kinds/times, immutable target guards, duplicate
suppression after reopening, malformed batches, late mapping failure rollback,
key-form isolation and the public group handler. Strict all-target Clippy,
warnings-denied Rustdoc, default CLI tests and CLI-reference regeneration
passed. The default CLI build retains an existing unused `clock::observe_login`
warning. No live account operation was performed for this chunk.

## Bounded live receive regression — 2026-10-10

[Content-free live report](offline-aggregated-receipts-live.json): one 20-second
window authenticated and stopped cleanly, using the saved account/profile with
read marking disabled, automatic reconnect disabled and no chat outbox sends.
Preflight found no queued action, receipt or chat send. Existing action summaries
(18 OK, one error, two sent/unknown), nine acknowledged receipt summaries,
registration summary, generation and journal head remained unchanged. Nine
additional checkpoints were retained; inbox, outbox, identity and session table
counts stayed unchanged. Counts and summaries do not prove every stored byte
unchanged. Receive-side protocol controls remain allowed in this window.

No fresh aggregate receipt or new message was validated; aggregate projection
remains unverified. Private config/logs remain outside Git under the ignored
account directory. An initial offline CLI invocation used an unsupported explicit
Boolean value for `--json`; it failed before any connection. The corrected run
used the actual switch form. No retry of the live window was performed. The
window ran on commit 19ca798, before the separate single-list parser fix.

## Key-based multi-target receipts — 2026-10-10

**Implemented/offline:** the same public handler now selects the first
participants child's required `key` when message_id is absent. Both forms are
parsed completely before the existing direct/group handlers commit all facts in
one transaction. No new route, outbound message, target snapshot, retry or resend
authority is introduced. The inner key selects the saved send; an unrelated root
ID remains untouched. If both inner attributes exist, message_id takes precedence.

[Exact Cmk.A03](rea/2.26.37.71-receipt-list-shape/parser-exact.txt)
`0308..0328` requires key and a nonnull participant array. Root status is parsed
before this branch; [DDG exact methods](rea/2.26.37.71-receipt-list-shape/key-aggregate-model.txt)
return that one status/key for every entry and separate target/time pairs.
DDG.CPB retains the remaining sublist in ReceiptMultiTargetProcessingJob. This
proves shared root status, not use of entry type or root timestamp. Existing
receipt scheduling/privacy/native user-row gaps remain separate.

The parser reads one clock default in seconds at `0340..0354`. Each nonnull
optional JID reaches device conversion and A08 time parsing at `0364..0390`.
[Complete optional-time Evidence](rea/2.26.37.71-receipt-list-shape/optional-time.json)
`ev_71525460df7acbe75a8ad272d57c7c781f9d4c52a7b5ad20f008cdeffd035b5b`
and [exact A08](rea/2.26.37.71-receipt-list-shape/optional-time-exact.txt)
show default only for absence; present malformed/overflow time throws through
the existing strict Java-decimal parser. Multiplication to milliseconds wraps.
Unknown or missing optional JIDs skip before time parsing. Entry tag/type is not
an independent receipt kind here. Empty arrays produce no facts; missing arrays,
invalid keys and unknown root status refuse with redacted diagnostics.

[Optional-JID Evidence](rea/2.26.37.71-receipt-list-shape/optional-jid.json)
`ev_8d5e2718747a6ec2c37e01480e80ce82896b1e1e392ecc96467a7b5eb14e73d1`
has a JADX duplicated-block warning. Its factory is independently retained as
[nullable A02](rea/2.26.37.71-receipt-list-shape/nullable-jid-factory.json),
[complete A00 smali fallback](rea/2.26.37.71-receipt-list-shape/jid-parse.json)
`ev_e84317abc282a6692ff8f69c9ad06eca12738f06f6791dfec236ca79b53d57b5`
and [server-only parser](rea/2.26.37.71-receipt-list-shape/server-jid-parse.json).
Unknown server strings fail the native factory and return null; our permissive
local Jid model is not sufficient admission for this network branch. Native
[device conversion](rea/2.26.37.71-receipt-list-shape/device-cast.json)
`ev_3811765fef30f798d4e2dc7f3f43a77cb8c919bd9e2fc839c9f826b0df7de245`
returns an existing device or a user primary; unsupported namespaces cannot be
projected into saved PN/LID facts by the existing record handlers. Full special
JID/runtime subtype equivalence remains unported.

[Clock dispatch](rea/2.26.37.71-receipt-list-shape/clock-delegate.json)
`ev_cdc634951bc5013cd41b7e5924bce12822fbb57304ddb848b69820797f473165`
calls [08B.A00](rea/2.26.37.71-receipt-list-shape/clock-selector.json)
`ev_c8581bbd22a4b74d482959f21bcb7e28794bccc011d5b6a0112a182d68f16e95`:
nonzero NTP baseline, then confirmed server elapsed baseline, then wall minus
saved offset. Rust uses its existing confirmed login/ping elapsed/offset, or local
wall time with no confirmed sample. The same sampled second is used for every
missing entry time. Historical inferred message/receipt clock samples are
excluded. Native NTP acquisition/preference is still unported, so absent-time
parity against an APK with an active NTP baseline is not established. The fact's
server_timestamp may therefore be this parser default, rather than supplied wire
evidence. Ordinary receipts retain their missing-time behavior; message_id
aggregates still require every entry time.

The failing-before encrypted-store regression produced zero instead of two
facts. After binary wire encoding/decoding the fix proves inner-key/root-ID separation, root read status
over conflicting entry played status, Unicode supplied time, confirmed-clock
fallback, absent/invalid JID skipping, original target exclusion, immutable intent/
attempt/progress and duplicate-free reopening. Malformed batch tests compare all
tables and the journal head. The public group test now runs both forms, proving
frozen target attribution and root delivery status for key aggregates. A clock
test excludes historical inferred samples, verifies whole-second login offset,
and reopens without writes. Shared aggregate rollback/authority regressions
remain passing; this does not establish complete native admission.

Validation: 902 workspace/all-feature tests passed (761 library, four codec,
nine crash, 54 CLI, one API-symbol, 73 tooling). Strict all-target Clippy,
warnings-denied Rustdoc, default CLI tests, CLI-reference check and experimental
CLI build passed after test-only style fixes. The default build retains the
existing unused clock::observe_login warning. No new live session was opened;
fresh key aggregate projection remains unverified.
