# Unordered hold configuration inputs, defaults and snapshot lifetime

**Confirmed static**, WhatsApp2.26.43.72, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
xtask DEX evidence, not REA Evidence IDs or runtime settings. The known REA
Android provider limitation was not retried. No account/app execution.

## Reproduction and bounded coverage

Method exports use, from repository root:
`target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'CLASS' --method 'METHOD' --signatures --exceptions > OUTPUT`.

| CLASS | METHOD | OUTPUT |
| --- | --- | --- |
| LX/0av; | <init> | config-constructor.txt |
| LX/0ax; | <clinit> | descriptor-declarations.txt |
| LX/00f; | <init> | boolean-descriptor-constructor.txt |
| LX/0AB; | <init> | integer-descriptor-constructor.txt |
| LX/00E; | A18 | boolean-descriptor-reader.txt |
| LX/00E; | A0d | integer-descriptor-reader.txt |
| LX/00E; | A02 | integer-resolver.txt |
| LX/1XK; | invoke | fallback-selector.txt |
| LX/1XK; | <init> | fallback-constructor.txt |
| LX/00E; | A0C | boolean-fallback.txt |
| LX/00E; | A01 | integer-fallback.txt |
| LX/08Y; | A19 | boolean-map-reader.txt |
| LX/08Y; | A1B | integer-map-reader.txt |

Field-write exports use
`target/debug/xtask apk-field-refs ../android-lab/apks/WhatsApp.1.apk --class 'LX/0av;' --field FIELD --type TYPE --writes-only > OUTPUT`:
A06/J ->delay-field-writes.txt; A0U/Z ->gate-field-writes.txt.
Each returned one direct DEX write, in the constructor. This does not rule out
reflection or prove the same object lasts across provider resets.

Large default-map methods are **bounded excerpts**, explicitly marked inside
the files. Generate complete exports with the same apk-smali command:

| CLASS | METHOD | retained excerpt |
| --- | --- | --- |
| LX/08Y; | <init> | default-map-constructor-excerpts.txt |
| LX/08Y; | A0H | boolean-default-helper-excerpts.txt |
| LX/08Y; | A07 | boolean-late-helper-excerpts.txt |
| LX/08Y; | A0B | integer-default-helper-excerpts.txt |

[excerpt-provenance.json](excerpt-provenance.json) records each full method's
byte count/SHA256 and retained half-open byte-offset ranges. Excerpts retain
class/method/signature headers, then instruction lines whose starting byte offset
falls in each listed range, followed where specified by all const/move opcode
destinations for the named register. These textual const/move censuses are not a
semantic value-flow or all-opcode write verifier. Full methods were generated/searched locally and selected instructions inspected; the excerpts do not claim full provider/default-map audit coverage.
An initial whole08Y scan located the selected methods/keys; unrelated entries
and callback selector0 are not interpreted here.

## Constructor samples the hold gate and conditionally reads the delay

[0av constructor](config-constructor.txt),0126..0170, reads0ax.A0C through
00E.A18 and stores the returned Boolean asA0U. If false, the branch0148 ->0416
loads long0 and joins the A06 store at016c. If true, it reads0ax.A0G through
00E.A0d, widens the signed int to long and storesA06. No clamp or unit conversion
appears between this int getter and the long store. This does not establish
upstream value validation or the downstream scheduler's treatment of negatives.

[0ax declarations](descriptor-declarations.txt) construct:

| Descriptor | Key | Constructor values after key | Selected reader |
| --- | --- | --- | --- |
| A0C /00f |0x9952|false,false|A18 uses first Boolean fieldA01|
| A0G /0AB |0x9953|60000,60000|A0d uses first integer fieldA01|

The [descriptor constructors](boolean-descriptor-constructor.txt) and
[integer equivalent](integer-descriptor-constructor.txt) bind those positions.
The [Boolean reader](boolean-descriptor-reader.txt) boxesA01 forA0D; the
[integer reader](integer-descriptor-reader.txt) boxesA01 forA02. Thus these are
explicit descriptor fallback inputs, not evidence the gate is enabled or the
runtime delay always equals60000. The prior [hold timer](../2.26.43.72-count-hold-release/README.md)
logsA06 as milliseconds and passes it to08Z.Cc2; scheduler internals remain open.

## Registry defaults may take precedence over descriptor fallback

The prior [provider dispatcher](../2.26.43.72-count-consumer-binding/provider-range.txt)
case66 (0x42) at09a8 constructs08Y, which extends00E. That is the constructor's
selected configuration binding. Its [A19](boolean-map-reader.txt) returnsA00;
[A1B](integer-map-reader.txt) returnsA02.

The [map constructor excerpts](default-map-constructor-excerpts.txt) show v14
Boolean builder and v13 integer builder, helper invocations and final
buildKeepingLast stores to those fields. Selected entries:

- A0H putsfalse at0x9952 and0x6f06 using its initially boxedfalse v1.
- The later A07 Boolean helper again putsfalse at0x9952 using boxedfalse v2.
- A0B puts60000 at0x9953. The constructor later also puts60000 there via v298,
  boxed from0xea60 at44da..44ee and used atbe24..be3c.

These observed default-map entries corroborate the descriptor inputs and add a
declaredfalse entry for base consumer scheduler gate0x6f06. No effective stored
values, complete duplicate-key/value-flow proof or map lifetime is inferred from
these bounded exports.

The [fallback callbacks](fallback-selector.txt), Boolean selector2/default and
integer selector1, route to00E.A0C/A01. The
[Boolean branch](boolean-fallback.txt) first checksA0h, then under its monitor
checks again, looks upA19 and uses the supplied descriptor fallback only if the
registry returns null. If both are null it throws Unknown BooleanField. It then
reads SharedPreferences with that default; selected cache/exposure branches
follow. The [integer branch](integer-fallback.txt) mirrors this withA1B/getInt.
So a registry entry takes precedence over descriptor fallback, and saved
preferences can override it. The outer00E.A06/config-owner/MobileConfig paths
and effective runtime precedence are not fully traced here; this selected
fallback path is not the whole configuration system.

## Consequences and limits

The gate and delay are constructor snapshots with one direct DEX writer each.
A false sampled hold gate stores delay0; an enabled gate reads an overridable
signed-int delay with declared60000 input/default entries. Base consumer
scheduler gate0x6f06 is separate from hold gate0x9952. These facts do not justify
turning either gate on or installing an unconditional60-second Rust timer.
Provider recreation, preference/config refresh, full registry collision/value
flow, scheduler timing/cancellation/failure and ACK ordering remain open. No
Rust network, retry/replay, key-generation or live account behavior changed.
