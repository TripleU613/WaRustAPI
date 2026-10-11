# Effective Boolean configuration: readiness and snapshot reads

**Confirmed static / runtime adoption unported**, 2026-10-11. WhatsApp
2.26.37.71, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
This extends the [existing provider/mapping trace](token-feature-inputs.md)
with the readiness writer, fallback cache effects, concrete context and
snapshot Boolean read. It does not make a saved AB reply an effective value.

## Resolver return branches

00D.A10 uses false-mode 00F.A03 -> A0E ->
[A07](apk/2.26.37.71-mobileconfig-boolean/resolver.txt). Its two independent
predicates are:

```text
ready = A0z() && A06.get()
shadow = A0A && A0C
```

The DEX at 0042..0056 initializes shadow false when A0A is false; the Java
local otherwise appears uninitialized. A0z's existing closure is
A0A && A0b && !A0Q.get(). Mapping A0L and context A0I must both be non-null.
On normal return, the branch selection is:

| Mapping/context | Ready | Shadow | Returned source |
|---|---|---|---|
| Missing either | Any | Any | Fallback callback |
| Both present | False | False | Fallback callback |
| Both present | False | True | Fallback, with wrapped provider comparison |
| Both present | True | False | Direct provider 09U |
| Both present | True | True | Wrapped provider Hln.A02, with fallback comparison |

True-mode cache hits in A05 return before ready-provider reads; provider
putIfAbsent can return a competing cached value. False-mode A10 does not take
that early A05 lookup or write provider results into A0N. On the ready shadow
path, fallback is invoked with [00F.A00](apk/2.26.37.71-mobileconfig-boolean/comparison-mode.txt),
which changes its selector while preserving its mode Boolean. Comparison and
provider errors can still prevent a normal return. A07 has no catch table.

The earlier claim that provider values were generic result processing is
insufficient: the fallback/shadow branch returns the original fallback at
028a even after evaluating native data for comparison. The ready shadow branch
returns Hln.A02 at 01de. Exposure/comparison helper A0C is not the adoption gate.

## Readiness mutation and setup

[Physical A06 references](rea/2.26.37.71-mobileconfig-boolean/ready-field-refs.txt)
find three sites: constructor assignment, A07 read and 09N.BJ1 read. The
[retained constructor](apk/2.26.37.71-token-feature-providers/provider-bindings.txt)
allocates A06 with false. Its reference is not evidence of a ready manager.

[A0q](apk/2.26.37.71-mobileconfig-boolean/refresh.txt) clears the setup marker
A07, configures a concrete 099 using CTW with preference/default getters for
26525, 26524, 28443 and 30441, then calls BJ1 with Boolean 26640 and false.
[CTW](apk/2.26.37.71-mobileconfig-boolean/manager-settings.txt) updates the
099 fields, synchronizing the A05/A00 assignments; A0c/A0d are assigned outside
that synchronized block.

[09N.BJ1](apk/2.26.37.71-mobileconfig-boolean/manager-init.txt) is the
lightweight-manager initialization flow, not a blanket readiness setter. It
handles already-started state, unit state, logout/session identity, missing
directory, manager construction and errors. At its terminal outcome dispatch,
only codes {3,5,6,10,12,13,14} reach the readiness block. That block performs a
Boolean read of 32343 (0x7e57), gets the concrete 07t through
[09N.A00](apk/2.26.37.71-mobileconfig-boolean/provider-owner.txt), and, if
non-null, sets inherited A06 true. Other terminal codes and exceptions do not
reach that set. Complete outcome-to-manager construction semantics are still
partly interpreted; those numbers must not be treated as a generic success
enumeration or inferred from log label positions.

The separate property callback [09W.CDt](apk/2.26.37.71-mobileconfig-boolean/property-callback.txt)
is an empty return. [Bpp](apk/2.26.37.71-mobileconfig-boolean/postfetch.txt)
selects exposure keys using A0z, updates a daily exposure timestamp only when
elapsed time is strictly greater than 86,400,000 ms, and dispatches 1fY.A00.
It does not set A06 or clear the Boolean value cache. REA reported missing
decompiled bodies for both callbacks; their actual DEX bodies settle this.
This is a direct physical writer closure, not a reflection/native mutation
or complete callback registration proof.

## Fallback cache and preferences

