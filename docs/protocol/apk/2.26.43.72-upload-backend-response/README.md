# Concrete upload backend response and retry boundaries

**Confirmed static**, pinned WhatsApp2.26.43.72/code264307238, APK SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`,
rehashed in this audit. Baseline `9eebb0b5`. Offline xtask DEX fallback; no
REA-provider/runtime output or live operation. The prior Android provider failure
is [retained](../2.26.43.72-retry-key-envelope-bytes/rea-provider-attempt.json).
This follows [transfer classification](../2.26.43.72-upload-result-classification/README.md).

## Reuse known backends rather than treating abstract dispatch as absence

The [base constructor caller scan](base-constructor-callers.txt) finds JAS and
JAT constructors. Existing [15Q selector and complete backend evidence](../../rea/2.26.43.72-media-upload-http/README.md)
already retain both implementations. Their effective selection for an actual
request remains separate. The previous transfer audit's abstract A04 boundary
was scoped to that trace, not proof that concrete code was unavailable.

In complete [JAS.A04](../../rea/2.26.43.72-media-upload-http/urlconnection-execute.txt),
03d0 invokes HttpURLConnection.getResponseCode into v2. That code is returned at
0492/04f2 without a new success-only remapping. For code>=400, an optional error
stream is read with0Q4.A02 limit0x100000 and passed to KjC.BvX; normal sub400
input is read with the same limit and passed to KjC.CDR with response headers.
Status/header marker precedes body handling. Callback execution/body-read failure
can throw before the integer returns. Limit semantics beyond the shown helper
argument, durable storage and worker retries are not inferred.

In complete [JAT.A04](../../rea/2.26.43.72-media-upload-http/tigon-execute.txt),
TigonResult.error is checked before body processing. A null body becomes an
empty stream. The selected body wrapper is read with0Q4.A02 limit0x100000;
nonnull TigonResult.response then supplies1gK.A00 as the returned code at04aa.
Code>=400 invokes BvX; sub400 invokes CDR with reconstructed headers. This
confirms the concrete source/read of the integer passed through K2k.A03 to
MtZ, while native Tigon's upstream response/status construction remains separate.
The [single direct field writer](tigon-status-writers.txt) is1gK's constructor;
this census alone does not authenticate the status or resolve every native path.

Both selected backends can therefore hand sub400 responses to the JSON callback;
that callback is not sufficient to prove reusable media. Transfer modes,
missing-path configuration and downstream adoption from the preceding audit
still apply. Rust's existing exact200 gate is not widened by this evidence.

## Explicit request retry flag; unresolved redirect/platform policy

JAT.A04 sets TigonRequestBuilder.retryable=false at017a, using v2=0 from00f0
without an intervening reassignment on the selected path. It also sets connection
15000ms, idle60000ms and the request deadline from JAT.A01. The
[builder constructors](tigon-builder-constructors.txt) retain distinct retryable,
replaySafe, connection/idle/request/response timeout fields. Kotlin default-mask
semantics, replaySafe's effective value and downstream enforcement are not newly
resolved; the explicit JAT retryable write does not need a default-value guess.
No universal exactly-one-network-attempt guarantee follows from a field setting.

JAS obtains its connection through [12F.A03](urlconnection-factory.txt), which
resolves0jV then calls [0jV.A03](urlconnection-open.txt). That method selects
URL.openConnection() when its supplied proxy equals NO_PROXY, otherwise calls
openConnection(proxy) and applies a separate helper. The no-argument branch does
not explicitly pass NO_PROXY. Effective proxy selection/authentication and
platform defaults remain untraced. There is no explicit redirect setter in these
selected opening methods or JAS.A04; this bounded negative result is not proof
that redirects are disabled. JAT builder constructors likewise do not establish
all native redirect policy. Do not silently copy platform assumptions into Rust.

Rust's three explicit reqwest builders retain their local no-redirect/no-retry/
no-proxy policy documented in [HTTP boundaries](../../http-client-boundaries.md).
No setting is changed, no header/proxy identity is invented and no replay added.
Remaining work includes effective selector/configuration, Tigon enforcement,
platform redirects/proxies, complete upstream status provenance and durable
result/retry lifecycle. No server acceptance or zero-restriction guarantee.

Reproduce new exports and byte-check the two reused concrete backend methods:

```sh
python3 docs/protocol/apk/2.26.43.72-upload-backend-response/reproduce.py ../android-lab/apks/WhatsApp.1.apk
```

All new files are pinned by checksums.json. Complete reused methods are not
copied again; the script validates the APK hash before any selector call.

Validation: all five new exports and both reused backend methods reproduced
byte-for-byte. APK coverage, formatting, regenerated CLI reference, OpenAPI
library-symbol test, whitespace and staged repository hygiene passed. The full
runtime workspace suite was not rerun for this research-only chunk.
