# Blocklist-triggered live-location disable request

**Confirmed selected static**, WhatsApp2.26.43.72 build264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected methods exported by xtask; no new REA Evidence ID, runtime
observation or live account operation. No Rust behavior changes.

## Cancellation requires an existing share record

The [preceding observer](../2.26.43.72-blocklist-location-obligation/README.md)
calls0zB.A02(jid,null,null,manager,02X.A0Y) after its conditional identity query.
A02's map lookup under A0c returns null immediately when no EVN exists
(0070..0078). The null second argument jumps0078 +0xb code units to008e,
setting v9=true. This selected caller therefore takes the matching-record path
without needing a supplied message key. No new blocked/unblocked Boolean is
examined here; reachability still depends on the upstream changed collection,
observer registration, existing state and query outcome.

For a PhoneUserJid selected by0Dn.A0d, EiZ/0Ws.A0E supplies the destination
mapping when nonnull. Missing mapping logs downgrade_to_pn and uses the original
identity. Other inputs also retain the original identity. This is a selected
producer rule, not authority to downgrade arbitrary Rust requests or synthesize
mappings. The EVN is removed under the original map key. Local model/state,
recipient-set, listener and scheduled-handler work follows; those helpers are
not fully reconstructed. A00 allocates a sequence while still under A0c.
After monitor exit and local callbacks, A0j may cause A0V, but either result
joins021e and queues SendDisableLiveLocationJob(destination,sequence).
A0j=false is not a send veto. Failure before reaching enqueue remains possible.

## Sequence and stale-job admission are separate

A00 reads live_location_sequence_number with default-1, obtains08q.A04()*1000,
uses that value when greater than the saved value, then adds1. It puts the long
through SharedPreferences.Editor.apply before returning it. For ordinary
non-overflowing values this is max(saved,clock_value*1000)+1. Actual clock
semantics, Java long overflow edge cases, cross-process/crash durability and
all sequence writers are not established; apply is not synchronous disk proof.
The job constructor stores rawJid/sequenceNumber. It binds the job parameter
A01 to rawJid, sets A03=true and invokes DCZ.A02 before constructing parameters;
full parameter semantics/dedup/retry policy remain open.

At execution A0L parses rawJid and returns on invalid input. It then calls
0zB.A0l(destination) and returns when sharing is currently enabled. A0l checks
the direct map and, for0Dn.A0k-positive identities, may check0Ws.A0K's alternate
identity. A missing EVN returns false. A0K(end,now) returns true for end==0 or
end>now; otherwise A0l calls A02 with the saved message key and expiry reason,
passes its result to A0C, and returns false. Thus this recheck can itself cause
expiry cleanup; it is not a pure Boolean read. A re-enabled unexpired share can
suppress an old queued disable. No generic retransmission rule follows.

## Exact selected outbound node and completion

A0L obtains a fresh0Sf.A0H request ID and constructs:

```xml
<notification id="REQUEST_ID" type="location" to="DESTINATION">
  <disable id="DECIMAL_SAVED_SEQUENCE"/>
</notification>
```

Outer ID and child sequence ID have different ownership and meaning. The child
uses Long.toString, not a fresh clock or the outer request ID. No extra namespace
or IQ wrapper is constructed in this method. Eoh registration fields are
A02=destination, A06=notification, A09=location and A08=request ID. It calls
0Sf.A0C(node,key,81) then untimed FfK.get, discards the returned object and logs
done on normal return. This proves selected construction/dispatch/wait order,
not socket/server acceptance or a universally untimed full operation.

Reuse [ACK API variant evidence](../2.26.43.72-ack-api-variants/README.md):
A0C normalizes the key, has readiness failure paths, and uses0Th.A01/2Gc0,
whose success wrapper intentionally discards reply content. Null get output
therefore does not prove no ACK; normal completion is not a blanket error-free
server response guarantee. Full job retry, ACK-reader classification and
crash/uncertain-send policy for this producer remain unknown.

## Rust boundary and reproduction

A bounded search of crates/whatsapp-api/src/transport for the job name,
live_location_sequence_number and literal location/disable found no matching
location-disable producer/state owner (unrelated ban-appeal location and
broadcast-name validation exist). That is not whole-repository absence proof.
Rust account_sync/blocklist still lacks native share/cache/sequence/durable
work authority. Do not add this packet on every blocklist delta, fabricate an
active share, copy native job retries in isolation, or ACK and discard the
notification. No supported Rust location feature or network correction is
claimed by this evidence-only chunk.

