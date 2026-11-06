# Photo lookup result dispatch and named-child selection

**Confirmed selected static / implemented offline**, APK2.26.43.72
build264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Focused DEX exports; no new REA Evidence records or runtime observation.

Reuse complete [6p1.CK4](../2.26.43.72-profile-photo-callbacks/get-result.txt)
and [6p1.A00](../2.26.43.72-profile-photo-callbacks/get.txt).
[0ln dispatch](../../authkey-outcomes.md#resulterror-dispatch) selects CK4 for
outer result and Bvj for outer error. CK4 calls0Tp.A0F(picture), not first-child
validation: the new complete A0F export confirms first matching direct child
in order, ignoring unrelated names and later duplicates. There is no root or
nested error-extension scan in this success callback.

With picture, CK4 reads its body/direct_path/hash and optional URL, then picture
id. A malformed java.net.URL or Integer.parseInt throws LIL. Without picture it
uses null metadata and the saved A04 ID. A null selected ID becomes-1; otherwise
it is parsed as signed32. Picture present but ID absent does not fall back to
A04. CAe receives the constructed H5d and saved A06 long. Do not infer full
Android URL-handler acceptance or native cache state from these branches.

The new constructor export binds the last String argument to A04; A00 uses that
field as the optional outgoing picture/id. Thus the no-picture branch retains
the request's cached-ID input rather than reading an unrelated extension.
Constructor A06 comes from SystemClock.elapsedRealtime(). It is the constructor's
clock sample forwarded to the callbacks, not a callback timeout duration.
Downstream latency/cache policy and all external constructor callers remain
untraced. The preceding error trace's saved-long source is resolved here, not
an instruction to alter Rust timers.

## Correction

Rust's generic inner-error search could mark a matching photo lookup result as
Error because of unrelated direct403 or nested401 extensions. The pinned
callback does not route either extension to Bvj. Matching saved
profile-picture-get / w:profile:picture / get / target / picture-image-query-url
requests now retain the outer-result outcome, independently of those extensions.
The same request-shape helper is shared with the already implemented outer-error
classification; outer error still records signed code and lookup404/410 metadata.
A kind label on a ping cannot borrow this result rule.

This change corrects callback selection, not native payload admission. Ok records
a correlated result without recognized producer refusal, not proof URL/id
validation, photo/cache adoption or publication completed. Raw reply remains
saved for explicit inspection; malformed URL/id handling and downstream CAe
state application remain unported. The upload-specific first-picture exception,
app-state inner refusal parser, source/callback authority and timers are unchanged.
No new query, ACK, refresh, retry, replay or live account operation.

The regression failed with Error before correction. It now tests picture-first,
metadata-first, no-picture and two-picture responses with competing root/nested
errors through the live callback owner. It verifies duplicate settlement refusal,
saved original reply, no photo_error metadata, reopen and no resend. The existing
photo test now checks both upload and lookup kind-only negatives.

## Reproduce

From repository root with built xtask and pinned APK:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
OUT=/tmp/wa-photo-lookup-reproduce
mkdir -p "$OUT"
target/debug/xtask apk-smali "$APK" --class 'LX/0Tp;' --method A0F --signatures --exceptions > "$OUT/first-named-child.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/6p1;' --method '<init>' --signatures --exceptions > "$OUT/lookup-init.txt"
```

Compare both files exactly. checksums.json covers all retained files except
itself. Shared complete methods stay under their existing identities; no APK,
image, bulk tree or private state is committed.

Verification:1537 workspace tests across nine suites including doc tests, strict
workspace/all-target/all-feature Clippy, fmt, OpenAPI library-symbol and
CLI-reference tests, APK coverage freshness and research hygiene passed.
The APK SHA256 was rechecked; both new DEX exports reproduce byte-for-byte.
Focused regression: `cargo test --locked -p whatsapp-api --all-features photo_`.
