# Retry key-query wire audit — 2026-11-05

**Confirmed static / bounded baseline match:** pinned APK2.26.43.72 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
[Retained manager/job/writer chain, exact gates and reproduction](apk/2.26.43.72-retry-key-query-wire/README.md).
No native runtime, account traffic or production Rust change.

The selected deferred retry job's prekey manager createsBulkGetPreKeyJob, which
uses Android Message87 to reach0v0.A00's key-query writer. The baseline is root
id/xmlns=encrypt/type=get/server to, key child, user/@jid per device. Rust's
current plain query matches this classic, non-identity, context-disabled branch.

Native retry source12 is a local job/request diagnostic input, not a selected
IQ attribute. A registration mismatch does not by itself put the requested JID
in identityJids: reason=identity is driven by that explicit set. PQ upgrades use
a separate reason=pq branch and require fetchPqPrekey. pqsupport=true is conditional
on that fetch flag. context_jid is gated by0x6530, nonnull context and JID-type
predicates whose complete mapping/effective runtime values remain unknown.
No unconditional extra fields are justified by this audit.

The manager suppresses in-flight device requests and clears entries strictly
after uptime+60000, not at equality. This is not proven to be the IQ callback
deadline or authority to replay a held query. Native large-list splitting uses
size/config predicates, not an established server maximum; Rust's32-device bound
remains local. Manager drain/partitioning, TEE/PQ/alias/context selection,
backoff/response deadlines and full recovery remain unfinished.

This finding closes the selected manager-to-serializer construction link. It
neither completes all outbound key queries nor changes ACK, backoff, identity,
query replay or live acceptance policy.

Validation: two filtered key-request construction tests passed. APK coverage,
staged whitespace and research checksum/link hygiene passed. No Rust source
changed, so the full workspace suite was not rerun for this research chunk.

### 2026-11-05: current prekey error backoff has scoped state

**Confirmed static, APK 2.26.43.72:** non-PQ error selector 76 moves affected
existing in-flight entries to pending only for 500..599. A01 mode and A00 posted
flag control scheduling; the 10/610 generator starts at zero delay. Delayed drain
clears posted flag but retains mode; explicit reset differs. Complete normal
admission does not establish a blanket stop of all key queries. **Unknown:**
network-to-listener attribution, external reset authority and durable job/write
recovery. No Rust retry, admission or reservation policy changed.
[Exact anchors, source transitions and delay derivation](apk/2.26.43.72-prekey-error-backoff/README.md).
