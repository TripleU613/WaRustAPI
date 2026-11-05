# Unified-session eligibility source — APK 2.26.43.72

**Confirmed static**, checked DEX TXT and digest-bound DEX class metadata;
not runtime observations or REA Evidence envelopes. Pinned APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reproduce selected methods with `xtask apk-smali APK --class CLASS --method
METHOD --signatures --exceptions`. Other retained inventories use apk-callers
for09F.BUx and apk-field-refs for09O.A0H:LX/0E2; with --writes-only.
Run `python3 inspect-direct-implementers.py APK` for the metadata inventory;
it refuses a different APK digest. Complete selected methods are retained.

## Global provider connects0Sb to09G

0Sb's constructor obtains its09F from00D.A02(0xed), not an arbitrary runtime
interface name.00D.A02 obtains the global06o and delegates toA03's shared
ID-keyed cache/resolution path. On a miss that path calls1Uw.A00 with ID237;
its shard selector(id>>14)&0x7f yields0 and delegates to1Uw.A01. The previously
[retained complete shard](../2.26.43.72-client-auth-token-membership/provider-shard-router.txt)
was re-exported in this investigation and matched byte for byte: case237
constructs09G. Cache hits return the earlier bound object; full provider cache
invalidation and substitutions are not a runtime observation in this trace.

09G's constructor obtains the current00X.A01 context and resolves provider0xee
through09H.A03, casts to09O and stores it. The context-specific resolver's full
scope/invalidation is not reconstructed here.09G.BUx directly delegates to that
09O.BUx. An all-DEX class-definition inventory finds two direct09F implementers,
09G and09O. This does not claim a complete census of inherited implementations,
subclasses, overrides, dynamic loading or every runtime binding. The separate
interface caller inventory has151 direct references; call-site count is not a
runtime trigger count or a list of independent protocol requirements.

## Predicate is loaded Me-record presence, not channel readiness

09O.BUx callsA07, then returns exactly whetherA0H:LX/0E2; is nonnull. A07
checks09a.A01, conditionally supplies1Wk selector1 to09a.A04, then calls09a.A00.
This is the selected lazy-load entry; callable data provenance and waiting/error
semantics remain open here. The predicate has no socket/channel state test in
this method. It cannot by itself prove primary-login success, current transport
readiness, credential freshness or server acceptance.

The A0H write inventory finds two write sites,09O.A00 and privateA01. Selected
A00 installs a supplied nonnull0E2 record, uses its PhoneUserJid/0Dx fields to
refresh identity/device projections, and clears throughA01 when both identity
fields are null or selected09P conversion fails. A null supplied record skips
the installation branch; it is not an unconditional clear instruction here.
The method returns the previous record. Full record validation, caller authority
and projection helpers are not established by these selected writes.

PrivateA01 clearsA0H and derived device fields/references, writes
registration_device_id=0 with apply and resets a separate AtomicInteger. That
selected clearing operation is not proof every account switch/logout calls it,
nor proof a durable credential/session reset completed. No account state was
read or modified in this research.

## Consequence for unified-session construction

The earlier [ID lifecycle](../2.26.43.72-unified-session-startup/README.md)
usesBUx before generation when the cached ID is absent or during refresh. With
this resolved implementation, those checks concern loaded Me-record presence.
The nonnull cached-ID fast path in0Sb.A03 returns before recheckingBUx, while
the identity observer and explicit refresh separately update the cache. Therefore
this trace does not establish an atomic send-time identity/readiness fence for
every startup node. It also does not authorize generating arbitrary Me records
or manufacturing IDs to satisfy a predicate.

Full startup gates, global/context provider lifecycle, record load authority,
clock synchronization, family447 writer handling, server response/ACK policy and
durable generation ownership remain open. No Rust behavior, sender, route,
account state or live account/network operation changed.
