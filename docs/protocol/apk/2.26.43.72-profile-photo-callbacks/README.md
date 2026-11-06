# Profile-photo requests and callback boundaries

**Confirmed static**, APK2.26.43.72 code264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Checked DEX disassembly, not runtime observation or REA Evidence envelopes.
Only selected own-photo upload and ordinary image lookup paths are interpreted.

## Addressing is confirmed through the helper, not inferred

Hyd.A00 creates an iq builder throughB1x.A0c, adds its ID and callsB22.A0F
with xmlns/w:profile:picture. A0F adds that attribute to the builder and returns
a new to attribute bound to0mL.A00. Hyd immediately appends that returned
attribute. 0mL.getServer returns s.whatsapp.net. A0F's return value is therefore
part of construction, not an unused namespace result. The older Rust comment
that the native setter omits to is refuted for this build. No wire correction
is necessary: the existing Rust to=s.whatsapp.net is correct.

Hyd adds type=set; target is conditional on its first flag being false. The
ordinary own-photo branch has id/xmlns/to/type, one picture/type=image child,
and supplied bytes. reupload is independently conditional; account-center
credential/encryption_metadata branches are separate and not reconstructed here.
0Tj.A01 copies the accumulated attributes/children/data into the final node.

6p1.A00 explicitly builds five outer attributes: id,xmlns,to,target,type=get.
Its image query adds query=url and type=image to picture; optional cached id,
preview size, common_gid attribute and nested tctoken have separate conditions.
The common_gid value is an attribute of picture, not another child. DBv target
normalization and bot/preview/configuration policies remain unported/unresolved.
Rust's current ordinary URL lookup preserves its existing normalized target.

## Callback timer and dispatch wrappers differ

Hyd.A00 passes duration0 and logical type25 (0x19) to0Sf.A0W (00e6..00f4).
6p1.A00 passes duration0/type26 (0x1a) to0Sf.A0R (0194..01a2).
[Shared registration/admission evidence](../2.26.43.72-abprops-callback-timer/README.md)
retainsA0W/A0A/A0R and0ln.A08: positive duration is required to post message4.
Neither selected photo producer installs that timer. A0R has an explicit
pending fallback distinct fromA0W's ready admission; this is not proof of
identical offline/reconnect policy or infinite callback lifetime. Both selected
BsS methods return immediately. No retry is scheduled inside those methods;
other callers/jobs and native lifecycle policy remain separate.

## Upload success is not a generic nested-refusal parser

Hyd.A00 setsA00 using2L9.A1X, whose complete method returns true only for
null input. For a nonnull upload, Hyd.CK4 takes the first response child and
requires picture via0Tp.A01/A02. Missing/wrong tag throwsLIL. It then reads
nullable id and has_staging (exact true), and invokesITY.CJw. Subsequent
response children are not inspected by this callback. Null-input removal
skips that picture parse and supplies null/false toCJw; its full producer/
state policy is not ported by Rust's nonempty JPEG-only queue.

**Confirmed mismatch / implemented offline:** typed profile-picture-set with
matching saved set/image/byte-content request and a result whose first child
is picture must not become a refusal because a later error extension exists.
Rust previously applied its generic nested-error search to that result. It now
ignores such error extensions only for this supported upload-success shape.
Outer error still recordsError/code. Missing/wrong first child, lookup and
removal retain separate behavior; this is not full photo reply admission.
The generic action row is an envelope outcome, not proof of native cache,
photo publication, image validity or complete native parser acceptance.

6p1.CK4 has a different parser: it finds picture by name, optionally constructs
java.net.URL, parses selected/cached id as signed int (missing becomes-1),
constructsH5d and calls8pX.CAe. URL/ID parse failures becomeLIL; missing picture
uses null metadata and the cached ID. These complete method bodies are retained,
but no Rust URL/cache/default/lookup-result adoption is claimed. Downstream
ITY/8pX effects and Bvj error callbacks remain open. Do not generalize About's
payload-ignoring callback or the upload rule to this lookup parser.

## Verification and reproduction

Existing builders now assert exact own-upload4/lookup5 outer attribute counts
and server address. Paused-clock test queues actual upload and lookup, confirms
no invented deadline at60s, tests valid result/outer error and duplicate claims,
and verifies upload picture-plus-error extension remainsOk without refusal code.
A mislabeled ping does not inherit this upload exception. Normal-driver test
covers the valid first-picture/result plus synthetic error403 extension,
subsequent receipt ACK, no resend and clean stop. The extension is a synthetic
fixture, not an observed server response. The earlier untraced640x640/square directive is removed from CLI/API guidance;
request bytes and JPEG validation bounds are unchanged. Native image cropping/
resize/JPEG parameters, reply/cache policy and live interoperability remain unfinished.

Use `target/debug/xtask apk-xref APK --anchor 'w:profile:picture'`, then
`apk-smali APK --class CLASS --method METHOD --signatures --exceptions`:
Hyd.A00/CK4/BsS (set/set-result/set-failure),6p1.A00/CK4/BsS
(get/get-result/get-failure),B1x.A0c (iq-builder),B22.A0F (namespace-helper),
0Tj.A01 (node-finalize),0Tp.A01/A02 (require-tag/tag-match),2L9.A1X
(removal-flag),0mL full class (server-jid). Exact SetProfilePhoto/GetProfilePhoto
log searches did not locate these paths; the namespace anchor did. Full methods
retain alternate branches without claiming coverage of all photo operations.
checksums.json covers every other file. No APK/media/bulk tree is committed.

## Error callback follow-up (2026-11-06)

The previously open Bvj callbacks are now retained in the
[error callback bundle](../2.26.43.72-photo-error-callbacks/README.md).
Both select1ud.A00; lookup404/410 routes to empty metadata while upload still
calls onError. Rust now preserves this diagnostic and correct direct-code
selection; downstream ITY/8pX effects remain unported/untraced.

## Lookup result follow-up (2026-11-06)

[Named-child and constructor trace](../2.26.43.72-photo-lookup-result/README.md)
closes the lookup result/error-extension mismatch. The saved callback long is
now bound to constructor SystemClock.elapsedRealtime(); downstream uses remain
unknown. Full payload validation and native cache adoption remain unported.
