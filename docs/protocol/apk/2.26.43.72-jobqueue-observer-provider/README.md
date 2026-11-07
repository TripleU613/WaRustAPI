# Jobqueue observer set and lazy manager provider

**Confirmed selected static DEX**, WhatsApp2.26.43.72/code264307238,
APK SHA-256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Baseline `9a1a3bb6`. Execution-free repository DEX inspector, no successful REA
Android analysis claimed after the retained Java-path failure. No live account,
request or Rust runtime change.

## Reuse the existing wake path

[Prior observer/wake evidence](../2.26.43.72-jobqueue-wake/README.md) already
establishes RNL registration on0AI, conditional manager association through Cgm,
NhR.A05 wake submission and QnY case33 opening the selector condition. It also
retains executor rejection/teardown limits. Those are not rediscovered here.
This bundle follows the previously unresolved construction/list boundary.

## Observer set input

[0fW constructor](facade-constructor.txt) obtains two sets via00D.A05(0x1ded)
and00T.A05(0x1d37), combines them through0AJ.A00(Set[]), and stores the result
as0fW.A09:Set. Both supplied sets are checked nonnull. The two keys are set
provider inputs, not proof that any particular observer is a member.
[00D adapter](app-set-adapter.txt) obtains/casts its object through00D.A02; for
07r sets it conditionally installs the00D.A05 provider and clears A06 after a
monitored flag read. [00T adapter](session-set-adapter.txt) obtains06o.A00 and
passes null Context, that instance and the selected key to00T.A04, then casts
the result toSet. Full backing registry/lazy set population remains untraced.

## Concrete manager creation and association input

[Exact NhR constructor caller census](manager-callers.txt) has one physical site,
0fW.A00. [Complete provider method](manager-provider.txt) synchronizes on the
supplied facade. If A00:NhR is absent it takes A09.toArray(RF1[]), copies that
array and converts it to a List, then passes it to the new NhR constructor.
The returned NhR is stored back in facade.A00 and reused on later calls.
The already retained NhR constructor iterates this List and invokes each RF1's
Cgm(this); the list therefore supplies the exact manager association argument.
This proves per-facade cached construction, not a global singleton or shared
instance identity across account/session providers.

The provider also reads integer properties419/420 and boolean12464 for NhR
constructor arguments. Their effective values/full meanings are not established
here. Later optional60000ms scheduling belongs to a separate0fb runnable; it
must not be relabeled a retry receipt deadline, backoff or queue wake period.
The provider's constructor list snapshot does not imply future additions to a
backing set will retroactively receive Cgm or that iteration succeeds under all
concurrent lifecycle conditions.

## Concrete observer factory selector

[Selected RNH.A00 context](observer-factory-case.txt) masks the supplied selector
with0x3fff and switches on it. Case245 jumps to103a, constructs RNL and returns
it at1044..1046. This resolves the concrete construction case previously known
only from the RNL constructor caller census. It does **not** bind either set key
above to that selector: a low14-bit factory case is not a complete provider ID.
No complete RF1 membership/list or global observer delivery claim follows yet.

The deterministic [extractor](extract-factory-case.py) validates the method
signature and exact case target, records the complete source text hash/length,
and retains only entry/switch and the direct-return case. Other302-way factory
branches are deliberately outside this view; no conclusion depends on them.

## Remaining boundary and reproduction

Next: resolve the backing set registries for0x1ded/0x1d37 to complete observer
membership; verify provider lifecycle/07o dispatch and deferred persistence/dedup.
No Rust resend or reconnect policy is added. Queue wake, requirement admission,
writer authority, socket write and server acceptance remain separate evidence.

```
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0fW;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/RNH;' --method A00 --signatures --exceptions > /tmp/retry-jobqueue-provider.txt
python3 docs/protocol/apk/2.26.43.72-jobqueue-observer-provider/extract-factory-case.py /tmp/retry-jobqueue-provider.txt
```

Use exact descriptors in other exports for the corresponding selectors. PCs are
inspector bytes; branch deltas/exception comments use code units. Direct caller
census excludes reflection and aliases. checksums.json pins every local artifact.

Follow-up: [compiled set membership](../2.26.43.72-jobqueue-observer-membership/README.md)
now resolves the second set to RNL binding0x300f5 and closes the selected
manager-association chain. Runtime delivery/lifecycle limits remain separate.
