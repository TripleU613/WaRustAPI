# Blocklist recovery: snapshot adoption, mode and MEX obligation

Confirmed selected static, APK2.26.43.72 build264307238; SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected DEX methods/catches/census, one selected persist-ID export and
[complete REA manager Evidence](../../rea/2.26.43.72-blocklist-migration-obligation/README.md).
No blocklist, account, APK or network operation ran. No Rust behavior changed.

## Close the selected ordinary adoption worker

Prior [8R7 ->7vW ->8ak](../2.26.43.72-blocklist-set-reconciliation/README.md)
left8bU unresolved. The genericObject constructor binds selector to$t, manager
toA00, supplied set toA01, metadata toA02 and hash toA03. Selector0 at0cae
calls1dH.A0O(null,metadata,hash,set) directly. This closes the equal-c_dhash
selected snapshot-adoption handoff, not full callback/account concurrency.
Other run selectors remain in the complete export but are not interpreted as
this operation. Selector1's dirty path is separate, below.

## Addressing mode is persisted authority, not inferred from mappings

3BU.A00 stores String.valueOf(Boolean) under a0aj.A08-generated key using the
literal is_blocklist_lid_based and namespace02X.A0j. A01 reads that key via
0aj.A0a: empty string returnsfalse; otherwise Boolean.parseBoolean. The full
0aj storage/key namespace, provider/account identity and current value are not
reconstructed; compiled defaultfalse is not an effective-state observation.

6pA.A02 delegates to this mode. The prior A03 admits directly when mode is true;
its false-mode migration checks remain separate. Four direct3BU.A00 invocations
are3Wl.Bk5,6p9.CK4,6pA.A01 and88m.A00. This is bounded direct-call coverage,
not every alias/native/reflection writer. Selected6pA.A01 clears the flag then
calls0W9.A03, whose consequences remain untraced. No Rust marker is invented
from the existence of a LID, PN/LID association, response name or success log.

## Dirty reconciliation merges and resolves before a different request

8bU selector1 (0b04) takes the supplied set, preserves entries that are0Dx,
and uses0Ef.A06 plus07W.A0M to form a union with the supplied iterable.
This helper adds both inputs; it is not subtraction/intersection. The prior
6p9.CK4 caller supplies878.A01.keySet, not an assumed local blocked cache.
The retained878 constructor binds its first map toA01; current6qD parser
passes the pn_jid association map there. Full association adoption remains
separate from these parsed keys.88m.A01
requests selected missing mappings through3KB.A02/A00, requiring1xM.A01=true;
remaining mappings/contact diagnostics andDCH.A01 are separate. Complete nested
lookup policy and source freshness are not established by this selected trace.

After0Ws.A0N lookup, it retains0Dx values plus the original0Dx entries and
calls88m.A00(hash,set,isRollback=false,dirtyAck=true). The selected catch is7gZ;
full task executor/retry scheduling and all error propagation remain unknown.
This is not replay of the original per-contact block/unblock IQ.

## Current MEX request and response binding

88m.A00 creates8ht using its7qm-typed constructor. Its matching constructor fixes$t=0
and preserves hash, set and finalBoolean. invokeSuspend's selector0 obtains
maps via0bY.AZS,0Ws.A0V and0bC.A0U, then builds GraphQlCallInput entries with
jid plus nested identifier. Initial setter keys include display_name,
guest_name,pn_jid,unknown_identifier,username; later conditional setters use
mapping/contact/record values. Full provider authority, exact field-choice
semantics and null-versus-omitted wire encoding remain open; do not send an
empty fabricated identifier object or assume every key is emitted.

The outer input sets dhash and blocklist. A true captured Boolean adds a
dirty_ack=true setter call.6oj.A0P wraps it under input. The0iT descriptor is
MigrateBlocklistLid in whatsapp-android-mex; dispatch reaches0iq through the
selected generic coroutine helper. Full final IQ/HTTP envelope, writer admission,
callback lifetime/deadline and replay policy are not traced here. Neither the
20s classic block/set timer nor a generic32s timer is copied into this request.

queries.json is the pinned asset selection (exact integers exported as strings):

| Field | Value |
| --- | --- |
| server doc_id | 25028600226770432 |
| client_doc_id | 12789525189554463941782259879 |
| operation_name_hash | 1278952518 |
| operation_text_hash | 9554463941782259879 |
| schema | whatsapp_mex |

