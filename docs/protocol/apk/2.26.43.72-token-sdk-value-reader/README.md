# Token SDK concrete value reader and context construction

**Confirmed static**, APK2.26.43.72 code264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Repository xtask DEX exports; no claimed REA Evidence envelope or target execution.
This follows [SDK defaults](../2.26.43.72-token-sdk-defaults/README.md).

## Bodyless base methods have concrete Java overrides

The interface-aware declaration scan for0A2.A06(JJZ)J finds0A3 andNv3, not just
the bodyless0A2 base declaration.0A3 returns the caller's supplied default.
Nv3 has a DEX implementation that reads a ByteBuffer. Thus the preceding base
method export was not proof that this read requires native analysis. The scan
covers114178 APK classes with declared superclass/interface ancestry; it does
not establish runtime replacements or instance selection by itself.

09u.A08 caches a latest context inA0Y and constructs it throughA01 when absent.
09u.A07 additionally caches per-config contexts in an AtomicReferenceArray,
validates the index and usesA08 for a missing entry. Both include account-unit
logout checks returningA0K for an empty user in unit type2. Invalid config indices
returnA0M. A01 asks its09l manager for getLatestHandle and its Java ByteBuffer.
With no usable handle/buffer, the explicit09m manager case returnsA0L; otherwise
it constructsNv3 even when data is absent. An underlying0EV manager can supply
PCc and wrapNv3 withQAh/PLr translation. Therefore default-only, directNv3 and
translated paths must remain distinct. Complete manager selection and lifecycle
are not inferred from constructor availability.

## Table representation and long read

Nv3's constructor sets LITTLE_ENDIAN, derives the root offset from the current
buffer position and its first relative int, then checks format/version fields.
The full constructor and catch scopes are retained; it caches vtable offsets,
includingA06 at slot0x0a for long values andA05 at0x0c for long metadata.
It also handles additional maps and unit metadata, not modeled by the cases here.
Oy1.A02 computes vector data start by adding table root, vtable field offset,
that location's relative int and4. Nv3.A02 reads metadata int at start+slot*4.
Nv3.A01 decodes the six-bit parameter type at specifier bits48..53. Long reading
requires type2 and uses the low16-bit slot at vector start+slot*8.

Nv3.A06 checks existing metadata. Flags0x6 trigger auxiliary handling (unless the
boolean suppresses the selected call); flags0x11 return the supplied default.
Otherwise an existing long-value vector yields getLong. No value-vector offset
returns literal0. IndexOutOfBoundsException/BufferUnderflowException in the
retained metadata/value ranges log and return the supplied default. This is a
bounded branch description, not permission to treat every malformed buffer as
zero or to bypass the preceding source/type gates.

## Source classification

Nv3.A07 reports DEFAULT__NO_DATA_ON_DISK withoutA03. Its A00 metadata dispatcher
selects the vector by type and returns sentinels on invalid types/slot reads.
P7S initializesA00=-239,A02=-495,A01=-751; literal17 is also handled specially.
For ordinary metadata, B1w maps nonzero(meta&0x10) to2 and bit0 adds1:

| Combined bits | Source classification |
| ---: | --- |
| 0 | SERVER |
| 1 | DEFAULT__SERVER_RETURNED_NULL |
| 2 | DEFAULT__SERVER_RETURNED_NULL_EMPTY_UNIT_ID |
| 3 | DEFAULT__MISSING_SERVER_VALUE |

Sentinel-239 and literal17 report missing value;-495 invalid type;-751 invalid
slot. These sources feed the previously retained0A2.Ark gate: only selected
SERVER/DEFAULT__SERVER_RETURNED_NULL paths proceed toA06; source reporting,
value reading, int override and silent-push precedence remain distinct stages.

The token specifiers are type2 with config/slot pairs865=423/101,909=444/110,
996=483/125,997=484/126. These SDK coordinates are not AB property IDs, times or
effective values. read-cases.json is explicitly an **analyst-derived** model of
35 metadata cases plus these coordinates. Its default-mask column describes
A06's mask alone, not whether0A2.Ark reachesA06. It is not an executed native or
APK oracle and cannot establish buffer hydration or effective token policy.

## Reproduce and remaining work

All exports use apk-smali with exact classes/methods and --signatures --exceptions:
09u.A07/A08/A01,Nv3.A07/A01/A02/A00/<init>,P7S.<clinit>,B1w.A00,Oy1.A02.
The override export uses apk-virtual-overrides --class 'LX/0A2;' --method A06
--signature '(JJZ)J' --include-interfaces on the pinned APK.

```sh
python3 docs/protocol/apk/2.26.43.72-token-sdk-value-reader/derive-read-cases.py > /tmp/token-read-cases.json
cmp /tmp/token-read-cases.json docs/protocol/apk/2.26.43.72-token-sdk-value-reader/read-cases.json
```

Remaining: manager/handle producers and persisted server-buffer provenance,
translation maps, cache invalidation/readiness across updates, silent-push and
override hydration, and connection/account selection into00E. Raw AB reply rows
still do not establish effective SDK policy. No Rust timing/admission change,
new network request, reset or live account operation; full parity remains open.

The subsequent [handle/cache follow-up](../2.26.43.72-token-config-refresh/README.md)
traces concrete handle producers, file framing and separate latest/session cache
transitions. It does not establish complete effective-policy hydration.
