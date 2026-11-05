# Typed sender-key future-counter retry

**Confirmed static:** APK2.26.43.72 code264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reuses [native future-counter trace and integer-domain limits](../2.26.43.72-sender-key-future-counter/README.md)
and its linked complete Eyj/0Yl/EtY/F2q/DxB exports. Exact future-counter message
selects -1202 -> internal10 -> wire6. No new APK exports, REA Evidence envelopes,
target execution or account traffic.

## Read-only effective matching-state view

Follow-up source inspection found an independently validated state view without
changing the dependency. SenderKeyRecord::deserialize already applies normal
reservation recovery, so the loaded record is the same effective position used
by group_decrypt. serialize(&self) emits its current states in order and appends
a local reservation field; it does not perform another ratchet advance. Exact
backend/schema identities and source facts are in source-identity.json.

The Rust adapter serializes this effective record only on a relevant failed
decrypt, keeps the temporary bytes in Zeroizing, and decodes a metadata-only
prost view: record.states field1, state.id field1 and chain field2, iteration
field1. Secret fields are skipped. First matching chain ID is selected in saved
order, including optional ID default0, matching backend lookup. This does not use
only the front state or export components. No state is changed or persisted.

The classifier runs only after backend InvalidMessage(SenderKey,...). In the
pinned backend these producers follow successful signature verification; future
counter and CBC failures share that enum. It reparses typed SenderKeyMessage and
checks the exact matching effective chain position. Both counters must be within
0..i32::MAX, subtraction must succeed, and difference must be >2000. Thus native
signed-int subtraction matches backend arithmetic in the mapped domain. It does
not inspect provider error strings and does not guess a cause when metadata or
integer-domain equivalence is missing. Boundary2000 stays with ordinary decrypt/
CBC handling; high-bit unsigned inputs remain generic and their native parity
is unresolved.

**Inferred compatibility mapping / implemented offline:** a confirmed excessive
jump becomes fixed redacted Error::SignalSenderKeyFutureCounter, then inbox
SenderKeyFutureCounter and private RetryCause::SenderKeyFutureCounter -> error6.
Signature failures still select error5 before any counter classification. No
MAC/plaintext authentication is claimed for the rejected payload. No delivery
ACK or ratchet/event commit is authorized by the retry cause. Existing count1,
original timestamp, routing, durable reservation and uncertain-write no-replay
behavior are unchanged. Offline CLI replay recognizes the outcome and sends nothing.

## Regression and remaining scope

cargo test --locked -p whatsapp-api --all-features future_counter
checks signed messages with deliberately invalid one-byte CBC payloads: jump2000
reaches generic CBC failure, jump2001 is typed, older matching state is selected
instead of newer front, invalid signature wins, reservation-adjusted positions
are honored, and high-bit input is left unmapped. These are actual backend calls,
not APK execution. No new high-bit/skipped-key recovery behavior is claimed.

Production-loop test undecryptable_message_requests_once_before_write_cut_and_never_replays
creates a correctly signed future message and checks error6/original timestamp,
reservation before failed write, no second retry/delivery ACK on reconnect and
unchanged sender-key rows. Existing missing-session and signature cases retain
errors1 and5. Complete high-bit/overflow, other primitives/producer cases, key/count
progression, routing/ACK policy and server acceptance remain open.

Validation:1329 workspace/all-feature tests passed. Formatting, strict workspace
all-target/all-feature Clippy, CLI reference regeneration, APK coverage, staged
whitespace and research checksum/link hygiene passed. Offline only.
