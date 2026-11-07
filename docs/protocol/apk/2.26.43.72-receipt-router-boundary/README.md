# Receipt router precedes ordinary status parsing — APK 2.26.43.72

**Confirmed selected static / implementation planning boundary**, build264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected DEX exports/reference results and a labelled derived declaration
summary; xtask fallback, no native execution, account traffic or REA Evidence IDs.
No Rust runtime behavior changes in this research chunk.

## Discover the dispatch layer instead of assuming every receipt reaches F10

[Direct caller query](handler-direct-callers.txt) found no direct 14h.BLn call.
That is not evidence the handler is unused: delivery is through message/handler
interfaces. A declared A03 override census found 1rA,1rH,1rJ,1rL; its
[derived summary](declared-a03-summary.txt) retains the coverage and names,
not unrelated override bodies. Receipt-specific 1rU inherits
[1rA.A03](base-route.txt) and overrides A05. At0546 the inherited route sets the
receipt tag, calls Epa.A00 for a nullable diagnostic value and invokes virtual
A05 at0536. **Negative result:** [Epa.A00](attribute-hash.txt) is an attribute-map
hash (excluding offline), NOT a receipt status classifier or handler type enum.
The exported [base A05](base-a05.txt) is not the receipt override either.

[1kz initializer](incoming-type-enum.txt) binds A08 to RECEIPT/receipt.
The already retained [complete1s2 reader](../2.26.43.72-about-callback-lifetime/reply-dispatch.txt)
0348 maps the root tag through1uY, then selects receipt. At0432..0484 it uses
the receipt tag, absence of a keys child, non-retry type and flag0x382c to choose
one queued wrapper route; other conditions have their own routes. This is wrapper
selection, not proof that keys receipts are dropped or skip all receipt handling.
Its legacy route at104e..1072 looks up the handler by root tag, constructs1uU
and invokes A03. [Wrapper constructor callers](legacy-wrapper-callers.txt)
find only this1s2.run direct caller. Full provider/runtime map completeness is
not claimed; [1rU constructor callers](router-constructor-callers.txt) are1rT.AL0/AL1.

## Actual receipt branch order

[Complete1rU.A05](receipt-route.txt) reads required root id, type with absent
`delivery`, optional recipient/participant/offline/from and metrics before
building F0R.A03 context at0204. The [initializer](route-sets.txt) defines:

- ordinary set: delivery, inactive, sender, played, played-self, read, read-self;
- call child set: offer, accept, reject, enc_rekey;
- simple type map: server-error ->69, peer_msg ->221, hist_sync ->212.

The selected order after context creation is significant:

1. First child participants with a NONNULL message_id invokes ordinary/status
   forwarding at028e BEFORE testing the ordinary root-type set. Typed/string
   null-carrier equivalence is not inferred from Rust's attribute enum.
2. Delivery with a recognized call child has its separate call handling.
3. Delivery with biz/feature-gated bot metadata has a separate branch.
4. Ordinary type-set membership at0482..048a forwards to status processing.
   Feature-gated view/status handling is adjacent and is not collapsed into it.
5. Simple mapped types, retry (including category=peer), enc_rekey_retry and
   root type=error receipts (requiring an error child) each take their separate paths.
6. The remaining path at0768 calls2Cx.A00 with the EXISTING Eoh context.

[Private ordinary forwarder](ordinary-forward.txt) chooses message code1 or277
from the status selector and calls [2Ai.A00](forward-message.txt). That helper
puts the code into Android Message.arg1 and carries the incoming wrapper as obj,
plus the loggable stanza ID. Code1 aligns with retained [15H registration](../2.26.43.72-ordinary-receipt-parse-error-dispatch/ordinary-registration.txt).
Complete consumer/provider scheduling remains a separate runtime boundary.

## Unknown root type is not automatically a fresh parse error

For otherwise admitted ordinary PN/LID metadata with no special child/class/type
route, an unknown root type can reach the last unhandled branch. The retained
[complete2Cx.A00](../2.26.43.72-retry-ack-dispatch/unhandled-receipt.txt)
records local diagnostic490 but copies Eoh, adds wire error487, and submits via
the selected wrapper path. It preserves recipient/edit subject to original
casts/writer selection. This is NOT the [1rg.A03 fresh parse-error descriptor](../2.26.43.72-retry-original-chat/parse-error-context-reader.txt),
which drops recipient/edit and aggregate children.

A first participants/message_id child can instead forward the same unknown root
kind to F10.A03's offline branch, which does not run its ordinary root-kind
allowlist before that branch. A participants/key child alone does not trigger
this early selection. Thus downstream F10's explicit unknown-root LIL branch
is not enough evidence to NACK every unknown incoming root type as a parse error.
Earlier assumptions based only on that downstream branch are refuted by this
router. No blanket error response should be added from a type string alone.

## Consequence and retained implementation gap

Rust currently has selected ordinary fact/ACK allowlists and proven native
parse-error causes. It has not yet ported this complete receipt router or its
unhandled copy487 path for unknown roots. The next correction must classify
routing before response construction, preserve copied versus fresh fields,
respect preceding root/context/typed-address failures and special routes, and
test capture-before-write, no fact/query/resend/identity work and no uncertain
write replay. It must not treat every valid native special receipt as an unknown
or a malformed ordinary one. Readiness, feature state, wrapper scheduling,
class/provider/cache/binary completeness and live acceptance remain unverified.
This research adds no live request or account operation and makes no ban claim.

## Reproduction

```sh
APK=../android-lab/apks/WhatsApp.1.apk
sha256sum "$APK"
target/debug/xtask apk-callers "$APK" --class 'LX/14h;' --method BLn
target/debug/xtask apk-callers "$APK" --class 'LX/1uU;' --method '<init>'
target/debug/xtask apk-virtual-overrides "$APK" --class 'LX/1rA;' --method A03 --signature '(LX/1k2;)V' --include-interfaces
target/debug/xtask apk-smali "$APK" --class 'LX/1rA;' --method A03 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1rN;' --method A05 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1rU;' --method A05 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1rU;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1rU;' --method '<clinit>' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/2Ai;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/Epa;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1kz;' --method '<clinit>' --signatures --exceptions
target/debug/xtask apk-callers "$APK" --class 'LX/1rU;' --method '<init>'
```

checksums.json covers every other file. The A03 census summary is explicitly
derived; all retained method exports are complete. The existing reader, ordinary
parser, context, unhandled and writer evidence are linked instead of duplicated.
No APK/private state or bulk tree is included.
