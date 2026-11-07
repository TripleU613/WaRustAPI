# Stateless retry configuration inputs and selected defaults

**Confirmed selected static / research only**, APK2.26.43.72 build264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected xtask DEX fallback methods plus retained same-build resolver
and lookup evidence. PCs are bytes; branch displacements16-bit units. No target
execution, new REA Evidence ID, account access or Rust runtime change.

## Retry controls use effective configuration accessors

The prior [retry input trace](../2.26.43.72-enc-state-pq-retry-inputs/README.md)
identifies the extra key-preparation switch as
`isStateless && integer(0x53e5)>0 && boolean(0x5d4c)`.
The selected native key-load branch is also used for positive incoming count;
this investigation addresses the independent zero-count stateless branch.

Fresh exports of00E.A0a/A02 and A14/A0D matched previously retained selected
methods. Reuse [integer getter](../2.26.43.72-token-bucket-configuration/integer-getter.txt),
[integer resolver](../2.26.43.72-count-hold-config/integer-resolver.txt),
[Boolean getter](../2.26.43.72-media-priority-selection/boolean-getter.txt) and
[Boolean resolver](../2.26.43.72-media-priority-selection/boolean-resolution.txt).
A0a invokes A02 with00G.A03 and null supplied default; A14 analogously invokes
A0D. These pass their legacy fallback and SDK readers into00E.A06. Null supplied
default does not itself mean the effective value is0 or false.

The existing [configuration precedence trace](../../privacy-mex-config.md#effective-integer-configuration-has-two-authorities)
and [REA records](../../rea/2.26.43.72-privacy-mex-config/README.md) distinguish
legacy cache/preferences/compiled defaults from ready MobileConfig selection.
Raw returned AB rows are not equivalent to effective accessor values. Rust's
abprops module already labels most returned rows as evidence, not a complete
MobileConfig application port. These two retry controls are not integrated.

## Selected compiled entries are0 and false

[Complete08Y.A0B](integer-default-helper.txt) boxes integer0 from v2 at0050 into
v12 at0056, then inserts key0x53e5 with v12 at16d2. The complete prefix contains
no branch and no other v12 destination. [Complete08Y.A0H](boolean-default-helper.txt)
boxes false into v1 at0016, then inserts key0x5d4c with v1 at0e84. Its complete
prefix contains no branch and no subsequent v1 destination before that insertion.
These are specific compiled map entries, not guesses based on omitted AB rows.

Existing [constructor excerpts](../2.26.43.72-count-hold-config/default-map-constructor-excerpts.txt)
show Boolean builder v14 and integer builder v13, then buildKeepingLast and
assignment to08Y.A00/A02. Corresponding [Boolean map reader](../2.26.43.72-count-hold-config/boolean-map-reader.txt)
and [integer map reader](../2.26.43.72-count-hold-config/integer-map-reader.txt)
return those maps. The broader local08Y scan found each selected literal key once,
but its complete class is not committed. Full later helper/computed-key precedence
and runtime overrides are not newly proven. Because buildKeepingLast is used,
the selected insertions alone must not be called a complete final-map audit.

## Both concrete initial SDK specifiers are zero

The retained [0El.A0L lookup](../2.26.43.72-token-mobileconfig-binding/lookup-map.txt)
allocates41080 longs and has2065 literal aput-wide sequences. Neither property
21477 (0x53e5) nor23884 (0x5d4c) has an explicit write in that complete sequence;
therefore each concrete initial array entry is Java-initialized0. The prior
[lookup binding](../2.26.43.72-token-mobileconfig-binding/README.md) establishes
0A8.B8Q indexing this array and selected DI/readiness boundaries. Array aliases,
reflection/replacements and runtime mutation remain outside the static result.

For the previously traced SDK integer adapters, a zero specifier produces0 and
an INVALID_SPECIFIER_MISSING_IN_WA_LOOKUP_MAP classification rather than a
normal nonzero-specifier SDK read. If that selected integer result is returned,
its >0 test is false, so the extra stateless switch is off regardless of the
Boolean operand. This does NOT imply every native call selects that result:
legacy preference/cache values and readiness/comparison paths remain distinct.
No effective Boolean SDK value is inferred merely from a zero Boolean specifier.

## Reproducible bounded derivation and remaining integration

Run `python3 derive-inputs.py` in this bundle. It must reproduce
[derived-inputs.json](derived-inputs.json) byte for byte. The script checks every
prefix opcode against a closed set, verifies the relevant reaching destinations
and boxed-zero/false insertions, and validates all2065 unique/in-bounds literal
lookup writes before reporting the two missing indices. It is pinned to these
methods, not a general Dalvik interpreter or a complete provider/default-map
value-flow analysis. Source hashes are recorded in the result.

The current Rust queue only handles aggregate count0 in its bounded minimal
branch. These findings support specific native default/SDK-zero paths but do not
prove that every stateless count0 message belongs to that branch. Integrating
carried keys still requires applicable configuration authority, durable direct-
distribution selection/upload exclusion, immutable signatures/wire and uncertain-
outcome no-replay behavior. Do not enable emission using raw AB rows alone, force
false to avoid the key path, or manufacture keys. Full native configuration and
retry coverage remain unfinished. No runtime gate, request or key mutation added.

Reproduce apk-smali with signatures/exceptions for08Y.A0B/A0H. A broad08Y export
was used only to locate the selected methods; the unrelated class tree stays
outside Git. Referenced getter/resolver methods and lookup retain their own
manifests. [checksums.json](checksums.json) pins all files here.
