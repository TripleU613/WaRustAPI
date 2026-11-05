# Sender-key empty-record and missing-chain retry causes

**Confirmed static**, APK2.26.43.72 code264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected EDR/EDY checked-DEX class exports are retained here. Existing
[Eyj.A01/0Yl.A00](../2.26.43.72-group-counter-error-paths/README.md) and
[EHq/EtY/F2q/DxB](../2.26.43.72-outgoing-retry-error-map/README.md) exports provide
the surrounding path. No target/account execution or REA Evidence envelope.

## Different native paths

Eyj.A01 checks the sender-key record list for emptiness at0014..0024 before
constructing Fj2 at0024..002e. Empty list jumps to035c, creates EDS with
No sender key for: context, and0Yl.A00 maps EDS -> -1008 -> internal1 -> wire1.
That includes a present record object with no states, not just an absent row.

For nonempty records, Eyj parses the sender-key message and searches by key ID.
Exhausting the search at0046 constructs EDR(String), No keys for: ID at0344..035c.
Its EDR catch table selects handler code-unit01c0 (byte0380), constructing
EDY(Throwable) at0382..038c. EDR and EDY both directly extend Exception; the
complete selected class bodies show constructor delegation and no overrides of
message/localized-message/toString. EDY(Throwable) delegates to Exception(Throwable).

The [Android cause-only Exception contract](https://developer.android.com/reference/java/lang/Exception#Exception(java.lang.Throwable))
uses the cause's string form for the outer message. The [default Throwable string
contract](https://developer.android.com/reference/java/lang/Throwable#toString())
prefixes the runtime class. Thus **inferred from confirmed APK code and platform
contract**, outer getMessage starts X.EDR: rather than the bare No keys for:
prefix. EHq passes that outer message with -1005; none ofEtY's selected exact/
prefix refinements match. It retains -1005 -> internal4 -> wire4. Applying bare
No keys refinement -> wire2 or direct EDR -> wire3 would discard this wrapper.
This resolves the earlier bounded uncertainty for this specific branch, not all
wrapped exceptions or every unknown-key error.

## Local corroboration and implementation

WrapperContract.java mirrors only the selected constructor semantics with a
synthetic ID, not the APK implementation. java-version.txt identifies OpenJDK27;
wrapper-result.txt confirms the class-prefixed outer message and failed bare-prefix
match. This local observation corroborates the documented platform contract but
is not Android runtime capture. Reproduce with javac -d /tmp/wa-chain-wrapper-classes
WrapperContract.java, then java -cp /tmp/wa-chain-wrapper-classes X.WrapperContract.
Reproduce EDR/EDY exports with xtask apk-smali APK --class 'LX/EDR;'
--signatures --exceptions (substituteEDY). Backend source identity is retained.

**Inferred provider equivalence / implemented offline:** receive_group checks the
loaded record is nonempty before backend parsing, preserving native empty-record
precedence. Empty -> None -> inbox MissingSession -> error1. At the subsequent
backend call a typed NoSenderKeyState now means no matching chain in that present
nonempty record. It becomes fixed redacted SignalMissingSenderKeyChain, inbox
MissingSenderKeyChain, RetryCause::MissingSenderKeyChain -> error4. No provider
strings or IDs enter diagnostics. Signature verification does not occur when
no chain key can be selected; this is not authenticated stored plaintext.

The production-loop regression adds malformed ciphertext with a stored empty
record (error1 before parsing) and a parsed message naming an unknown chain in
an installed nonempty record (error4). It checks original timestamp, reservation
before failed write, no repeated retry/delivery ACK on reconnect/redelivery and
unchanged sender-key rows. Existing missing/signature/future cases retain1/5/6.
Offline CLI replay sends nothing. Count/key progression, other producer policies,
full native routing/ACK parity and live acceptance remain open.

Validation:1329 workspace/all-feature tests passed. Formatting, strict workspace
all-target/all-feature Clippy, CLI reference regeneration, APK coverage, staged
whitespace and research checksum/link hygiene passed. Local platform-contract
fixture passed separately; no APK or live account execution.
