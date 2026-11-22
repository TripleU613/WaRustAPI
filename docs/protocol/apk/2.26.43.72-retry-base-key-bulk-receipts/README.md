# Receipt-driven bulk retry base-key cleanup — APK 2.26.43.72

**Confirmed selected static.** Build264307238; pinned APK SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`. Eight complete method exports include signatures/exception tables, three caller inventories and one hash-checked negative construction probe. The existing complete F21.A05 is reused and reproduced rather than duplicated. These are repository xtask outputs, not REA Evidence. No Android execution, effective configuration, live database or account traffic was observed.

## Producer and receipt eligibility

The earlier [Fc1 selector17 bulk deletion](../2.26.43.72-retry-base-key-schema/bulk-delete-excerpt.txt) consumes address-to-message-key sets. [Constructor inventory](bulk-constructor-callers.txt) identifies19 invoke sites. Bounded local discovery found [F21.A00](bulk-map-producer.txt) constructing selector17; it does not prove every possible dynamic selector producer. A [complete-coordinator probe](coordinator-construction-probe.json) found no Fc1 reference in0WJ, so searching only the storage coordinator would miss this producer.

[F21.A00 callers](map-producer-callers.txt) are four sites inside F21.A05. Its [previous complete export](../2.26.43.72-receipt-null-fact-actor/legacy-apply.txt) matches the current extraction exactly; it is the receipt consumer already traced from the native parser/event in that bundle. [Current direct callers](receipt-dispatch-callers.txt) include15H.A04 and Fd4.run. This is the ordinary MessageStatusUpdateHelper path, distinct from the status-story and VoIP single-delete producers.

A00 iterates Emh items. [1e4.A03](received-target-predicate.txt) accepts exactly numeric codes5,8,13,15,17,18 and otherwise returns false. For that category, message B96==21 logs permanent-send-failure and skips target processing. Other receipt categories use a separate generic status-update branch. These are numeric predicates in this method, not a newly inferred complete wire-type table.

Selected target records are built by [EZd.A00](target-normalization.txt). If0xW.A08 is false and code13 or8 belongs to an outgoing key, it substitutes code5; otherwise it preserves the code. The [Em7 constructor](target-record.txt) retains device, additional user, original message, resulting code and timestamp. Full0xW settings provenance/effective values are not established.

## Device-keyed cleanup map and ordering

A00 groups target records by default EoY built from the complete DeviceJid (F3k.A04, previously traced). Each value is a LinkedHashSet of original1dW message keys. Collection requires from-me=true or [1dX.A0H](ownership-predicate.txt). That helper additionally accepts a selected type64/Dwf message with own-user BVC and A0z=false. Device identity and the full message key are retained; there is no bare-user/global-message-ID delete.

After collecting the cleanup map, A00 runs its distinct A03 processing, then selected message/status-store updates and bookkeeping. A nonempty cleanup map queues Fc1 selector17 under base_key_delete through0YM.A02 at0454. The [runnable constructor](bulk-runnable-constructor.txt) stores the map inA01 and F21 inA00, matching the earlier selector17 arm. That arm expands original/normalized addresses and executes EbJ's exact message/recipient deletion. The [executor trace](../2.26.43.72-retry-base-key-delete-triggers/delete-executor.txt) establishes ThreadPoolExecutor submission, not awaited completion.

The cleanup map is formed before later per-message result maps; this selected code does not filter it by every final returned result. Exceptions can still prevent reaching submission. No single outer transaction, cleanup cancellation on later failure, crash atomicity or committed-deletion-before-return guarantee is established. This is receipt-driven cleanup, not a timestamp TTL. Global/periodic retention remains untraced.

## Separate pending-prekey confirmation

[F21.A03](pending-prekey-confirmation.txt) is not the message_base_key table delete. It conditionally consumes Eoe's message-ID/device-keyed in-memory values, with EeT.A01 and feature0x48ce gating; selected namespace handling can map the device's user while preserving device index. For each retained address/base-key pair it acquires the coordination lock, loads the session under F22's monitor and checks presence of pendingPreKey plus its preKeyId. It compares the pending baseKey bytes with the retained bytes via Arrays.equals. Only equality calls [F1Y.A06](pending-prekey-clear.txt), which clears pendingPreKey and its presence bit, followed by0Yh.CrT storage. Mismatch/absence skips clearing; selected EDQ is converted to AssertionError. Shared outer transaction, all cache producers, persistence outcomes and effective flags remain unknown.

Do not replace this with unconditional pending-prekey removal on a receipt, erase ratchets, or conflate it with per-message retry-base-key cleanup. This trace establishes the selected native distinction, not a complete Rust implementation recipe. Rust base-key persistence, corresponding lifecycle integration and higher-count retry policy remain unported; counts above two remain refused. No runtime code, wire field, identity reset, replay or live operation changed.

Reproduce: `python3 docs/protocol/apk/2.26.43.72-retry-base-key-bulk-receipts/reproduce.py ../android-lab/apks/WhatsApp.1.apk`. The script checks the APK and every export plus the reused complete dispatch. checksums.json covers all bundle files except itself.
