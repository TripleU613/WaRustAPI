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

## Tags

Operations are organized by tag. The full list:

- Service
- Accounts
- Store
- Registration
- Registration Lab
- Session
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
`docs/api/openapi.json` in the same change.** The CLI coverage test
(`whatsapp-cli`) enforces that every command named there exists and every
command in the CLI is named there. The document is the single source of truth
for the API surface and is edited directly (it is JSON, so the CLI can embed
and serve it without extra dependencies).

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
