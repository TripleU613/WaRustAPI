# Receipt count preprocessing, copies and ACK inputs

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Four-topic recipe](apk/recipes/2.26.37.71-receipt-count-preprocessing.json) and
[new complete helpers](apk/2.26.37.71-receipt-count-preprocessing/targets.json)
extend [recipient counts](receipt-recipient-count.md). Historical snapshots
are reused below, not rewritten. Offsets are method-relative bytes.

## D7J preprocessing does not use encrypted device cardinality

D7J.CHJ skips count processing and reaches the final callback-parameter branch
when 1gd.A1A(message) is true. Otherwise it computes:
`group = 0DE.A0m(chat)`, `otherMulti = 0DE.A0Q(chat) && !0DE.A0i(chat)` and
`special = 1ht.A03(message)`. A group remote-resource diagnostic does not
return; processing continues. Complete A1A/A0Q and preprocessing registration
remain unknown; these expressions are not replaced by guessed public JID types.

For group or otherMulti, it casts chat to 0c3 and obtains 1jL through
0bA.A0B. If key.fromMe AND message.A0y are true, its group/otherMulti branches
skip the count assignment (00ac..00c0 -> 00d6). Otherwise it computes:

- Group: `1jL.A06() + (special ? 1 : 0) - 1`.
- OtherMulti: `1jL.A06() + (special ? 1 : 0) - (1jL.A0a(account) ? 1 : 0)`.

The group formula reaches the shared store at 00d2 via the backwards goto
at **00f8**: 00f8 + 2*(-0x13) = 00d2. That write site is not exclusively the
literal-two branch. The otherMulti formula writes at 011a. Signed integer
arithmetic is shown without a clamp or positive-count validation.

When neither group nor otherMulti is true, 0080 branches directly to **00c0**,
bypassing the fromMe/A0y tests above. Only special AND 0DE.A0l(chat) then writes
literal **2** at 00d0..00d6. Requiring those bypassed ownership/flag tests for
this branch would misread the control flow. Its upstream A1A and complete
special metadata authority remain separate guards; this is not a universal
ordinary direct-message recipient count.

After any assignment, a nonnull 3Ya parameter causes explicit NPE construction
and throw (00d6..00e8); null returns. This selected body does not establish a
successful callback or durable count write. No SQL appears in this method.

## Participant maps and own-user membership

1jL.A06 returns the size of A0A when its A00 mode is nonzero, otherwise A0B.
A0a gets local PN through Aq0 and local LID through Apx, then first looks up
the mode-preferred identity through A0G(identity,false). A hit returns true.
On miss it inspects both maps and may log the disagreement. In nonzero mode,
a present PN in A0B is a fallback true result; zero mode has no equivalent
LID-map fallback and returns false after the miss. Map creation/default mode
and complete account contents remain unknown; arbitrary directory aliases are
not consulted or invented here.

A0G returns null for null input. Inputs passing A0Z/A0X/A0P use A0A when
mode is nonzero or the explicit Boolean is true; otherwise lookup uses A0B.
Remaining inputs use A0B. It returns the cast map value; it does not calculate
or populate membership. Full predicate-to-public-class mapping and map loader
collisions remain unknown. Thus the formulas use native participant caches,
not number of devices in a Rust send reservation.

## The other physical writes reuse existing sources

The [ten-write inventory](receipt-projection-count-evidence.txt) now has immediate
value/branch attribution for every physical site, while transitive producer
and normal-send initialization authority is still incomplete:

- [Bp2.A0P](apk/2.26.37.71-receipt-ownership/incoming-message-flag.txt)
  assigns message A06 from Bp2.A03 at 0058..0060 only under C6a key.fromMe.
  That received/restored own-key branch also sets status 4 and A0y=true.
  [Cih.A01](apk/2.26.37.71-group-envelope-own/stanza-builder.txt) reads
  Cih.A02 into v28 (00ac..00b4), passes it as v43 (016c..0170), and the
  [Bp2 constructor](apk/2.26.37.71-group-envelope-own/message-stanza-constructor.txt)
  receives it as v40 and stores A03 (0050..0058). This proves transfer, not
  the upstream meaning/default of Cih.A02 or ordinary authored-message seeding.
- [D7E.Byt](apk/2.26.37.71-receipt-ownership/message-flag-copy.txt) copies
  source A06 to destination at 0068..0070 under nonnull source B1Q
  (0036 branches past that block when null). It does not recompute the count.
  Its source lifecycle and all transitive copy callbacks remain separate unknowns.
- [DMZ.run](apk/2.26.37.71-message-ack/server-background-worker.txt) writes
  literal 2 at 027c..0282 when 1ht.A03 is true, the supplied worker count
  (loaded from DMZ.A00) is zero, and 0DE.A0l(chat) is true, after its earlier
  ownership/status checks. The existing
  [constructor](apk/2.26.37.71-message-ack/server-worker-constructor.txt)
  preserves that integer input. The worker subsequently persists message A06
  as recipient_count, separately from the supplied receipt-server timestamp.
  The value 2 is not evidence that two users have received the message.
- Cursor restoration and status/broadcast writes are attributed in
  [recipient-count producers](receipt-recipient-count.md).

## Remaining runtime boundary

**Unknown/unported:** normal outgoing initialization/transition, Cih count wire
producer, participant-map authority and preprocessing hooks, effective policy,
full native receipt_user admission/namespace and primary identity cleanup.
Closing these immediate writes does not create a native user-row ledger or
prove a safe resend. Immutable device targets and raw Rust DeliveryFacts stay
separate. No Rust messaging runtime, private keys/identities, request IDs,
account waits or live request changed. Existing APK CLI commands reproduce
this source-only research; no HTTP route or live capability was introduced.

Validation: four topics / nine files / 41,524 bytes reproduce from the
pinned APK exactly. Cached self-diff verifies four unchanged topics and four
metadata artifacts. CLI reference regeneration, API/library-symbol checks,
formatting and diff hygiene pass. Historical reused source remains unchanged.
No Rust runtime code changed, so no fresh workspace/runtime or live test is
claimed for this source-only chunk.

Follow-up: [text type inserter](text-type-inserter.md) identifies 1hl.BJK and
confirms D7J membership in the before set; other hook effects remain unknown.
