# Native response to a server-origin ping

2026-11-04. **Confirmed static in APK 2.26.43.72**, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
The Rust receive loop already updated its clock for `iq/get` with namespace
`urn:xmpp:ping`, but sent no protocol response. A current-build reader-to-writer
trace establishes the missing response below. It is not the periodic client
keepalive `iq/get xmlns=w:p`, and it does not copy a request ID.

No service connection or real account was used. An unanswered native ping is a
confirmed compatibility omission; its effect on a particular account restriction
or server disconnect is **unknown**. This correction guarantees neither full
startup parity nor freedom from restrictions.

## Current reader-to-writer path

Complete REA method Evidence and checked DEX routing are retained under
[2.26.43.72-server-pong](rea/2.26.43.72-server-pong/README.md):

1. [1rA.A03](rea/2.26.43.72-server-pong/ping-parser.json) selects IQ type `get`
   plus namespace `urn:xmpp:ping`. It calls `1lZ.A0t` with `t` parsed through
   `0HF.A01(value, 0L)`. This branch does not require a particular child,
   retain an ID for the reply or direct the reply to the stanza's `from`.
2. [1lZ.A0t](rea/2.26.43.72-server-pong/ping-callback.json) forwards
   [A0B(timestamp)](rea/2.26.43.72-server-pong/ping-event.json), whose Android
   Message has `arg1=4` and Bundle key `timestamp`. The concrete
   [1lY.CPm](rea/2.26.43.72-server-pong/reader-handler-routing.txt) rewrites
   `what=0`, preserving `arg1`, and sends it to the reader Handler.
3. [1lY.handleMessage](rea/2.26.43.72-server-pong/reader-handler.json) case 0
   invokes [0ZJ.A0q](rea/2.26.43.72-server-pong/ping-connection-consumer.json).
   [1uW.A00](rea/2.26.43.72-server-pong/reader-event-id.json) reads `arg1`.
   On 4, A0q updates the clock from timestamp × 1000 and local time, invokes
   the writer with [0ZJ.A07()](rea/2.26.43.72-server-pong/pong-writer-command.json),
   then invokes a separate `1lQ.A03` helper. The helper's broader effects are
   not interpreted here.
4. A07 creates a Message with `arg1=0`. The concrete
   [0vI writer adapter](rea/2.26.43.72-server-pong/writer-handler-routing.txt)
   rewrites `what=2`; its Handler case 2 delegates to `0v0.A00` without changing
   `arg1`. [1i7.A00](rea/2.26.43.72-server-pong/writer-event-id.json) preserves
   command 0; its special nested-command cases are unrelated.
5. [0v0.A00](rea/2.26.43.72-server-pong/writer-dispatch.json) command 0 emits
   this exact stanza shape:

   ```xml
   <iq type="result" to="s.whatsapp.net"/>
   ```

   The independent [checked DEX](rea/2.26.43.72-server-pong/writer-dispatch-checked.txt)
   confirms switch command 0 at `00f0` targets `21c8`. At `21d0..220a`, it
   allocates exactly **two** attributes: string `type=result` and typed JID
   `to=0mL.A00`. At `220a..2222`, it constructs `iq` with that array and writes
   it; no ID, namespace or child is added. The
   [current server-JID trace](rea/2.26.43.72-active-iq-audit/server-jid.txt)
   establishes that `0mL.A00` has empty user and server `s.whatsapp.net`.

Representative complete REA Evidence IDs:
`ev_ecb884dcca3412c07ee7ec1809fe030cf37ced8d68c864373e38773b72d33726`
(reader), `ev_27b68c87f6d2855c2d9c2555d246e39dec370a32af4a8e37450e8a187881c097`
(connection consumer), and
`ev_193d29d77f5215f4eb005c31edfbebf2d6c5f7fa6b846f1aa375ff3e095bef8a`
(writer). Bytecode offsets above are method-relative **bytes**. The full writer
contains other commands; their presence is not a claim that those commands are
semantically interpreted or ported by this chunk.

## Rust behavior

After an authenticated node is durably captured and its existing clock handling
runs, `connection::drive` now recognizes exactly `iq` + `type=get` +
`xmlns=urn:xmpp:ping`, sends the native two-attribute result and continues
receiving. It handles the request while activation is pending as well as after
activation; the native consumer has no active-result prerequisite for this path.
A pong does not satisfy our active IQ or mark the account ready.

The response is sent to the constant typed server JID. It does not reflect
arbitrary `from`, `id`, namespace, timestamp, child content or other attributes.
An absent/unusable `t` does not prevent the response: native parsing supplies 0
and still reaches the writer. Existing clock validation remains unchanged and
does not promote invalid time into a confirmed clock sample.

No durable send intent, retry job, account mutation or new feature was added.
Each newly received qualifying ping can receive a pong, including a repeated
identical ping; deterministic capture keys still deduplicate the raw evidence.
A failed socket write ends that connection through the existing error path.
The pong is not queued or reconstructed from historical captures on a later
connection. The actor's separately bounded recoverable-reconnect policy is not
changed.

## Offline regressions and limits

Three focused tests pass:

- `native_server_ping_is_answered_before_activation_and_after_capture`: injects
  the request before active confirmation; checks exact native output (including
  no echoed ID), committed capture and clock source before releasing the fake
  socket write; covers repeated request and absent ID/bad timestamp; confirms
  readiness still requires the real active result. Account rows and journal
  remain unchanged; capture/clock provenance survive reopening the encrypted
  store.
- `server_pong_does_not_answer_other_iq_types_or_namespaces`: the periodic-client
  namespace `w:p`, IQ result/set, absent namespace/type and another stanza kind
  do not produce a pong or end receiving.
- `failed_server_pong_write_keeps_capture_and_is_not_queued_for_reconnect`: a
  failed fake write ends the epoch with capture retained; a new injected
  connection over the reopened store does not resend the historical pong.

These exercise the real node-level receive loop with injected channels, never
an external server. Exact current active/passive request construction is a
[separate current trace](rea/2.26.43.72-active-iq-audit/README.md). Other startup
IQ ordering, effective bootstrap gates, reconnect reason selection and native
keepalive scheduling remain outside this bounded correction.
