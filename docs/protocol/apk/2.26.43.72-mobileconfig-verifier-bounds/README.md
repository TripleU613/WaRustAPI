# MobileConfig structural verifier bounds

**Confirmed selected static evidence**, APK 2.26.43.72 code 264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reconstructed arm64 libmobileconfig-jni.so: 852440 bytes, SHA256
`5c50f623849463f4b71b662dbceabeef1a5f4dc36f56f0154c9048ef0d09255c`.
No native execution, mutation experiment or live account/network operation.

The [reader framing trace](../2.26.43.72-mobileconfig-reader-framing/README.md)
binds body verifier 0xa2ee4. This follow-up resolves its shared structural
helpers. It does not establish the complete schema, valid real fixture contents
or installation/activation semantics.

## Root and table bounds

Verifier state starts with body base and size, two 32-bit limit values loaded
from **0x1cd10: 64 and 1,000,000**, alignment byte +0x18 set to 1, and a vector
arithmetic cap +0x20 of **0x7fffffff**. Counter names and full reset/decrement
lifecycle are not established here; do not call them network limits or timeouts.

Helper **0x9b050** checks a four-byte span at a body-relative position. When
alignment-byte bit 0 is set it requires position divisible by 4. It requires
total size **strictly greater than 4**, then position <= size−4. These exact
conditions differ from simply checking position+4 <= size on a four-byte body.

Root-offset helper **0x9ac7c** first uses that span check, reads a 32-bit value
and compares it as signed against 1. Thus selected accepted deltas are positive
signed values, excluding zero and the high-bit-set range. It then requires the
relative field position plus delta <= size−1. It returns the delta or zero;
this alone does not verify a table at the resulting target.

Table helper **0x9acec** checks the table's four-byte offset word and computes
its vtable position by subtracting the sign-extended table word. It increments
two 32-bit counters and checks each new counter against the corresponding
initialized limit. The counter writes happen before the limit result, so a
false return is not evidence that verifier state remained unchanged. It checks
the vtable header span and alignment, requires an even vtable byte length, and
checks the complete vtable span against body size. Full table/object semantic
meaning is delegated to subsequent field checks.

## Fields, vectors and strings

Offset-field helper **0x9ad9c** looks up a 16-bit vtable entry. An absent entry
or zero field offset returns true; a present field must pass 0x9ac7c at its
relative location. This is an optional-field structural check, not proof that
the field exists. Scalar helpers **0x9af08** and **0x9af64** likewise accept
absent entries and otherwise check aligned spans of 8 and 4 bytes, respectively.
Their availability assumptions depend on prior table verification.

Vector helper **0x9afc0** takes a positive element stride and:

- verifies the count word through 0x9b050;
- requires count **strictly less** than floor(cap/stride);
- calculates needed bytes = 4 + count×stride;
- requires body size **strictly greater** than needed bytes and relative
  vector position <= body size−needed bytes.

The span may end exactly at the body end when vector position is nonzero,
despite the strict comparison against whole-body size. If an optional output
end-position pointer is supplied, it is written **before** the final span test;
callers must still use the boolean result. The script covers nonnegative
positions and positive strides only, not all machine overflow/pointer states.
Wrappers 0xa3cd0/0xa3ce8 use strides 4/8 and accept a null vector pointer.

String helper **0x9adfc** accepts null as optional. For nonnull strings it
checks the byte-vector span (stride 1), then requires an in-range **zero byte**
at the computed end. String-vector helper 0x9ae8c walks four-byte offsets and
applies the string helper to each target, returning false on a failed child.
This is structural termination checking, not UTF-8, content or wire validation.

The retained main verifier invokes these and other nested helpers across its
fields. Some field-specific nested table/vector helpers remain unresolved, so
the above does not amount to a full MobileConfig schema implementation.

## Required late scalar is distinct from optional checks

At **0xa34b8–0xa34ec**, the main verifier requires a nonnull body pointer,
resolves its root table, requires a vtable large enough for entry **0x1c**, and
requires that entry to be nonzero. It then reads the selected 32-bit scalar
and accepts that late condition only when it equals **0x1e240 (123456)**.
Unlike the optional helpers, this late check rejects absence. The scalar's
semantic name remains unknown, and it is not the same vtable entry as the
earlier schema-check helper's **0x22** field merely because both compare to
the same number. Other preceding checks still apply before a true return.

## Reproduction and limits

```
python3 docs/protocol/apk/2.26.43.72-mobileconfig-verifier-bounds/inspect-bounds.py ../android-lab/output/metrics-native/reconstructed-reproduction/libmobileconfig-jni.so > /tmp/mc-bounds.json
cmp /tmp/mc-bounds.json docs/protocol/apk/2.26.43.72-mobileconfig-verifier-bounds/bounds.json
```

The inspector pins member/ELF identity, limit literals and selected BL targets.
Boundary examples are analyst transcriptions, **not native tests** and not a
full table verifier. A passing root/vector example is only a selected-helper
projection. LLVM retains helpers, wrappers and selected main-verifier entry/end
windows; the previous bundle retains the whole main routine. Nearest JNI_OnLoad
headings are not local identities.

rea-bounds.json retains complete REA Evidence
`ev_af7a11e3507f3ba6da8f6f1467ceaeffe0d5ac6e7c2c38f8343ab829f4db7ce8`,
command `e scr.color=0; pd 36 @ 0x9afc0`, exit 0, untruncated output, unchanged
member hash and an entrypoint warning. This is a bounded vector-helper window;
provider version/external inputs are not bound and annotations are not runtime
memory observations.

**Remaining:** complete field/nested-table schema, counter lifecycle, valid
source fixtures, provider/rejection cleanup behavior, upgrade/installation,
durable active-state admission and retry/account ownership. No Rust runtime or
saved account state changed.
