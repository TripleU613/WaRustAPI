# APK AD JID flag decoding and zero-device class selection

2026-10-11, **implemented/offline**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Four-topic recipe](apk/recipes/2.26.37.71-ad-jid-decoding.json) and
[complete signatures/instructions/catches](apk/2.26.37.71-ad-jid-decoding/targets.json)
retain the Java node reader, AD reader, string builder and writer. Offsets below
are method-relative bytes. This corrects the binary-zero assumption in
[typed recipient admission](receipt-user-recipient.md).

## Flag byte is not an agent

124.A01 recognizes AD_JID tag **247** at 007e..0086 and reaches A02 at
00f6..00fc. A02 reads the first byte, unsigned device byte and user string.
The [complete REA A02](rea/2.26.37.71-ad-jid-decoding/ad-reader.json) and exact
instructions agree: bit **1** selects LID, bit **128** selects hosted. Other
bits do not produce an agent. The selected servers are:

| flags & 0x81 | Server |
| --- | --- |
| 0 | s.whatsapp.net |
| 1 | lid |
| 128 | hosted |
| 129 | hosted.lid |

The device-99/hosted mismatch branch reports a diagnostic and continues;
it is not a parser rejection. Rust retains that acceptance without copying
telemetry. The previous reference-only decoder recognized just four byte values
and treated every other value as an agent on the PN server. Thus flag 3 selected
PN/agent 3 instead of LID/agent 0. Rust now uses the confirmed bit mask and
retains the received device and wire encoding kind.

1Qe.A00 writes AD tag 247, derives bit 128 from its hosted predicate, adds bit 1
for 0e1, then writes the device and user (0132..017a). There is no arbitrary
agent field. The Rust encoder preserves its four supported canonical namespace
flags and now refuses nonzero agents in this AD encoding rather than placing an
agent number in the flag byte. No saved local address is rewritten.

## Binary zero and literal text zero differ

[124.A03](rea/2.26.37.71-ad-jid-decoding/jid-string.json) omits the device suffix
when its integer is zero: at 001c..0024 it appends an empty string, not `:0`.
Nonzero values append colon and decimal device. A missing/empty user returns the
server string. After AD decoding, A01 passes the constructed string to 0D1.A00
at 0102..0108, then caches that actual JID in 0hq. A zero AD device therefore
reaches the plain user factory for the supported user namespaces.

Literal text `user:0@server` instead reaches the generic factory with the colon
still present and constructs a DeviceJid. Its failure of UserJid.class remains
correct. Rust's JidKind records wire encoding; it cannot substitute for this
native class construction. The typed recipient getter now permits Plain or
AgentDevice with both components zero, while retaining the explicit-text-suffix,
nonzero-device, namespace and special-object checks. It returns the plain user
form for ACK emission, matching the reconstructed UserJid and the writer's
zero-component JID_PAIR path at 009c..00fa; raw capture/decoder encoding kind
stay intact. This also prevents an
otherwise valid zero-AD user recipient from being treated as absent and projected
onto an ordinary outbox receipt fact. Original target guards remain intact.

Three negative fixtures formerly put agent numbers in binary AD flag bytes.
They now carry those unsupported addresses as text attributes, preserving
identity-query/refusal and group-reference attribution rejection without
claiming that native AD encoding contains an agent field.

The separate [JNI createNewJid](rea/2.26.37.71-ad-jid-decoding/jni-create-jid.json)
calls the same text factory and returns null on 08j. Its
[key/value packaging](rea/2.26.37.71-ad-jid-decoding/jni-key-value.json) caches the
supplied JID and raw string. These helper bodies do not prove the native codec's
own string construction, activation or agreement with every Java reader branch.
The proven zero result above is the Java 124 path, corroborating the earlier
[observed zero-AD contact ingestion](chat-actions.md). Native JNI decoding,
interop tuples, primary hosted alias normalization and complete special grammar
remain separate work.

## Nested JID tags are refused (2026-10-13)

Every JID reader takes its user and server parts through one string reader
(`read_string_value` in `binary/decoder.rs`). That reader always refused a JID
value, but only after reading it, and a JID's own parts were read through the
same function. Nested JID tags (for example `FA FA FA …`) therefore recursed
once per byte. A network-input audit measured a stack overflow, which is a
process abort and not a recoverable error, at 400–700 nested bytes in debug and
2,000–5,000 in release on a 2 MiB thread. A compressed frame of about 4 KB
(4 MB inflated, inside every limit) aborted both builds, before the frame was
captured. The reader now refuses the four JID tags (247 AD_JID, 246 FB_JID,
245 INTEROP_JID, 250 JID_PAIR) before reading them, with "expected string"
(class `invalid_node`). No input that decoded before is rejected: a JID in a
string position was always refused. For a malformed nested JID the reported
class can differ (for example `unexpected_eof` or `invalid_jid` before,
`invalid_node` now). The measured depths are observed in a scratch run, not
retained. Test:
`nested_jid_tags_are_refused_without_recursion`.

**Evidence boundary.** The retained 124.A02 above reads the AD user part with
`A09(stream, tag)`; A09's body is not retained, so whether the APK accepts a
nested JID tag there is **unknown**. The refusal is our own bound. A nested JID
in a string position has not been seen in any trace. Only the server can send
one: peer content arrives inside `enc` bytes.

## Checks and update workflow

Independent literal binary fixtures failed before implementation: noncanonical
flags created the wrong agent/server, and encoding a nonzero agent did not refuse.
The zero-AD recipient regression also failed before correcting its getter.
Tests retain AD encoding kind, verify user-recipient ACK copying, keep literal
`:0` nullable and prevent zero-AD qualifiers from adding ordinary outbox facts.
Existing API/CLI behavior changes only through these codec/receipt paths; there
is no new endpoint, option, schema, request or automatic replay.

```sh
target/debug/xtask apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-ad-jid-decoding.json \
  --output docs/protocol/apk/2.26.37.71-ad-jid-decoding --check
```

The generated nine files / 56,482 bytes reproduce exactly from the pinned APK.
For another release, review/remap these four methods before exporting a new
generation and using apk-research-diff; these topics have no automatic anchors.
No live request, saved identity/key operation, ID regeneration, reservation
rewrite or server-wait change ran. Static evidence does not prove ban immunity.

Validation: **926 workspace/all-feature tests** passed, plus default CLI/API-symbol
checks, strict workspace Clippy, warnings-denied Rustdoc, CLI build, formatting,
CLI-reference/OpenAPI coverage and repository hygiene (2,699 local links;
130 checksum bundles / 27,450,116 bytes). Four complete REA envelopes match the
pinned subject hash. No live receipt acceptance or cross-release parity is claimed.

Follow-up: [generic receipt ACK primary user encoding](receipt-primary-ack.md)
now canonicalizes decoded binary zero PN/LID `to`/`participant` values. The
complete writer class census explains why this rule cannot apply globally to
explicitly constructed DeviceLid addresses.
