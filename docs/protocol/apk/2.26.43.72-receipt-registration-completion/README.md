# Receipt callback registration and completion — APK 2.26.43.72

Confirmed selected static, build264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected DEX methods/catches and direct callers, exported using the
repository disassembler. These are not REA Evidence envelopes or runtime proof.
No account, viewer or network operation was performed.

## Registration precedes dispatch

0Th.A01 has two direct callers, 0Sf.A0B/A0C. A0B normalizes the Eoh key
through A02, creates FfK, registers through 0Th.A01 at0028..002e, then calls
0Sf.A05 at0052..0058. A01 checks duplicates under A04's monitor and still
puts a new 2Gc wrapper at00a4..00aa; discriminator is0 (0096..0098).
Registration precedes this dispatch call, not proven socket completion.

The direct A0B caller census includes read, played, retry, media-error and
order-status-failure receipt jobs, peer-message and status-send jobs. This
census is not full reflection/native/aliasing coverage or proof every caller
has the same policy. Complete played A0L constructs EW7/Message selector38,
calls A0B at02d6..02dc, then FfK.get at02de..02e4.

## Callback completion is not receipt acceptance

2Gc.BsT discriminator0 forwards null to the held callback at006e..007e;
the nonzero branch forwards the supplied object. The retained constructor binds
the supplied callback to A01 and the discriminator to $t. Both share elapsedRealtime
metrics/optional cleanup. FfK.BsT stores the object, sets A02=true and releases
the latch. Its no-argument get waits, then returns A01 when A02=true, otherwise
throws ExecutionException. The timed overload also has TimeoutException.
The played job uses the no-argument overload; this does not establish absence
of timeouts elsewhere. No outer numeric error is interpreted in these methods.

Combined with the retained current [handler](../2.26.43.72-receipt-callback-key/README.md),
which can invoke BsT(node) with an error present, this closes the selected
local callback/future path. It does not turn error-bearing ACKs into server
acceptance, nor prove every specialized receipt state transition succeeds.
Rust keeps its separate acknowledged/refused diagnostics, strict correlation,
durable callback ownership and no-replay guards. No production mismatch was
established and no behavior change is justified by this trace.

Unknown: all A05 failure paths, FfK failure producers, effective native job retry
policy, complete specialized state, aliases and live outcomes. Native duplicate
callback replacement is not permission to replace/replay Rust uncertain intent.

## Reproduction

Run from the repository root with the pinned APK and current built xtask;
choose an empty output directory, then compare files with this bundle.

```sh
APK=../android-lab/apks/WhatsApp.1.apk
OUT=/tmp/receipt-registration-repro
mkdir -p "$OUT"
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'LX/2Gc;' --method '<init>' --signatures --exceptions > "$OUT/wrapper-init.txt"
target/debug/xtask apk-callers "$APK" --class 'LX/0Th;' --method A01 > "$OUT/register-callers.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0Th;' --method A01 --signatures --exceptions > "$OUT/register.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0Sf;' --method A0B --signatures --exceptions > "$OUT/wrapper.txt"
target/debug/xtask apk-callers "$APK" --class 'LX/0Sf;' --method A0B > "$OUT/wrapper-callers.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/2Gc;' --method BsT --signatures --exceptions > "$OUT/callback.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/FfK;' --method BsT --signatures --exceptions > "$OUT/future.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/FfK;' --method get --signatures --exceptions > "$OUT/get.txt"
target/debug/xtask apk-smali "$APK" --class 'Lcom/whatsapp/messaging/receipts/jobqueue/job/SendPlayedReceiptJobV2;' --method A0L --signatures --exceptions > "$OUT/played-job.txt"
```

checksums.json covers every other bundle file with bytes and SHA256.
