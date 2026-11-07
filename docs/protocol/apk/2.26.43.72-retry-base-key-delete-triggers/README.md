# Retry base-key delete producers — APK 2.26.43.72

**Confirmed selected static.** Build264307238, pinned APK SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`. Complete method exports include exception tables. Bounded excerpts carry checked full-source byte lengths/hashes, explicitly omit other instructions and exception tables, and do not stand for entire runnable bodies. All exports are repository xtask outputs, not REA Evidence. No Android execution, runtime flags, database outcomes or account traffic were observed.

This follows the [two direct single-delete arms](../2.26.43.72-retry-base-key-addresses/README.md), which invoke0WJ.A0r through Fei selector15 or Feq selector33. It resolves selected producers, not every base-key deletion route or global retention policy.

## Status-state producer

[Ens.A01](status-input.txt) consumes FvR updates: per-item device, timestamp, message key and receipt-code getters, with address mapping, status-key construction and batching. It calls [Ens.A02](status-state-batch.txt), whose logs explicitly identify StatusStateManager/processStatusStateUpdateBatch. Do not label this ordinary chat delivery handling merely because the input contains receipt codes.

A02 opens a local database/transaction scope, loads stored status state, and has multiple guards for missing state, permanent failure, target/device eligibility, receipt codes and existing state. In its selected update continuation it constructs the device's default session address, passes the original Dwc/F1A-compatible status key and Ens into Feq with selector33, and submits base_key_delete through0YM.A02 at052e..0538. The [constructors](status-runnable-constructor.txt) retain the field mapping for the generic object overload; the prior retained Feq33 arm reads F1A.A01 and delegates to0WJ.A0r.

Scheduling is inside the status-state transaction: the later normal path calls1hk.A00 at0592, closes the transaction handle, then closes the database. The enqueue is earlier. This does not prove deletion executes after commit, rolls back with status-state changes, or is cancelled if a later instruction fails. Selected receipt-code comparisons include13/17, but a complete semantic table for every state/code and effective flag is not established here. This is conditional status-state cleanup, not evidence that every delivery/read/played receipt must delete a key.

## VoIP enc_rekey receipt producer

[Feo.run excerpt](voip-receipt-excerpt.txt) retains switch selector46 entry, the Android Message.arg1 dispatch and the selected receipt branch. The c5/197 dispatch reaches ETr at10e4. Its VoipStanzaChildNode tag is compared with enc_rekey; only that selected tag enters cleanup construction. It uses the ETr device's user, literal call_rekey and boolean true through [DBb.A0g](rekey-message-key.txt) to construct1dW, and [F3k.A04](default-address.txt) to construct the device's default session address (E78.A02/E7N.A03).

It obtains F2A.A0P's0YM executor, builds Fei selector15 with address/key/F2A through the [factory](runnable-factory.txt), and queues base_key_delete at11a8. The [constructor](runnable-constructor.txt) stores address/key/F2A in the fields read by the prior Fei15 arm. After scheduling, control invokes handleIncomingSignalingXmppReceipt through VoipNativeImpl.A0o. This proves enqueue-before-native-invocation ordering, not completed or committed deletion before native processing. No TTL or ordinary-message ACK deletion rule follows.

## Execution and bounded discovery

[0YM.A02](delete-executor.txt) wraps a named Runnable with0YO.A01 and calls ThreadPoolExecutor.execute. It does not await a result or inspect a timer. Full queue ordering, worker count, wrapper behavior, crash handling and database contention are untraced.

The [factory inventory](factory-callers.txt) has39 invoke sites across36 distinct methods; constructor and alternate submit-helper inventories are also retained. A local discovery scan of those factory methods located the positive Feo path; this is not exhaustive selector dataflow or proof no dynamic caller can supply15. Direct F2A class inspection did not reveal the construction; its delegate is reached from the outer Feo handler. The alternate [submit helper](runnable-submit-helper.txt) constructs the same runnable and invokes08Z.CbD. Inspected Euf.A02, Eu4.A01, FeA.run and FUY.CK4 sites instead supply selectors4,38,46 and42 respectively: [Euf](other-selector-euf.txt), [Eu4](other-selector-eu4.txt), [FeA excerpt](other-selector-feA-excerpt.txt), [FUY excerpt](other-selector-fuy-excerpt.txt). These bounded negative sites are not cleanup producers.

Ordinary chat base-key cleanup, bulk-map producer/trigger, periodic TTL, all status branches, complete alias migration and crash behavior remain open. No Rust base-key persistence, higher retry counts, wire fields, replay or live operation changed. Counts above two remain refused.

Reproduce with `python3 docs/protocol/apk/2.26.43.72-retry-base-key-delete-triggers/reproduce.py ../android-lab/apks/WhatsApp.1.apk`. The APK and complete-source identities are checked before matching exports/excerpts. checksums.json covers all bundle files except itself.
