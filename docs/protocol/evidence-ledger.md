# Evidence ledger

Labels (README 6.1): `Observed` · `Derived` · `ReferenceOnly` · `Unknown`.
Every important wire constant used by the code points here through a
`ledger_ref` string. **No `Observed` transport fact exists yet**: nothing has
been captured from the pinned Android client or a live server.

Reference client: `com.whatsapp` 2.26.36.75 (code 263607533), APK sha256
`ce746890…922e9`, signing cert sha256 `39:87:D0:43:…:5D:44`. Full record:
`crates/whatsapp-api/fixtures/reference-manifest.json`.

Reference implementations (web multi-device profile, **not** Android primary):
`tulir/whatsmeow` @ 2375e175 (MPL-2.0); `oxidezap/whatsapp-rust` @ 4a0721b (MIT
headline; see ADR 0005 for AGPL files).

| Ledger ref | Fact | Value in code | Label | Source / notes |
| --- | --- | --- | --- | --- |
| `framing.length_prefix` | Frame length prefix width | 3 bytes big-endian | ReferenceOnly | whatsmeow `socket/constants.go` `FrameLengthSize`; whatsapp-rust `framing.rs`. Web uses WebSocket; Android transport (direct TCP vs WS) is `Unknown` (`transport.endpoint`). |
| `framing.max_frame_len` | Maximum frame body | `1 << 24` | ReferenceOnly | whatsmeow `FrameMaxSize`. Local product limit defaults to 1 MiB (`Limits`). |
| `framing.connection_header` | One-time header before first frame | `WA 0x06 <dict_version>` | ReferenceOnly | whatsmeow `WAConnHeader`; whatsapp-rust `WA_CONN_HEADER`. |
| `binary.frame_flags` | Payload flag byte; bit 1 = zlib | `0x02` | ReferenceOnly | whatsmeow `binary/unpack.go`. |
| `binary.tags` | Structural tag bytes | LIST_EMPTY=0, DICT_0..3=236..239, INTEROP_JID=245, FB_JID=246, AD_JID=247, LIST_8=248, LIST_16=249, JID_PAIR=250, HEX_8=251, BINARY_8/20/32=252/253/254, NIBBLE_8=255 | ReferenceOnly | whatsmeow `binary/token/token.go`; whatsapp-rust `token.rs`. |
| `binary.tokens` | Token dictionary | v3, 236 single + 4×256 double | ReferenceOnly | `fixtures/tokens/web-md-v3.json` (from whatsapp-rust `tokens.json`, MIT; byte-identical to whatsmeow). Android app is known to use its own dictionary versions; **must be extracted from the pinned APK** before any live use. |
| `jid.servers` | Server names | `s.whatsapp.net`, `g.us`, `lid`, `broadcast`, `newsletter`, `c.us`, `hosted`, `hosted.lid`, `msgr`, `interop` | ReferenceOnly | whatsmeow `types/jid.go`. |
| `jid.ad_domain_types` | AD_JID first byte → server | 0→user, 1→lid, 128→hosted, 129→hosted.lid, else agent | ReferenceOnly | whatsmeow `NewADJID`. |
| `noise.pattern` | Handshake pattern | `Noise_XX_25519_AESGCM_SHA256` (IK resume also used by web) | ReferenceOnly | whatsmeow `NoiseStartPattern`; whatsapp-rust `consts.rs`. README 8.3: hypothesis until confirmed. |
| `noise.prologue` | Prologue | `WA 0x06 <dict_version>` | ReferenceOnly | Same bytes as the connection header in the references. |
| `noise.trust_root` | Root public key for cert chain | `1423 7557 4d0a … a8ee 6b` | ReferenceOnly | whatsmeow `handshake.go` `WACertPubKey`; whatsapp-rust `WA_CERT_PUB_KEY`. Signature scheme XEdDSA. |
| `noise.issuer_serial` | Intermediate issuer serial | 0 | ReferenceOnly | whatsmeow `WACertIssuerSerial`. |
| `noise.cert_chain` | Chain structure and check order | `CertChain{intermediate,leaf}`; details/signature(64) | ReferenceOnly | whatsmeow `verifyServerCert`; schema `WACert.proto`. |
| `noise.record_limit` | Max Noise record size on the wire | — | Unknown | Needed to accept `snow` (65,535-byte limit); ADR 0004. |
| `schemas.*` | Protobuf schemas | `WACert`, `WAWebProtobufsWa6`, `WACommon`, `WAAdv` | ReferenceOnly | `proto/whatsmeow/` @ 2375e175. |
| `transport.endpoint` | Chat endpoint (host/port/URL, TCP vs WebSocket, TLS layer) | — | Unknown | Web uses `wss://web.whatsapp.com/ws/chat`; Android is different and not observed. |
| `login.client_payload` | `ClientPayload` fields for a primary Android device | — | Unknown | whatsmeow builds a WEB/companion payload; primary markers, version fields and startup requests are not observed. |
| `registration.endpoints` | HTTPS registration endpoints | — | Unknown | Not observed. |
| `registration.payload_encoding` | Request/response encoding | — | Unknown | Not observed. |
| `registration.prerequisites` | Keys/ids needed before first request | — | Unknown | Section 7 requires generating them before the first request that needs them. |
| `registration.verification_steps` | OTP / PIN / other steps and cooldowns | — | Unknown | Not observed. |
| `messaging.id_format` | Wire message-ID format | — | Unknown | Web references use uppercase hex of random bytes; tickets carry `wire_id: None` until observed. |
| `e2ee.*` | Session/message versions, prekey upload, padding, retry | — | Unknown | Gate ADR 0005. |
| `devices.*` | Device list discovery/invalidation | — | Unknown | |
| `media.*` | Media construction | — | Unknown | |
| `store.*` | Local store facts (not wire) | see ADR 0002 | Observed | Tested offline in this repository. |

## Dated server observations

None. `server_observation` in the reference manifest is `null`.
