# Successful client pong schema and one-shot callback lifetime

**Confirmed static**, APK2.26.43.72 code264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No target execution, external socket or live account operation. Existing xtask
DEX exports are not REA Evidence envelopes. The preceding
[client ping callback/manager trace](../2.26.43.72-client-ping-errors/README.md)
retains1xa.CK4 and the manager response worker. This continuation closes its
selected typed-reader and recoverable-schema-error gaps.

## Required success fields and reader semantics

1xa.CK4 first checks iq tag, then requires from through8B6.A0F with allowed
classesJid/UserJid. A0F callsA0J with targetJid, null default and no equality
requirement. It then checks class assignability; rootJid in the allowed list
admits its subclasses. This selected check does not compare from with the
request's server destination. It is schema parsing, not a proof of authorized
origin for arbitrary network input.

0Tp.A0A prefers the already typed JID attribute. If absent/untyped, it parses
the string throughJid.Companion.A02. It logs non-protocol-compliant JIDs, but
that selected log branch does not reject them. Class-cast failure returnsnull.
Full native string-JID grammar is not reimplemented here: Rust uses its existing
bounded Jid parser for text and accepts already parsed Jid values. Those local
bounds remain a supported-path limit, not newly asserted native restrictions.

CK4 obtains the original request's id as a required String, then requires reply
id equality. Its String length bounds are-9007199254740991..9007199254740991;
Java String length cannot reach those outer values, so no new small ID limit
is justified. Next it requires signed-long t within inclusive
1577865600(0x5e0c5180)..4102473600(0xf486c780). Finally type must equalresult.
Missing required values do not default. A0J's supplied String defaults in this
path are equality constraints, not substitutes for absent data.

A0J's number branch calls0Cd.A08 -> A09(radix10), which usesUTF-16 charAt,
Character.digit, explicit sign handling and negative accumulation with overflow
checks. It accepts decimal BMP digits and optionalASCII +/-, not whitespace,
radix prefixes, fractions, supplementary-digit scalars or signed64-bit overflow.
The existing Rust java_decimal_long primitive supplies those numeric semantics;
this path additionally applies the callback's inclusive timestamp bounds.
The returned timestamp is logged byCK4, not adopted into the confirmed clock.

## Malformed result consumes IQ ownership but not the manager response

0ln.A02 removes the pending callback under its lock before scheduling the
worker. 1xb.run calls0ln.A01, which invokesCK4 only forresult andBvj only forerror.
CK4 throwsLIL(String) on a failed typed-reader result before schedulingFea(24).
That constructor sets isRecoverable=true. The router catchesLV5, records its
error and checksLV5.A01, which reads this flag; its fatal RuntimeException
branch is skipped for this constructor. No ping-manager response is scheduled.
The manager's outstanding ping timer remains separate from the consumed IQ
callback. A second same-ID result/error cannot reacquire that callback.

This is recovery from schema parsing within the current receive path, not an
instruction to reconnect immediately or retry the ping. The full native metrics
sink/callback dispatcher and unrelated exception classes remain out of scope.

## Rust correction and explicit limits

The prior normal driver cleared a live ping for any matching iq/result. It now
tracks its response claim separately from the outstanding ping deadline. Only
an exact, owned, unexpired result/error consumes that claim. Forresult it checks
required supported JID from, request-bound id, timestamp grammar/range and exact
type before clearing the outstanding ping. A malformed result emits only a
fixed field-name diagnostic, retains the original timer and continues receiving.
A later same-ID valid result or error cannot resurrect the consumed claim.

Non-replyIQs do not consume the claim. Matchingerror retains the preceding
native-derived liveness handling and does not borrow result-schema requirements.
Disconnect/restart drops this in-memory state; no callback is restored from saved
captures. No deadline extension, replay, identity/clock mutation or new wire
request is added. Current local ping cadence/timeout and full native effective
configuration remain separate gaps; this is not full ClientPingManager parity.

## Verification and reproduction

The paused-time actor regression injects an ID-onlyresult, then a well-formed
same-ID result anderror. A server-origin ping still receives its pong, and the
original outstanding client ping must time out. The regression is run against
parent production code as a red check and against corrected code as green.
The schema test covers required fields, text/typed JIDs, inclusive bounds,
Unicode/sign inputs, malformed/overflow/millisecond-looking times and a changed
ID. Existing intended successful-pong fixtures now carryfrom/t; malformed
fixtures remain explicit and are not automatically enriched by the fake link.

Source reproduction uses existingxtask apk-smali APK --class CLASS --method
METHOD --signatures --exceptions:8B6.A0F/A0J,0Tp.A0A,0Cd.A08/A09,0ln.A01/A02,
1xb.run,LIL/LV5(fullclasses). apk-callers0ln.A01 records both direct worker
callers; the selected one-shot/default worker path is retained. The preceding
bundle retainsCK4/Bvj,Fea's manager response and constructors. No complete
upstream sender authorization or native JID grammar equivalence is claimed.
Checksums cover every other file; raw APK/private traffic remain outside Git.
