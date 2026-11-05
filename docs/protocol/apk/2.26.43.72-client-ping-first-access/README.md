# Client ping configuration uses a distinct first-access cache

**Confirmed static**, APK2.26.43.72 code264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No target execution, account operation or claimed REA Evidence envelope.
This closes a configuration-mode gap from the
[response-relative scheduling correction](../2.26.43.72-client-ping-scheduling/README.md).
Runtime effective timing/configuration adoption remains unported.

## Caller options are part of the value contract

The retained1A7.A05/A07/A06 call00E.A02/A0D with00G.A02 for periodic interval,
manager timeout and alarm/Runnable Boolean selection. 00G.<clinit> constructs
A02 with firstAccessValue=true and exposure option02X.A01. By contrastA03,
used by the previously traced token getters, has firstAccessValue=false.
The common property ID/getter alone therefore does not specify cache behavior.

00G.A00 changes exposure option but preserves the first-access Boolean. In
particularA02 converts toA05, stilltrue. The constructor writes that Boolean
toA01, and the toString method labels it firstAccessValue. No enum value is
inferred solely from its obfuscated name.

00E.A0h selectsA07 when firstAccessValue=true andA0P otherwise. A01's integer
legacy path checks that selected cache, rechecks under its monitor, then reads
registered fallback/preference value and puts it inA07 for first-access options.
It does not simply re-read the newest received AB row on every call. A missing
registered/default value with no supplied fallback throws; a wrong preference
type is not silently converted into a guessed integer. Existing complete
[integer fallback](../2.26.43.72-token-bucket-configuration/integer-fallback.txt)
and [resolver](../2.26.43.72-token-bucket-configuration/configuration-dispatch.txt)
retain those exact branches.

## Conditional SDK branches and existing-entry precedence

A13 requiresA0E && A0b && !A0S.get(); A06 additionally evaluates lookup/context
presence and readyA08. These gates are not inferred from raw AB response values.
The prior [effective-integer audit](../../privacy-mex-config.md#effective-integer-configuration-has-two-authorities)
retains concrete adapters and initialization/selection flags. Keepalive uses its
first-access option within that resolver; this audit does not choose its runtime
mode or effective flags.

| Selected case | First-access behavior |
| --- | --- |
|Existing nonnullA07 entry |Selected cache lookup can return it before another underlying read; A01's legacy path also rechecks it under lock.|
|Empty cache, legacy result selected |A01 gets compiled/preference integer, insertsA07 and returns it.|
|Empty cache, SDK direct-return path |A06's first-access path usesA07.putIfAbsent at025c..0274; an existing concurrent entry wins. No unconditional packaged-default selection is justified.|
|SDK comparison-return branch |A06 reads SDK, invokes its legacy lambda with00G.A00 at02a8..02b8, compares, then usesA07.putIfAbsent at02de..02f0. The copied option remainstrue, so successful A01 can populateA07 before that insertion.|

**Derived static consequence, conditional:** in the final comparison row, with
an initially empty cache, the traced integer lambda completing normally and no
external cache mutation, its legacy read populatesA07 before the SDK insertion.
putIfAbsent then returns that existing value, which A06 returns at02ee..02f0.
This is not proof that all first-access reads always choose legacy, that every
account reaches the comparison branch, or that missing SDK specifiers can be
replaced by compiled values in other branches. The direct SDK path is distinct.

A06 cacheA0d is another cache, controlled byA0c. For first-access options the
selected flow bypasses its ordinary fast-read branch and ultimately usesA07.
Treating all cache fields as one generic cache would lose this precedence.
The checked literal specifier table lacks the selected ping entries, as retained
in the previous bundle; missing mapping is not a duration value or permission
to select a different branch.

## Reset and lifetime boundaries

FreshA0t export exactly matches the retained ready-cache-reset artifact. When
A0c is enabled it creates/replacesA0d; it does not accessA07. A0u resetsA09
and invokes SDK setup/readiness work through0A7, without directly resettingA07.
Thus the selected readiness/cache reset does not justify refreshing the first
ping's effective values from a new raw reply.

Physical field census finds oneA07 assignment, in00E's constructor, and reads
only inA01/A06/A0C/A0h/1XK.invoke/1XL.invoke. The constructor creates a fresh
ConcurrentHashMap. Selected map uses in the complete getters/adapters are
get/put/putIfAbsent, not a directclear/remove. The JSON failure branch in1XL
removes a persisted preference; that is not anA07 cache clear. The constructor
and direct field census do not exclude reflection/native mutation or aliases.
Full configuration-owner DI scope, replacement and identity-switch lifecycle
remain unclosed. This is an in-memory instance cache, not evidence for a durable
first-access snapshot surviving every process restart or a cache reset per socket.

## Rust gap and port obligations

The preceding runtime correction independently restarts the next ping interval
from accepted result/error and leaves maintenance scheduling separate. Its
locally selected durations remain unchanged. Before adopting native timing:
close registered final defaults/preference hydration and applicable SDK gates;
preserve first-access options and cache winner semantics; bind cache lifetime
to the actual native owner; preserve distinct initial/subsequent interval,
manager timeout and callback timeout; retain alarm/Runnable selection limits.
Do not turn raw AB absence into default authority, clear first-access state on
reconnect, or apply every new AB row as an immediate timer update without the
corresponding producer/lifecycle proof. No new runtime/config/clock/reset/retry
or live account behavior is introduced by this research chunk.

## Reproduction and verification

Existingxtask apk-smali APK --class CLASS --method METHOD --signatures
--exceptions exports00G(fullclass),00E.A0h/A0t/A0u/A13/A0C/<init>,1XK.invoke and
1XL.invoke. Exactapk-field-refs targetsclass00E, fieldA07,
typeLjava/util/concurrent/ConcurrentHashMap;. The constructor excerpt names
its full-export SHA256 and byte-PC interval. CompleteA01/A06 are linked above;
prior1A7/1xa/Fea exports establish the callers and accepted-response schedule.
FreshA0h/A0t bytes match the prior pinned artifacts exactly. These checks prove
selected source identity and trace, not effective values or live server behavior.
Checksums cover every other file; no raw APK or bulk decompiled tree is included.
