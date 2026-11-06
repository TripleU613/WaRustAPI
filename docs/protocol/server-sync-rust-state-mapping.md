# Server-sync native gates versus current Rust state

Source audit, 2026-11-05. **Confirmed source** means the current repository,
not APK execution or live account state. Native requirements, including the selected positive registration-device-ID
role predicate, come from the
[checked current lifecycle](server-sync-lifecycle.md) and
[versioned fetch producer](app-state-versioned-fetch.md). No runtime change.

## State mapping

| Native obligation | Actual Rust authority | Supported conclusion and gap |
| --- | --- | --- |
| Product/role predicate09F.BU7 | transport/login.rs primary_login; connection.rs primary session setup | Native09G/09O.BU7 calls BU8(false): after initialization it reads registration_device_id default0 and tests positive. Rust checks numeric phone registration and primary login payload, but has no equivalent persisted native device-ID authority; initialization/writers and full runtime binding remain unclosed. Do not create a companion bootstrap default. |
| Held collection version/LT hash | SavedCollectionState loaded by held_fetch::queue_in and reserve_in | Existing complete logical-index checkpoints are required, normalized and frozen; reservation checks equality again. This is authenticated held-state admission, not dirty announcement metadata. |
| Advertised dirty version | No dedicated notification lifecycle row in the current receive dispatcher | A notification cannot update SavedCollectionState.version or LT hash. Native dirty_version assignment belongs to separate metadata, including its zero sentinel. |
| syncd_dirty / companion_syncd_dirty recovery block | No established native-equivalent saved gate | App-state server backoff is a timed server-error directive, not this native recovery state. Account generation fences provenance; it is not a recovery-clear signal. |
| companion_syncd_critical_bootstrap_state | No established equivalent | Missing held state or action rows do not prove native bootstrap-state1, nor a safe default value. |
| Existing/ready syncd mutations | Prepared pending patch checkpoints and queued durable actions exist | ensure_no_pending_collection_in and held_fetch::pending_in detect overlapping unsettled patches. They are exclusion guards, not a port of native mutation ready flags or A0c eligibility. A queued fetch is not an outgoing mutation. |
| One sync job across write, response, pagination and rescan | Generic action reservation plus held-fetch admission phase | Actions hold exact wire/outcome uncertainty and account generation. No native-equivalent notification scheduler flight/completion owner follows from an Admitted phase. |
| Server-directed retry wait | app_state_sync::SERVER_BACKOFF and server_waiting_in | Receipt-time wait survives through explicit action settlement. Preserve it independently; do not reuse it as bootstrap or dirty-version authority. |
| Result application | Pure binding/header inspection and separate verification/application primitives | inspect_response explicitly classifies structure and provenance, not authenticated MAC verification or automatic held-fetch result installation. Generic ActionStatus::Ok is a correlated IQ result, not proof of collection advancement. |

## Audited source boundaries

- [AccountRecord](../../crates/whatsapp-api/src/session.rs) stores generation,
  phone/JID, keys, registration, observed credentials and last authentication;
  there is no declared role/bootstrap/recovery field in that record.
- [Primary login](../../crates/whatsapp-api/src/transport/login.rs) requires
  matching registration/phone and a numeric server-issued login matching the
  configured phone. Payload platform is Android; companion pairing is not
  synthesized by that builder.
- [Held admission](../../crates/whatsapp-api/src/transport/app_state_sync/held_fetch.rs)
  queues explicit selected collections from existing complete held checkpoints;
  reserve_in validates sealed generation/request/baselines, then honors server
  wait and overlapping pending patches. Its module explicitly disclaims result
  verification, state advancement and a flight lease.
- [App-state sync](../../crates/whatsapp-api/src/transport/app_state_sync.rs)
  has version-aware request construction, pending patch exclusion, durable
  server backoff and redacted response inspection. Searches for native gate
  names and ready-mutation flags found no implementation in these selected
  modules. This is not proof against all future or external store writers.

## Consequences for integration

The existing sealed admission record is reusable provenance for an explicit
held-version intent; it cannot establish native notification eligibility or
create scheduler ownership retrospectively. Implement notification metadata,
role/recovery eligibility, one-flight ownership and authenticated response
completion together at their respective transaction boundaries. Preserve exact
original wire and uncertain reservations. A missing gate remains unknown until
its source is mapped; it is not false merely because a Rust row is absent.

The [connection-flight design](app-state-connection-flight-design.md) and
[inline fetch application specification](app-state-inline-fetch-application-spec.md)
remain the integration contract. Generic action pacing is already implemented;
it must not be reported as a missing scheduler gate or as the completed native
single-flight lifecycle. No blanket server_sync ACK, automatic query, new
identity, copied dirty version or default recovery preference was introduced.


## Current native role body bound (2026-11-05)

The [retained physical BU7 declarations, BU8 and initialization boundary](apk/2.26.43.72-notification-handler-census/README.md)
now establish the selected preference predicate. This narrows the unknown from
an opaque role function to concrete saved device-ID provenance and initialization.
Rust has no declaration matching that preference in the audited account/login
sources. A missing row must not silently select a native default before the
initializer/writer mapping is established. Other role/bootstrap/recovery gates
remain independent.


## Existing role evidence and current writer census (2026-11-05)

The earlier [role/recovery audit](server-sync-role-recovery.md),
[read-self authority](read-self-authority-current.md) and
[own-identity ingress](own-companion-ingress-current.md) already trace selected
initialization and reset behavior. They qualify earlier unclosed wording above;
full Rust migration/account authority is still missing. The new
[exact preference-string census and selected writer value flow](apk/2.26.43.72-notification-handler-census/README.md)
finds8 references. Pairing writes DeviceJid.getDevice and updates the cache;
Me clear and selected cleanup reset both to0. Account-switch reader is scoped
through a supplied selected-account argument, not presumed current-account.
These native observations do not authorize Rust identity/role resets, and
preference/cache operations are not an atomic protocol-store migration.
