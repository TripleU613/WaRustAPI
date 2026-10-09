# Ordinary text store insertion boundary

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Four-topic recipe](apk/recipes/2.26.37.71-text-store-insertion.json) and
[complete methods](apk/2.26.37.71-text-store-insertion/targets.json) continue
[text store submission](text-store-submission.md). Offsets are method-relative
bytes; branch displacements are signed DEX code units, multiplied by two.

## Authored-message dispatch and result

0zC.A0A starts with message-key fromMe and A0y. The fromMe && !A0y branch
at 0026 jumps to 0448 (0026 + 0x211 * 2), which invokes A0E(message, **1**).
This insertion argument is distinct from the caller's scheduling reason.
There is no direct 178.A0H setter in A0A: neither integer is proof of native
message status 1 or 2. Earlier A09 preprocessing and A0E/helper effects remain
separate prerequisites.

A0A reads 34T.A06 at 0252 and carries it into 2jQ.A03 on its selected result
construction at 03b8..03d2. The result also carries A00/error into A01 and a
separate boolean into A02. A09 performs further result bookkeeping, so this
is not a claim that every outer worker result equals this raw insertion result.
The constructor of 34T maps its first boolean to A06, second to A01, third to
A02, fourth to A03, fifth to A04 and sixth to A05; the C0l argument becomes A00.
Field suffixes do not establish their argument order or complete semantics.

The already retained [A0E body](apk/2.26.37.71-receipt-recipient-count/message-insert-count-context.txt)
invokes 0V9.A06 at 08a8. It subsequently checks message.A0j against -1, with a
separate transient-message path; a void return alone is not durable-row proof.
The selected successful path calls Aya.Bqh(message,35X) at 093c, reads
35X.A00/A01, then constructs 34T with C0l.A08 and first boolean true at
0954..097a. Remaining booleans retain their actual producer values rather than
being assumed false. Transaction success marking and subsequent staged Ahi
selector-8 callback do not establish network transmission or receipt admission.
Provider composition, callback semantics and all failure/duplicate paths remain
**unknown** here. [A09 source](apk/2.26.37.71-text-status-initialization/settings-preprocessing.txt)
retains the outer preprocessing and result updates.

## Transaction wrapper and registry hooks

0V9.A06 skips 1im transient messages with a diagnostic and returns at
0000..004a. For other messages it resolves the type registry 1gj through 1hh,
requires A02(type) and an A00(type) result implementing **1hj**, and otherwise
throws IllegalStateException with Unsupported message type. The supported path
opens the store and transaction, constructs AiX selector **10** using 0VB and
the original message, and calls 1hh.A01(message, lambda). It then marks success,
closes the transaction and store at 00de..00f0, and records elapsed metrics.
Retained exception tables show close/rethrow and suppressed close failures.
There is no direct message-status setter or receipt_user SQL in this wrapper.
Concrete inserter and lambda effects are still **unknown**.

1hh.A01 independently repeats the type support/inserter check. Its supported
path has this confirmed order:

1. Iterate the A03-provided before hooks, each 1hq.CHJ(message, **null**).
2. Invoke the supplied Function0 at 00b0..00b6.
3. Call 1hj.BJK(message) at 00b6..00bc.
4. Iterate A02-provided after hooks, again CHJ(message, **null**).

The supported loop's terminal branch at 00e2 lands at **010c**, skipping the
unsupported diagnostic. Unsupported branches land at **00fa** and emit
message cannot be inserted into the database / update /
fmessage-database-inserting-not-supported, then return. No exception table
catches hook/inserter failures in A01; those propagate to its caller.
**Inferred:** if registry support changed between the wrapper's check and this
second check, the latter could diagnose and return before insertion. No such
runtime registry change has been observed. BJK returns void; no affected-row
count is proven by this interface boundary.

The null CHJ argument means the nonnull-result failure branch retained in
[D7J preprocessing](receipt-count-preprocessing.md) would not be selected by
this invocation. **Unknown:** D7J membership in either provider iterable has
not been established; implementing 1hq alone does not prove registration.

## Limits and reproduction

**Unknown/unported:** ordinary type-zero concrete inserter, AiX selector-10
body, exact before/after provider composition, complete normal send-state
progression, settings/namespace admission, native user-row membership and
primary identity cleanup. Do not substitute immutable targets, device counts
or raw delivery facts for receipt_user existence. No Rust runtime, account,
new message ID, live send or receive operation changed.

Existing offline APK CLI tools reproduce the recipe and compare cached bundle
files. This adds static source coverage, no HTTP route or live capability.
Validation: four topics / nine files / 79,656 bytes reproduce exactly from
the pinned APK. Cached self-diff verifies four unchanged topics and four metadata
artifacts. CLI reference, API/library-symbol, formatting and repository hygiene
checks pass (2,003 local links; 92 bundles / 18,769,751 bytes). No fresh workspace
runtime suite or live validation is claimed.

Follow-up: [main-row callback](text-main-row.md) closes AiX selector 10 and
separates storage type from persisted message status; type-zero owner is unknown.
