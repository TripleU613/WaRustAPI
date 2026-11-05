# Contact-check factory selection: constructor flags before timer selection

**Confirmed static**, APK2.26.43.72 code264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No live account operations, APK execution or claimed REA Evidence envelope.
This narrows the factory gap identified in the [caller-specific timer audit](../2.26.43.72-contact-queue-atomicity/README.md)
and [backoff observer trace](../2.26.43.72-contact-protocol-backoff/README.md).

## Refuted shortcut: the lightweight path is not the current Rust query

The previously retained HxV.A0B is querySyncPhoneNumberLightweight. It creates
Hck(UserJid,String) with null user/phone text and sets A0C=true, A0I=true,
A0E=property5751 (0x1677). The selected constructor does not initialize A0F;
Java's Boolean field default is false. Hck.A00 does not write A0F and builds
H6w. H6w's constructor copies Hck.A0F to H6w.A0J. 0pD.A06 starts aggregate
v40=false and sets it true from that per-user A0J at05f6..0602. Only a true
aggregate emits query/disappearing_mode at0ddc..0dfa. Thus the selected
lightweight phone factory does not request that protocol.

Rust contacts::queue_check always adds contact,lid,disappearing_mode. That is
confirmed vocabulary and a locally composed request, not exact parity with
this lightweight factory. Prior live acceptance of an envelope does not prove
native factory/caller parity or that every optional protocol is appropriate.
No request field is removed or timer inferred from this negative comparison.

## Regular phone factory and another phone entry differ too

HxV.A0C accepts only preset0vC.A0J orA01, resolves property5751, property5839
(0x16cf) and0eE.A0L, then calls HcZ.A00. For a nonempty phone its selected
common factory writes contact A0C=true; it also unconditionally sets Hck.A0L,
A0B andA0I true. Its fifth Boolean input is copied toHck.A0F; A0C supplies
constant true for that argument. It also supplies true to the A0Q input.
The builder/record/writer chain therefore selects disappearing_mode, and
additional picture/business/username flags beyond Rust's three protocol names.
The full wire remains conditional on the selected writer/configuration paths;
this is a traced flag comparison, not an executable complete wire oracle.

HxV.A0E uses presetA0U and the common factory with the corresponding A0F
argument false. The same protocol name's existence elsewhere cannot be used
to describe all phone queries as one constructor profile. Its other flags and
context also remain attributable to that separate caller.

| Selected native path | Hck.A0F / disappearing-mode flag | Dispatch evidence |
| --- | --- | --- |
|HxV.A0B lightweight phone |false, never assigned by this factory |previous bundle retains32000ms call to0pD.A07 |
|HxV.A0C regular phone |true supplied toHcZ.A00 |callsHxV.A02, which supplies32000ms to0pD.A07 |
|HxV.A0E other phone entry |false supplied toHcZ.A00 |also callsHxV.A02 with32000ms |
|Rust contacts::queue_check |always emits disappearing_mode |exact native profile remains unbound |

These selected query paths use HxV's A0H helper, whose Hzo observer binding is
retained in the backoff bundle. A separate0p7 sync call uses128000ms; there is
still no universal usync timer. Input context/mode alone cannot select a complete
caller policy, and no request is tagged as a native factory merely because it
shares those strings.

## Register/constructor identity and reproducibility

HcZ.A00's full descriptor has UserJid,6p2,preset,String,int, then seven Booleans.
Its fifth Boolean is zero-based parameter9, registerv15 in this static method.
The regular A0C caller passes registers v3..v14; caller v12 is that argument
and is set true at0088. In A0E, its v2..v13 range supplies caller v11=false.
H6w's full constructor at00b6..00be establishes the A0F -> A0J field transfer.
Those names must not be equated solely by their obfuscated letter.

Current0vC.<clinit> identifies A0J as INTERACTIVE_QUERY and A0I as
INTERACTIVE_MULTI_PROTOCOL_QUERY. Both use0vD.A04/0vE.A05; their own current
initializers retain interactive/query string values. Distinct preset objects
also carry other fields, so equal visible mode/context strings are not proof
of equal factory semantics. The selected constructor and full writer are
retained rather than reducing the comparison to an attribute scan.

Reproduce with existing xtask apk-smali APK --class CLASS --method METHOD
--signatures --exceptions:0pD.A06,Hck.<init>/A00,H6w.<init>,HcZ.<init>/A00,
HxV.A0C/A02/A0E,0vC/0vD/0vE.<clinit>. The prior bundle retains HxV.A0B and
Hzo binding/callback evidence. Checksums cover all other files here; no APK,
user data or bulk decompiled tree is committed.

## Remaining implementation requirements

Before changing the Rust producer, select and port the intended native factory
including protocol flags and applicable effective configuration, not a timer
alone. Bind its observer/backoff scope, clock assumptions and native read gates;
retain saved request identity and uncertain outcomes. Full wire-condition,
configuration, integrity/adoption and per-scope wait behavior remain unfinished.
This is a static factory comparison; runtime send/timeout policy is unchanged.
