# Media error response: retry producers and receipt context

**Confirmed selected static / research only**, APK2.26.43.72 build264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected xtask DEX fallback methods with full signatures and exception
ranges; PCs are bytes and branch displacements16-bit units. No execution,
new REA Evidence ID, account request or Rust runtime change. Continues the
[mode-input trace](../2.26.43.72-media-error-mode-inputs/README.md).

## Retry state is selection state and can survive reprocessing

The previous exact F1Z.A03 write census contains five sites. EuT.A01 was already
traced as setting true before later retry-job enqueue. The remaining four are
now bound to their selected producers:

| Direct writer | Selected source / condition | Result |
| --- | --- | --- |
| F2q.A01 at00b2 | nonzero EtY status; EtY.A00() returns true | A03=true before placeholder selection and later branches |
| F2q.A03 at00d6 | selected old-counter retry branch | A03=true; DqX.A0B=23; invokes A02 |
| F2q.A09 at014e | typed FSV, logged StopAndSendRetry | A03=true and A00=1 before subsequent processing calls |
| Fc1.run at088a | saved Dd3 bitField0 mask0x2 present | A03 restored from isSendRetryReceipt_ |

[Complete F2q methods](decrypt-retry-processing.txt) include A01/A02/A03/A04/A09.
A03's alternative branch explicitly logs RECEIVED_WITH_OLD_COUNTER and calls
0x0.A0a with500 rather than selecting retry; its choice also depends on A04(),
2L9.A1N and feature0x41a0. Their full meaning is not newly classified. A01's
EtY predicate body and complete status policy remain unknown here. A09 has
separate FSW/FSZ actions; do not turn every typed stop into retry or error491.
A02 branches on A03 to dispatch selected retry handling; this is not proof that
an outbound receipt reached a socket or was accepted. No reset-to-false writer
appears in the prior exact census except the restored metadata value, which can
be false. This does not rule out defaults, reflection/native writes or replacement
objects.

The complete Fc1.run body is already retained as
[identity-worker.txt](../2.26.43.72-identity-ack-dispatch/identity-worker.txt).
Its fresh export was byte-identical, so no duplicate was committed. At0848 it
reads1k8.A02 bytes, checks nonnull/nonempty, parses Dd3, and checks mask0x2 at087e
before restoring A03 at088a. Parse failure logs decode failed and continues
through its retained branch; an absent bit does not write A03. This is saved
container metadata, not an incoming server flag proven by a field name.

[Exact metadata field refs](retry-metadata-refs.txt) find two direct sites:
F2C.A00 writes Dd3.isSendRetryReceipt_, Fc1.run reads it.
[Encoder F2C.A00](retry-metadata-encoder.txt), in its selected v45=true branch,
reads F1Z.A03 at0058, sets metadata presence mask0x2 and copies the value at0074.
It serializes through DBe.A1a at00a8 into v25 and passes it as the second byte
array to1k8's constructor at013a. The
[container constructor](saved-container-constructor.txt) stores that argument
v35 into1k8.A02 at0078; the first array is separately A03. This closes the
selected encode/container/decode value chain. Storage transaction, fsync,
recovery atomicity and all encoder callers are not established by this trace.
No Rust persistence format or retry replay rule is changed.

## Receipt permission comes from decryption pass selection

[EmK.A04 writer census](receipt-context-writers.txt) finds its constructor alone;
[constructor](receipt-context-constructor.txt) copies its boolean argument at001e.
[Constructor caller census](receipt-context-callers.txt) finds Ed7.A00 alone.
[Ed7.A00](decrypt-callback-factory.txt) copies its supplied boolean v32 into v19
at0040, forwards it through v25 to EmK at0130, and wraps that context in FKc.
Other branches construct FKe/En5/EmL and must not be equated with FKc. This
narrows the prior EmK unknown to Ed7's caller selection.

[Exact Ed7.A00 caller census](decrypt-factory-callers.txt) has two invokes, both
F2q.A04. In [A04](decrypt-retry-processing.txt) at09da..09ea it obtains F1Z.A0A(),
checks it through2L9.A1W and XORs that result with1 to produce the first call's
sendReceipt at09fe. The [predicate](nonnull-predicate.txt) returns object!=null;
the [selector](secondary-ciphertext-selector.txt) reads Dxs.A0A or, for Dxr,
Enw.A01. Thus this selected first pass enables receipt permission exactly when
that secondary descriptor is absent. This is a descriptor-presence rule at the
selected invoke, not a blanket rule derived from plaintext MIME or local support.
A later conditional pass invokes Ed7 at0a84 with that descriptor and current v2
as the boolean. Its full control-flow value provenance remains unclassified here;
do not assume all callback construction has the first pass's permission value.
The semantic names and producer policy of the selected Ezj descriptors are also
not fully established. Full virtual/runtime resolution is outside direct census.

## Consequence and next proof

**Confirmed:** retry suppression can be restored from metadata; receipt permission
changes with selected decryption context. **Inferred:** a Rust implementation
that classifies only the final plaintext or only successful sends loses native
response-decision inputs. **Unknown:** secondary pass policy, native invalid-media
classification versus locally unsupported features, complete stop-result producers,
BX2 dispatch, direct writer routing and durable response ownership. Preserve
ratchet/capture ownership and do not add generic491 responses or repeat a retry
after an uncertain write. Next trace the secondary descriptor producer/pass and
saved-metadata transaction consumers, then compare the applicable Rust path.

Reproduce `apk-smali <apk> --class '<descriptor>' --method <name> --signatures
--exceptions` for EmK.<init>,Ed7.A00,F2q.A01/A02/A03/A04/A09,F2C.A00,1k8.<init>,
2L9.A1W,F1Z.A0A. Direct reference files use `apk-callers` or `apk-field-refs` with
exact class/member, field type Z and writes-only for EmK. Referenced earlier
bundles retain their own manifests. [checksums.json](checksums.json) pins this
bundle. No server acceptance or restriction/zero-ban guarantee is established.