The already traced fallback A0D reads a cache, then locks/rechecks, resolves
the Boolean default map and reads SharedPreferences with that default. Missing
both map and caller default throws Unknown BooleanField. Its false-mode call
to [A0u](apk/2.26.37.71-mobileconfig-boolean/fallback-cache-write.txt) only
puts the Boolean into A0N. [A0F](apk/2.26.37.71-mobileconfig-boolean/preference-presence.txt)
only checks preference contains(id); it is not a preference write or an
adoption operation.

The [complete physical virtual-declaration census](rea/2.26.37.71-mobileconfig-boolean/fallback-cache-overrides.txt)
finds A0u in 00D, 0Ck and 0Dw. The latter two use their own maps. The
[07t constructor](apk/2.26.37.71-token-feature-inputs/default-map-constructor.txt)
calls its 00D constructor; no 07t override appears in that census. This does
not establish cache invalidation order or authority after a later AB update.

## Concrete Boolean context and specifier fields

[099.AVn](apk/2.26.37.71-mobileconfig-boolean/boolean-default.txt) supplies
default = ((specifier >>> 61) & 1) == 1 to
[AVo](apk/2.26.37.71-mobileconfig-boolean/boolean-context.txt). The wrapper:

- Validates unit type bits 54..59 against the factory's A08 through
  [A06](apk/2.26.37.71-mobileconfig-boolean/unit-validation.txt).
- Requires parameter type bits 48..53 to equal 1 for Boolean.
- Selects config index bits 32..47, then uses latest context A08 or cached
  per-config context A07 according to the 0E8 mode.
- Returns the supplied default on a mismatch, with optional source metadata;
  errors and sampling paths are not silently converted into values.

[A07](apk/2.26.37.71-mobileconfig-boolean/config-context.txt) checks the index,
cached AtomicReferenceArray entry and logout state before constructing/cache
adoption. [A08](apk/2.26.37.71-mobileconfig-boolean/latest-context.txt) has its
separate synchronized latest snapshot cache.
[A01](apk/2.26.37.71-mobileconfig-boolean/snapshot-factory.txt) gets the latest
handle/buffer and builds Lwj, optionally wrapped in IIY for the Java manager.
The [Lwj constructor](apk/2.26.37.71-mobileconfig-boolean/snapshot-owner.txt)
establishes its 09H base. The optional IIY path remains untraced here.

[09H.AVo](apk/2.26.37.71-mobileconfig-boolean/snapshot-boolean.txt) applies
enabled Boolean overrides first. It then consults the
[silent-push selection](apk/2.26.37.71-mobileconfig-boolean/silent-push-selection.txt)
whose timestamp/type/null checks can select an overlay value or default.
Otherwise metadata source can suppress a snapshot read; eligible values reach
virtual A09. No native AVn invocation alone establishes a server result.

[Lwj.A09](apk/2.26.37.71-mobileconfig-boolean/snapshot-read.txt) is a managed
Boolean read. With a non-null table and Boolean specifier, it uses low 16 bits
as parameter index. A missing Boolean vector returns false. It reads a byte;
mask 0x06 triggers metadata/exposure lookup, and mask 0x11 retains the supplied
default. Otherwise bit 7 supplies the Boolean. Bounds/buffer errors are logged
and return the current value, which can be the supplied default. The exact
table construction, translation, source metadata, wrappers and exposure
effects remain outside this complete semantic closure.

## What this permits next

StatusInfraConfig's P(id) predicates require this effective Boolean pipeline,
not the raw rows surfaced by abprops-query --show. The next required closures
are fallback cache invalidation/update ordering, A0z's mutable control inputs,
manager snapshot/translation/overlay storage and correlated AB application.
The [status-dispatch gate](status-dispatch-gate.md) remains unported until those
authorities can be reproduced; no flags, defaults or migration timestamps
were fabricated in real account state.

The [recipe](apk/recipes/2.26.37.71-mobileconfig-boolean.json) retains 22 DEX
topics with full signatures, exceptions and register flow. Twenty-three full
REA envelopes retain engine/evidence identity, including the unavailable
callback-body observations. No new runtime behavior or live request was made.

```sh
target/debug/xtask apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-mobileconfig-boolean.json \
  --output docs/protocol/apk/2.26.37.71-mobileconfig-boolean --check
```
