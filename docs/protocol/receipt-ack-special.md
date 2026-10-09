# Specialized read, played and server-error ACK messages

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Fourteen-topic recipe](apk/recipes/2.26.37.71-receipt-ack-special.json),
[complete selected methods](apk/2.26.37.71-receipt-ack-special/targets.json).
Offsets are method-relative bytes; catch tables use code units.
This extends [generic ACK dispatch](receipt-ack-dispatch.md), reusing the
[ACK parser](apk/2.26.37.71-message-ack/ack-wire-parser.txt) and
[0yG handler](apk/2.26.37.71-receipt-ack-dispatch/ack-handler.txt).
It adds source coverage, not a Rust state transition or live claim.

## Parser selection and specialized producers

**Confirmed:** 1Qy.A02's receipt type dispatch recognizes read, played,
played-self, read-self and server-error. It builds 1gc with false direction
at 01e0..01ea after selecting/normalizing its remote chat. The read branch
at 0236..025c converts the nullable stanza participant through 0DK.A00 and
calls 1Dz.A11. Played and played-self call A16; server-error calls A17.
Read-self takes the shown readreceipts attribute/227 path rather than this
A11 branch. These paths subsequently reach generic A1E forwarding; specialized
processing is not a substitute for the full-key callback removal.
The [read-setting continuation](read-setting-sync.md#read-self-ack-wire-producer-continuation)
now connects read-self/227 to the preference consumer; full subtype conversion,
broadcast routing and effective dispatch remain unknown. A type named server-error is distinct from the outer
numeric error attribute and is not a guessed registration refusal.

**Confirmed:** 1Dz.A11 calls A0I with participant and key. A0I builds a Bundle
with msgid from key.A01, remote_chat_jid from key.A00 through 0DE.A09 and
participant from the supplied DeviceJid through that same string helper.
It creates Android Message with **what=0, arg1=86 (0x56), arg2=0** and the
Bundle as obj. The constructor argument order is retained; 86 is not what.
Null string conversion semantics remain unknown.

1Dz.A16/A17 each build a one-key array and DDH with null device, null original
user, null stanza key, null metadata, the key's chat, status argument **10**
(played/played-self) or **12** (server-error), and long time argument **0**.
DDH.<init> stores array A07, chat A03, device A02, integer A00, long A01,
stanza A05, metadata A06 and original user A04. A0N packages it as Android
Message **what=0, arg1=262 (0x106), arg2=0**. The zero long is not an observed
peer timestamp; no receipt_user column or row insertion is demonstrated here.
These argument values must not be equated with 5/8/13 live user-timestamp slots.

## Dispatch reads arg1, not what

**Confirmed:** 1UB.A00 returns Message.arg1. 1Dy.CDr takes that selector and
asks 0y8.A0F. True clones the Message and calls A06; false sets what=0 and
posts to its own handler. A06 sets what=1, restores arg1 to the selector and
posts to 0yG. The previously retained 0yG case 1 ultimately invokes 0y8.A00;
full optional instrumentation and exception behavior are not reinterpreted here.

A0F's A0O-enabled branch admits both 86 and 262 explicitly. Otherwise, or for
non-hardcoded selectors, its fallback requires SparseIntArray A02 to resolve a
nonnegative index below handler list A0F.size(). Effective A0O and registration
contents remain unknown. A positive selector is not unconditional admission.

0y8.A00's A0O-enabled branch resolves Mpi.A00 N12 and selects **N12.A0d for
86**, **N12.A0Z for 262**. It resolves the provider, casts to 0jT and calls
BFC(Message,selector). The fallback uses the registered index/list, asserts
index bounds, invokes BFC and throws if the selected handler returns false.
Missing enabled handler selection also throws. Full N12 provider construction,
concrete BFC application and durable message-state effects remain unknown.
This now identifies the next concrete consumers to trace rather than inventing
an update from producer argument names.

In the false CDr branch, 1Dy.handleMessage case 0 calls 0iS.A0r then A0p.
A0p checks selectors 4 and 87 specially; other values, including 86/262, are
cloned and forwarded through 0fY.CDs. Full 0fY implementation and registration
remain unknown. Neither dispatch route proves handler completion or SQL commit.

## Consequences for the Rust port

**Confirmed scoped negative:** these selected producer/dispatch methods do not
insert a receipt_user row, adopt a peer key, decrypt a captured message or issue
a retry. That does not establish absence of downstream effects. Actual N12
A0d/A0Z consumers and fallback 0fY.CDs need tracing before adding message-state
updates. Generic receipt ACK completion remains separate from specialized state
messages and from peer delivery/read/played receipts.

No Rust runtime behavior, private account, key, ID, reservation or wait changed.
No live request ran. Existing durable ACK presence guard remains in place;
callbacks or status arguments do not authorize replay or fabricate user-row
membership needed by primary identity cleanup. Full identity recovery and
receive/messaging parity remain unfinished. Existing offline research tools
expose reproduction; no HTTP route, CLI command/flag or live capability is added.

```sh
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-receipt-ack-special.json \
  --output docs/protocol/apk/2.26.37.71-receipt-ack-special --check
```

Validation: fourteen selected topics/methods, nineteen files / 81,579 bytes
reproduced exactly from the pinned APK. Cached self-diff verified fourteen
unchanged topics and four indexed artifacts. CLI reference regeneration, API
library-symbol verification, formatting and whitespace checks passed. Repository
hygiene checked 1,726 local links and 63 checksum bundles / 12,666,281 bytes.
No fresh runtime or live result is claimed; the prior functional ACK guard
passed 812 workspace/all-feature tests.
