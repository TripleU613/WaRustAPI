# Self-user ACK conversion — APK 2.26.43.72

**Confirmed selected static**, build264307238; pinned APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected0EG/2LK class exports through xtask fallback, not REA Evidence
or runtime execution. No account operations.

[0Dd.A00](../2.26.43.72-message-ack-chat/jid-parser.txt)02ca..02f6 returns
barestatus_me ->0EG and lid_me ->2LK. Both classes here extend UserJid, have
empty user, corresponding self server and type11. Each overrides
getPrimaryDevice to unconditionally throw UnsupportedOperationException.
They do NOT inherit ordinary UserJid primary-device construction or return null.

[0De.A02](../2.26.43.72-message-ack-chat/user-factory.txt) can also create each
singleton for its corresponding domain, irrespective of user. Therefore
[0De.A04](../2.26.43.72-call-sender-text/user-cache.txt) can populate bare and
qualified self-domain cache keys. Bare keys resolve to the same singleton under
the selected factory/insertion paths. Qualified self domains are absent from
0Dd's normal domain switch: their result depends on whether cache is populated.
This change classifies ONLY the two exact bare names; it does not guess a
cache-hit/null result for arbitrary qualified self aliases.

[0Dn.A0k](../2.26.43.72-ack-participant-success/user-types.txt) admits type11;
[0EB.A00](../2.26.43.72-ack-participant-success/device-conversion.txt) therefore
invokes its getPrimaryDevice override and throws. For an ordinary message ACK,
[1rH.A03](../2.26.43.72-distribution-ack-admission/native.txt)0452..0464 converts
participant THEN originalfrom before successA10.0488..049a does the same before
refusalA0z. A valid recipient selects the message key but does not replace the
originalfrom used in conversion. Thus a self-valued participant OR from can
prevent either terminal dispatch, even when selected recipient matches intent.
This is distinct from the ordinary nonnull-participant success-only filter.

The existing [reader generic-exception evidence](../2.26.43.72-call-null-sender/README.md)
retains the capture/read-on path; no new fatal message-ACK gate or reconnect is
introduced. Required-id/numeric/challenge459 precede class dispatch, and special
message diversions remain separate. A self-valued recipient is itself a UserJid
and has precedence: Rust cannot process that self chat, so it retains uncertainty
instead of falling back to a different matching ordinaryfrom.

[0Dn.A0R](../2.26.43.72-call-ack-admission/sender-types.txt) excludes type11.
Bare self call senders are nonnull and now reject with otherwise valid fields,
while459 diversion remains earlier. No qualified-alias/binary equivalence or
full native callback/payload/recovery behavior is inferred.

## Reproduce

From checkout root, pinned APK outside Git; compare complete output bytes:

```sh
cargo build --locked -p xtask
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0EG;' --signatures --exceptions > status-self.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/2LK;' --signatures --exceptions > lid-self.txt
```

Previously retained complete parser/factories/predicates/dispatch are linked
above. Static failure paths do not establish server acceptance or no-ban proof.
