# Bulk prekey refresh request and callback

APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
**Confirmed static:** this continues the
[identity-notification scheduler](identity-notifications.md). It traces named
job inputs into wire attributes, not a live identity transition.

## Job and wire request

[BulkGetPreKeyJob constructor](apk/2.26.37.71-bulk-prekey-refresh/bulk-job-constructor.txt)
keeps the normal `jids` list separately from `identityChangedJids`, plus context,
fetchPqPrekey, contextRawJid, sessionScopeOrdinal and isPqSessionUpgrade.
PQ session upgrade requires PQ fetching; these booleans are distinct. The
[run method](apk/2.26.37.71-bulk-prekey-refresh/bulk-job-run.txt) reconstructs
DeviceJid lists, prepares their combined set through `Cio.A02(..., false)`
(`0056..00a0`) and emits handler message **what=0, arg1=87, arg2=0**
(`01da..01e8`). It carries the lists as `jids` and `identityJids`; the reason-8
context is used separately for job telemetry, not copied as a numeric wire
`reason` here. **Unknown:** Cio.A02's preparatory state effects.

The [existing full 0y0.A00 dispatcher](apk/2.26.37.71-read-receipt-job/read-dispatch.txt)
selects 87 -> `16ae`, registers a Bot callback at `1738..175e`, and builds the
request at `1786..193a`. For each requested device, membership in the
identityJids HashSet (`1792..179a`) selects an extra attribute:

```xml
<iq id="fixture-query" xmlns="encrypt" type="get" to="s.whatsapp.net">
  <key>
    <user jid="15555550100@s.whatsapp.net"/>
    <user jid="15555550100:3@s.whatsapp.net" reason="identity"/>
  </key>
</iq>
```

Addresses/selectors above are synthetic. `reason="identity"` belongs on the
selected **user**, not the IQ or key (`17b6..17c8`). It does not replace the
normal key query with the separate `encrypt/identity/user` identity-only read.
The native builder independently adds key-level `pqsupport="true"` for PQ
fetching (`1806..1824`) and guarded `context_jid` (`1824..1886`). Ordinary
identity-notification legacy fetching has null context and PQ false, so neither
attribute is emitted. The separate conditionally queued PQ request remains
unported; no fabricated PQ support is sent by this change.

## Callback and retry boundary

[Bot constructor](apk/2.26.37.71-bulk-prekey-refresh/bulk-callback-constructor.txt)
retains the requested devices, B78 scope, PQ-fetch and upgrade booleans. It does
not receive identityJids as a separate adoption-authority list.
[Bot.A03](apk/2.26.37.71-bulk-prekey-refresh/bulk-callback-result.txt) requires a
list, sorts its children with DOB selector 32, reads DeviceJid/error code per
row (`0072..00ae`) and otherwise parses identity, registration, optional type,
key and signed key, optional device-identity and identity_auth. Type defaults
to 5 only on absence; a present type must contain exactly one byte. Identity_auth
includes version, signature and certs; its presence is not proof of verification.

Parsed candidates are handed to `1Dz.A1O` (`03a0..03d8`); requested-device
success/error paths reach `A15` and `A1T` (`03de..04ce`).
[Bot.A01](apk/2.26.37.71-bulk-prekey-refresh/bulk-callback-error.txt) handles the
outer error across requested addresses, deriving B76 with scope and B77 selected
by PQ mode, then calling A1T (`0056..0064`). **Unknown:** complete A1O/A15/A1T
verification, pin/session mutation, callbacks, sorting authority and partial-set
semantics. Parsing public keys here does not establish that they can safely
replace an existing authenticated pin.

[Job exception policy](apk/2.26.37.71-bulk-prekey-refresh/bulk-job-exception-policy.txt)
returns `!isPqSessionUpgrade` (`001a..0024`). Native job framework retry behavior
remains untraced. Rust's once-only durable reservations and uncertain/refused
state remain unchanged; this callback does not authorize generic retry/reset.

## Implemented request capability

**Implemented/offline tested:**
`messaging::key_request_many_with_identity_changes(recipients, changed, id)`
adds the user reason only for selected devices. Existing `key_request_many`
passes an empty changed set, preserving ordinary requests. Local bounds remain
1..=32 recipients; changed inputs must fit that bound and belong to the requested
set. Fixed errors omit addresses. These subset/budget refusals are local guards,
not demonstrated server limits.

Matching uses existing Rust exact user/server/agent/device correlation across
Plain/AD_JID encodings. PN, LID and individual companion devices remain distinct.
This is **inferred/local policy**, not a complete native DeviceJid.equals port.
The builder performs no store access, queuing, key verification or adoption.

```sh
cargo run -p whatsapp-cli --features experimental-transport -- \
  peer-keys --chat 15555550100@s.whatsapp.net --preview-identity-change
```

The experimental flag invokes `peer_keys::identity_change_preview` before store
opening. Output contains namespace, IQ type, user reason and encoded byte count
with a fixed offline selector; no address or wire body is printed. It conflicts
with `--show`/`--refresh`. It creates no HTTP route, action or transmission.
An actual fresh-binary check with a nonexistent store and absent key environment
returned the four preview fields (62 encoded bytes for the synthetic PN input)
and created no files. Normal queue/inspection semantics remain unchanged.
Preview success is
not evidence that an identity changed or that a server accepted a refresh.

The [fixture](../../crates/whatsapp-api/fixtures/prekeys/2.26.37.71-identity-query.json)
manually transcribes selected/unselected wire attributes from the dispatcher
using synthetic addresses. Tests compare the complete relevant IQ/key/user
attributes, exercise comparison codec round-trip, companion and namespace
mismatch refusals, encoding equivalence, local bounds, redacted preview and CLI
flag conflicts. It is source-derived structure, not executed APK output.

Automatic identity-notification refresh, Cio preparation, callback authentication,
PQ/context handling and native deferred ACK state remain **unported**. The next
step is A1O/A15/A1T and Cio, followed by durable notification/request correlation
and application of the existing explicit identity-change policy. No real account
was opened and no fresh live request was sent for this work.

## Reproduce retained evidence

The [six-topic recipe](apk/recipes/2.26.37.71-bulk-prekey-refresh.json) retains
11 files / 70,703 bytes: job construction/run/exception policy and callback
construction/error/result.
The full dispatcher is reused rather than copied into another bundle.

```sh
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-bulk-prekey-refresh.json \
  --output docs/protocol/apk/2.26.37.71-bulk-prekey-refresh --check
cargo test -p whatsapp-api --all-features identity_change_
```

Use `apk-research-diff` on retained release bundles for topic/file changes. A
same-bundle self-diff verifies checksums and retention, not release equivalence.
