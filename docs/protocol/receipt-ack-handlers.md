# Receipt ACK concrete handler bindings

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Eighteen-topic recipe](apk/recipes/2.26.37.71-receipt-ack-handlers.json),
[selected source](apk/2.26.37.71-receipt-ack-handlers/targets.json),
[physical scan and literal fixture](receipt-ack-handler-inventory.txt).
This closes provider binding left unknown by
[specialized ACK dispatch](receipt-ack-special.md). Offsets below are method-relative
bytes. Runtime handler configuration and complete persistence remain unknown.

## Complete switch visibility

**Confirmed local defect/fix:** xtask's switch formatter previously retained only
64 cases, then appended `...`. That hid both required factory entries and failed
to validate a malformed tail. It now renders every declared packed/sparse case;
the DEX u16 count bounds the walk, and incomplete tables refuse with a diagnostic.
The existing research export byte budget still applies. The regression failed
before the fix. Literal 37.71 header/jump words verify late packed cases; a sparse
65-case fixture checks its late entry and truncated tail too.

Historical committed snapshots remain immutable. Comparing snapshots made by
old/new renderers can show newly visible cases on an unchanged APK; that is a
renderer change, not proof of changed WhatsApp behavior. Export both versions
with the same renderer before interpreting an APK release diff.

## Providers resolve to concrete consumers

**Confirmed:** N12.<init> assigns A0d through 00C.A00(0xb74), and A0Z through
00C.A00(0xb82). A00 creates 059; its constructor stores the ID, and get lazily
resolves it through 00C.A02, caching the object under a monitor. A02 delegates
to A03; A03's uncached branch calls 1UE.A00 at 0082..0088 and caches non-null
results. Its cache-hit path returns the existing object.

1UE.A00 selects a factory family using `(id >> 14) & 0x7f`. Both IDs select
family zero / A01. A01 masks with 0x3fff and selects A0R for the interval
0xae8..0xbfe (0d64..0d88). A0R's packed switch explicitly maps **2932/0xb74
→ 0a3a → new 14c**, and **2946/0xb82 → 0ae2 → new 17J**.
These are provider bindings, not an inference from matching selector numbers.
The previous 0y8.A00 dispatcher connects read selector 86 to A0d, state selector
262 to A0Z. Physical constructor scans independently find one caller for each,
1UE.A0R. The 31-method BFC name scan is recorded without claiming registration
or a complete interface implementation inventory.

## Read ACK handler is gated

**Confirmed:** 14c.BFC selector 86 branches to 01ca, parses remote_chat_jid,
participant and msgid from the bundle, and constructs 1gc(chat, id, false).
Missing msgid returns true without scheduling. It then calls 0DE.A0i; that helper
returns true only for a non-null JID whose getType() is **5**. A false result
also returns true without scheduling. The type number is established here;
its complete subtype mapping is not reconstructed by this chunk.

For the admitted chat, it resolves Cif and its executor, then submits
DNf(key, Cif, participant, **15**) at 025e..026e. The matching Object constructor
stores participant in A00, Cif in A01, key in A02. DNf.run case 15 starts at
07a8: absent participant/user or absent resolved 0ZY/8BW returns. Otherwise it
constructs a 72k lookup using participant resolution, Cnc.A01(message ID) and
Cif.A00(chat). Its predicates on 72k.A03, 8BW.A0L and 7uw.A02 can return early.
The remaining path at 11e4..121c names `onReadReceiptReceivedByServer` and calls
1b7.A0V(record, 1cE.A09, 7Mb.A0K, false). **Unknown:** full lookup semantics,
these enum meanings and transitive persistence. No general peer read timestamp
or receipt_user insertion is established by this bounded path.

## Played/server-error ACKs enqueue a status model

**Confirmed:** 17J.BFC selector 262 branches to 11ee, casts obj to DdM, resolves
Cmn and calls A03(model, **0**, **5000**, **0**) at 1214..1222. The integer comes
from the entry's v8=0 on this branch. It returns true. This packages the prior
DDH status arguments 10/12 and zero time; it does not replace them with peer time.

Cmn.A03 creates an AzY completion, constructs DMc carrying all four arguments,
and submits it through 1f0.A02(runnable, **13**). DMc.run calls Cmn.A04 with those
saved inputs, completes AzY with null on both normal and throwing exits, and
rethrows on failure. The 13 is an executor argument, not a receipt_user status.

**Confirmed boundary, incomplete trace:** the retained full Cmn.A04 asserts its
execution context, reads model size/stanza metadata and loops model entries.
Its elapsed-budget path invokes DdM.CPB for deferred work; later paths query
keys, status and time via BSJ/Ccf/CeD. **Unknown:** complete per-status application,
lookup/admission, deferred-job behavior, SQL/cache effects and error handling.
The full method is retained so this boundary can be extended without repeating
the provider investigation. Do not interpret source availability as closure.

## Runtime and validation limits

This chunk changes offline inspection output only. Rust messaging/receipt state,
account identities, reservations and private stores are untouched; no live
request or chat send was performed. Explicit read/played controls and primary
cleanup remain incomplete as tracked in the messaging checklist. Validation
covers late-case visibility, malformed tails and reproducible bounded exports;
it does not establish live specialized-handler parity.

Checks passed: 813 workspace/all-feature tests; strict workspace/all-target Clippy;
Rustdoc with warnings denied; formatting; CLI-reference regeneration and API symbol
check. Exact export verifies 23 files / 395,604 bytes; checksum-validated self-diff
reports all 18 topics and four metadata artifacts unchanged.
