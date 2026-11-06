# Individual report list bounds and result model

**Confirmed selected static / implemented offline**, APK2.26.43.72
build264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Focused complete DEX exports; no new REA Evidence records or live observations.
Reuse the [RPC, manager, exact Dzb constructor, Rv2 and Shk](../2.26.43.72-report-callback-timer/README.md).

## List baseline is zero; diagnostics are not server policy

Dzb's retained individual-report overload passes the long returned by B23.A03
as each list lower bound. B23.A03 initializes it to0, attaches the generated
request id, and returns that same0. Therefore the three list declarations are
0..210,0..5,0..5 in this build. 0Tk.A07 checks List.size against those inclusive
bounds. Empty nonnull lists pass; there is no requirement to invent a message
child merely because the schema permits them. This corroborates the existing
Rust empty-list subset against the current build, not server acceptance.

A07's out-of-range nonnull branches call00L.A0C(false,diagnostic) then return
true. A0C constructs an AssertionError, passes it to Log.e(Throwable), and
returns without an explicit throw. Null list logs its diagnostic then returns
false. Thus these bounds are generated diagnostics, not proof of a hard
constructor exception or a server restriction threshold. Rust's supported subset
still constructs no message children and does not copy this permissive fallback.
No larger list or message-payload capability is added.

## Request correlation and optional report ID

6oi.A0b requires the response tag iq and returns Dzb's original request node;
it does not by itself compare IDs. Rv2 selector1 reads original request id,
then supplies it as the expected response id to8B6.A0J with equality enabled.
The same reader validates type against literal result. The helper path
6of.A1Y(1) yields [type]; 6oj.A0v fills [id] and returns -9007199254740991,
and6oi.A0s returns +9007199254740991. String lengths use Java String.length,
not numeric parsing of the ID. The huge negative lower bound is generated
schema metadata, not a negative allowed-ID number. The expected string check is
exact equality, not normalization or a guessed new nonce.

A0J's string branch traverses the named child path, reads the selected attribute,
uses String.length bounds, compares a supplied expected string via equals, and
returns null with a stored diagnostic when the selected value fails. The full
method retains other typed branches without claiming they are all reconstructed.

Shk.A00's [report,id] path has lower1 from6oj.A0p and upper20. These are UTF16
code units through String.length, not bytes or Unicode scalar count. Absent,
empty, overlong or otherwise unselected ID produces no7NL from this helper.
Importantly Rv2 stores that nullable model without requiring it, and the RPC
returns7UQ with nullable report ID. An invalid/missing report ID does not alone
prove whole-envelope refusal in this selected chain; do not add that guard.
Child selection/duplicate policy and complete schema-error handling remain open.

RNG.A0H requests from through8B6.A0F with Jid and UserJid classes. A0F obtains
the generic parsed JID and applies isAssignableFrom against the supplied classes.
This is not an explicit server-only class restriction; Rv2 stores the nullable
result and does not compare it to request destination in this constructor.
Earlier transport/source/callback authority and complete JID parsing remain
separate. No new server-source requirement or relaxed source guard is invented.

## Corrected result-extension classification

The manager tries Rv2 selector1 success before its LIL fallback to the error
model. The selected success model does not inspect unrelated root error or
list/user/error extensions. Rust's generic action parser did, turning a valid
report/id result plus extensions into Error. The red regression reproduced
this. Matching saved kind=report and supported spam/set/spam_list wire forms
now retain their correlated outer-result outcome independently of those
extensions. A report label on a ping cannot claim this rule. Outer errors,
32s live deadline and other producers' inner-error rules remain unchanged.

This is callback/envelope classification, not the full native schema or report
application port. Generic Ok does not prove report-ID projection, server
moderation acceptance, native cache effects or source-policy completion. Raw
reply and settlement order remain durable. No field, report ID, reported
message, retry, replay, ACK or live account operation is fabricated.

The regression uses a valid-shaped report result with both root403 and nested
user401 extensions, a live written owner, duplicate settlement refusal, saved
reply, encrypted-store reopen and no resend. A mislabeled ping retains generic
refusal behavior. Existing timeout/error tests cover the separate32s boundary.

## Reproduce

From repository root with built xtask and pinned APK:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
OUT=/tmp/wa-report-response-reproduce
mkdir -p "$OUT"
target/debug/xtask apk-smali "$APK" --class 'LX/B23;' --method A03 --signatures --exceptions > "$OUT/list-baseline.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0Tk;' --method A07 --signatures --exceptions > "$OUT/list-bound.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/00L;' --method A0C --signatures --exceptions > "$OUT/bound-assertion.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/6oi;' --method A0b --signatures --exceptions > "$OUT/request-binding.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/RNG;' --method A0H --signatures --exceptions > "$OUT/response-source.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/8B6;' --method A0J --signatures --exceptions > "$OUT/schema-value.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/8B6;' --method A0F --signatures --exceptions > "$OUT/schema-jid.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/6of;' --method A1Y --signatures --exceptions > "$OUT/type-path.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/6oj;' --method A0p --signatures --exceptions > "$OUT/report-id-minimum.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/6oj;' --method A0v --signatures --exceptions > "$OUT/request-id-path-minimum.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/6oi;' --method A0s --signatures --exceptions > "$OUT/request-id-maximum.txt"
```

Compare every retained .txt byte-for-byte. checksums.json covers every retained
file except itself. Shared complete parent methods remain under their original
identities. No APK, bulk tree, message content or private state is committed.

Verification:1539 workspace tests across nine suites including doc tests, strict
workspace/all-target/all-feature Clippy, fmt, OpenAPI library-symbol and
CLI-reference tests, APK coverage freshness and repository hygiene passed.
All eleven helper exports reproduce byte-for-byte from the pinned APK.
Focused regression: `cargo test --locked -p whatsapp-api --all-features report_result_ignores_unrelated_error_extensions`.
