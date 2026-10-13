# API reference

The OpenAPI 3.1 document at `openapi.json` (this directory) describes every operation
of the library and CLI: registration, login, messaging, receipts, groups, device
management, contacts, events, media and more.

## Serving the document

Start the local HTTP control plane:

```sh
cargo run --locked -p whatsapp-cli -- serve --bind 127.0.0.1:8787
```

The document is served at `GET /openapi.json`:

```sh
curl http://127.0.0.1:8787/openapi.json
```

## Viewing the document

Use any OpenAPI 3.1 viewer. For example, with `@redocly/cli`:

```sh
npx @redocly/cli preview-docs docs/api/openapi.json
```

Or visit [redoc.ly](https://redoc.ly) in your browser and paste the URL.

For a human-readable CLI reference generated from this document, see [docs/cli.md](../cli.md).

## Fields

Each operation includes:

- **`x-cli`** — the exact CLI command to invoke this operation (e.g.,
  `register --code 123456`). Operations that exist only over HTTP do not have
  this field.
- **`x-transport`** — where the operation is currently served:
  - `cli`: invoked through the CLI today; HTTP route is the contract.
  - `http`: served by `whatsapp-cli serve` over HTTP today.
- **`x-live`** — boolean, true if the operation contacts WhatsApp servers and
  needs `--allow-live-request`.
- **`x-secret-input`** — boolean, true if a parameter contains sensitive data
  (OTP, phone, credential, token, key); the CLI redacts it and never logs it.
- **`x-long-running`** — boolean, true if the operation may block (e.g., a receive
  loop or a server wait).
- **`x-library`** — array of Rust module paths naming the library and CLI functions
  that implement this operation (e.g., `["whatsapp_api::accounts::*",
  "whatsapp_cli::accounts"]`). Checked by the test `openapi_library_symbols_exist`
  to ensure every documented operation has a corresponding implementation.

## Tags

Operations are organized by tag. The full list:

- Service
- Accounts
- Store
- Registration
- Registration Lab
- Session
- Token
- Messages
- Outbox
- Receipts
- Chat Actions
- Groups
- Contacts
- Devices
- Media
- Events
- Account Safety

## The rule

**Every new or changed CLI command, flag, library capability, HTTP route,
request/response field or server-response code is documented in
`docs/api/openapi.json` in the same change, with `x-cli`, `x-transport`,
`x-live` and `x-library` set honestly.** The CLI coverage test
(`whatsapp-cli`) enforces that every command named there exists and every
command in the CLI is named there. The test `openapi_library_symbols_exist`
enforces that `x-library` names the Rust entry point. The document is the
single source of truth for the API surface and is edited directly (it is JSON,
so the CLI can embed and serve it without extra dependencies).

## Store routes (opt-in)

`whatsapp-cli serve --store-routes --token-file <new path>` (built with
`experimental-transport`) adds read-only, redacted store inspections for the
account selected by `--store`/`--account`/`--key-env`: `GET /v1/receipts`,
`/v1/prekeys`, `/v1/outbox/suspended`, `/v1/actions` (list form only),
`/v1/identity-conflicts`, `/v1/outbox/group-preparations`,
`/v1/diagnostics/own-lid` and `/v1/outbox/{local_id}/versions` (malformed ID
400 before the store opens, unknown ID 404 `unknown_local_id`). These refuse any
query string with 400, so CLI-only options are never silently ignored.

Three counts-only group inspections take exactly one query parameter, as the
OpenAPI document names it: `GET /v1/groups/receipts?request_id=<id>`,
`/v1/groups/key-queries?request_id=<id>` and
`/v1/groups/refresh-status?id=<action id>`. The value must satisfy the
library's ID rule (ASCII alphanumerics and `-_.:`, at most 128 bytes) and is
not percent-decoded; a missing, renamed, repeated, empty, encoded or extra
parameter is 400 before the store opens. Nothing saved under the ID answers
200 `null`, as the CLI prints. A key-query record the summary cannot count (a
legacy outcome without a frozen plan, or a damaged row), and own-LID sources
that disagree or are malformed, answer 422 `saved_state_unsupported` with the
library's fixed reason (never an address); other read failures stay 500.

All store routes are declared with `x-transport: ["cli", "http"]` and the
`storeBearer` security scheme.

```sh
whatsapp-cli --store acct.redb --account me --key-env WA_KEY \
  serve --store-routes --token-file ./serve.token
curl -H "Authorization: Bearer $(cat ./serve.token)" http://127.0.0.1:8787/v1/prekeys
```

The server refuses non-loopback binds with store routes, writes a new 256-bit
token to a new file (mode 0600 on unix; an existing file or symlink is refused;
the token is never printed or logged), checks the store opens at startup, then
opens it per request. The file is removed on Ctrl-C, SIGTERM or SIGHUP; a
killed or crashed server can leave it behind, still unreadable to others but
useless, and that path is then refused until you delete it. Answers: 404
`store_routes_disabled` when not enabled, 401 with `WWW-Authenticate: Bearer`
for a missing or wrong token (the scheme name is case-insensitive), 400 for a
query string the route does not take, 405 for other methods, 503 `store_unavailable` only while another
command holds the store, 500 `store_read_failed` for any other read failure
(details only in the server log).

Caveats: "read-only" means no protocol data changes; each request still takes
the store's ownership lock briefly, so a command starting during a request
(for example `listen` or one phase of `outbox-send`) can fail to open the store
and must be rerun; nothing is sent in that case. Use `127.0.0.1` rather than
`localhost`, so a client does not try another address a different local user
could listen on. Outputs are the CLI's: `peer_ticket` is an unsalted SHA-256
of the address, so a phone number can be recovered by guessing; refused
actions include the server's error text. Treat responses as account data.
Before adding a route, confirm its output is no more revealing than these;
never serve send, queue or live operations this way.

## Adding an operation

1. Follow the existing path convention in the document (e.g.,
   `/v1/messages/send` for the send operation).
2. Set `operationId` to a terse, unique identifier.
3. Set `x-cli` to the exact command string (e.g., `send --to $JID $message`).
4. Set `x-transport` to `cli` or `http`.
5. Set `x-live` to true if it contacts WhatsApp; false otherwise.
6. Add parameters via `#/components/parameters/*` (reuse common ones like
   `accountId`).
7. Add responses via `#/components/responses/*` and reusable schemas in
   `#/components/schemas`.
8. Include descriptions on every property.
9. Run `cargo test -p whatsapp-cli` to verify coverage.

Offline development utilities outside `whatsapp-cli` are recorded in the
root `x-research-tools` extension with `x-cli`, `x-transport` and `x-live`.
These entries do not create HTTP routes or add commands to the control CLI.
The `apk-proto-fields` entry records its validated descriptor-reader capability.

Authenticated own-device ordinary text synchronization can emit the existing
`MessageUpdated` / `Edited` event with the parent's original key. Loaded text
kind, terminal and sender-time ordering checks apply. Peer/media/special/orphan
edit receive and edit sending remain unfinished; fresh live delivery is unverified.
See [native evidence](../protocol/message-edit.md).

Own-device ordinary image caption edits also emit `Edited`, preserving the
original sealed media reference. Empty or absent captions decline; edited keys/URLs cannot
redirect the attachment. Other media edits and fresh live delivery are unverified.
See [native clone behavior](../protocol/image-caption-edit.md).

Own ordinary video/GIF and document caption updates now use `Edited` with the
original media reference and document filename. The native one-way GIF/video
kind exception is enforced; empty captions decline for all three families.
Peer policy and fresh live interoperability remain unverified.
See [decoder context and type evidence](../protocol/media-caption-edit.md).

Selected `pqkey` directory replies are explicitly unsupported by the current
session backend and cannot silently become classic encryption bundles. Identity
diagnostics still validate classic metadata only; PQ preview/application defer.
Some recipients requiring fresh PQ sessions remain unsupported.
See [native processing and the backend gap](../protocol/pq-bundle-boundary.md).

Identity refresh inspection and session preview now expose nullable
`pq_signature` with signature-only status, value length, nullable verification
result and `key_validation="unported"`. This does not enable PQ session creation
or adoption. See [original-byte signature evidence](../protocol/pq-signature.md).

Experimental `signal::kem` provides ML-KEM-1024 primitive generation,
encapsulation and expanded-key decapsulation with redacted, zeroized secrets.
It does not enable hybrid sessions or adopt PQ bundles. See
[API, native vectors and limitations](../protocol/rust-kem.md).

`signal::kem::initial_hybrid_keys` derives the initial root/chain pair from
ordered, already-computed EC and KEM agreements. It does not create or adopt
a session; see [the evidenced hybrid KDF and remaining integration](../protocol/hybrid-kdf.md).

`signal::direct::incoming_prekey_kem` borrows paired KEM metadata from a bounded
prekey envelope with redacted Debug. All incoming prekey inspectors reject
incomplete pairs; KEM receive remains unsupported before saved-state changes.
See [wire fields, version bytes and remaining integration](../protocol/prekey-kem-wire.md).

Experimental `signal::wire::encode_signal` and `encode_prekey` emit the native
version byte/MAC and paired KEM fields from supplied encrypted bytes and keys.
They do not advance/adopt/send sessions; see
[API arguments, bounds and wire validation](../protocol/signal-wire.md).

Experimental `signal::hybrid` initializes staged Alice/Bob ML-KEM-1024 records
and prepares native encrypted messages with candidate records. It does not
authorize/adopt/store/send keys or consume prekeys; see
[API arguments, atomic commit contract and offline exchange](../protocol/hybrid-session.md).

Existing text and encoded `signal::direct` preparation now handles correctly
owned held version-4 sessions with native wire/MAC and pending KEM metadata.
An explicit matching peer pin and account/ratchet checks are required; supplied
bundle signature checks still run. See
[normal API/outbox integration and remaining PQ gates](../protocol/held-hybrid-send.md).
