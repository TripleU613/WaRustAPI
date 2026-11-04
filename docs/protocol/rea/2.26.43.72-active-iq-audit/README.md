# Current activation IQ construction check

**Confirmed static**, APK 2.26.43.72 SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete checked DEX class output; no network operation.

- [builder.txt](builder.txt), raw `LX/277;.A00(String,boolean)`: `iq` attributes
  `id=<argument>`, `xmlns=passive`, `type=set`, `to=0mL.A00`; true selects the
  sole empty `active` child, false the `passive` child. No extra metadata or
  token is added by this builder. SHA-256
  `9fbff1e8bf5dfa1598776eb57dc7a035f7445721157855384a0e537ba8f21bf0`.
- [server-jid.txt](server-jid.txt), `LX/0mL;`: singleton has empty user and server
  `s.whatsapp.net`. SHA-256
  `9b7609bba92f00a97aeba1053898348d6f4c39cd17cb1c8a0001067ae16856d3`.
- [callback.txt](callback.txt), `LX/2IT;`: result callback `CK4` posts Handler
  command8 with the saved constructor int and arg2=0 when handler exists;
  delivery/error callbacks log warnings. SHA-256
  `403772e9c65c9e8a0c7f7a9cc79708a5b3e0eb8c1ec8af980a1bb89d9c30ed9b`.
  REA lookup `X.2IT` did not resolve a class, so the checked DEX fallback was used.

Rust `transport/messaging.rs::active_request` matches the traced true-builder
shape; its existing unit test checks exact attributes/child. No wire change was
needed. Native callback semantics do not prove the full subsequent bootstrap
sequence or equivalence of Rust's readiness/presence transition. Those remain in
the [startup audit](../../network-startup-audit-2026-11-04.md).

Reproduce each complete class with `xtask apk-smali APK --class 'LX/277;'
--signatures --exceptions`, substituting `LX/0mL;` and `LX/2IT;`. The caller path
is being retained by the separate startup continuation; this leaf does not claim
that every connection state unconditionally invokes the true branch.
