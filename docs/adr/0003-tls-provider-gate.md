# ADR 0003: Explicit rustls/ring provider for registration HTTPS

Status: owner-approved TLS exception · 2026-09-17

## Current decision (supersedes the historical provider gate below)

The owner approved relaxing the pure-Rust dependency requirement to advance
onboarding. Registration HTTPS now uses rustls 0.23.45 or newer with its
explicit `ring` provider, pinned by Cargo.lock. This replaces
`rustls-rustcrypto` and removes its older WebPKI/RSA dependency chain.
`ring` includes native C/assembly and needs a C build toolchain; this feature
must no longer be described as entirely pure Rust. The dependency auditor
allows `ring` only for the registration HTTPS feature, while retaining other
native dependency restrictions. No OpenSSL/system TLS is introduced.

The CLI's explicit `experimental-signup` feature composes the approved library
HTTPS feature and receives the same narrow `ring` exception. Its complete
normal dependency graph is audited separately; default CLI builds stay unchanged.

Certificate and hostname validation, explicit Mozilla roots, HTTPS-only
origin restrictions, no automatic retries/redirects/proxies, deadlines and
response bounds remain unchanged. No process-global provider is installed.
The registration feature remains experimental because protocol interoperability
is not established. Provider selection alone does not certify production use.

Reference: [rustls ring provider](https://docs.rs/rustls/0.23.45/rustls/crypto/ring/index.html).

## Historical decision

## Findings

- `rustls-rustcrypto` on crates.io: only `0.0.2-alpha` (0.0.1-alpha yanked),
  depends on rustls ^0.23. Its documentation warns against production use.
- `reqwest` with `rustls-tls-manual-roots-no-provider` and `http2` brings no
  crypto provider into the graph (verified in a probe: no `ring`/`aws-lc-sys`).
- Graviola includes assembly and is excluded under the strict contract.

## Decision

Production HTTPS support is **blocked**. No HTTP client is part of the
library graph yet; registration and media HTTPS operations are represented
by typed errors (`ProfileIncomplete`, `Unsupported`). When an experimental lab
build needs HTTPS, it must use `reqwest` (no default features,
`rustls-tls-manual-roots-no-provider`) plus `rustls-rustcrypto` behind a
feature whose name contains `experimental`, with normal certificate
validation. Certificate validation is never weakened.

## Exit criteria

A reviewed pure-Rust rustls provider with a maintenance track record, or an
explicit decision by the project owner to relax the contract for TLS.

## Experimental implementation update — 2026-09-17

Feature `experimental-registration-https` now implements the lab-only path
described above with reqwest 0.12, rustls 0.23, and
rustls-rustcrypto 0.0.2-alpha. It builds a per-client rustls configuration
instead of installing a process-global provider, uses the compiled Mozilla
WebPKI roots, keeps certificate and hostname validation enabled, accepts only
the comparison registration origin, disables redirects, proxies, transparent
decompression and automatic retries, applies bounded connect/whole-request
timeouts, and incrementally caps response bodies at 1 MiB. The dependency
audit covers this feature graph and confirms no denied native crypto crate is
active. This does not change the production-blocked status: the provider is
still alpha.
