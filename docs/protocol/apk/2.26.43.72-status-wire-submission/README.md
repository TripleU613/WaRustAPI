# Status job rendering and submission — APK 2.26.43.72

**Confirmed selected static**, build264307238; pinned APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected DEX methods via xtask fallback, not REA Evidence or runtime
execution. No account operations. The onRun method is retained completely;
only selected persistence/render/submission/wait paths are claimed traced.

## Persisted hash and wire-attempt boundaries

SendE2eStatusJob.A0L contains the actual status send pipeline. The selected
part-hash path at1aac..1b0e checks operation/part state and persists
stanza_devices_hash in status_send_part with row_id,partcode, prior-state and
parent statusUUID predicates. Failure paths report missing/failed persisted
part hash instead of treating construction as successful. Exact feature/target
selection and every failure return remain unported.

After buildingEng but before0Sf submission,3d6a calls privateA0E(targetSet).
For its selected batched/backfill predicate and nonnullF2a part suffix, A0E
requires a nonempty target set and owns an outer transaction. It writes literal
Long0 (3q4.A0d) toreceipt_device_timestamp only for matchinguuid,part_code and
recipient JID rows whose timestamp isNULL or<0. SQL anchor is
MARK_BACKFILL_WIRE_ATTEMPTED; batches use0x3cd(973) target chunks. It commits,
closes DB and invokes a separate callback only if rows changed. This marker
precedes submission and is NOT a delivery timestamp or proof of socket write.
No Rust receipt timestamp schema or resend policy is inferred.

## Selected root attribute renderer

At37ce..3a1e, params and prepared values feedEwq/EmY; EmY's constructor binds
messageType,phash,addressingMode,bucketing,time,disableDeviceFanout to fields.
At3a26..3b3a the renderer emits:

| Attribute | Selected condition/value |
|---|---|
| t | nonzero prepared time, converted by2LB.A06 (time domain/scaling not traced here) |
| to/from | to when1dW.fromMe or edit8; from otherwise |
| type | preparedEmY message type |
| id | original1dW id |
| phash | nonnull prepared hash |
| bucketing | nonnull prepared value |
| addressing_mode | nonnull prepared value and not0Dn.A0V(selected JID) |
| participant | nonnull prepared DeviceJid |
| recipient | nonnull prepared UserJid |
| edit | nonzero edit integer, String.valueOf |
| device_fanout | literalfalse whenEmY.A05 istrue |

The disable flag comes fromparams.A0B, which combines a set helper andA04/A0D
operation predicates. Those predicate/value producers remain incomplete: this
is NOT a universal requirement to emitdevice_fanout=false. It does not justify
adding optional attributes with invented or blank values to Rust sends.

Renderer extensions invokeEwq.A03 and selectedFvG implementations, then optional
metadata helpers. Ewq.A02 finds/rebuilds a named child while preserving its bytes/
children, or adds one. Selected calls setmeta session_scope,status_h,status_ots
underEF8/value-presence conditions. Exact session-scope/hash/original-time
producers and extension behavior remain untraced. Root and child metadata must
not be conflated or assumed mandatory merely because their names occur here.

Ewq.A01 wraps the prepared addresses,key,Eoh,children,attribute map and counters
asEng. Its large-stanza path logs diagnostics then constructsEng: no hard size
rejection is established by that branch. The disassembler's unknown0xb3 in
logarithmic diagnostic scale arithmetic is retained and not interpreted.
Existing Rust payload limits remain local policy, not native limits proved here.

## Submission and ACK wait are separate evidence

At38a8..38f2 the job builds expectedEoh from prepared routing,id,classmessage and
nonnull/nonzero edit. AfterA0E it obtains Android Messagewhat0/arg1=8 containingEng,
submits via0Sf.A0B(Message,Eoh), then callsA0P(expectedEoh,future). The final
queue/writer byte encoding, actual socket write, callback ownership/fallback,
ciphertext transaction and complete postwrite effects remain untraced.

A0P checks16j.A0U feature: enabled usesFuture.get(300000ms); disabled usesget()
without that local timeout. OnTimeoutException it calls0Sf.A0T(expectedEoh,error),
marksEsB.A04 and logs/rethrows. Other exceptions mark that field depending on
E5W/E5X cause before rethrow. No reconnect/replay or universal5-minute server
requirement is inferred. Existing Rust configured timeouts remain unchanged.

## Reproduce

From checkout root with pinned APK outside Git; compare complete output bytes:

```sh
cargo build --locked -p xtask
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'Lcom/whatsapp/productinfra/status/sendflow/SendE2eStatusJob;' --method A0L --signatures --exceptions > send-job.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'Lcom/whatsapp/productinfra/status/sendflow/SendE2eStatusJob;' --method A0E --signatures --exceptions > pre-submit-marker.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'Lcom/whatsapp/productinfra/status/sendflow/SendE2eStatusJob;' --method A0P --signatures --exceptions > ack-wait.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/3q4;' --method A0d --signatures --exceptions > marker-zero.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/EmY;' --method '<init>' --signatures --exceptions > attribute-values.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'Lcom/whatsapp/productinfra/status/sendflow/E2eStatusJobParams;' --method A0B --signatures --exceptions > disable-fanout-predicate.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/Ewq;' --method A01 --signatures --exceptions > final-envelope.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/Ewq;' --method A02 --signatures --exceptions > child-metadata.txt
```

Displayed offsets are bytes, branch deltas code units. Existing sender-key stanza
regressions check current Rust emission, not full native renderer parity. Next
work is metadata/operation producers and actual0Sf registration/writer admission.
No runtime change, live operation, replay authority or no-ban claim.
