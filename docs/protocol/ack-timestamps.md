# Retained server ACK time

2026-10-09. APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.

**Confirmed static:** the already retained [ACK parser/worker trace](message-ack.md)
shows `1Qy.A02` at 049e..04be parsing `t` through `0Gv.A01`, defaulting to
zero on missing/invalid input, then multiplying the signed long by 1000. DEX
multiplication wraps. `1Dz.A10/A0H` forwards the value through dispatcher
arg1=19; `0zC.A0U` queues `DMZ`. The worker receives this value without another
conversion. Its ordinary SQL at 07e6..0856 writes the supplied value to
`message.receipt_server_timestamp`, even where memory can preserve an older
positive value. This is existing evidence reused for an implementation; no new
APK discovery or whole-worker parity is claimed.

**Implemented/offline tested:** matching success ACKs now record
`DeliveryFact.server_timestamp` separately from local `observed_at`, atomically
with the existing once-only acceptance fact. The shared Java decimal parser
accepts signed BMP decimal digits and rejects whitespace, overflow, supplementary
digits and malformed text according to the retained Java reference. Seconds-to-ms
conversion uses wrapping multiplication, including negative and zero results.
Those values are observations, not a trusted local clock or positive receipt time.

Historical facts missing this field deserialize to **None** and omit it again
when serialized. They are not rewritten to zero. Current success ACKs lacking a
valid `t` explicitly retain **Some(0)**. Error ACKs and other facts remain None;
this change does not claim their timestamps are absent from the APK. Duplicate
terminal ACKs cannot replace the first observation. Recipient, participant,
InFlight and ambiguity/corruption guards remain unchanged. No identity rotation,
reservation reset, replay or native `receipt_user` membership is introduced.

## Reproduction and inspection

`status <local-id>` already emits the saved ticket/progress/facts JSON. Its API
schemas are corrected to that actual shape, including structured device contact
JIDs, rather than the previous invented flat state/booleans. `listen` performs
authenticated ACK processing. No command, flag or HTTP route is added.

The encrypted-store regression exercises all **479 cases** in the independent
[executed host Java oracle](../../crates/whatsapp-api/fixtures/mex/2.26.37.71-numeric-reference.json)
plus a missing attribute, then reopens each store and checks both timestamps,
once-only behavior and lack of replay eligibility. The oracle is host Java
semantics, not APK execution; its [source](MexJavaNumericOracle.java) and prior
[validation scope](mex.md) are retained. Separate tests cover error ACKs and
historical JSON omission versus current explicit zero.

```sh
cargo test -p whatsapp-api --all-features acknowledgement_timestamp_uses_java_oracle_and_survives_reopen
cargo test -p whatsapp-api --all-features error_ack_does_not_claim_a_success_timestamp
cargo test -p whatsapp-api --all-features historical_facts_keep_unknown_server_time_without_migration
```

Validation: 832 workspace/all-feature tests passed, including 480 timestamp
store/reopen cases within one regression. Strict all-target Clippy,
warnings-denied Rustdoc, default CLI tests (25 plus API-symbol), regenerated CLI
reference, formatting and whitespace checks passed. Repository hygiene checked
2,128 links and 105 existing APK bundles. No qualifying live ACK or account
migration was performed.

**Unknown/unported:** complete native ACK status admission, special message
classes, counter/token persistence and receipt-user row authority. No qualifying
live ACK was observed in this change; no account was opened and no new message
was sent. Existing accepted IDs must not be renamed or replayed to manufacture a
timestamp test. This retains raw acceptance metadata without changing send policy.
