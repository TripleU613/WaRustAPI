# Send-job state path and an inert post-insert listener

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Fourteen-topic recipe](apk/recipes/2.26.37.71-send-job-state.json),
[complete signatures, operands and catches](apk/2.26.37.71-send-job-state/targets.json),
[independent raw switch bytes and negative findings](send-job-state-evidence.txt).
Offsets below are method-relative bytes. No Rust runtime or account change.

## Post-insert listener does not establish transmission

The already retained [0zC.A04](apk/2.26.37.71-text-store-submission/store-worker-target.txt)
iterates A0r's values before insertion, discarding each value without a shown
callback invocation (0018..002c). Later, only when isWrittenToDb is false and
the map is nonempty, 0162..017a
constructs **DNs selector 49** with the listener, result and message, then queues
it using 1f0.A02 and local scheduling code 25. The true result at 00e8 skips
the failure diagnostic and latter map iteration directly to return at 017c.

**Confirmed scoped negative:** DNs.<init> stores the supplied selector without
remapping. DNs.run's packed switch represents exactly **0..48**; its default
at **000e** is return-void. Therefore running selector 49 invokes no listener
and constructs no send stanza or receipt row in this method. The independent
raw APK bytes confirm size=49/first_key=0 and the immediate return, rather than
relying solely on rendered switch text. Nothing here proves map population or
scheduler execution occurred live. Other selectors remain separate workflows.

1f0.A02 obtains an Android Message with the supplied Runnable callback, sets
**arg1=25** and sends it to A01 Handler. This is distinct from what=25 and is
not native message status 25. Full Handler lifecycle and callback execution
remain untraced. This closes one previously unknown immediate listener body;
it cannot be used as the ordinary store-to-network authority.

## Actual send-job execution and bounded refusal paths

SendE2EMessageJob.A0G is the concrete onRun path, anchored by its diagnostic.
It returns for the duplicate flag at 0050..0064. Its complete body, including
special TEE/expiry/subtype paths and exception handling, is retained, but these
are not all ported or semantically closed by this trace.

At 08ba..0974, a present message, retryCount==0 and **A03 false** admit the
candidate already-accepted check. A03 is exactly a nonnull, nonempty
**targetDeviceRawJids** test. It compares message.A08 against native 4 through
1hg.A04, whose [ordering includes equal ranks](receipt-admission.md).
Additional property 0x591b/Ayk.A1X and AzZ.A03 gates control whether this
candidate reaches the warning/report and terminal path. If the latter gate is
false, it continues at 0974. **Do not turn the diagnostic into an unconditional
native ACK-abort rule.** Effective properties and Ayk.A1X semantics remain
unknown here; Rust's existing terminal-fact replay refusal is not weakened.

At 0974..09a2, native A08==**22** plus Ayk.A1X true takes the ABORTED diagnostic
and terminal path. At 0c90..0caa, retryCount **greater than 4** takes the high
retry-count terminal path. This is native job bookkeeping, not permission to
add four network retries, rotate identity, or reset Rust reservations. Complete
outer scheduling and retry requirements remain unknown/unported.

The native path reads the send count through 0zq.A01 for the selected ordinary
message branch at 09d0..0a18. A01 queries message_send_count.send_count by
positive message row_id; absence returns zero. Its A00 delegate only validates
row_id and main storage through assertions. No increment or native status setter
appears in either selected method. This count is not the job retryCount and not
a server-directed wait. Full assertion behavior and count writers are unknown.

## Native states 20 and 21 reach a separate asynchronous worker

**Confirmed:** CSH.A00 first builds Cnn send-event metadata and calls Cnn.A00,
then at 0070..0082 passes the original message key, null callback and **20**
to 15A.A05. CSH.A01 only builds/forwards metadata in its retained body;
it does not make that status call. Cnn.A00 converts through A02 and invokes
0zO.A01. Its downstream metadata/metrics implementation remains untraced here;
forwarding metadata is not itself receipt_user row creation.

SendE2EMessageJob.A0G also calls 15A.A05 with **20** at 0ee4..0ef2 under
AzZ.A0I, and with **21** at 0f4c..0f5a on a different selected path. Their
complete enclosing branches are retained; these calls do not establish a
universal sequence, nor new authored-text states 1/2 at the insertion boundary.
A04 returns **false** for message types **58/69/77**, otherwise it returns
the A03 explicit-target-set predicate. These integers are not decoded into
public content names from this branch alone.

15A.A05 requires a nonnull key, otherwise throws. It queues **DM8 selector 6**
with the key, manager, nullable callback and supplied status through 1f0.A01
using scheduling code **36**. DM8.<init> preserves those fields. DM8.run's
selector 6 at 0726..0798 resolves an existing message through the cache helper,
then COS.A00(key,false) on a miss. A second miss logs nosuchmessage and returns.
It neither recreates a missing message nor assigns a new request ID. A found
message is passed to **15A.A07** with the original status and callback. Full
cache/lookup authority is still unknown.

The [already traced A07](receipt-state-only.md#a07-admission-and-played-eligibility)
rejects nonadvancing status before mutation, otherwise applies its separate
eligibility and persistence paths. Its true return is not proof every database
write succeeded. This worker is separate from 15A.A02's positive-time per-user
receipt writer; no user-row existence may be inferred merely from scheduling it.

## Rust and remaining authority

**Unknown/unported:** full fresh-job creation/queue authority, transitive send
metadata, actual live user-row admission/seeding, effective properties and
complete ordinary native status progression. Primary identity cleanup still
requires actual receipt_user membership. Immutable targets, raw DeliveryFacts,
ACK timestamps and native state updates are not substitutes.

The exact receipt_user search repeats the fourteen previously resolved literal
sites; its absence in these selected bodies is not global absence of indirect
SQL. CQl was already documented as prior device-receipt target selection, and
is not presented as a fresh resolver or new discovery. No account was opened,
no message/receipt was sent, and no retry or identity policy changed.

Existing apk-smali, apk-research-bundle, apk-research-diff and apk-xref expose
this evidence; no new CLI flag or HTTP route is introduced. For a new release,
re-anchor into a separate generation: this exact pinned snapshot and cached
self-diff do not establish cross-release or live equivalence.

```sh
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-send-job-state.json \
  --output docs/protocol/apk/2.26.37.71-send-job-state --check
cargo run -p xtask -- apk-research-diff \
  docs/protocol/apk/2.26.37.71-send-job-state \
  docs/protocol/apk/2.26.37.71-send-job-state
```

## Validation

The fourteen topics select one method each. The exact source export reproduced
**19 files / 268,818 bytes**; cached self-diff reports fourteen unchanged topics
and four unchanged metadata artifacts. Independent raw bytes verify the listener
switch size/default, and retained-source assertions verify the original
failed-write branch, DM8 selector 6 and native 20/21 call arguments.

CLI-reference regeneration, API/library-symbol checks, formatting and whitespace
passed. Repository hygiene checked **2,138 links / 106 APK checksum bundles**,
22,800,258 source-bundle bytes. Research/docs only: no fresh runtime suite or
live network test is claimed. The preceding 832-test runtime validation remains
a separate completed change.
