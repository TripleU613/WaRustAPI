# Explicit incoming message timestamps

2026-10-09. APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.

**Confirmed static:** the retained [group-envelope sources](apk/2.26.37.71-group-envelope-own/)
show `CnN.A02` 0012..001e reading optional root `t`. `CgC.A01`
00c2..00e0 reads `CaF.A09`, supplies a clock-derived fallback to `0Gv.A01`,
then multiplies the signed long by 1000. At 01c4..01d0 it stores the boxed
result in `Cih.A0J`. `Cih.A01` 0018..0032 requires a timestamp and unboxes it.
The already retained [Java parser evidence](receipt-clock.md) establishes signed
BMP decimal parsing; DEX long multiplication wraps. These are existing sources
reused for a Rust correction, not a new APK discovery.

**Implemented/offline tested:** incoming text/media/protocol events, reactions,
keep changes, poll votes and group-history notices share the explicit `t` parser.
Valid signed Java decimal values survive as millisecond `server_timestamp`,
including negative, zero and wrapping results. Previously ASCII parsing plus
nonnegative/checked multiplication silently discarded some valid values.

**Local policy / unported:** missing or malformed input remains unknown (`None`).
We deliberately do not substitute the APK's current-clock fallback during
historical projection: that could make old expiring content appear fresh. Existing
expiration checks remain in force. No stored event is migrated or reprojected by
this change; no journal marker is removed. Timestamp evidence is not clock
synchronization, receipt admission, a send permission or native user-row authority.

## Reproduction

The [host Java oracle](MexJavaNumericOracle.java) now has a `validity` mode,
distinguishing successful zero parsing from failure. Executed Temurin Java
21.0.12.1+1 LTS reproduced all existing parsed/scaled columns unchanged and added
validity to all 479 [public numeric cases](../../crates/whatsapp-api/fixtures/mex/2.26.37.71-numeric-reference.json).
This is host Java behavior, not execution on Android.

The regression applies these 479 inputs to six event projections (2,874 cases),
checks missing timestamps, expired/unknown content and a future BMP-digit
expiration case, then tests encrypted atomic journal publication and reopen with
three explicit signed/BMP/wrapping timestamps. Restoring committed captures does
not duplicate events. Existing CLI `events` JSON exposes event timestamps;
`events --summary` remains redacted. No new command or HTTP route is supplied.

```sh
cargo test -p whatsapp-api --all-features transport::inbox_events::tests
```

**Unknown/live unverified:** no qualifying live message was received in this
change. No real account was opened, no network request or message was sent, and
native missing-time fallback parity remains unimplemented.

Validation: 835 workspace/all-feature tests passed. Strict all-target Clippy,
warnings-denied Rustdoc, default CLI tests (25 plus API-symbol), regenerated
CLI reference, executed Java oracle, formatting and whitespace checks passed.
Repository hygiene checked 2,156 links and 107 existing APK bundles. No live test.

## Attachment expiry continuation

**Implemented/offline tested:** the ordinary attachment retrieval check now
uses the same explicit stanza timestamp parser as event projection. The prior
ASCII/nonnegative/checked path could reject a still-unexpired attachment whose
Unicode-decimal root `t` was already accepted for its event. Both directly saved
routes and original historical capture lookups use the shared parser; absence
of either authenticated route or valid time still refuses access. Expiry remains
checked on retrieval, not only when the event was published. No download,
plaintext export, view-once consumption or reservation policy changes.

The new offline test checks all 479 oracle inputs plus missing time and a future
BMP-decimal time through both saved-route representations (962 checks), and a
missing-route refusal. Existing ordinary media projection, reference revalidation,
view-once exclusions, cached authentication and export tests remain separate.
No account or network operation ran; new live attachment behavior is unverified.

```sh
cargo test -p whatsapp-api --all-features transport::media::tests
```

**Bounded follow-up / unknown:** the transport numeric-input scan also found
ASCII conversions in `groups::apply_refresh_in` (truncated size), `image_upload::route`
(route lifetime) and `clock::observe` (the separate inferred sample). They were
not changed: message timestamp evidence alone does not establish their native
numeric conversion, bounds or authority. This scan is not a complete numeric
parser inventory and makes no parity claim for those distinct inputs.

Validation: 836 workspace/all-feature tests passed. Strict all-target Clippy,
warnings-denied Rustdoc, default CLI tests (25 plus API-symbol), regenerated
CLI reference, formatting and whitespace checks passed. Repository hygiene
checked 2,159 local links and 107 APK bundles. No fresh live test.
