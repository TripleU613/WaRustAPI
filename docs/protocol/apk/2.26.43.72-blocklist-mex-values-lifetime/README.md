# Migration variable values and callback lifetime — APK 2.26.43.72

Confirmed selected static, build264307238; APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected DEX exports/catches plus
[complete REA map export](../../rea/2.26.43.72-blocklist-mex-values-lifetime/README.md).
No APK, account, migration request or native callback was executed.

## Direct setters retain null at the intermediate object/map layer

Prior [MigrateBlocklistLid builder](../2.26.43.72-blocklist-migration-obligation/README.md)
initializes several nested identifier fields through GraphQlCallInput A0A/A08
with null. Both direct setters obtainNha and callNha.A00. Successful A00 checks
mutability/key then appends key AND value to its ArrayList, including null;
there is no value-null filter. Repeated named setters append another pair.

GraphQlCallInput constructor retains a0iH factory and null object-node pointer;
A02 lazily getsNha through0iH.A01. It takes a pool entry or constructs one,
then invokesNhZ.A06; that requires refcount increment to1 and sets mutable=true
from false. Whole pool/recycle safety and all alias lifetimes remain untraced.
The checked mutation guard can throw; this is not unconditional mutation of
arbitrary stale/immutable objects.

A03 createsTreeMap and invokes privateA01. It iterates everyNha key/value pair,
converts the value through privateA00 and callsMap.put even when that value is
null. A00's initial null branch returnsnull; nestedNha similarly becomes a
TreeMap. Duplicate names therefore use the last pair at this intermediate
layer. No key is removed merely because an initial direct setter suppliednull.
This closes object/map retention, not final wire null/omission behavior.

0i1.Axc exports its rootGraphQlCallInput throughA03. In contrast,0i1's own
primitive String/Boolean/Number wrapper methods skip null arguments. The
migration's nested directGraphQlCallInput setter path must not be replaced by
that different top-level wrapper's omission rule. Final JSON serializer,
configuration, exact list/null encoding and transmitted bytes remain open.

## Coroutine error handling is bound to literal false

2LE.A0Q createsIE8(selector3) and calls0iq.ARv. The prior complete
[0iq export](../2.26.43.72-prekey-lid-mex-operation/operation.txt)
showsARv createsKmF andCxG(selector16), thenAQY. Constructors bindCxG's
continuation toA00 and supplied error-policy function toA01. Selected case16
sets0id.A00=CxE(selector43), then createsCxG(selector15) and stores it in
0id.A01 via the shared tail at0d2c. Constructor field order is checked,
not guessed from argument position. The retained6oh.A0d is only a cast/non-null
check on the supplied callback, not a builder/deadline mutation.

CxE case43 resumes the captured continuation with its incoming value. CxG
case15 takesHbw, invokes the supplied policy, and on false resumes failure
usingGiw(Hbw) wrapped by0dg.A00. IE8 case3 calls2L8.A1A; its complete body
boxes const0, so the selected policy is FALSE. A helper's name is not evidence
of true. The indefinite-error-wait hypothesis is not established by this path.

Fresh0id export matches the retained
[callback adapter](../2.26.43.72-prekey-lid-mex-callback/binding.txt) exactly:
A05 invokesA00; A08 invokesA01 and returns its Boolean. The retainedBaseMex
error/throwable forwarding uses that adapter. Thus selected errors reach the
continuation and the prior migration coroutine'sGiw catch ->7Db ->manager7gZ,
not its7Da mode/cache adoption path. Duplicate/concurrent callback ownership,
all lower transport error admission and account generation remain separate.

## Selected default and non-Pando32s send branch

The [prior migration builder](../2.26.43.72-blocklist-migration-obligation/migration-coroutine.txt)
uses2LA.A0Y; [checked factory](../2.26.43.72-prekey-lid-mex-operation/concrete-factory.txt)
constructs a fresh0iq. Its constructor'sA00=32000ms andARv match the retained
shared operation methods exactly. This selected producer does not callBZr,
change the timeout, or pass the builder to another deadline setter in its
shown direct path. Its callback initializer only writes0id callbacks.

The existing [operation/handle trace](../2.26.43.72-prekey-lid-mex-operation/README.md)
shows freshA03=false selects0j3 (notRsH).0j3 still chooses Pando versus IQ at
runtime. Its non-Pando branch passes the saved32000ms to0Sf.A0W, whose callback
registration/expiry scheduling is already separately traced. That is a
selected non-Pando send deadline, not a universal migration/Pando deadline,
outer wait duration, socket-write limit or permission to retry. AQY discards
the returned operation handle; this wrapper shows no new retained handle or
cancel hook. Full cancellation/lower lifecycle remains unproven.

## Rust comparison and limits

No dedicated Rust migration sender or mode adoption is added. Existing generic
group MEX construction uses a different nonnull input and is unchanged. Do not
invent null fields, copy wrapper omission semantics, assume error callbacks hang,
or assign a universal32s retry loop from this evidence. Final wire serializer,
identifier provider authority, effective Pando policy, writer/account checks,
callback races and complete migration adoption remain unported.

## Reproduction

From repository root, pinned APK and current built xtask:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
OUT=/tmp/blocklist-mex-values-lifetime-repro
mkdir -p "$OUT"
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'LX/6oh;' --method A0d --signatures --exceptions > "$OUT/callback-cast.txt"
target/debug/xtask apk-smali "$APK" --class 'Lcom/facebook/graphql/calls/GraphQlCallInput;' --signatures --exceptions > "$OUT/input.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/Nha;' --signatures --exceptions > "$OUT/object-node.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/NhZ;' --method '<init>' --signatures --exceptions > "$OUT/node-init.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/NhZ;' --method 'A06' --signatures --exceptions > "$OUT/node-activate.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0iH;' --method 'A01' --signatures --exceptions > "$OUT/node-factory.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0i1;' --signatures --exceptions > "$OUT/variables.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/2LE;' --method 'A0Q' --signatures --exceptions > "$OUT/await-helper.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/CxG;' --method 'invoke' --signatures --exceptions > "$OUT/callback-initializer.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/CxG;' --method '<init>' --signatures --exceptions > "$OUT/callback-constructors.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/IE8;' --method 'invoke' --signatures --exceptions > "$OUT/error-policy.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/IE8;' --method '<init>' --signatures --exceptions > "$OUT/error-policy-init.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/2L8;' --method 'A1A' --signatures --exceptions > "$OUT/error-policy-value.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/CxE;' --method 'invoke' --signatures --exceptions > "$OUT/success-callback.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/CxE;' --method '<init>' --signatures --exceptions > "$OUT/success-init.txt"
```

Linked0iq constructor/ARv and0id adapter were freshly exported and compared
with their retained methods. Complete synthetic methods retain all selectors
so the identified branches can be checked; other selectors are not interpreted
as this operation. checksums.json binds every other file here by bytes/SHA256.
Static evidence does not prove live acceptance, non-delivery or zero bans.
