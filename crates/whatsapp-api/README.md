# whatsapp-api

Standalone async Rust library intended to register and operate as a WhatsApp
**primary device**. See the repository `README.md` for the implementation plan
and `../../STATUS.md` for what is actually working.

## Status: experimental, offline gates only

Nothing in this crate has exchanged a message with a WhatsApp server. The
Android primary-device registration, login payload and transport endpoint are
`Unknown` in the evidence ledger. Every protocol-facing operation returns a
typed `Error::Unsupported` or `Error::ProfileIncomplete` naming the missing
facts. What does work, and is tested offline:

- typed domain model, identifiers and public API contract;
- encrypted transactional store (`redb`, XChaCha20-Poly1305 envelopes,
  authenticated record identity, single-writer ownership, generation fencing);
- durable event journal with consumer cursors and at-least-once replay;
- durable outbox with idempotent enqueue and honest send tickets;
- bounded binary-node codec driven by a `ReferenceOnly` token dictionary;
- cancellation-safe frame reader, experimental Noise XX adapter (feature
  `experimental-noise`), certificate-chain binding over an injected verifier;
- durable registration state machine with bound challenge handles, atomic
  prerequisite preparation, cooldown resumption, outcome reconciliation and
  atomic successful credential commit;
- a conservative, injectable comparison-build HTTP registration flow that
  covers the observed `/v2/code` → `/v2/register` happy path in offline tests
  without enabling it for the incomplete active profile.

## Usage sketch

```rust,no_run
use std::sync::Arc;
use whatsapp_api::model::ids::AccountId;
use whatsapp_api::store::EnvKeySource;
use whatsapp_api::WhatsAppClient;

# async fn demo() -> whatsapp_api::Result<()> {
let client = WhatsAppClient::builder()
    .store_path("protocol.redb")
    .account_id(AccountId::new("main")?)
    .key_source(Arc::new(EnvKeySource::new("WHATSAPP_API_MASTER_KEY")))
    .build()
    .await?;
println!("{:?}", client.capabilities());
# Ok(()) }
```

## Features

- `experimental-noise`: compiles the `snow`-based Noise adapter. `snow` is
  used without its `std` feature because that feature pulls in `ring`.
- `experimental-registration-https`: compiles the constrained, lab-only
  reqwest/rustls registration transport with the alpha pure-Rust
  `rustls-rustcrypto` provider. Normal certificate/hostname validation stays
  enabled; this is not production grade.

## Licence

MIT OR Apache-2.0 for first-party code. Third-party inputs and their licences
are recorded in `fixtures/` and `../../docs/protocol/`.