Asset is assets/whatsapp-android-mex_client_persist_ids.json, SHA256
`a30a4b3ce55150747820bbf353e5d2e742fde19e2a11dd3e43b094d7088ae797`.
These are public release identifiers, not account credentials or live results.

Response code selects xwa2_migrate_blocklist_lid, requires a nonnull blocklist
collection, parses each jid through0Dx's factory (logs/skips null results), and
constructs7Da(metadata made of empty maps,dhash,set). Missing selected structure
returns7Db(null); caughtGiw becomes7Db(error). The retained7Da constructor binds
all three fields unchanged. Parsing a typed result is separate from safe state
adoption and complete validity of every skipped element.

Manager88m.A00 requires7Da;7Db becomes7gZ. Only isRollback=false writes the
mode true AND invokes1dH.A0O with returned data. isRollback=true skips BOTH
writes, but still reaches its completion log. The log is not proof that mode or
cache changed. Successful submission, parsed response, mode persistence, shared
set adoption and durable/account-safe commit are distinct boundaries.

## Rust comparison and remaining work

Rust retains matched=false cache uncertainty and its visible reconciliation
warning; it has no verified addressing-mode adoption, MigrateBlocklistLid
dedicated sender, native mapping merge or complete migration callback lifecycle. Current
manual full-fetch support is not declared equivalent to the native migration
operation. No normal action is reset/requeued and no live operation is added.

A bounded Rust source/fixture census for MigrateBlocklistLid, its response root,
is_blocklist_lid_based and dirty_ack found no matches in crates/**/*.rs/json.
The inspected transport/mex.rs exposes the group-info constructor, not this
operation. This is not a claim that generic raw-node facilities cannot send
arbitrary data; it establishes no dedicated implementation for this obligation.

This closes selected worker/mode/request/response links and adds a named outbound
request obligation. Full identifier authority/serialization, runtime mode,
nested lookup/adoption, scheduler/deadline, callback/account generation and
native rollback semantics remain open before implementation. No account-ban
cause or prevention guarantee is inferred from this static evidence.

## Reproduction

From repository root with pinned APK and current built xtask:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
OUT=/tmp/blocklist-migration-obligation-repro
mkdir -p "$OUT"
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'LX/878;' --method '<init>' --signatures --exceptions > "$OUT/metadata-init.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/8bU;' --method 'run' --signatures --exceptions > "$OUT/adopt-worker.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/8bU;' --method '<init>' --signatures --exceptions > "$OUT/adopt-worker-init.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/3BU;' --signatures --exceptions > "$OUT/mode-state.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0aj;' --method 'A0a' --signatures --exceptions > "$OUT/mode-read.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/6pA;' --method 'A02' --signatures --exceptions > "$OUT/mode-enabled.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/6pA;' --method 'A01' --signatures --exceptions > "$OUT/mode-rollback.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/88m;' --method 'A00' --signatures --exceptions > "$OUT/migration-submit.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/8ht;' --method 'invokeSuspend' --signatures --exceptions > "$OUT/migration-coroutine.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/8ht;' --method '<init>' --signatures --exceptions > "$OUT/migration-coroutine-init.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/7qm;' --signatures --exceptions > "$OUT/migration-helper.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/6oj;' --method 'A0P' --signatures --exceptions > "$OUT/input-wrapper.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0Ef;' --method 'A06' --signatures --exceptions > "$OUT/merge.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/07W;' --method 'A0M' --signatures --exceptions > "$OUT/merge-add.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/88m;' --method 'A01' --signatures --exceptions > "$OUT/lookup.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/7Da;' --method '<init>' --signatures --exceptions > "$OUT/result-init.txt"
target/debug/xtask apk-callers "$APK" --class 'LX/3BU;' --method A00 > "$OUT/mode-write-callers.txt"
target/debug/xtask apk-mex-queries "$APK" --operation MigrateBlocklistLid > "$OUT/queries.json"
```

checksums.json binds every other bundle file by bytes/SHA256. Existing complete
sources remain linked in their original checksum bundles. No raw APK or bulk
class dump is committed; full selected synthetic methods retain all selectors
so their discriminator/branch claims can be checked.
