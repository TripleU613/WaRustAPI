# Feature configuration

## AB-property initial read (2026-10-08)

**Confirmed static, APK 2.26.37.71**, SHA-256 `1755111a…7d5c`:
`Hgb.A00(VZZI)` builds `iq type=get to=s.whatsapp.net xmlns=abt`, with
`props protocol=1` and a hash from `ab_props:sys:config_hash` when neither
force nor refresh selection is set. Missing state supplies the empty string.
`Aym.A1L` still adds that empty hash attribute. `GAZ.A1T` selects singleton
`1S7`, whose getServer is s.whatsapp.net and user is empty. The callback is
`IWy` discriminator 1, registered with 32000 ms. That timer is an APK request
timeout, not a server wait or permission to repeat uncertain requests.
[Scoped source and negative results](abprops-receive-evidence.txt) retain the
construction, callbacks, storage, concrete getters and defaults windows.

`0iS.A12` reads login success props/abprops integer hints into AtomicReferences;
these are not property values. `14f.A09` abprops notifications reach a forced
fetch only when sending-ready 09Y.A06 qualifies, with a separate MobileConfig
path gated by 00D.A0A. Neither gate is assumed here. No exact w:props anchor
was found. SQLite props tables in 0gZ/0KO store generic name/value or key/long
state; their existence does not identify the ABProps configuration store.

**Confirmed callback:** IWy.C8f case 1 reads props and optional erid; metadata
includes ab_key, hash, refresh (default 86400 seconds, converted to ms),
refresh_id (default 0) and delta_update (Boolean.parseBoolean, default false).
Nonempty config_code identifies configuration rows; config_value is required,
and config_expo_key is optional. Otherwise event_code/sampling_weight are
required integers. SparseArray.append collects configuration tuples with last
value winning for duplicate IDs, before typed value conversion.

A full update with a nonempty hash clears/replaces previous configuration;
a delta retains existing state and does not require a nonempty hash. A full
hashless/empty-hash reply skips value application (our not_modified diagnostic),
although metadata may still change in the APK. Full refresh intervals are
clamped to at least 600000 ms; these are local scheduling metadata. Backup,
exposure-key, sampling, app-version, refresh-time and invalidation operations
are separate from identity/session state. Complete application and scheduling
are not ported by this read path.

**Confirmed typed Boolean:** base 00D.A15 has no code body. Direct subclasses
07t/0Ck/0Dw return the same 07t.A00 map (two by delegation). 07t constructor
passes Boolean builder v14 through A0J, then assigns its built map to A00.
A0J registers 35661 at 1ed2..1ee6 with the false Boolean held in v1.
The constructor alone has no literal 35661; searching helpers was necessary.
07t.A07 also contains a registration, but it is not assumed to be the active
constructor path. 0Ci.A00 tests A15 membership, parses Integer.parseInt, then
stores value != 0. Text true is not a valid Boolean value in this path.
Malformed last values are skipped, not replaced by earlier duplicate values.
This establishes AB-property typing, not effective MobileConfig precedence.

**Implemented/offline:** `abprops-query` queues one initial read on the existing
durable action queue. Repeating returns the same ID even after refusals,
malformed responses or an uncertain reservation. `abprops-query --show` reads
correlated saved evidence, with fixed lifecycle/failure codes, configuration
and sampling row counts, delta/hash-presence flags and an explicit returned
35661 Boolean. Missing values stay null, never an inferred default. Invalid last
values set removal_value_invalid. Hashes, ab_key, exposure keys, other values,
weights/codes, raw errors and identifiers are not printed. Inspection applies
nothing to runtime feature policy. Local duplicate-props, 16384-row and 64-KiB
UTF-8-value guards are resource/ambiguity policies, not server requirements.
Queue/show have no HTTP route; listen later transmits the queued read once.

