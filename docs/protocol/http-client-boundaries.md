# HTTP client and replay boundaries

2026-11-07. **Confirmed local source**, baseline `f7552102`. This is a bounded
inventory of explicit reqwest construction and its current call sites in
`crates/whatsapp-api/src`, not new APK evidence, live traffic or native parity.
No runtime behavior changes. Searches for reqwest, hyper, ureq, Client::builder,
Client::new and ClientBuilder found three explicit reqwest client builders;
the public WhatsApp ClientBuilder is the separate socket/runtime facade.

| Request flow | Builder and caller | Existing destination/admission boundary |
| --- | --- | --- |
| Registration POST | [wire.rs](../../crates/whatsapp-api/src/registration/wire.rs), ExperimentalHttpsTransport::build/send | Constructed registration URL must begin with the fixed https://v.whatsapp.net/ origin. with_settings validates central transport settings; new supplies bounded defaults. Builder validates whole-request timeout and supplied User-Agent. |
| Media upload POST | [image_upload.rs](../../crates/whatsapp-api/src/transport/image_upload.rs), client/upload_prepared | upload_source and upload_prepared validate the profile; registered matching account and owned upload reservation remain required. URL comes from approved route host and explicit upload-path construction. |
| Ordinary attachment GET | [media.rs](../../crates/whatsapp-api/src/transport/media.rs), open -> image_upload::client | Profile validation precedes work. media_url requires HTTPS/approved host and rejects credentials, explicit nondefault port and fragment. Account/source and durable download reservation checks remain separate. |
| View-once prefetch GET | [image_download.rs](../../crates/whatsapp-api/src/transport/image_download.rs), download_view_once -> image_upload::client | Profile validation and registered matching account precede candidate discovery; selected URL and owned download reservation are checked before send. This is ciphertext prefetch, not opening or consumption. |
| Appeal POST | [ban_appeal.rs](../../crates/whatsapp-api/src/transport/ban_appeal.rs), submit_ban_appeal -> build_client | Fixed https://acs.whatsapp.com/graphql destination. appeal_response_limit validates central transport settings before credential/client lookup. Token provenance/submission ownership are not proved by these settings. |

All three selected builders explicitly set HTTPS-only mode, no redirects,
retry::never, no proxy discovery, referer(false), and disable gzip/brotli/deflate/
zstd decompression. Each installs a rustls ring-provider configuration with
safe default protocol versions, WebPKI roots and no client authentication.
Whole-request and connect deadlines are supplied explicitly. Configured User-Agent
is applied when present; this is not proof it is required on every native flow.
Media POST/GET additionally set Accept-Encoding: identity at their call sites.
Registration and appeal set their content type and length explicitly. The
registration builder also bounds its idle pool to one per host.

**Negative finding:** no missing redirect/retry disable flag was found among
these selected builders. No code change is justified by that search. Disabling
reqwest retries does not prove exactly one on-wire packet, successful application,
or that an application worker never sends another request. Durable reservations,
response classification, socket reconnects and explicit operator actions have
their own state machines and tests. This inventory does not authenticate a
server response or authorize any live request.

## Remaining differences and questions

The three builder implementations duplicate TLS/HTTP settings; this audit does
not refactor them or assume they may safely share all admission/error policies.
Media builders rely on validated callers; registration and appeal validate at
their own documented boundary. Future callers need their own verification.

Native redirect/retry/deadline/connection-pool policy, effective HTTP version,
TLS extension/fingerprint behavior, mandatory request-specific headers, and full
server acceptance remain unproven by this Rust source inventory. Continue
per-request APK construction and response/order/persistence audits from
[network correctness](network-correctness-audit.md),
[media network outcomes](media-network-outcomes.md),
[registration lab](registration-lab.md) and [appeal evidence](ban-appeal.md).
Neither zero bans nor exhaustive coverage of all possible dependency/feature
transport paths follows from three text matches. The selected scope is the
explicit builders and their direct source callers inspected here.

Reproduce the bounded discovery from the repository root:

```sh
rg -n 'reqwest::Client::|Client::new|ClientBuilder|hyper::|ureq::' crates/whatsapp-api/src
rg -n 'image_upload::client\(|client\(config\)|build_client\(config\)' crates/whatsapp-api/src/transport
```

Then read each complete builder and caller; search output alone does not prove
the settings run before a request. No live endpoint or APK was contacted.

Validation: APK coverage, formatting, regenerated CLI reference, OpenAPI
library-symbol test, whitespace and staged repository hygiene passed. This
source-audit/documentation-only chunk did not rerun the full runtime workspace
suite; the preceding runtime verification remains separate.
