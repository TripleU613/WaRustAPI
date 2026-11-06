# Root stream-end token and native logout reporting — APK 2.26.43.72

Historical research snapshot at570bca74. The identified Rust gap is now
[implemented with coordinated capture/termination tests](../../node-serializer.md#root-stream-end-captures-and-consumes-the-receive-epoch-2026-11-06).
Statements below about outstanding implementation describe that earlier snapshot.

**Confirmed selected static**, build264307238, APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected xtask DEX exports, not REA Evidence/runtime observations.
Research only; no live accounts or Rust behavior changes in this chunk.

## Token2 is control flow before ordinary node construction

[Complete123.A01](../2.26.43.72-group-missing-identification/current-jid-attribute-reader.txt)
reads a list header/count, then description token. At0018..0022 token2 returns
null BEFORE ordinary description conversion and listcount validation. Its
meaning is not dependent on looking up the textxmlstreamend in a dictionary.
A raw-string description spellingxmlstreamend is not this exact token branch.
The retained frame readerA08 decrypts/authenticates, handles compression and
callsA01 for selected compressed and uncompressed paths, then returns its node
or null. Full fragmentation/header/error/cipher lifecycle remains separate.

The [complete1s2.run](../2.26.43.72-about-callback-lifetime/reply-dispatch.txt)
at0058 callsA08; its null result takes0064->1196. That block logs logout with
isServerAckKick=false, invokes1lX.C2o(-1) and RETURNS. It does not read another
stanza or send an ACK on this selected path. Server-ack-kick has a separate
reason18 branch; corrupt-stream has separateLIL/onError handling. These
native reason values are not Rust error codes or universal server retry rules.

## Concrete listener and disconnect reporting

The [complete0ZJ.A0z initialization](../2.26.43.72-about-callback-lifetime/owner-initialization.txt)
constructs1lY(this) in v8 at04e4, retains that value through0c00 and passes it
as1s2's1lX argument. Retained1lY.C2o posts what1,arg1=reason,arg2=0.
Its handleMessage case1 checks its finished flag: when set, takesA1M with
quit-if-logout-finished diagnostic; otherwise callsA1G(reason).

A1G delegates toA0j. A0j callsA0X, remaps input-1 to localreason4, then calls
A1Q(false,reason). A1Q invokes1rh.A00 when present (that method forwards offline_resume=false
to0Ta; it is not proof of socket closure), clears selected
0YC flags, invokes14E cleanup, and checksA1X. Only if that early-quit predicate
isfalse does it pass the reason through0uu.A01, callA0n(effectiveReason),
notify writerCd4, invoke0YS.BtH(false,effectiveReason), then0uu.A03.
Concrete0YU.BtH postsmessage4 withboolean/reason asarg1/arg2. The prior
[handler disconnect flow](../2.26.43.72-iq-shutdown-eligibility/README.md)
provides further callback cleanup context. FullA0X/A1X/A0n/0uu reason mapping,
provider identity and global reconnect/backoff authority remain open.

This proves selected root-end termination and its local report chain; it does
NOT establish unconditional logout persistence, credential removal, account
ban/refusal, no-reconnect forever or safe retransmission of pending work.

## Confirmed Rust gap; implementation still outstanding

Current comparison dictionarysingle[2] isxmlstreamend. Decoder can turn
rootlist8/count1/token2 into an ordinaryNode with that description. Its empty
list variant instead fails validation. Normalconnection::drive has terminal
rules forfailure/stream:error, but no root control-token handling; its generic
undecodable path may capture and continue. Thus the selected native immediate
termination is not yet implemented consistently across Rust receive paths.
No passing test of existing terminal-error behavior should be treated as proof
that this gap is fixed.

Low-levelmessaging::read_node/read_frame/captured waits and three CLI comparison
receive loops have different decoded/undecodable interfaces. A coordinated fix
must retain authenticated capture and stop the session without manufacturing
ACK/delivery facts, while classifying reconnect eligibility explicitly. Native
C2o(-1)->local4 is not enough to invent a new ban/refusal policy. Do not patch
only a textualxmlstreamend name check: the actual control is rawroot token2,
not an arbitrary raw string or a nested child name.

Required regression scope for implementation: root LIST_8/LIST_16/empty-list
prefixes followed bytoken2, compressed/uncompressed frames, captured originals,
no subsequent read/ACK/write/settlement, queued/inflight uncertainty, duplicate/
trailing input treatment and controls for rawtext names, nested nodes and
ordinary terminal errors. Exact nestednull/child construction and all native
trailing-frame semantics remain untraced; no broader exception policy follows.

**Offline control:** existingundecodable_terminal_stanzas_end_the_session_without_reconnect
passes unchanged. It coversfailure/stream:error and unreadable-then-EOF, not
stream-endtoken2. No new runtime capability, reconnect rule or live operation.
The broader network objective remains incomplete.

## Reproduce

From checkout root with pinned APK outside Git; compare complete output bytes:

```sh
cargo build --locked -p xtask
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1rh;' --signatures --exceptions > reader-owner.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/123;' --method A08 --signatures --exceptions > frame-reader.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1lY;' --method C2o --signatures --exceptions > logout-submit.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1lY;' --method handleMessage --signatures --exceptions > observer-handler.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0ZJ;' --method A1G --signatures --exceptions > logout-proxy.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0ZJ;' --method A0j --signatures --exceptions > logout-state.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0ZJ;' --method A1Q --signatures --exceptions > logout-report.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0YU;' --method BtH --signatures --exceptions > disconnected-submit.txt
target/debug/xtask apk-callers ../android-lab/apks/WhatsApp.1.apk --class 'LX/1s2;' --method '<init>' > reader-callers.txt
```

Existing complete A01/run/initialization exports retain their own recipes and
checksums. PCs are bytes; branch deltas code units. Direct callers and selected
binding do not establish every provider/lifecycle or runtime interleaving.
