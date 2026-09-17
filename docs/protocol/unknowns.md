# Unknowns and blockers (G1/G2)

Ordered by what blocks the first decisive result (README section 0).

1. **Android registration protocol** (`registration.*`): endpoints, payload
   encoding, required prerequisites and their generation timing, verification
   steps, cooldown semantics, returned credential material. Without this the
   library cannot reach `Registered`; `RegistrationService::begin` returns
   `Blocked(ProfileIncomplete)`.
2. **Transport endpoint and layer** (`transport.endpoint`): host, port, TCP
   vs WebSocket, whether TLS wraps Noise on Android.
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