**Encoding correction:** our encoder previously omitted every empty text
attribute. The APK retains attributes through 0hs.A0Q and writes each in
1Qe.A01, including the initial hash. The Rust encoder now preserves explicit
empty strings through action save/decode/send. A literal raw BINARY_8(0)
fixture verifies preservation, not equality to the APK's packed-empty marker
selection. Existing byte/string packing equivalence and Android execution are
not established by that fixture. No IDs, identities or reservations are rotated.

**Unknown / unfinished:** effective 35661 remains unknown until a supported
response and complete resolver precedence are established. This command does
not disable removal guards or claim effective feature state, even if the server
returns false. Full/delta cache application, MobileConfig mapping/native reads,
refresh hints/scheduling, exposure and sampling application are unfinished.
The response remains encrypted in the existing action store. No historical
Android preferences or automatic server-config refresh are invented.

The [eight-topic recipe](apk/recipes/2.26.37.71-abprops.json) and
[baseline](apk/2.26.37.71-abprops/targets.json) preserve 13 methods / 13 files /
97174 bytes, including exception tables. Bulk 07t defaults stay local; compact
constructor/entry windows establish the specific Boolean. String-free helpers
still require reviewed relocation in another release.


## Saved-account validation (2026-10-08)

**Observed live:** preflight found no eligible outbox sends, queued actions,
pending receipts or selected-chat device refresh. One initial read was queued;
repeat queue returned its original ID. A bounded 32-second `listen` without
`--send-outbox` authenticated and became active, then ended before the window
completed. Saved inspection reports `outcome_unknown`: the action is `sent`,
with no correlated reply. All row counts, hash/delta flags and removal value
remain null. No effective Boolean or configuration adoption is established.

**Observed captured terminal shape:** `stream:error` has no outer attributes
or numeric code; its child is `ack`, with attribute names `class`, `type`, `id`.
This is not proof that the configuration query caused the closure, and does
not establish a ban or a server wait. `listen` currently collapses the terminal
reason into a generic connection-ended error; improving that diagnostic and
tracing the rejected acknowledgement remain follow-up work. Unsupported `ib`,
retry, PSA, encrypt and MEX traffic was also observed, not newly implemented.

**Observed locally after closure:** one new action remained `sent`; all prior
actions were preserved. Selected peer pins/device inspection, receipts and
outbox matched their preflight values. These limited comparisons do not claim
that every store table was unchanged: login and captures were persisted.
Repeating queue returned the same ID locally and made no second transmission.
The uncertain read is retained, with no reset, retry or new ID. Private captures
and pre/post output stay in ignored encrypted/private local storage.

**Verification:** 699 workspace/all-feature tests passed (582 API unit, four
codec integration, nine crash-matrix, 44 CLI unit, one API-symbol, 59 xtask).
Strict workspace/all-target Clippy, warning-free Rustdoc, 24 default CLI checks,
experimental API-symbol check, regenerated CLI reference, experimental CLI
build and formatting passed on system Rust 1.99.0. The initial Clippy failure
was item placement after a test module and was corrected. A filtered API-package
invocation ran zero matching API-symbol tests and is not counted; the actual
CLI integration check passed. Pinned Rust 1.98.1 verification is not claimed.

Repository hygiene passed: 912 local links, package licenses, tracked-file
budgets and 3,840,464 bytes across five checksum-verified APK bundles.

## Token Boolean evidence (2026-10-09)

**Confirmed static/local:** [token feature inputs](token-feature-inputs.md)
closes false default registrations for 15491 and 14303 and preserves the
cache/preferences/MobileConfig getter boundary. `abprops-query --show` reports
`token_counterpart_value`, `token_pending_namespace_value` and their respective
`_invalid` flags using the existing last-value/Java integer conversion rules.
Missing/non-applicable values remain null; no APK default is substituted.
These are selected returned properties, not effective gates, and do not activate
refresh or adopt configuration. Effective MobileConfig results remain unknown.
