# Retry original-chat selection — APK 2.26.43.72

**Confirmed static / bounded Rust correction.** APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete checked DEX exports retain signatures and exception tables. No native
execution, account, network request or live retry occurred. These local exports
are not REA Evidence envelopes.

## Original message key includes a chat as well as nested id

The complete [F10.A04 parser](../2.26.43.72-retry-receipt-keys/receipt-parser.txt)
reads `recipient` using `0Tp.A0A(UserJid.class,"recipient")` at0050..0064.
If nonnull it uses that value; otherwise0098..00a8 takes Eoh.A02 through
**0Dn.A00**. The latter strips DeviceJid to its userJid, or delegates the other
JID forms to0Di.A00. F10 then calls its **A01 alias helper** at00b4..00be before
constructing the original message key with nested id at00c0..00c6.
The [message-key constructor](../2.26.43.72-retry-message-correlation/message-key-constructors.txt)
stores remote chat and message id separately. Message-id equality alone is not
that native key.

F10.A01 has effective-property, special-JID and mapping branches, including
0x9.A00 and0f5.A07; complete meanings/alias parity are not ported here. Rust's
existing saved peer→LID relation is used for correlation only; receipts do not
create mappings or replace identity.

The typed JID reader0Tp.A0A first uses an existing JID attribute or parses text,
checks/logs protocol compliance and class-casts it, returning null on a class-cast
failure. Thus an unusable explicit recipient can fall back to Eoh.A02 in native
F10. The Rust correction instead refuses a present unsupported/malformed
recipient as a **local supported-path boundary**. It does not claim full native
fallback equivalence.

## Context-layout search and its limits

Eoh constructor stores its first JID argument in A02; Efa.A00 forwards mutable
Efa.A02 to that argument. Field-writer and constructor/freeze caller inventories
are retained. Candidate bodies inspected here show:

- 1rH.A03 builds context from root `from`, `participant`, `recipient` and root id
  and has `class=receipt` ACK handling. This is not by itself proof of the whole
  incoming retry receipt factory.
- 1rg.A03 builds a parsing-error/NACK context from root fields. Its diagnostics
  explicitly identify parsing-error handling; it is not normal retry admission.
- 0x0.A04 builds context from an18U message object, with a conditional recipient.
  Its selected body is not an incoming receipt parser.

These bounded results avoid repeatedly treating a shared-context writer as the
missing incoming retry factory. Complete inbound context registration remains
open; the direct F10 original-key selection above is independently retained.
No ACK/NACK wire changes are based on these candidate bodies.

## Rust correction and validation scope

[Original-chat correction](../../retry-original-chat.md) makes a plain user-like
recipient select RetryRequest.chat when present; otherwise it normalizes root
from without device suffix. The requesting device still comes from participant
or from. Invalid/group/device-form recipients refuse locally. Text device0 syntax
is also refused because Rust's text JID parser can normalize an explicit zero
suffix to a plain identity. This is not a universal native UserJid grammar claim.

The normal direct driver additionally compares the selected chat against the
saved original peer and its already-saved LID before session work or queries.
Wrong chat with a matching participant, or wrong explicit recipient, produces
only local `not_the_original_chat`; no prepared retry/refetch is committed.

All TXT files are complete stdout from `apk-smali --class CLASS --method METHOD
--signatures --exceptions`, `apk-callers`, or `apk-field-refs` for their named
subjects. context-model.txt retains the whole Eoh class (7methods). Field writes
query exact Efa.A02:Jid with --writes-only; parser callers query F10.A04.
No source export is declared whole-program semantic coverage. Checksums cover
all files; direct native methods are not executed by Rust regressions.
