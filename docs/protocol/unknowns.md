# Unknowns and blockers (G1/G2)

Ordered by what blocks the first decisive result (README section 0).

1. **Android registration protocol** (`registration.*`): payload encoding,
   URL assembly, required prerequisites and their generation timing,
   verification steps, cooldown semantics, returned credential material.
   *Progress 2026-09-17:* host `v.whatsapp.net`, path fragments and the
   parameter vocabulary are client-observed (ledger `registration.host`,
   `registration.vocabulary`). `RegistrationService::begin` still returns
   `Blocked(ProfileIncomplete)` because encoding and flow are unobserved.
2. **Transport endpoint and layer** (`transport.endpoint`): port, TCP vs
   WebSocket, whether TLS wraps Noise on Android. *Progress:* hosts
   `g.whatsapp.net`, `g-fallback`, `e1`–`e16.whatsapp.net` are client-observed.
2b. **Handshake pattern negotiation** (`noise.client_patterns`): the client
   contains classic XX/IK **and nine ML-KEM512 hybrid** Noise patterns. If the
   server requires a hybrid pattern for new primary registrations, `snow`
   cannot be the backend (no such patterns) and a pure-Rust ML-KEM512
   implementation would need review. Must be observed on the wire.
3. **Login payload** (`login.client_payload`): primary-device markers,
   version fields, startup requests, resume behaviour.
4. **Noise record sizes** (`noise.record_limit`) to accept or reject `snow`.
5. **Android token dictionary**: the vendored dictionary is the web v3 list.
   The APK must be inspected (lab, offline) to extract the Android dictionary
   and version byte; `xtask` should gain an extractor once the location in
   the APK is established.
6. **Signal backend** (ADR 0005): licence decision or clean-room plan.
7. **TLS provider** (ADR 0003) for registration/media HTTPS.
8. **Wire message-ID format** (`messaging.id_format`).

## How to resolve

Evidence must come from the pinned APK (static analysis in the lab) and from
dated captures with a controlled test number. Do not copy web-profile
constants into `Observed` rows. Record each observation with date, APK
version, and whether the server accepted the reference client that day.
