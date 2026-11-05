# Classic incoming local prekey retry cause and lookup ordering

**Confirmed static**, APK2.26.43.72 code264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reuses complete checked-DEX [F22.A04 local-key producers and matching-session
reuse](../2.26.43.72-group-counter-error-paths/README.md),
[0WJ.A0O catch table and EtY/F2q/DxB](../2.26.43.72-outgoing-retry-error-map/README.md).
No new APK extraction/execution or REA Evidence envelope; no live account traffic.

F22.A04 first looks for matching current/previous session version/base key,
promoting an older match if needed; matching paths reach decrypt without loading
local signed/ordinary prekeys. New bootstrap then loads signed key, and optional
ordinary key. Missing/invalid local records produce EDR at0570..05b2 and05e8..062c.
0WJ.A0O catches this EDR as -1003, EtY admits it, F2q internal3 and DxB wire3.
Missing PQC key instead uses EDQ; it is not covered by this classic mapping.

## Adapter ordering and typed cause

Backend source identity/facts are retained. Its process_prekey checks identity
trust, then promotes matching session before local key-store lookup. Rust's
classic adapter formerly eagerly deserialized both supplied local records before
that decision. A corrupt unused record could therefore reject a pending prekey
envelope that the held session could decrypt. The adapter now borrows record
bytes and decodes/validates ID only when get_pre_key/get_signed_pre_key is called.
Missing, mismatched-ID or malformed selected record returns the backend's typed
InvalidPreKeyId/InvalidSignedPreKeyId at that lookup. No durable mutation occurs.

**Inferred provider equivalence / implemented offline:** only classic incoming
message_decrypt_prekey maps those selected local-key errors to the fixed redacted
SignalIncomingLocalPrekey. Direct and classic pairwise-group inbox boundaries
preserve IncomingLocalPrekey; normal receive chooses RetryCause::IncomingLocalPrekey
-> retry/error3. Outgoing bundle diagnostics, hybrid/KEM processing and unrelated
Signal failures are unchanged. No provider string or key identifier is logged.

The registration factory's signed-material availability/identity/signature checks
still occur before the normal inbox enters this adapter. This change does not
relax that gate, treat database faults as retryable keys, or solve every missing/
corrupt local-material path. Later key-pair integrity errors and full native
hybrid/direct producer equivalence remain separate open work.

## Offline tests and corrected MAC fixtures

The direct real-crypto test checks absent ordinary key, corrupt selected ordinary/
signed records and a follow-up pending prekey envelope using the matched saved
session while both unused local records are corrupt. It authenticates the follow-up
plaintext without consuming a prekey. This proves on-demand lookup rather than
merely testing a new error label.

The production-loop uncertain-write test adds missing ordinary key, requested
signed-ID mismatch and a classic pairwise group prekey case. It constructs real
synthetic prekey envelopes, asserts error3/original timestamp, then confirms no
repeated retry/delivery ACK after reconnect and unchanged local prekey rows.
Existing missing-session/signature/future/missing-chain cases retain1/5/6/4.
Offline CLI replay recognizes the result and sends nothing.

Three earlier MAC-refusal fixtures flipped the final byte of the whole prekey
envelope. That could alter an outer prekey ID and exercise key lookup rather
than nested MAC authentication. A test-only helper now parses the outer envelope,
flips the nested Signal message's trailing MAC byte and reconstructs the classic
envelope with unchanged IDs/base/identity. Companion authority and primary LID
MAC-refusal tests retain their authentication/rollback assertions with the correct
mutation. The production key-error fixtures explicitly select missing IDs instead.

Reproduce: cargo test --locked -p whatsapp-api --all-features signal::direct::tests
and undecryptable_message_requests_once_before_write_cut_and_never_replays.
No server acceptance, full count/key/routing/ACK policy or zero-ban guarantee is
established; the broad network audit remains unfinished.

Validation:1329 workspace/all-feature tests passed after correcting the nested
MAC fixtures. Formatting, strict workspace/all-target/all-feature Clippy, CLI
reference regeneration, APK coverage, staged whitespace and research checksum/
link hygiene passed. Offline only.
