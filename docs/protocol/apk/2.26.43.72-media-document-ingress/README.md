# Current document ingress and nonnegative size — APK 2.26.43.72

**Confirmed selected static / research only**, build264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected DEX exports via xtask fallback; no target execution, new REA
Evidence IDs, account traffic or Rust runtime change. Byte PCs are method-relative;
branch displacements use16-bit code units. This updates the current-build boundary
of [ordinary documents](../../document-media.md), preserving the older evidence.

## Core descriptor matches Rust

[Di0.dynamicMethod](document-descriptor.txt) and the [exact decoded fields](document-fields.txt)
confirm url1, mimetype2, title3, fileSha2564, fileLength5, pageCount6, mediaKey7,
fileName8, fileEncSha2569, directPath10, mediaKeyTimestamp11, contactVcard12,
thumbnail metadata13..19, caption20 and accessibilityLabel21. The current nested
descriptor contains no viewOnce field; this says nothing about unseen wrappers.
Rust's core Document tags and separate caption20/fileName8 extraction agree.
Its known view-once wrapper exclusions remain; no unknown-wrapper replay rule is
invented from absence of a nested flag.

The field output was previously decoded directly from this pinned APK while
locating audio, then retained as a bounded document-shaped candidate. It is
copied byte-identically here, not inferred from the older BZB class name.
Full name-helper bodies are not duplicated; no offline raw-smali schema
regeneration success is claimed.

## Select the real ordinary converter, not every hash reader

[Encrypted-hash field census](encrypted-hash-refs.txt) finds TEN direct DEX
read/write sites. A field reader alone does not establish ordinary ingress.
The initially selected [FPY.CR7](candidate-medianotify-converter.txt) has special
medianotify/other branches and is not an Eel.A01 direct caller. Its full export
is retained as a bounded candidate, not silently treated as the ordinary parser.

The exact [Eel.A01 invocation census](projector-callers.txt) finds FOUR sites:
COi.A04,Cf3.CR7,FOt.CR7,FPq.CR7. [FOt.CR7](document-converter.txt) tests the
DiW document-message bit, excludes `medianotify`, constructs9JR and invokes
Eel.A01 at0062 with the caller's mode/update inputs. It also has other branches;
only this selected document path is classified. [FPq.CR7](alternate-converter.txt)
is retained as another direct caller; its complete wrapper/alternative-context
semantics are not equated with FOt.

These are direct DEX invocation counts, not complete reflection/JNI/runtime
coverage. Full envelope dispatch and the caller mode/booleans remain untraced.

## Document size differs from audio/video at this leaf

[Complete Eel.A01](document-projector.txt) initializes/updates8tC with mode/context
predicates. At00d2..00e8 it reads fileLength and throws only when signed value is
NEGATIVE (`if-ltz` at00de), so zero reaches Ch9 in this selected parser. This is
different from audio/video's positive-size checks; do not copy those conditions
across media kinds merely because the same downstream hash setters are used.

Rust currently rejects zero in BOTH ordinary Descriptor admission and shared
crypto verify_kind. This is a **confirmed leaf-policy difference/local nonempty
limit**, not proof that every native zero-byte encrypted document succeeds end to
end. Model/source construction, crypto framing, transfer finalization, UI/export
and retry behavior for empty documents have not been followed. No partial fix
removes only one guard and strands a reservation; no zero-length network support
is claimed or enabled from this leaf alone.

At0104..011c the selected plaintext hash must be32bytes before DBc.A1K; at0128..
0140 a present encrypted hash must be32 before DBd.A1R. Caller update/presence
conditions are independent. The earlier byte-setter/message/model chain establishes
local Base64 representation and downstream hash fields. Malformed hash error14
is a local Dxp parser argument, not an HTTP or proven ban/retry code. Media key,
thumbnail and timestamp handling precede these checks; no new native crypto
algorithm is derived merely from schema/field names.

## MIME and labels are not universal native allowlists

Unlike the selected audio MIME selector, Eel stores raw mimetype through Ch5 at
02fc..0306; this selected method does NOT call an equivalent MIME allowlist.
This bounded absence is not evidence that every MIME passes every native layer.
Rust's token syntax/256-byte base-MIME bound for document export stays explicitly
local, and the Opus exact-MIME fix is not copied onto generic documents.

Title and filename are separate: nonempty title at018e..01ae and filename at01d6..
01f8 pass through StringUtils.A0F with limit65536 before their different message
setters. Caption has its own field/setter at01bc. StringUtils truncation semantics
are not re-derived here. Sender filename is never used as Rust's output path;
existing explicit create_new/private-file export behavior is unchanged.

DirectPath is separately inspected/stored and missing paths are logged on a
selected branch, not universally thrown. contactVcard at0306 can set message.A05
for a distinct local subtype. Thumbnail fields and page counts have other paths.
Native downloader/context handling for all of these remains separate from this
core descriptor/export comparison.

## Conclusion and remaining scope

**Confirmed:** core current document tags, selected ordinary converter-to-parser
chain,32-byte hash guards, distinct label fields and nonnegative parser size rule.
No confirmed outbound request-construction or result-adoption bug was fixed in this
pass. Rust changes are source provenance comments only. The nonempty/size/MIME
bounds remain labelled local; no automatic download, retry, response relaxation
or sender-controlled path adoption is introduced.

**Unknown:** complete wrapper/medianotify routing and update modes, zero-byte
crypto/transfer end state, contactVcard/thumbnail/provider handling, native URL/
range/status recovery, account/output authority and live interoperability.
No server acceptance, restriction-cause or zero-ban claim is made.

Reproduce `apk-field-refs <apk> --class 'LX/Di0;' --field fileEncSha256_
--type 'Lcom/google/protobuf/ByteString;'`; `apk-callers <apk> --class 'LX/Eel;'
--method A01`; `apk-smali <apk> --class '<descriptor>' --method <name>
--signatures --exceptions` for Di0.dynamicMethod,Eel.A01,FPY/FOt/FPq.CR7.
Field output came from `apk-proto-fields <apk> --class 'LX/Di0;'`.
[checksums.json](checksums.json) pins all retained files including this report.
