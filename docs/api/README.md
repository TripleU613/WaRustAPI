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
