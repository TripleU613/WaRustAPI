# Unknowns and blockers (G1/G2)

Ordered by what blocks the first decisive result (docs/design/README.md section 0).

Own-device ordinary text edits are now projected with loaded parent kind,
terminal and sender-time ordering checks. Peer edits still require the effective
native integer property 2983; a baked default or guessed 15-minute window is not
proof. Media/caption, gated poll/event, future-proof wrapper, orphan and expiry
lifecycles remain open. See [the receive evidence](message-edit.md).

1. **Android registration protocol** (`registration.*`): payload encoding,
   URL assembly, required prerequisites and their generation timing,
   verification steps, cooldown semantics, returned credential material.
   *Progress 2026-09-17:* host `v.whatsapp.net`, path fragments and the
   parameter vocabulary are client-observed (ledger `registration.host`,
   `registration.vocabulary`). A separately pinned newer build (2.26.37.70)
   statically confirms the full base/path assembly, URL-encoded form body and
   JSON response vocabulary, but the exact 2.26.36.75 artifact and a live
   capture are still needed before changing the active profile.
   `RegistrationService::begin` still returns `Blocked(ProfileIncomplete)`
   because response meanings and the complete flow are unobserved.
   The comparison build's common identifier fields and public key-bundle byte
   encodings are now mapped and implemented offline. The opt-in
   [persisted lab factory](registration-lab.md) generates keys and signs using
   the upstream Signal convention; accepted ID ranges, Android signing
   semantics and mandatory-field timing remain unobserved. Default registration
   is still blocked; mock success is not evidence of server acceptance.
2. **Transport endpoint and layer** (`transport.endpoint`): port, TCP vs
   WebSocket, whether TLS wraps Noise on Android. *Progress:* hosts
   `g.whatsapp.net`, `g-fallback`, `e1`–`e16.whatsapp.net` are client-observed.
   The 2.26.37.70 comparison statically selects 443/5222 with a port-80
   fallback. Its built-in Java path is not WebSocket: primary/fallback
   gateways use raw TCP on 443/5222, while numbered edges use TLS on 443 and
   raw TCP on 5222. Port 80 wraps raw TCP with bounded HTTP/1.1 chunked streams
   using `POST /chat` and host `c.whatsapp.net`. Active remote configuration
   and pinned-profile equivalence remain open.
2b. **Handshake pattern negotiation** (`noise.client_patterns`): the client
   contains classic XX/IK **and nine ML-KEM512 hybrid** Noise patterns. If the
   server requires a hybrid pattern for new primary registrations, `snow`
   cannot be the backend (no such patterns) and a pure-Rust ML-KEM512
   implementation would need review. The newer comparison's selector map is
   known and its compiled defaults select classic mode (`NONE`, variant
   `IKKEM`), but its remotely configured effective mode/variant must be
   observed at runtime.
3. **Login payload** (`login.client_payload`): primary-device markers,
   version fields, startup requests, resume behaviour.
4. **Noise record sizes** (`noise.record_limit`) to accept or reject `snow`.
   The 2.26.37.70 comparison frame writer uses a three-byte big-endian
   length and rejects bodies at or above 16,777,216 bytes. This framing
   bound is much larger than `snow`'s 65,535-byte limit; it does not establish
   the largest record emitted or accepted by the server.
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

## Comparison token dictionary: double-byte page 0 index 1

Observed 2026-09-20 on a block-list fetch reply: an `item` attribute name
decoded as `B3b` where the official parser (`LX/8nc.A02`, 2.26.37.71) reads
`active`, and the web-md-v3 fixture places `active` at double-byte page 0
index 1. The comparison dictionary (`TokenDictionary::comparison_2_26_37_70`)
therefore has at least one wrong double-byte entry; encoding still works
because unknown names fall back to raw strings. Re-derive the double-byte
pages from the APK and add a decode/encode round-trip test for every entry.
