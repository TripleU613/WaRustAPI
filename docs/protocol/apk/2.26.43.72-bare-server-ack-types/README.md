# Bare server ACK types — APK 2.26.43.72

**Confirmed selected static**, build264307238; pinned APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected classes via xtask fallback, not REA Evidence envelopes or
runtime execution. No account operations.

The retained [bare factory](../2.26.43.72-call-sender-text/bare-factory.txt)
returns0mL for s.whatsapp.net,7Ks for g.us andJ9P for call. These complete
classes confirm all extend Jid directly, use empty user and their corresponding
server, and return types13/16/20. They are neither0DY message chat norUserJid
recipient. [0Di.A00](../2.26.43.72-call-temp-ack-text/chat-cast.txt) returns null
for non0DY; [typed recipient accessor](../2.26.43.72-ack-address-accessors/typed-address.txt)
returns null after failed class cast. Hence none can establish selected message
from and none can override a valid from as recipient.

## Bound the cache before applying bare types

Earlier work deliberately deferred bare subtype rejection because0Dd.A00 checks
cache first. This change does not infer cache-independent types from factory
alone. The retained [field census and cache analysis](../2.26.43.72-call-sender-text/README.md)
identify initializer/parser/A03/A04 as the four direct0De.A00 field sites;
[initialization](../2.26.43.72-call-sender-text/factory-init.txt) creates an empty
cache. [Key formatting](../2.26.43.72-call-sender-text/cache-key.txt) uses a bare
domain only when user is empty, otherwise inserts@. Therefore producing one
of these three bare cache keys requires empty user and that exact domain.

[A03](../2.26.43.72-call-sender-text/newsletter-cache.txt) rejects empty trimmed
user before insertion, and only inserts newsletter domain.
[A04](../2.26.43.72-call-sender-text/user-cache.txt) inserts after successful
[A02](../2.26.43.72-message-ack-chat/user-factory.txt). Empty-userPN fails the
[5..20 constructor](../2.26.43.72-pn-ack-text/phone-grammar.txt); g.us/call are
not A02 user domains and throw. Thus these selected insertion paths cannot
replace those bare keys. This resolves the bounded producer question, not
reflection, hidden aliases, native mutation or arbitrary generic cache writers.
status_me/lid_me are NOT folded in: A02 can create those user singletons and
A04 can cache their bare keys, a distinct path intentionally left separate.

[0Dn.A0R](../2.26.43.72-call-ack-admission/sender-types.txt) includes20, excludes
13/16. With valid call fields, barePN/group now reject through EW4/wrappedLIL;
barecall passes this sender-type predicate. Required-id/numeric and459 ordering
are unchanged. These are nonnull sender identities, not recoverable parse
failures. Full payload admission/recovery is not implemented.

## Reproduce

From checkout root, pinned APK outside Git; compare complete output bytes:

```sh
cargo build --locked -p xtask
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0mL;' --signatures --exceptions > pn-server.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/7Ks;' --signatures --exceptions > group-server.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/J9P;' --signatures --exceptions > call-server.txt
```

Existing complete cache/parser/predicate evidence is linked rather than copied.
Binary JID equivalence, user cache aliases, all other families, generic receipt
routes and actual server policy remain open. No live or zero-ban claim.
