# Photo error callback selection and lookup-empty outcome

**Confirmed selected static**, APK2.26.43.72 build264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Focused complete DEX methods, no new REA Evidence record or runtime execution.
This follows [request/success/timer evidence](../2.26.43.72-profile-photo-callbacks/README.md).

Hyd.Bvj passes 1ud.A00(response) to ITY.onError(int). 6p1.Bvj calls
6oj.A05, which delegates to the same 1ud.A00. Reused complete
[1ud.A00](../2.26.43.72-status-privacy-fetch-callback/error-code.txt) and
[1ud.A01](../2.26.43.72-prekey-upload-replies/error-parser.txt) show direct
error children in order, skipping children lacking code. The first present
code wins, including invalid/empty code ->0; it does not skip a malformed code
for a later valid one. Nested user errors do not supply this callback code.
Reused [0HF.A00](../2.26.43.72-prekey-upload-replies/error-number-parser.txt)
checks TextUtils.isEmpty, calls Integer.parseInt and catches NumberFormatException
to the supplied default0. The [blocklist follow-up](../2.26.43.72-blocklist-reply-dispatch/README.md)
retains its byte identity and existing host Java reference corpus. Signed32,
Java decimal parsing/default0 differs from the generic u16 diagnostic.

6p1.Bvj compares code404 (0x194) and410 (0x19a). These two codes construct
H5d with the saved target, null string/URL/byte-array metadata, id=-1 and the
preview-dependent selector, and call8pX.CAe with the saved long. Other codes
call8pX.CAd with saved target/type, code and long. Thus these are empty-metadata
callback outcomes, not permission to retry and not proof every error means
empty photo. Hyd upload has no corresponding404/410 success-like branch.
The call signature binds the empty arguments; downstream H5d/cache application,
8pX/ITY implementations and the saved-long meaning remain untraced here.

## Implemented correction and boundary

The generic action recorder previously searched nested user errors and then
selected the first error even if uncoded. A red regression confirmed nested401
could override the actual direct404. For matching saved photo set/image/bytes
or get/picture/image/query=url requests, outer errors now use the first direct
coded child, preserving the existing blocklist rule independently. A kind label
alone cannot claim photo policy. Result admission, source/callback ownership,
request construction, timer and retry behavior are unchanged.

Optional durable ActionOutcome.photo_error carries signed code and
empty_lookup; it is serialized and shown by actions. empty_lookup=true means
only a supported lookup whose native code is404/410. The generic action status
remains Error for outer IQ/error and generic error_code remains u16 compatibility
data. This metadata records callback classification, not native cache changes,
photo publication, absence of a remotely stored photo, or server-side success.
Legacy absent metadata remains unknown; no backfill or historical rewrite.

Tests cover upload versus lookup, direct/nested/uncoded competing errors,
first invalid code before valid410, missing/empty/overflow/signed/Unicode codes,
legacy optional serde compatibility, duplicate settlement, encrypted-store
reopen and no resend. No live account operation or inferred ban trigger.

## Reproduce

From repository root with built xtask:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
OUT=/tmp/wa-photo-error-reproduce
mkdir -p "$OUT"
target/debug/xtask apk-smali "$APK" --class 'LX/Hyd;' --method Bvj --signatures --exceptions > "$OUT/set-error.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/6p1;' --method Bvj --signatures --exceptions > "$OUT/get-error.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/6oj;' --method A05 --signatures --exceptions > "$OUT/lookup-code-wrapper.txt"
```

Compare these three files exactly; checksums.json covers every retained file
except itself. Shared helper/host-reference exports stay under their existing
identities. No APK, image, bulk tree or private state is committed.

Verification:1536 workspace tests across nine suites (including doc tests),
strict workspace/all-target/all-feature Clippy, fmt, OpenAPI library-symbol and
CLI-reference tests, APK coverage freshness and research hygiene passed.
Focused regressions: `cargo test --locked -p whatsapp-api --all-features photo_error_`.

## Lookup result follow-up (2026-11-06)

[Named-child and constructor trace](../2.26.43.72-photo-lookup-result/README.md)
closes the lookup result/error-extension mismatch. The saved callback long is
now bound to constructor SystemClock.elapsedRealtime(); downstream uses remain
unknown. Full payload validation and native cache adoption remain unported.
