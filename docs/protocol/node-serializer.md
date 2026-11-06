# Binary-node serializer compatibility

## Empty child content follows the APK (2026-11-06)

**Confirmed selected static, APK2.26.43.72:**
[Complete serializer and conditional-size-provider exports](apk/2.26.43.72-node-serializer/README.md)
show1r2.A01 omits content for an empty child array, but preserves a nonnull
zero-length byte array. Rust now omits both the content slot and LIST_EMPTY
suffix for NodeContent::Children([]), including nested nodes. Attributes,
nonempty children and byte content keep their existing encodings. The in-memory
model and persisted intents are unchanged.

**Offline regression:** literal byte fixtures distinguish omitted children,
empty bytes, attributed nodes and nested nodes. The regression failed before
the fix on `00 f8 02 19 00` versus APK-derived `00 f8 01 19` for synthetic iq.
Existing codec properties already normalize zero-child content to Empty.
This corrects construction; it does not prove server rejection of the old form.

**Confirmed selected static / negative:** list-size and raw-byte width branches
match existing Rust at actual transport sizes. Native bounded serialization
uses a conditional provider; default returnsnull, installed provider is feature/
state/heap dependent. Selected bounded-buffer E5X occurs before record encryption;
framing E5W occurs after encryption/counter consumption. No universal native
serialization cap or automatic retry follows. Runtime budget values, remaining
size producers and disconnect callback authority stay open.

**Open follow-through:** optional colon nibble12 packing exists in nativeA06;
Rust does not currently select it. Native inbound decoding, actual producers,
complete string/token context and full codec parity remain unverified. No live
account operation or decoder behavior change is included here.

## Packed nibble colon and padding match the APK (2026-11-06)

**Confirmed selected static / implemented, APK2.26.43.72:**
[Complete123.A07/A0A reader exports and existing writer context](apk/2.26.43.72-packed-nibble/README.md)
close the colon-alphabet follow-through above. NIBBLE_8 digit12 is colon;
actual13..15 are invalid. Odd strings contain2*pairs-1 actual digits, so the
last low nibble is ignored, regardless of its value. Oddzero-pair headers
are invalid; empty even values are admitted. HEX_8 retains all16 hex digits.

Rust now packs colon in its existing nibble contexts, accepts packed colon,
rejects invalid actual data instead of emitting NUL, ignores padding before
validation and rejects negative digit counts. Two independent literal fixtures
failed before the fixes and pass afterward; the reader matrix separates actual
invalid digits from ignored padding and valid hex. No new request, retry,
response-settlement authority or live account operation is introduced.

This resolves the specifically named colon/alphabet/count gap, not complete
context-dependent serializer parity or the effects of every native parser fault.

## Names and body text use different string rules (2026-11-06)

**Confirmed selected static / implemented, APK2.26.43.72:**
[Complete node constructors and typed-JID writer with existing A01/A05/A06](apk/2.26.43.72-string-context/README.md)
show token lookup remains enabled for names, but unknown tag/attribute names
and JID_PAIR servers do not nibble/hex pack. Users and attribute values allow packing.
Node text content is converted to bytes in0Tp's constructor, then serialized
without token lookup or packing. Rust now follows these distinct contexts.

**Offline regression:** independent literal byte vectors cover a numeric tag,
numeric attribute name, packed attribute value, token-looking body and colon
body. They failed before the fix. Existing codec properties already normalize
text content to bytes. Decoded token Text can now re-encode as raw byte content;
wire spelling is canonicalized while text bytes remain unchanged. No automatic
text-to-JID conversion or persistence/settlement/retry change is included.

**Unknown:** A05's context-sensitive automatic JID conversion, full special
class tuples, native default charset outside ASCII and parser fault routing
remain separate evidence gaps. No live operation or complete codec parity claim.

## Legacy interop tuples cannot erase a nonzero integrator (2026-11-06)

**Confirmed selected static / local identity correction, APK2.26.43.72:**
[Complete native244 reader and interop identity class](apk/2.26.43.72-interop-identity/README.md)
show integrator is an address component. Rust's existing reference245 decoder
previously discarded its integrator and encoder always wrotezero. That could
collapse distinct wire identities. The decoder now explicitly refuses nonzero
integrators it cannot represent; zero preserves its prior reference behavior.

The literal legacy-schema regression failed before correction and covers
integrators1/2/65535 plus zero control. This is not native244 support or a
claim that the APK accepts reference245/246. Native244 has different fields/
domain discriminator and remains unported. No identity adoption, durable-row
migration, resend or live operation is added; old persisted identity ambiguity
and full native conversion require separate evidence.
