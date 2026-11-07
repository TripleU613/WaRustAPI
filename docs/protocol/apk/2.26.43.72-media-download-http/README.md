# Shared media download HTTP boundary — APK 2.26.43.72

**Confirmed selected static / research only**, build264307238, APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Checked complete DEX exports via repository xtask fallback; no APK execution,
new REA Evidence IDs, live requests or Rust runtime change. DEX PCs below are
byte offsets within a method; branch displacements are code units (multiply by2).

## Construction and header conditions

[Shared method](shared-download-method.txt) is `0lO.A01(M3d,K1R,URL,long,long)`.
It contains BOTH Tigon and HttpURLConnection paths, despite the latter's marker
name `MediaDownloadConnection`. Do not label the whole method a single stack.
Tigon GET construction is at0368..0376. Its connection/idle timeouts are15000/
30000ms at0376..0386; request timeout separately comes from Jwl.A00. The builder
sets retryable=false at0390 (v12 initialized0 at0078). This is transport retry
configuration, not proof of no higher-level rescheduling.

Both branches explicitly set `Accept-Encoding: identity` (03a2..03b0 and
093a..0948). This confirms this selected leaf agrees with Rust's identity header;
it does not establish every downloader or wrapper's final wire headers.
HttpURLConnection sets connect15000/read30000 at0918..092c and User-Agent from
0lR.A02 (0220..0232,092c..093a). Tigon does not add a User-Agent in this selected
builder block; stack/global interceptor defaults remain unknown. Rust's configured
UA and total timeout must not be called identical to either native timeout model.

`Range` is omitted only when start==0 AND end==-1; otherwise constructed as
`bytes=<start>-` with end appended when end!=-1. Both branches show the same
condition/format at03b0..041e and0968..09d6. Thus a range header is not universally
mandatory. Rust currently requests a full ciphertext with no Range; caller/input
binding remains necessary before calling this native full-transfer parity.

Selected conditional headers: X-FB-Socket-Option=TCP_CONGESTION=bbr uses a config
boolean; X-FB-nc-oc requires nonempty M3d.A06; X-FB-Pad requires nonnull0mQ.A00.
HttpURLConnection's X-FB-Request-Analytics-Tags requires nonnull analytics input.
Host is explicitly set, with route/original URL/socket selection controlled by
earlier branches. These are not evidence to fabricate telemetry, route overrides,
companion secrets or other operation metadata in Rust. Complete producer/value
and Tigon property/interceptor parity remain unknown.

## Response acceptance is wider than Rust's current200-only rule

At0682..0696 the Tigon path accepts200 or206 before its error path; its no-error
and nonnull-response prerequisites are at0672..0682. The HttpURLConnection path
accepts200/206 at0a5a..0a6c. Its status precedes response_headers_received marker
at0a1e..0a30; this is marker ordering, not proof of durable native persistence.
Both paths specially inspect416 Content-Range containing `*/`; the selected
method can return that response to higher callers instead of always throwing.
A416 is not thereby a successful full-file download.

Rust currently persists every non200 as server_refused, including206/416. This
is a **confirmed leaf-policy difference**, not yet a confirmed supported-operation
bug: native callers handle resumable offsets, ranges and policy/length state.
Do not simply accept all2xx, drop ciphertext authentication, retry416, or add a
resume request from this leaf. Full response-header/range semantics must first be
bound to the supported encrypted-image and ordinary-media transfer profile.

## Exact DEX callers and bounded exclusions

[Caller census](download-callers.txt) finds FOUR direct DEX callers of0lO.A01:
MuY.A03, MuZ.A03, MVv.A04 and N41.Cb6. No reflection/JNI completeness is claimed.
[MuY](caller-muy.txt) and [MuZ](caller-muz.txt) forward their two long arguments
and retain X-WA-Metadata/connection metadata; these wrappers do not establish
zero/-1 input values. [MVv](caller-mvv.txt) clamps its selected start to0 when
nonpositive and chooses either a bounded end or-1 at024a..028a. It has a distinct
416 handling branch at02f0 onward. [N41](caller-n41.txt) is explicitly marked
PlainDownloadTransfer; its200/206 length policy at0248..029e and subsequent
Content-Range parsing show why leaf status acceptance is not whole-transfer
success. Full caller upstream selection and crypto/result consumers remain open.

[Exact string census](header-get-xrefs.txt) finds11 Accept-Encoding literals
and73 GET literals across DEX. These counts are search boundaries, not complete
HTTP request counts. [JAU export](tigon-download.txt) was inspected because it
contains both literals, but NO call chain from0lO or the supported encrypted
operation to JAU was established. It is retained as a bounded candidate, not
asserted to be the selected download implementation. Other anchor hits remain
unclassified; no percent-complete or mandatory-header claim follows.

## Reproduction and integrity

Run `target/debug/xtask apk-xref <apk> --anchor Accept-Encoding --anchor GET`;
`apk-callers <apk> --class 'LX/0lO;' --method A01`; and `apk-smali <apk>
--class '<exact descriptor>' --method <method> --signatures --exceptions` for
0lO.A01, MuY.A03, MuZ.A03, MVv.A04, N41.Cb6. JAU was exported without a method
filter. [checksums.json](checksums.json) pins all raw text bytes and SHA256.
