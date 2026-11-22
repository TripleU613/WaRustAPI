# Post-login startup stanzas

What the Android client writes after the login exchange, and what this client
writes instead. Everything here is **offline**; no live observation is claimed.

APK **2.26.43.72** (SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`).

## Selected startup block (confirmed static)

The selected connection bootstrap runs, in order:

1. **Push-config GET** — `X.1rj.A02` builds `iq type=get to=s.whatsapp.net
   xmlns=abt` with a `props` child (`protocol=1`, `hash=<saved or empty>`, and
   `refresh_id` when the predicate holds). Retained evidence:
   [`AB-property initial read`](feature-configuration.md#ab-property-initial-read-2026-10-08).
   Re-anchored in 43.72: `X.HBE.A00(force, refresh, refresh_id)` builds the same
   IQ (`abt`, `props protocol=1`, `hash`, optional `refresh_id`) and registers a
   32 s callback timer.
2. **Unified session** — `X.0Sb.A02` then sends the node through `X.1rj.A03`
   (the same fire-and-forget writer the availability stanza uses):
   `ib` with one child `unified_session [id=<string>]`, no IQ wrapper, no
   request ID, namespace, type or destination. Retained evidence:
   [node and ID lifecycle](apk/2.26.43.72-unified-session-startup/README.md).
3. **Availability** — a conditional bare `presence type="available"` writer
   call (`X.1rj.A01`). Retained evidence:
   [startup availability](apk/2.26.43.72-startup-availability/README.md).

## `unified_session` id (confirmed static)

`X.0Sb.A00` generates the string as

```text
String.valueOf((clock_ms + 0xf731400) % 0x240c8400)
```

i.e. three days (`259200000`) added to the app's corrected clock reading and
reduced modulo seven days (`604800000`). `X.08q.A00` returns
`elapsedRealtime + offset` when a saved offset is nonzero, otherwise
`currentTimeMillis - offset`; it is an offset-corrected wall clock, not a random
identifier. `X.0Sb.A03` caches the value and can refresh it on explicit refresh
or an identity-observer event; the startup call uses the cached/generated string
and returns no node only for an uninitialized account.

Rust equivalent: `transport::unified_session` computes the same expression from
`clock::server_now` (the confirmed server-time sample written by login success
and pings), falling back to the local wall clock when no sample exists. The
node is written once per connection, after activation is confirmed and before
availability.

**Limits.** The APK's offset producers/NTP sync, refresh cadence and the server's
meaning/acknowledgement of this stanza are not established. Java evaluates the
expression in signed 64-bit arithmetic; the Rust helper uses the same `+`/`%`
shape with a non-negative remainder so a clock before the epoch cannot produce a
negative id. The 0..64 String bound seen in `X.0Tk.A06` is a diagnostic
validator; it is not shown to be a server limit.

## This client's startup sequence

After the activation IQ is confirmed:

1. `ib`/`unified_session` (`transport::unified_session::startup_node`)
2. `presence type="available"` (`messaging::available_presence`)
3. a queued display-name presence, when one is pending (`presence::take_push_name`)
4. reserved outgoing receipts (`receipts::reserve_pending_owned`)
5. queued one-shot actions (drain trigger)
6. prekey upload when the store wants one (`prekeys::wants_upload`, startup trigger)
7. the next pending device query (`devices::DeviceCursor`)

The push-config GET is **not** sent automatically: it is available as the
explicit `abprops-query` action and is queued only when the operator asks. The
APK's own fetch is also conditional (stale refresh interval or a server
`abprops` notification that requires message sending to be ready), so this
client does not send it at every startup.

## Still open

- The push-config GET at startup, with its staleness/notification predicate and
  reply application. Manual action only today.
- Whether the APK writes availability on every connection or only under the
  selected condition; this client always writes it after activation (its own
  choice, recorded in `messaging::available_presence`).
- History sync, `ib` variants other than `unified_session`, and any startup
  stanza gated behind AB properties the server enables.
- No live validation of this ordering: it is a static trace plus offline tests.
