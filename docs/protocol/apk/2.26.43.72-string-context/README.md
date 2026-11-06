# String encoding depends on node position — APK 2.26.43.72

**Confirmed selected static**, build264307238; APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected xtask DEX exports, not REA Evidence or runtime observations.
No live account operations.

## Node tag/name and attribute-value contexts differ

[Complete1r2.A01](../2.26.43.72-node-serializer/node.txt) invokesA05 for the
tag with(false,true), attribute names(false,false), and string attribute
values(true,true). [CompleteA05](../2.26.43.72-node-serializer/string.txt)
looks up tokens first, optionally interprets unknown text as a JID when its
second boolean is true, and otherwise callsA06 with its first boolean as
packing permission. Thus names still use tokens but do not nibble/hex pack
unknown strings. This is not a ban on numeric or hex names: raw encoding is
used for that fallback. Attribute values retain their packing permission.

The retained typed-JID writerA00 usesA05(user,true,false), and for its
protocol-compliant JID_PAIR server usesA05(server,false,false). AD_JID user
also uses(true,false). Other special classes have separate tuple branches;
their complete field schemas/class conversion remain separate unported
work. This change does not reinterpret the Rust special tuple models.

Rust previously used the same token/packing helper for names, values, users
and servers. It now preserves token lookup while disabling packing for names
and server strings. User/value packing remains enabled. Token dictionaries,
attribute order and typed-JID selection are unchanged.

## Node text content is raw bytes, not a token/packed string

0Tp's constructor(String tag, String content, attrs) callscontent.getBytes()
when content is nonnull, then delegates to its byte-array constructor. The
latter storesA01 byte content.1r2.A01 invokesA06(output,bytes,false), bypassing
A05 token lookup and disabling packing for body bytes. The exported constructor
uses Java's default charset; literal ASCII evidence needs no charset inference.
Rust preserves its existing UTF-8 model for non-ASCII text; complete native
runtime charset equivalence is not newly claimed here.

Rust NodeContent::Text previously passed through token/packing lookup; now it
uses the same raw length-prefixed byte path as byte content. Empty/nonempty
body presence is retained. A decoded text model re-encodes canonically as raw
bytes; exact original token spelling is not preserved, consistent with the
model's existing byte/text normalization. No persisted intents or cryptographic
plaintext/key/ciphertext bodies are rewritten by this codec correction.

## Literal regression and limits

Synthetic existing web dictionary fixtures show numeric tag123 and numeric
attribute name123 usingfc03+ASCII, while value:1 remainsff01c1. Bodyget uses
fc03+ASCII instead of dictionary token41; body:1 stays rawfc02+ASCII. The new
test failed before the correction on the tag's ff82 123f versusfc03 313233.
An old broad round-trip test assumed token content would retain its Text
variant; its fixture now explicitly uses byte content, with independent literal
tests preserving the stronger wire claim. This is offline testing, not proof
of server rejection of the prior alternate encoding.

**Remaining unknown:** native A05 can auto-convert eligible string text to JID;
Rust keeps explicit AttributeValue::Jid and does not yet copy that conversion.
Native constructor/cache/type rules, nullable/unknown namespace handling and
complete context-specific inbound fault routing require their own evidence.
No blanket text-to-JID conversion, new request/retry, settlement or live operation.

## Reproduce

From checkout root, pinned APK outside Git; compare retained output bytes:

```sh
cargo build --locked -p xtask
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0Tp;' --method '<init>' --signatures --exceptions > node-constructors.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/1r2;' --method A00 --signatures --exceptions > jid-writer.txt
```

PCs are byte offsets; branch deltas are code units. These complete selected
methods do not establish complete serializer, special tuple or account parity.
