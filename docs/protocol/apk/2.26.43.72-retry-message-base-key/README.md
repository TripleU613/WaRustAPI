# Retry message base-key storage — APK 2.26.43.72

**Confirmed selected static.** Version code264307238, APK SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`, rechecked locally for this pass. Thirteen complete selected checked-DEX method/caller exports, with signatures and exception tables for methods. These are repository xtask outputs, not REA Evidence. No Android execution, traffic or server acceptance was observed.

## Selected ordinary-receipt branch

[163.A04](retry-session-gate.txt) obtains the held session through 0WJ.A14/A0M and copies its `Dha.aliceBaseKey_` through [EZK.A00](session-base-key.txt) and [DBd.A1a](byte-string-copy.txt), which calls ByteString.toByteArray. This proves the stored bytes' field provenance; no fixed length, curve encoding or prefix is established by these getters.

At0032, a remote-registration mismatch follows cleanup and returns the **old** base key at0090. It does not enter count-2 recording. A missing session jumps to0136 and returns null. Matching registration reaches009a:

| Selected count/condition | Selected result |
| --- | --- |
| count<=2, count!=2 | null return, no A0s call |
| count==2 | A0s(address, message key, current base key), then null |
| count>2, A15 returns true | return current/old base key through the wait branch |
| count>2, A15 false, address integer A01!=3 | null, no recording |
| count>2, A15 false, A01==3 | 32i.A03 flag chooses wait/base-key return or null |

PCs are bytes; printed branch deltas are code units. At009c, +0x31 targets00fe; at00a8, +0x11 targets00ca; at00b2, +0x42 and at00c6, +0x38 target0136. Do not call A01==3 a particular cryptographic algorithm without tracing that discriminator. Effective 32i flags remain unknown. The returned-key consumer and deferred requirements are retained separately in [retry query/ACK evidence](../2.26.43.72-retry-query-ack/README.md).

## Persistence and comparison

[0WJ.A0s](save-message-base-key.txt) creates a HashSet containing the original address and the result of 0YG.A02(...,"saveMessageBaseKey","message_base_key"). Equal addresses deduplicate. It passes the selected set through 0Wk.A07/A08 before entering the selected local database transaction scope. For each set member it writes table `message_base_key` through 0KU.A08 with:

- `msg_key_remote_jid`: message-key remote JID raw string;
- `recipient_id`, `recipient_type`, `device_id`: EoY.A04/A01/A00 of that member;
- `msg_key_from_me`: message-key A02 boolean;
- `msg_key_id`: message-key A01 string;
- `last_alice_base_key`: supplied bytes;
- `timestamp`: selected clock value divided by1000.

Selected normal completion calls 1hk.A00, closes transaction/database handles and calls 0Wk.A03 on the coordination set; exception handlers are retained. The full set-coordination helper semantics are not traced in this bundle. Clock source details and crash durability are not established here. Selected replacement API, fresh schema index and explicit deletion were subsequently traced in [the schema follow-up](../2.26.43.72-retry-base-key-schema/README.md); global TTL and actual migration outcome remain unknown.

[0WJ.A15](compare-message-base-key.txt) locks original and normalized addresses and selects `last_alice_base_key` using normalized recipient ID/type/device, message remote JID, message ID, and from-me (`!=0` for true, `=0` for false). It reads at most the first cursor row. No blob/null cursor/no row yields false. A present blob is compared with the currently loaded session's alice base key using Arrays.equals, and that boolean is returned. No local nonempty/fixed-length guard or ordering clause is established by this method; selected fresh-table uniqueness is traced in the schema follow-up, without proving the state of any live database.

[0YG.A02](address-remapping.txt) only considers its selected address-type integer0 for remapping. It uses phone-user lookup, 0x536a and address predicates, and may select 0xC/0Dw paths while preserving device/scope fields via F3k.A06. The selected missing-mapping path invokes a diagnostic before returning the original address; other early branches also return the original address. Effective migration flags and complete mapping policy remain unknown. Do not replace the saved record with a bare message-ID hash or assume only the original address is stored.

## Call inventory and limits

The direct DEX invoke inventory finds three callers each for [A0s](save-callers.txt) and [A15](compare-callers.txt): 163.A04; [location wrappers A0n/A11](save-location-wrapper.txt), which use `location_msg_id` with from-me=true; and [FfB.call](voip-peer-failure-call.txt), whose retained strings and calls identify a VoIP peer-decryption-failure path. The VoIP method also records at count2 and compares at higher counts under its own session checks. It is not evidence of ordinary fresh-bundle completion. Reflection/native/dynamic call completeness is not claimed.

**Inferred implementation consequence:** count2 recording and higher-count comparison are linked local state, not an extra outgoing stanza attribute or unconditional count2 network fetch. Rust still lacks this per-message/address base-key record, and counts above two remain refused. Do not enable them from this storage trace alone.

**Unknown:** full receipt-key/session replacement flow, every alias/migration branch, address discriminator semantics, effective flag values, full schema migration/TTL, full job reentry and native/runtime acceptance. No Rust runtime change in this bundle.

Reproduce all exports with `python3 docs/protocol/apk/2.26.43.72-retry-message-base-key/reproduce.py ../android-lab/apks/WhatsApp.1.apk` after building the existing xtask. `checksums.json` covers all retained files except itself.
