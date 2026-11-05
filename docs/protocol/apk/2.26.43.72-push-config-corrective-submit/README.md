# Corrective push-config construction and admission — APK 2.26.43.72

Confirmed static checked DEX TXT, not runtime observations or REA envelopes.
Pinned APK SHA256, reverified2026-11-05:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Command: `xtask apk-smali APK --class CLASS [--method METHOD] --signatures
--exceptions`. Complete selected methods are retained and checksummed.
Only the identified construction, admission and callback branches are interpreted;
this is not a complete send/reconciliation implementation.

## Same-tag merge matters to the emitted shape

Ebv.A00 receives Fuc callback, Ee7 state, optional account-count Integer,
optional settings Long, provider token String, platform String and optional
pkey String. The provider token gets an explicit nonnull check.6oj.A12 obtains
a request ID through0Sf.A0G, separate from the provider token.

B1w.A0a creates a fresh named0Tj builder.0Tj.A03 adds its node argument as
a child;0Tj.A04 instead merges a same-tag fragment's attributes, children and
content into the builder. B21.A0P finalizes its first builder and adds that
node as a child of the second. Applying these actual helper bodies to the
selected construction yields one iq with one config child, not multiple
config wrappers despite repeated config-builder allocations:

```text
iq [to=server JID, type=set, xmlns=urn:xmpp:whatsapp:push, id=request ID]
  config [id=provider token, platform=provider, additional selected attributes]
```

B21.A1A supplies the server destination. DBi.A0h supplies namespace AND
literal type=set. The finalA04 merges an intermediate IQ fragment into the
outer IQ. Same-tag merging's conflict/child-group behavior is retained in full
but not generalized here to every duplicate or nonempty merge case. This
shape must not be inferred by treating every builder helper as append-child.
The prior config GET/version=1 and token GET/pn writers are separate requests.

## Selected attributes and limits of generated checks

The config id is the provider-token input, not the generated outer IQ ID.
The selected body handles platform, app_mute, pkey, num_acc, settings and
voip_payload_type. Ee7.A00 and bit0x20 influence app_mute construction; those
state bits' complete producer semantics remain open. Optional pkey input and
a positive remote0x1014 value gate the VOIP value in this body. Missing account
count defaults to1 before Long conversion. Full enum/string writer behavior,
state bit policy and per-platform requiredness remain incomplete.

The generated account-count validator is called with bounds0..5, while
DBh.A1X supplies0..9007199254740991 for selected Long values.0Tk.A05 compares
a nonnull Long, invokes00L.A0C on a bound violation, then still returns true.
That assertion helper's [logging-only body](../2.26.43.72-client-auth-token-writer/README.md)
was previously traced. These numbers therefore are not proven hard writer
rejection or server limits. Null optional and null required values have
separate branches. No fabricated clamp or permissive server policy is added.
The0Tk.A06 String-length checks have the separately traced diagnostic behavior;
this does not establish every enum/string writer helper.

## Submit gate differs from the client-auth-token gate

Ebv builds FUT selector0 (client config) and invokes0Sf.A0W with request
family1 and32000ms, returning that boolean. A0W delegates toA0A with both
last flags false. A0A requires0AI.A06=true andA0J channel nonnull. It reads
A0O/state2 for its diagnostic, but does not gate admission on that value in
this selected method. If admitted it registers the callback/timer through
0ln.A08 BEFORE constructing/dispatching the writer Message, then returns true.
This is local admission, not server acceptance or completed configuration.

The positive duration enables callback checking. The false dropIfOffline
flag reaches the previously traced [writer gate](../2.26.43.72-iq-promotion-shutdown/README.md):
when its other predicates allow, a disconnected writer can defer this Message
in its deque. The token request instead usesA0Q/A06, which rechecks connected
readiness and supplies a different drop policy. Those families must not inherit
each other's queueing behavior without tracing their ownership and timeout rules.

If admission fails, A0A removes the ID from the queued registry throughA0E,
logs and returns false; it does not directly invoke BsS in this body. This
also differs from tokenA06's immediate delivery-failure callback. The service
caller previously traced supplies a null Fuc callback, so a local true return
cannot be interpreted as an eventual onSuccess notification to that service.

## Selector0 callbacks do not rotate the client-auth token

The retained [FUT bodies](../2.26.43.72-client-auth-token-failure/callbacks.txt)
show selector0 CK4 logging and optional Fuc.onSuccess, without the token
selector's cat persistence/rotation/held-dispatch chain. BsS logs and invokes
optionalFuc.Bwt. Bvj parses an error code;409 with platform other than adm
invokes1uj.A00 before optionalBwt. That helper's complete effects and the
error parser are not reconstructed here. Other error codes still reach the
selected optional failure callback. No token pending-slot restoration or
new replay authority is inferred for this config callback family.

## Implementation boundary

Corrective sender shape, builder merging, local admission and selected
callback effects are now connected. Complete attribute grammar, trigger/state
policy, writer/reconnect races, downstream helper effects, generation binding
and durable reconciliation remain open. No Rust config sender, retry, route
or live account operation is added. Existing uncertain holds remain unchanged.
