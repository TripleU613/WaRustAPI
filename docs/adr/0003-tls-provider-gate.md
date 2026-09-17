# ADR 0003: Production HTTPS is blocked; no TLS provider selected

Status: accepted (blocked) · 2026-09-17

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
