# Project handoff review — 2026-11-04

This is an offline review of `main` at `a13021b03195f4474959de07ea1136d9c5b4b1db`,
not a fresh interoperability test. No account state was imported or changed,
registration requested, login attempted, or message sent.

## Assessment

**Confirmed from source:** this is an existing primary-device implementation
with an explicit experimental comparison driver, not a project to restart.
Keep the `WhatsAppClient` account actor, encrypted transactional `ProtocolStore`,
durable outbox and `SyncEngine` event journal. The three workspace packages are
`whatsapp-api`, `whatsapp-cli` and `xtask`. Initial support is Linux x86_64;
Rust 1.98.1 is pinned. See [architecture](../design/architecture.md) and
[working agreement](../../AGENTS.md).

**Historical observations recorded by the project:** registration, primary login,
direct/group encrypted messages, receipts, images, voice notes, replies,
reactions, polls and status have varying live evidence. November 1–2 changes
also include direct/group edit and revoke, document/video sends, trusted-contact
tokens, retry answers and app-version persistence. Server ACK, delivery/read
receipt and human display/playback confirmation are distinct milestones.
See [status](../../STATUS.md) and [feature matrix](../feature-parity.md).
These account outcomes were not independently repeated in this review.

**Confirmed from source:** the default pinned profile remains gated; live-capable
behavior is explicitly selected through the comparison driver and feature flags.
`client.rs`, `runtime/comparison.rs` and `transport/connection.rs` show this split.
Queue acceptance is not transmission, and `listen --send-outbox` can drain
eligible queued work. Preserve targeted tickets and existing request IDs.

## Default-build correction found during review

**Confirmed locally:** `cargo run --locked -p xtask -- reference-manifest <apk>`
failed with E0433 because the unconditional `chat_settings::edit_payload` called
`signal::direct::text_payload`, which exists only with
`experimental-signal-crypto`. All-feature tests and Clippy did not expose it.
The encoder now has the same feature gate as its dependency. Existing live
callers already require that feature; payload bytes and validation are unchanged.
Default and all-feature builds/tests must both be kept in the continuation gate.

## What prevents treating this as complete

1. **Account reliability is unresolved.** The November 2 status records all four
   previously registered lab accounts as banned and live work stopped. A later
   saved voice resend returned `sent`, with completed registration unconfirmed.
   These are historical reports, not current account inspections. The version
   mismatch correction and volume/pattern hypotheses do not establish why the
   service restricted the accounts. No causal or ban-prevention claim is justified.
2. **Protocol parity remains selective.** The ordinary driver implements only
   evidenced branches. Retry paths, identity/companion lifecycle, effective
   feature configuration, receipt policy and disappearing-message behavior have
   remaining gaps. Some existing branches lack fresh qualifying live samples.
   Calls, companion pairing and full sticker/community support are not complete.
3. **Operator/API surfaces differ.** OpenAPI includes CLI-only contracts and
   implemented HTTP routes. `x-transport`, `x-live` and `x-library` must be checked
   per operation; the OpenAPI file is not proof that every operation is an HTTP
   endpoint. See [API documentation](../api/README.md).
4. **Several summaries are stale.** `src/lib.rs` still says no server message has
   ever been exchanged; `repository-map.md` describes no socket or transmission.
   The feature matrix still says normal-runtime retry answering is absent,
   although `connection.rs` invokes `retry_answer::answer` and `complete_refetch`.
   Its document/video and edit/revoke rows contain older blockers alongside newer
   live results. Consult source plus dated evidence before selecting work.
5. **APK provenance is split across builds.** The default profile is 2.26.36.75;
   most retained method evidence is 2.26.37.71, with a smaller 2.26.37.70 core
   comparison. The available 2.26.43.72 APK has a recorded comparison, not a full
   retrace. Obfuscated class identities cannot be carried across builds by name.

## Research inputs available to the next session

**Observed during this review:** the GitHub `nothing_yet` prerelease exposes
`WhatsApp.1.apk`, 155,531,089 bytes. Downloaded outside the checkout, it matches
SHA-256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Google aapt2 reports `com.whatsapp`, version **2.26.43.72**, code **264307238**.
Google apksigner verifies v1/v2/v3/v3.1 signatures and reports SDK-dependent
signer certificates; the API 24–32 signer matches the documented
`3987d043d10aefaf5a8710b3671418fe57e0e19b653c9df82558feb5ffce5d44`.
This checks artifact identity/signature integrity, not protocol equivalence.
The APK remains outside Git; follow [APK analysis](apk-analysis.md) to obtain it.

Existing `xtask` DEX/xref/smali, bundle/remap/diff, coverage and manifest tools
should remain the reproducible project research path. REA/JADX can supply focused
class/method/caller evidence when useful. Retain selected evidence with exact APK
hash, build, method and confirmed/inferred/unknown labels; do not commit raw APKs
or bulk decompilation. No Android device is required for static work.

## Recommended continuation

1. Reconcile the current feature/blocker matrix against source and dated results,
   preserving the default/comparison distinction. Avoid repeating completed traces.
2. Source-audit correction (2026-11-04): wrong-OTP recovery already exists in
   commit `c7ff30d`; the older unknowns entry and this review's initial next-task
   recommendation were stale. The current REA continuation traces the separate
   PIN/password `guess_wait` and ports its preservation without enabling retries.
   [Evidence and remaining work](rea/2.26.43.72-registration-recovery/README.md).
3. Add offline fixtures and state/restart tests for that evidenced behavior; expose
   diagnostics through the existing CLI and update OpenAPI/status in the same chunk.
4. Continue one protocol boundary at a time, with explicit acceptance criteria.
   Runtime retry coverage and group distribution behavior deserve priority over
   adding unrelated feature breadth. Native `.so` analysis is useful only when a
   concrete unresolved producer or cryptographic boundary leads there.
5. Resume live validation only for an explicitly selected authorized account and
   action, after inspecting its private saved state. This clean clone contains no
   ignored private lab state. Do not create new identities, reuse historical
   numbers, resend acknowledged messages or infer that old cooldowns have expired.

## Validation

The reviewed head's latest CI run succeeded, but its Rust lint/test/package jobs
were skipped for a documentation-only change. That green run is not a fresh
whole-workspace test result. Repository hygiene was rerun locally and checked
3,716 links and 167 checksum bundles totaling 32,149,347 bytes before this note.
Fresh local validation passed:

- `cargo fmt --all -- --check`;
- `cargo build --locked --workspace`;
- `cargo test --locked --workspace`: **426 passed**, none failed;
- `cargo test --locked --workspace --all-features`: **1,105 passed**, none failed;
- `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings`;
- repository hygiene and `git diff --check`.

The reviewed [CI run](https://github.com/TripleU613/WaRustAPI/actions/runs/41535967635)
and [dependency-security run](https://github.com/TripleU613/WaRustAPI/actions/runs/41655710983)
are distinct from those fresh local checks. No live behavior or wire encoding changed in this review.
