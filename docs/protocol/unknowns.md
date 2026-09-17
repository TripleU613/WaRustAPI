# Unknowns and blockers (G1/G2)

Ordered by what blocks the first decisive result (README section 0).

1. **Android registration protocol** (`registration.*`): payload encoding,
   URL assembly, required prerequisites and their generation timing,
   verification steps, cooldown semantics, returned credential material.
   *Progress 2026-09-17:* host `v.whatsapp.net`, path fragments and the
   parameter vocabulary are client-observed (ledger `registration.host`,
   `registration.vocabulary`). A separately pinned newer build (2.26.37.70)
   statically confirms the full base/path assembly and URL-encoded form body,
   but the exact 2.26.36.75 artifact and a live capture are still needed
   before changing the active profile. `RegistrationService::begin` still
   returns `Blocked(ProfileIncomplete)` because the complete flow is
   unobserved.
2. **Transport endpoint and layer** (`transport.endpoint`): port, TCP vs
   WebSocket, whether TLS wraps Noise on Android. *Progress:* hosts
   `g.whatsapp.net`, `g-fallback`, `e1`–`e16.whatsapp.net` are client-observed.
   The 2.26.37.70 comparison statically selects 443/5222 with a port-80
   fallback, but the active profile and transport layering remain open.
2b. **Handshake pattern negotiation** (`noise.client_patterns`): the client
   contains classic XX/IK **and nine ML-KEM512 hybrid** Noise patterns. If the
   server requires a hybrid pattern for new primary registrations, `snow`
   cannot be the backend (no such patterns) and a pure-Rust ML-KEM512
   implementation would need review. The newer comparison's selector map is
   known, but its remotely configured PQ mode/variant must be observed at
   runtime.
3. **Login payload** (`login.client_payload`): primary-device markers,
   version fields, startup requests, resume behaviour.
4. **Noise record sizes** (`noise.record_limit`) to accept or reject `snow`.
5. ~~**Android token dictionary**~~ *Resolved 2026-09-17 for content and
   order*: `xtask apk-tokens` recovers the ordered dictionary from the APK
   bytecode and it is identical to the web v3 list. Still open: the
   dictionary **version byte** (`binary.tokens_version`) and the structural
   tag bytes remain `ReferenceOnly` for the pinned profile. Version byte 3 is
   statically observed only in the separate 2.26.37.70 comparison.
6. **Signal backend** (ADR 0005): licence decision or clean-room plan.
7. **TLS provider** (ADR 0003) for registration/media HTTPS.
8. **Wire message-ID format** (`messaging.id_format`).

## How to resolve

Evidence must come from the pinned APK (static analysis in the lab) and from
dated captures with a controlled test number. Do not copy web-profile
constants into `Observed` rows. Record each observation with date, APK
version, and whether the server accepted the reference client that day.
