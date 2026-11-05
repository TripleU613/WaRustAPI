# Lower-level IQ map ownership and push-config producer — APK 2.26.43.72

Confirmed static checked DEX TXT, not runtime observations or REA envelopes.
Pinned APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Commands: `xtask apk-smali APK --class CLASS [--method METHOD] --signatures
--exceptions`; `xtask apk-field-refs APK --class CLASS --field FIELD --type
Ljava/util/Map;`. Complete selected methods are retained; the setup excerpt
is labelled and linked to its full retained source. The7Ow constructor is
filtered to exact(0Tp,1rt) from five overloads. Every file is checksummed.

## Reader and writer share a new map in the selected setup

The [full startup method](../../rea/2.26.43.72-authkey-outcomes/startup-trigger-checked.txt)
creates ConcurrentHashMap v13 at0916. It passes v13 to0ZJ.A0S at0962,
through1r7.A03 to1rC.AL1, and into1rD.A00. It also passes the same v13 as
1rj's last constructor argument at0a1e; that constructor stores it inA03.
The selected setup excerpt has no intervening v13 reassignment. These are
aliases of one heap map, not independent callback copies. The new map does
not restore old entries from durable storage in this selected setup block.

The [reader branch](../2.26.43.72-iq-reader-admission/processor.txt) removes
from1rD.A00 before invoking its1ru result/error callback, then forwards a
nonnull-ID response toward the ordinary0ln registry. The exact field inventory
has two reads in that reader method plus the constructor write. The writer's
field inventory has ten reads in0v0.A00, one in1rj.A02 and its constructor
write. This is a bounded field-access inventory, not an alias/escape census
or proof no other consumer can mutate an already-shared Map reference.

## Push-config GET is a concrete producer

1rj.A02 increments its integer counter, converts it withInteger.toHexString,
constructs1rt(id), puts1rv(writer, request) under that ID in the shared map,
then calls1r2.Cyu(requestNode,1). Map registration precedes node writing.
The counter/hex format is not proof of globally collision-free IDs across
connection generations or disjointness from0Sf-generated IDs.

1rt builds iq/get with xmlns=urn:xmpp:whatsapp:push, to0mL.A00 (the server
JID as traced in the [token writer evidence](../2.26.43.72-client-auth-token-writer/README.md)),
id=input, and child config/version=1. Its allowed version list contains0/1.
The selected String helper's previously traced diagnostic length behavior
must not be reinterpreted as a new hard server/numeric bound. This request
is push configuration, distinct from the client-auth-token GET/pn writer;
sharing the namespace does not make their inputs or callbacks interchangeable.

1rv extends1ru and stores the writer and original1rt. 1ru.A02's1rv branch
constructs7Ow(response, originalRequest), iterates its parsed list through
variant visitor84d into eight String holders, constructs2Dz and invokes
1lZ.A1G. A1G packages the configuration throughA0S and submitsCPm to the
reader observer. The exact selected7Ow constructor is retained for further
header/value tracing; the eight field names, value grammar, consumer effects,
complete trigger/failure/timeout policy and persistence remain untraced here.
No configuration is invented or enabled in Rust.

## Generic token node path does not explicitly add this entry

The previously traced token request uses the generic envelope233 path,
whose writer branch calls1rj.A03(node). The [retained A03](../2.26.43.72-iq-promotion-shutdown/node-write.txt)
only delegates to1r2.Cyu(node,1); neither this body nor the selected generic
writer branch explicitly inserts a lower-level callback record. Thus the
new push-config example is not evidence token submission also registers1rv
or receives its configuration side effects.

This is deliberately a scoped absence: complete Cyu/interceptor behavior,
escaped map aliases, concurrent contents, ID reuse/collision and every other
writer family are not proven. It does not establish that a matching low-level
entry can never exist for a token ID. Native map presence remains separate
from successful wire delivery or safe replay authority.

## Implementation boundary

Shared-map identity and one configuration producer/result bridge are now
connected. Other lower-level families, complete configuration parsing and
application, callback races, generation binding and durable reconciliation
remain open. No Rust sender, retry, route or live account operation is added.
Existing uncertainty and no-replay behavior remain unchanged.
