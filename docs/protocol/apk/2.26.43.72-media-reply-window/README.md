# Media refresh callback window — APK2.26.43.72

**Confirmed selected static**, build264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Checked xtask DEX fallback, not REA Evidence or runtime observation.

[0VM.A07](../../rea/2.26.43.72-media-route-lifetimes/request-clock-checked.txt)
constructs media_conn IQ/set to the server with xmlns=w:m. At0122..0130 it
passes32000ms and logical type124 to0Sf.A0R. [Complete A0R](registration.txt)
first calls A0W; its fallback registers through0ln.A08 before enqueueing.
[Shared A0W/A0A registration and timer sources](../2.26.43.72-abprops-callback-timer/README.md)
establish registration before dispatch, without timer restart on promotion.
The nominal32s timer is not a strict native reply-age check: delayed Handler
execution can compete with response claim. No Android scheduling equivalence
or automatic retry is inferred.

## Sender admission: bounded negative finding

The [complete1xY callback class](../../rea/2.26.43.72-media-route-lifetimes/callback-class-checked.txt)
contains no root from/xmlns check. CK4 replaces the root with its media_conn
child at0030..0038 before parsing fields. Bvj iterates errors; registration
has no separate expected-sender argument. The [selected IQ reader](../2.26.43.72-iq-reader-admission/README.md)
and [ordinary callback dispatch](../2.26.43.72-iq-response-ownership/registry-A01.txt)
forward result/error by ID without a selected sender comparison. This is not
a global absence proof: upstream authentication, provider bindings and alternate
routes remain open. No invented sender whitelist was added to Rust.

## Rust correction

[Canonical behavior, I/O cap and tests](../../media-network-outcomes.md#media-refresh-reply-window-2026-11-06).
The CLI previously used the generic30s captured wait. It now uses the media
family's32s pre-write deadline. Configured per-record I/O bounds remain in force;
a configured30s socket bound still ends a silent read before32s. Other families
retain their generic30s bound. No config override, automatic refresh or replay.

```sh
sha256sum ../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0Sf;' --method A0R --signatures --exceptions
```

Linked complete methods retain their own recipes/identities. checksums.json
covers every other file here. No live account operation or ban guarantee.
