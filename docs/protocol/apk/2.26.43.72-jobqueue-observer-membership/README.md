# Jobqueue connection observer compiled set membership

**Confirmed selected static DEX + derived initializer interpretation**,
WhatsApp2.26.43.72/code264307238, APK SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Baseline `75b31a2f`. Repository offline DEX inspector; no APK execution or
successful REA Android output claimed after its recorded Java-path failure.
No live account, runtime change or automatic retry added.

## Set keys resolve to concrete compiled arrays

[Previous facade/provider evidence](../2.26.43.72-jobqueue-observer-provider/README.md)
combines00D.A05(0x1ded) and00T.A05(0x1d37) before passing the RF1 snapshot to
NhR. This bundle resolves their compiled membership rather than assuming
RNL's constructor existence proves installation.

[00D.A02](app-lookup.txt) resolves via06o.A00 and00D.A03. The
[retained global resolver](../2.26.43.72-client-auth-token-observer/provider-resolve.txt)
reaches1Uw.A00 on cache miss. [Root dispatch](root-dispatch.txt) chooses a factory
from (id>>14)&0x7f; root0 routes to1Uw.A01. The
[retained A01 range chain](../2.26.43.72-client-auth-token-membership/provider-shard-router.txt)
routes0x1ded=7661 throughA0L because0x1d9f < id <=0x1eb8.
[Selected A0L case](app-set-shard-excerpt.txt) maps7661 to0292, reads07q.A8E,
then branches to088a to construct07r(scope,array). The other set's
[00T.A04](session-lookup.txt) reaches1Uw.A00 through its supplied scope;0x1d37
=7479 routesA01->A0K, whose [retained case7479](../2.26.43.72-client-auth-token-membership/set-binding.txt)
at066a reads07q.A8D and uses its shared07r construction tail at095a.

[Derived members](derived-members.json), from the complete already retained
[07q initializer](../2.26.43.72-prekey-upload-replies/factory-set-arrays.txt):

| Input set | Static array | Binding IDs |
| --- | --- | --- |
|00D key0x1ded|A8E|0x1c1af|
|00T key0x1d37|A8D|0x300f1,0x300f2,0x300f3,0x300f4,**0x300f5**,0x300f6,0x300f7,0x1096,0xc6b|

The [derivation script](derive-members.py) accepts only the initializer's observed
straight-line const/new-array/aput/sput-object/return instruction vocabulary,
interprets all10696 instructions and all511 array stores, then asserts the exact
selected arrays. It preserves full source hash/length. This is reproducible
static interpretation, not execution of Android code or observed runtime values.

## Connection observer binding closes the selected association chain

For0x300f5, root selector=(id>>14)&0x7f=12 reachesRNH.A00. Its low14-bit case
is245. [Previously retained case245](../2.26.43.72-jobqueue-observer-provider/observer-factory-case.txt)
constructs and returnsRNL. Thus the second compiled set contains the concrete
connection observer. The first set's distinct ID is not a duplicate that can
remove this member in the combined-set deduplication path.

The [retained combined-set/iterator/default-resolver trace](../2.26.43.72-client-auth-token-membership/README.md#first-readiness-set-includes-the-token-manager-binding)
shows member IDs retain their source resolver through0AJ, and08x.next resolves
and caches each object;07r's default1Vo resolver calls1Uw.A00.00T's selected
set path does not apply00D's global-resolver replacement. This binds the member
to the direct factory path above. The earlier facade snapshots that Iterable
into its RF1 array/List; NhR invokesCgm(this) on each entry, so the positive
RNL->manager association is established for this selected construction chain.
The retained RNL constructor registers on0AI, and BzL/BzN can wake the associated
manager as previously traced. This is stronger than a matching provider key alone.

## Limits

This closes the compiled RF1 membership gap for the connection observer, not
all ten bindings' human meanings, effective runtime membership after mutation,
global provider instance identity,07o dispatch completion, deferred persistence/
dedup/backoff, or account-generation/restart ownership. No universal wake or
request admission guarantee follows. A wake still only causes requirement/group
rechecks before writer authority and socket activity; it is not server acceptance.
Rust durable no-replay holds remain unchanged.

## Reproduction

```
python3 docs/protocol/apk/2.26.43.72-jobqueue-observer-membership/derive-members.py docs/protocol/apk/2.26.43.72-prekey-upload-replies/factory-set-arrays.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1Uw;' --method A0L --signatures --exceptions > /tmp/jobqueue-set-shard.txt
python3 docs/protocol/apk/2.26.43.72-jobqueue-observer-membership/extract-set-case.py /tmp/jobqueue-set-shard.txt
```

Use other exported descriptors for exact method selectors. The excerpt labels
omitted unrelated cases and retains source hash/length plus the complete switch
and selected direct-return path. PCs are inspector bytes; branch deltas/exception
comments use code units. checksums.json pins all local artifacts.
