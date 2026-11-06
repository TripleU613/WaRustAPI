# Blocklist metadata store declarations and write boundaries

**Confirmed selected static**, WhatsApp2.26.43.72 build264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected DEX exports/catches. No APK/account/device execution.
[REA writer body unavailable](../../rea/2.26.43.72-blocklist-store-writes/README.md)
is a retained negative, not source-body authority. This follows the
[metadata worker](../2.26.43.72-blocklist-metadata-adoption/README.md).

## Declaration census is not active interface binding

The superclass-only override query for0bY.BQ1 returns only the abstract
interface declaration and explicitly excludes interface dispatch. A bounded
all-class method declaration export finds exactly three BQ1 declarations in
this APK:0bY abstract,0bZ concrete,12d concrete, each(Map)->void. This establishes
available declarations, not the runtime DI binding for7to's provider0xb43.
No claim that all interface targets or dynamic implementations are resolved.

12d.BQ1 iterates entries, callsBPz per entry and ignores the Boolean result.
12d.BPw similarly callsBPx and returns the input keys as a list, irrespective
of individual Boolean results. Those wrapper return shapes are not proof every
entry was persisted; BPx/BPz implementation and deployment selection remain open.

## Concrete0bZ username writer has normalization and side effects

Before its monitor,0bZ.BQ1 converts each supplied value through7k8.A00.A00.
For nonnull/nonempty input7k8 trims both ends through2LC.A0z ->0Cf.A0R, then
0Cf.A0j removes **all leading** characters in the supplied array, here @ only.
Trim uses0Fv.A01, already retained as Character.isWhitespace OR isSpaceChar;
no locale lowercasing appears in this selected normalizer. Null/empty input
uses87n.A01, whose initializer carries the empty string. Trim occurs before
leading-@ removal, so whitespace newly exposed after removing @ is not trimmed
again by this function. 87n constructor checks nonnull and stores the string in A00 without a
lexical-validation branch. Exact runtime Unicode tables are not executed.

Under its monitorBQ1 compares normalized values withAW0 (null treated as empty),
keeps changed tuples, treats empty new values separately and consults0Ws.A0V
before selected3LI.A04 deletion-related calls. Nonempty write candidates map
through0Wv.A07 row IDs; row_id==-1 is filtered from that selected write map.
DB-not-ready returns an empty result list for that branch. A ready DB opens a
transaction, invokes12g.A02 for each candidate, then calls1hk.A00/close and closes
the DB wrapper. Delegated0bY.BQ1 and3LI.A01 invalidation-related calls follow;
selected changed entries also schedule FJ4 notifications outside the monitor.
Complete DEX/catch ranges retain branches not summarized here. No single atomic
transaction over row storage, delegation, invalidation and notification is proven.
The exact3LI operation effects, AW0 ownership and notification semantics remain
open; do not replace this with a guessed blanket deletion/overwrite in Rust.

## Display writer and database-value boundaries

0bZ.BPw resolves input JIDs through0Wv.A07 into row-keyed values. When0Gs.A0D is
false it logs db-not-ready and uses an empty result list. Only the shown
size-equality branch delegates the original map to another0bY.BPw. Ready DB
uses0aq.A00 transaction,12g.A01 per entry,1hk.A00/close thenDB close. Returned
LIDs are reconstructed from row IDs and filtered to0Dx; returned keys are not
a generic acknowledgement of all input strings. Collision/invalid-row semantics
and0KU concrete wrapper behavior still require separate evidence.

12g.A01 andA02 target lid_display_name by lid_row_id with separate display_name
and username ContentValues. Each calls0KU.A02 then, when its returned count is
not positive,0KU.A09 with argument5. The username fallback additionally supplies
display_name=""; the shown positive-count branch does not clear display_name.
No affected-row/insert-result enforcement is shown here. These are exact call
arguments, not an untraced assertion about backend conflict or commit guarantees.

## Consequence and next boundary

Rust's unified blocked-row projection still lacks equivalent separated contact
metadata ownership, normalization, changed-value/deletion policy and partial
backend outcome handling. Previous snapshot parsing/membership corrections do
not establish these writer semantics. Before installing a metadata adapter,
resolve active provider binding and concrete0Ws/3LI/0KU/contact-store ownership.
No production code, sender, retry, migration, alias installation or live request
is introduced. Static existence is not live commit/acceptance or zero restrictions.

## Reproduction

From repository root with pinned APK and built xtask:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
OUT=/tmp/blocklist-store-writes-repro
mkdir -p "$OUT"
sha256sum "$APK"
target/debug/xtask apk-virtual-overrides "$APK" --class 'LX/0bY;' --method BQ1 --signature '(Ljava/util/Map;)V' > "$OUT/superclass-only.txt"
target/debug/xtask apk-smali "$APK" --class '' --method BQ1 --signatures --exceptions > "$OUT/username-definitions.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/12d;' --method BPw --signatures --exceptions > "$OUT/display-wrapper.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0bZ;' --method BPw --signatures --exceptions > "$OUT/display-store.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/12g;' --method A01 --signatures --exceptions > "$OUT/display-values.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/12g;' --method A02 --signatures --exceptions > "$OUT/username-values.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/7k8;' --method A00 --signatures --exceptions > "$OUT/username-normalize.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/2LC;' --method A0z --signatures --exceptions > "$OUT/trim-bridge.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0Cf;' --method A0R --signatures --exceptions > "$OUT/trim-bounds.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0Cf;' --method A0j --signatures --exceptions > "$OUT/leading-at.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/87n;' --method '<clinit>' --signatures --exceptions > "$OUT/empty-username.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/87n;' --method '<init>' --signatures --exceptions > "$OUT/username-model.txt"
```

checksums.json binds every other bundle file. The bounded BQ1 declaration export
contains only the three selected method bodies/declaration, not an app tree.