From repository root with the pinned APK and rebuilt xtask:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
OUT=/tmp/blocklist-location-disable-repro
mkdir -p "$OUT"
target/debug/xtask apk-smali "$APK" --class 'LX/0zB;' --method A02 --signatures --exceptions > "$OUT/cancel.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0zB;' --method A00 --signatures --exceptions > "$OUT/sequence.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0zB;' --method A0l --signatures --exceptions > "$OUT/recheck.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0zB;' --method A0K --signatures --exceptions > "$OUT/active.txt"
target/debug/xtask apk-smali "$APK" --class 'Lcom/whatsapp/locationsharing/jobqueue/job/SendDisableLiveLocationJob;' --method '<init>' --signatures --exceptions > "$OUT/constructor.txt"
target/debug/xtask apk-smali "$APK" --class 'Lcom/whatsapp/locationsharing/jobqueue/job/SendDisableLiveLocationJob;' --method A0L --signatures --exceptions > "$OUT/send.txt"
```

checksums.json binds every other bundle file by bytes and SHA256. Compare each
export byte-for-byte. Branch operands are code units: multiply by2 before adding
to displayed byte offsets. Complete selected bodies include their catch tables;
no bulk synthetic job tree is included.

## Lifecycle and retry eligibility follow-up

**Confirmed selected static:** lifecycle.txt completes the other six declared
methods, including private logging helper, readObject, added/canceled logs,
exception decision and provider injection. readObject calls defaultReadObject
and rejects null/empty rawJid with InvalidObjectException. It does not validate
sequenceNumber or fully parse the JID there; A0L performs the later parse.
A0K only logs cancellation; no socket cancellation is proven by that hook.
Cf0 restores137/0zB service references, not saved protocol acceptance.

A0N logs the Exception and returns true. Reuse the already-retained complete
[builder, JobParameters, runner and requirement check](../2.26.43.72-bulk-key-job-completion/README.md):
constructor parameters bind rawJid as groupId, persistence=true, retryCount100
and ChatConnectionRequirement. It calls DCZ.A02, not the separate A01 helper
that adds OfflineProcessingCompletedRequirement. Four freshly exported builder
methods matched their retained bodies. These parameters do not prove100 sends,
actual persistent-database durability, group serialization/dedup or restart
acceptance. connection-ready.txt proves BWq requires a nonnull0AI service and
returns true exactly when0AI.A0O returns true; its full readiness semantics
remain untraced here.

The retained Qlq runner increments attempt count on Exception; RuntimeException
is rethrown before A0N. Other exceptions reach this true decision and then
Job.A0M: satisfied requirements loop through the attempt-budget check;
unsatisfied requirements enter deferred/requeue handling. Every later A0L
attempt performs its current-sharing check and allocates a fresh outer ID,
while retaining the job's sequenceNumber. The callback-result-discarding
contract remains distinct from exception eligibility. No explicit delay is
present on that selected in-runner loop; global scheduling/backoff remains
outside this claim. Native retry eligibility does not authorize replaying a
Rust durable uncertain send or rotating its ID to escape uncertainty.

Reproduce the additional retained files with the same APK/OUT variables:

```sh
target/debug/xtask apk-smali "$APK" --class 'Lcom/whatsapp/locationsharing/jobqueue/job/SendDisableLiveLocationJob;' --signatures --exceptions > "$OUT/class.txt"
target/debug/xtask apk-smali "$APK" --class 'Lcom/whatsapp/infra/jobs/requirements/ChatConnectionRequirement;' --method BWq --signatures --exceptions > "$OUT/connection-ready.txt"
python3 - <<'PYCODE'
from pathlib import Path
root=Path('/tmp/blocklist-location-disable-repro')
lines=(root/'class.txt').read_text().splitlines()
selected=['# Complete selected lifecycle methods; constructor and send method retained separately.']+lines[:4]
keep=False
for line in lines[4:]:
    if line.startswith('.method '):
        keep=any((' '+name+'(') in line for name in ['A00','readObject','A0J','A0K','A0N','Cf0'])
    if keep:
        selected.append(line)
    if line=='.end method' and keep:
        selected.append('')
        keep=False
(root/'lifecycle.txt').write_text('\n'.join(selected)+'\n')
PYCODE
```

Remaining gap: complete deferred-job persistence/scheduling and final ACK
failure classification, native active share/sequence authority and Rust recovery
integration. No production retry policy changed. This closes selected retry
eligibility, not the broad network correctness audit.
