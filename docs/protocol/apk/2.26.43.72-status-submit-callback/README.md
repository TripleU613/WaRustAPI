# Status submit callback registration and future meaning — APK 2.26.43.72

**Confirmed selected static**, build264307238; pinned APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected DEX methods via xtask fallback, not REA Evidence or runtime
execution. No account operations. Research-only; current Rust settlement,
callback ownership, timeouts and no-replay remain unchanged.

## Registration precedes queue submission

[SendE2eStatusJob](../2.26.43.72-status-wire-submission/README.md) passes outbound
Android Messagewhat0/arg1=8 containingEng plus expectedEoh to0Sf.A0B. A0B normalizes
key throughA02, constructsFfK, calls0Th.A01(callback,key), THEN callsA05(message,
this,false,false) to queue. Returning the future does not prove encoding/socket
write.1i7.A00 classifies this outbound message byarg1=8; its exceptions usearg2
only for other codes. This differs from inbound generic ACK handlerwhat6.

A05 has a readiness path that logs and returns without queue submission; this
selected return does not undo prior registration. Other paths activate a sender,
prepare/copy Message and invoke1A0.CdY. The selected methods do not establish
which concrete writer/channel is bound, actual byte encoding, socket completion
or global cleanup on disconnect. No native silent-drop policy is copied to Rust.

## Callback key normalization is narrower than message-chat selection

0Sf.A02 unwrapsfromDeviceJid only when getDevice()==0. For participant it first
uses0EB.A00 conversion, then unwraps only device0. Other key fields stay in a
copiedEfa or the originalEoh is returned. DevicegetDevice uses original integer
for the retained0EE/0EM classes: an admitted256 with low byte0 is NOT original0.
This is separate from1rH ordinary message-chat normalization, which unwraps a
DeviceJid base regardless of device number. No global to_non_ad rule follows.
Self-user participant conversion may throw before registration through its
retained getPrimaryDevice override; this is not an inbound reader gate.

0Th.A01 locks the full-key callback map. Duplicate key logs/reports an error,
then replaces viaMap.put; it does not reject or complete the prior callback in
this selected path. This does not prove unique native operation ownership and
is not permission to weaken Rust's original-reservation guards.

## Future completion carries no acceptance value

A01 wraps the innerFfK in2Gc with discriminator0 (constructor retained). Its
BsT case0 invokes inner.BsT(null), regardless of supplied callback object, then
records elapsed timing/metrics. BsR forwards failure. FfK.BsT stores its value,
sets successful completion flag and releases aCountDownLatch. Its get returns
stored value (therefore null here); BsR stores exception and get throws
ExecutionException. Timed get throwsTimeoutException if latch wait expires.

The [generic callback route](../2.26.43.72-receipt-callback-key/README.md) reaches
BsT after callback lookup without itself asserting ordinary message success.
Thus this status future can signal that callback processing finished, but its
normal return is NOT an accepted/refused/delivered result. Ordinary/async status
state handlers remain separate. Full-key fallback, challenge diversion and
independent ID maps must retain their own evidence and authority limits.

## Timeout removal is key-based

0Sf.A0T normalizes expectedEoh again, removes that key under lock and invokes
removed callback.BsR(exception) outside the lock. It does not compare a stored
callback owner object. Missing key returns without failure delivery. If a
replacement duplicate occupied that key, this selected code has no original-
owner check; no observed race or full native lifetime guarantee is claimed.
The status job's feature-controlled timeout handler rethrows after this cleanup.
No durable Rust reservation deletion, replay or reconnect follows from it.

## Reproduce

From checkout root, pinned APK outside Git; compare complete output bytes:

```sh
cargo build --locked -p xtask
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0Sf;' --method 'A0B' --signatures --exceptions > register.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0Sf;' --method 'A0T' --signatures --exceptions > timeout.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0Sf;' --method 'A02' --signatures --exceptions > key-normalize.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0Th;' --method 'A01' --signatures --exceptions > callback-insert.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0Sf;' --method 'A05' --signatures --exceptions > queue.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1i7;' --method 'A00' --signatures --exceptions > message-code.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/2Gc;' --method '<init>' --signatures --exceptions > wrapper-constructor.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/2Gc;' --method 'BsT' --signatures --exceptions > callback-success.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/2Gc;' --method 'BsR' --signatures --exceptions > callback-failure.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/FfK;' --method 'BsT' --signatures --exceptions > future-success.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/FfK;' --method 'BsR' --signatures --exceptions > future-failure.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/FfK;' --method 'get' --signatures --exceptions > future-get.txt
```

PCs are bytes; deltas code units. Remaining work: concrete1A0/writer registration,
last encoding/prewrite owner checks, ciphertext transaction and full callback/
disconnect lifecycle. No live execution, generic receipt producer claim or no-ban
proof; local Rust controls are not tests of native callback futures.
