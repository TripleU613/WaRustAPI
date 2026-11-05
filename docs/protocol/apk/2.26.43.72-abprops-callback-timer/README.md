# AB-properties callback timer and expiry ownership

**Confirmed static**, WhatsApp APK2.26.43.72 code264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Checked DEX disassembly, not runtime observation or REA Evidence envelopes.

## Producer and admission

The exact string `ab_props:sys:config_hash` re-anchors older Hgb.A00 to
HBE.A00. The latter constructs the abt/get/props protocol1 query and passes
32000ms (0x7d00), logical type220 (0xdc), and HyT selector2 to0Sf.A0W
(00fe..010c). A0W passes two false flags toA0A. Its ready branch requires
0AI.A06 and nonnull0Sf.A0J; it calls0ln.A08 before constructing/dispatching
its writer message. Its not-ready branch calls0ln.A0E and returns false;
HBE ignores this return. The complete meaning of those readiness fields and
A0E's effects are not established here. This differs from A0R's explicit
fallback registration: do not transfer that fallback to this producer.

## Registration time is not successful-write time

0ln.A08 inserts the callback into queuedA0I. Only a strictly positive duration
posts handler message4 via sendMessageDelayed and records timer tracking.
It passes the input boolean as message arg1; this AB-properties ready path
passes false. Zero/negative duration does not install this timer in A08.
The selected A0C promotion method moves ownership intoA0H and records an
elapsedRealtime timestamp, without restarting/replacing the timer. Thus a
fresh32s budget after successful local write is not supported by this chain.
Android handler clock/suspend behavior and actual scheduling are not measured.

## Expiry is an ownership event, not an independent reply-age predicate

The retained full handler in [promotion evidence](../2.26.43.72-iq-promotion-shutdown/README.md)
selects message4 at07a2, callsA03 with arg1==1, then removes timer tracking.
A03 removes queuedA0I first, otherwise sentA0H, under the registry monitor;
only a found callback schedules8bj selector21/BsS. HyT.BsS selector2 returns
immediately (0014). This selected callback adds no automatic AB-properties
retry or configuration mutation. Other jobs/callers remain separate.

[Response ownership evidence](../2.26.43.72-iq-response-ownership/README.md)
retains completeA0H/A02: response lookup cancels the tracked message4 before
GET ofA0H; actual response dispatch subsequently REMOVES A0H. Neither selected
response method compares reply age against32000. A timer already executing
may compete with that response claim. **Inferred possible ordering:** a response
can cancel an overdue but not yet dispatched timer; passing nominal duration
alone does not prove native callback expiry. This is not an observed race or
permission to accept replies after expiry has claimed ownership.

## Rust boundary and next port

Normal Rust LiveReplies is connection-scoped and consumed once, but has no
per-action expiry event. This remains an explicit compatibility gap. A port
needs producer-specific policy, a defined pre-write registration boundary,
expiry/response claim ordering and no resurrection/replay of durable Sent
rows. Applying32s to every action, measuring from successful write, or treating
expiry as proof of non-delivery would each exceed this evidence. This research
chunk changes no sender, timeout configuration, saved state or live behavior.
AB-properties parser/configuration precedence and scheduling remain separate.

## Reproduction

Use `target/debug/xtask apk-xref APK --anchor 'ab_props:sys:config_hash'`,
then `apk-smali APK --class CLASS --method METHOD --signatures --exceptions`:
HBE.A00 (abprops-query), HyT.BsS (abprops-failure), 0Sf.A0W (active-send),
0Sf.A0A (send-admission), 0Sf.A0R (send-wrapper), 0ln.A08 (register),
0ln.A03 (expire). Existing linked bundles retain fullA0C/A0H/A02/handler bodies.
`checksums.json` covers every other file here. No APK or bulk tree is committed.
