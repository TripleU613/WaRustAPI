# MobileConfig MEX error callback and selected message

**Confirmed static**, APK2.26.43.72 code264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Static DEX evidence, not REA Evidence envelopes, observed traffic or APK execution.
Follows the [fetch/result route](../2.26.43.72-mobileconfig-fetch-route/README.md).

## Selected failure handoff

The retainedIDq selector0 installsIE9 selector30 as its error callback. Generic
IE9 constructor storesHmm inA00 and the captured handler inA01. Selector30 at
182e..185e buildsID2 selector20 with the supplied error and handler, then uses
Hmm.A01's existing executor/direct dispatch. It returns the shared Boolean helper
result; this is not an implemented retry verdict.

ID2 selector20 at05e0..0622 castsHbw, reads its error list, formats a selected
message for the log, obtains the selected message again and calls
MobileConfigFetcherHandler.onComplete(false,message). Null message becomes
"Unknown error"; an empty string is not replaced by that fallback. It then jumps
to the shared Unit return at0ac0..0ac6. No selected Java instruction queues a
retry, advances an ep_refresh_id preference, clears a table or calls the success
worker. Native onCompletion behavior and the generic GraphQL transport's own
retry/timeout policy remain separate unknowns, so this is not a claim that every
failure anywhere is nonretryable. No numeric server code or backoff is fabricated.

selected-callbacks.json contains only these selected branches, dispatch switches,
required common return and complete catch metadata. It is explicitly an excerpt,
not the entire multiplexedinvoke bodies. Full export hashes and byte counts pin
its source; the script regenerates the full exports before checking hashes and
selecting ranges. None of the listed catch ranges covers these selected cases
(addresses in the source catch table are code units, versus instruction bytes).
Bulk unrelated selectors stay outside Git.

## Error ranking is stable, not first-entry selection

6oe.A19 callsHbw.A00(list), then the selectedNap.Aga(). Hbw chooses the lowest
rank, preserving the earlier entry on ties:

| Rank | Condition |
| ---: | --- |
| 0 | Hxy with extensions.summary true under optBoolean |
| 1 | OtherHxy with extensions.severity equal to critical ignoring case |
| 2 | LIM exception family |
| 3 | Other eligible error |

Hxy stores root JSON inA01 and optionalextensions object inA00; summary/severity
classification reads extensions, whereas Aga reads rootmessage via optString.
The full selector, constructor and critical classifier are retained. Empty lists
throwNoSuchElementException. Hbw construction wraps a non-LIM throwable inLI8,
appends each server error object asHxy after the failure entry, and throws if the
resulting list is empty. Malformed JSON entries do not become valid errors by
being counted. Complete Android coercion and every exception subtype remain
unported; this table does not implement them in Rust.

The declaredAga override scan covers114178 APK classes and findsHxy,LIM,7MI plus
the interface. Hxy returns a rootmessage string with empty fallback;LIM uses its
message, otherwise cause/"Unknown Failure" string;7MI returnsdetail. This is
static declaration coverage, not a guarantee against runtime subtype replacement.
The selected message is not a complete structured error, response acceptance,
server-required wait or account restriction classification.

## Reproduction and impact

```sh
python3 docs/protocol/apk/2.26.43.72-mobileconfig-fetch-error/derive-error-excerpts.py --apk ../android-lab/apks/WhatsApp.1.apk > /tmp/mobileconfig-errors.json
cmp /tmp/mobileconfig-errors.json docs/protocol/apk/2.26.43.72-mobileconfig-fetch-error/selected-callbacks.json
```

Full helper exports use apk-smali --signatures --exceptions forIE9/ID2 factories
and constructors,6oe.A19,Hbw.A00/<init>,Hxy.A02/<init>. Interface-aware override
scan targetsNap.Aga()Ljava/lang/String;. ReuseHmm.A01 and handler delegation from
the prior bundle. This narrows the previously unported general error-ranking
research in mex.md for the current build; no old-build name equivalence is assumed.

Rust still has no MobileConfig fetch/hydration adapter, so this chunk enables no
new request or generic ACK behavior. Native persistence, effective configuration,
WWW/error subpaths, full transport retry and timeout policy remain open. No
private account state, identity reset, live request or account operation.
