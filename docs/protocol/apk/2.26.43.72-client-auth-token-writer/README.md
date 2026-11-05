# Client-auth-token request writer — APK2.26.43.72

**Confirmed static**, checked DEX text exports, not runtime observations or
REA Evidence envelopes. APK SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`
reverified2026-11-04. Commands: `xtask apk-smali APK --class CLASS --method
METHOD --signatures --exceptions` and `xtask apk-xref APK --anchor auth_token`.
The constructor export was filtered to exact prototype
`LX/Dzb;-><init>(Ljava/lang/String;Ljava/lang/String;I)V` from28 constructor
overloads; no other overload's selector meaning is transferred to this one.
Offsets are method-relative bytes, branch deltas are DEX code units.

## Selected constructor28 grammar

The retained [1jA.A01 request manager](../../rea/2.26.43.72-authkey-outcomes/client-auth-token-request.json)
constructs Dzb(generatedId, input,28). In that exact overload, rsub-int/lit8
at001a..001e computes28-selector; selector28 therefore enters00a6. The
other branch builds an app-state coex request and is not the token request.

The selected branch creates an iq builder and adds namespace
urn:xmpp:whatsapp:push, destination0mL.A00, type=get and the supplied ID
through B23.A03.0mL has empty user and getServer=s.whatsapp.net, establishing
the server destination rather than an arbitrary phone JID. It builds one pn
child, conditionally writes the supplied input string as UTF-8 bytes through
0Tj.A05, then adds that child and stores the completed node in Dzb.A00.
0Tj.A01 retains byte content when no child nodes exist. The bounded object
shape for a nonnull input is therefore:

```text
iq xmlns=urn:xmpp:whatsapp:push to=s.whatsapp.net type=get id=<generated>
  pn: UTF-8(<manager input>)
```

This is builder grammar, not an executed encrypted wire vector or proof every
input is accepted by the service. The pn label does not establish that the
input is a phone number. No auth_token child or attribute is created by this
selected branch. The exact auth_token string anchor has25 physical references
elsewhere; their shared spelling is not proof of the same request family.

## String validation is not a numeric ID or hard length gate

B23.A03 validates the ID with0Tk.A06 using lower0, upper9007199254740991 and
optional=false, then adds the id attribute when that helper returns true.
A06 compares Java String.length; it does not parse the ID as an integer. The
large upper constant is not evidence of a53-bit numeric IQ-ID requirement.
B23.A03 returns long0, reused as the pn input's lower bound; its upper is300
and optional=false. Empty nonnull strings satisfy that lower bound.

**Confirmed qualification:** A06 calls00L.A0C(false,diagnostic) for a null
nonoptional value or an out-of-range length. The selected00L.A0C creates an
AssertionError, passes it to Log.e and returns; there is no explicit throw in
that method. A06 returns false for null, but true for a nonnull out-of-range
string after logging. Thus a nonnull long string still reaches A05 in these
selected bodies; copying300 as a proven server rejection or hard native
constructor limit would be unjustified. This does not authorize sending such
an input. Logging-internal failure behavior is not exhaustively audited.

## Eligibility and recovery remain separate

The earlier1jA.A01 evidence checks0AI.A0O. When not ready it saves input in an
AtomicReference; A00 later getAndSet(null)s one value and forwards it when
nonnull. The request uses callback FUT selector1, transport discriminator314
and32000ms via0Sf.A0Q. Its successful parsed-token branch stores
pref_client_auth_token and reaches conditional held-request selection as traced
in [challenge ownership](../2.26.43.72-challenged-request-ownership/README.md).

**Unknown:** full input provenance/authenticity and eligibility in the incoming
push path, readiness predicate semantics, complete reply schema/converters,
refusal/backoff handling, generation/account binding and durable restart
reconciliation. This field is not a registration JSON prerequisite inferred
from its name, and this request is not proven mandatory on every account.
No Rust sender, guessed input, fabricated token, automatic challenge retry or
replay authority is added. Existing uncertainty holds remain.

## Checksums

- [address-helper.txt](address-helper.txt): `0a3f4593e81c8fb588150474f1b3fdd9688f50df41a0af82c295dba9836d99c9`
- [assert-policy.txt](assert-policy.txt): `624cde421ffd23991d9104849d5d30c6f8add802b5daa811c5f493d69c21c443`
- [charset.txt](charset.txt): `69ac55ea0e4e83cb125dffb15ef6c4483509af82776f34bd4de1783bc6e00c47`
- [constructor-selected.txt](constructor-selected.txt): `46930a4ee8390c0b984e0ed231adf228cd9caafa77a55ecd773ae97b0ee93a08`
- [destination-class.txt](destination-class.txt): `9b7609bba92f00a97aeba1053898348d6f4c39cd17cb1c8a0001067ae16856d3`
- [id-helper.txt](id-helper.txt): `f41a1c8f84b2ab0bf70b1845e9a41f428c8ce3c28f63be969d523f59385c5fe4`
- [input-validation.txt](input-validation.txt): `f59ac3f687221f42dc9f8bc9fd1c6d218245d8b2a2d0c5e809eef6fcc31244b9`
- [node-build.txt](node-build.txt): `e2c7fc22d708d059ddb35a863669c67b7e12209cdac91376cd966d15ad2a4f41`
- [text-storage.txt](text-storage.txt): `e5dd80acb479040ba7097a31c3c925ad5673a5f9f72e5a33aea6d4ec44e69d4a`
- [type-helper.txt](type-helper.txt): `203fcd9e64b071140ef6579afb8cceada18c23762247371686f9889150e55be8`
- [wire-xrefs.txt](wire-xrefs.txt): `e506886fd6debb79c0e867d7212f2301eb05bb8ceef4eb9acdc5a84ce6b95b89`
