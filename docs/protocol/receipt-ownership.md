# Receipt ownership short circuit and message flag provenance

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Eight-topic recipe](apk/recipes/2.26.37.71-receipt-ownership.json),
[complete methods](apk/2.26.37.71-receipt-ownership/targets.json),
[ten physical flag writers and scoped excerpts](receipt-ownership-flag-evidence.txt).
This narrows an admission prerequisite, not a complete Rust receipt_user port.
Offsets below are method-relative bytes; field-scan offsets are APK DEX bytes.

## The caller short circuits before the helper

**Confirmed:** in [15A.A02](apk/2.26.37.71-receipt-admission/receipt-admission.txt),
0192..019a loads message key 1gc.A02. When true, the branch at 0196 goes to
01ac, which sets v2=0; the next test at 01ae goes to 01ea, the ordinary path.
The branch displacement is in code units: 0196 + 2*0x0b = 01ac.
It does not call 1gd.A0d and does not evaluate the foreign-input rejection.
The key is independently labelled `from me` in 152.A00's edit-validation log
at 0af8..0b12 (retained in the flag inventory excerpt).

**Confirmed:** when key.A02 is false, 1gd.A0d is called. Its exact expression is
`(key.A02 || (message.A0h == 64 && BO0(Bps.A00))) && !message.A0y`.
This follows its branches to 002e/0038, not a name-based interpretation.
Therefore only the type-64 own-Bps.A00 case with a clear flag can make this
helper true for the false-key caller. Bps.<init> sets the message type to 64;
it does not initialize its UserJid field A00 in the retained constructor.
The field's complete producer/meaning remains **unknown**. Do not call all
type-64 messages ordinary authored messages or invent a nonnull sender.

For the false-key/helper-false branch, 01b6..01e6 obtains the receipt DeviceJid.
BO1(device) true, or Cju.A01(device) true, goes through the special admitted path.
Otherwise property true skips the input; property false follows that same
special path at 04c6. This does **not** imply the ordinary 01ea path or guarantee
a user-row write. All later pool selection still matters.

**Confirmed port consequence:** ordinary own-key inputs need neither this flag
helper nor the foreign-input property to pass this ownership check. Requiring a
known A0y value for every outgoing receipt would add a guard absent here. The
positive-time, status, message-state/type, read-setting, manager exclusion and
counterpart checks remain separate; the short circuit does not bypass them.

## Property identity and defaults

**Confirmed:** AzZ.<clinit> 0062..0074 constructs A04 using integer 0x8abc
(**35516**) and true v1, through 1xf.A0u(int,boolean). A0u passes that boolean
to both arguments of 09P.<init>(int,boolean,boolean). 09P.toString labels the
values prodDefaultValue/debugDefaultValue, so both baked defaults are **true**.
15A.A02 0084..0096 resolves this descriptor through 00D.A13 and keeps the
result in v7; its 01e2 predicate uses that resolved value.

**Unknown:** this account's effective property value, descriptor exposure/cache
initialization and complete server-to-MobileConfig mapping. The existing
[Boolean resolver trace](peer-identity.md#companion-removal-property-and-effective-value-2026-10-08)
is separate from baked defaults; an account override can supersede a default.
No property setter, invented effective value or new refusal recovery is added.

## Flag writes are not a universal ownership label

**Confirmed:** an all-DEX exact writes-only scan of 178.A0y:Z finds ten
physical methods. The inventory retains their exact DEX offsets, full method
signatures, source-dump digests and local write windows. Constructor 178.<init>
sets false at 0030..0034. Bp2.A0P(178,DFI) starts v2=true and, when its builder
key.A02 is true, calls A0H(4) and sets A0y=true at 002a..002e. It also supplies
message.A0p from CjN.A00; full device producer authority is **untraced**.

1fm.A04(List,Set) tests the selected message key, sets A0H(4) and writes true
at 0f22..0f26. 1Uu.A00 writes true at 01a4..01a8 under its key test. 12y.A02
writes true at 0354..0358 under a key test; its surrounding logs identify
NewsletterMessageProtobufHelper, not a general database row loader. D7E.Byt
copies source-flag truth to the destination through true v2 at 00e4..00e8.
These are distinct creation/restoration/copy contexts, not proof that all local
outgoing messages acquire this flag.

Other write windows are 152.A00 at 0dbc..0dc0, 153.A00 at 07da..07de,
1gz.A00 at 390a..390e and 3Pe.run at 0e6e..0e72. Their selected register values,
complete path inputs and persistence/reload policy remain **unknown** here.
3Pe is a discriminator-switch runnable; the excerpt is not its whole contract.
The full methods were examined locally and their digests retained, but only the
listed windows are committed as scoped evidence. No complete flag semantics,
reflection absence or runtime origin reconstruction is claimed.

## Remaining implementation inputs

The [live one-column writer](receipt-user.md) and
[bulk import writer](user-receipt-import.md) remain separate. This ownership
short circuit removes one mistakenly broad prerequisite for ordinary own-key
receipts. Full state initialization/projection, effective read-setting gates,
normal-send seeding, PN/LID counterpart authority and later pool/post-write
behavior still need tracing before general user-row membership or primary
identity cleanup can be implemented. Existing immutable targets and raw
DeliveryFact observations must not be promoted to receipt_user existence.

No Rust runtime, account, key, request ID or server wait changed. No network
operation ran. Existing apk-smali, apk-field-refs and apk-research-bundle expose
reproduction; the API research entry stays CLI-only and x-live=false. This new
recipe has no verified remap anchors: re-anchor its selectors manually before
exporting a new release. Cached file diff is not semantic parity proof.

Ownership validation: eight topics / 13 files / 59,820 bytes reproduce exactly
from APK 37.71. Cached self-diff verifies eight unchanged topics and four metadata
artifacts, not cross-release equivalence. CLI-reference regeneration,
OpenAPI/library-symbol checks, formatting, whitespace and hygiene passed.
Hygiene checked 1,486 local links and 39 bundles / 9,331,935 bytes. Flag-writer
excerpts are separately attributed outside that bundle. No fresh workspace
runtime test or live account operation is claimed.
