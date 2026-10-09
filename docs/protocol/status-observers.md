# Deferred message-status observer path

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Fourteen-topic recipe](apk/recipes/2.26.37.71-status-observers.json),
[retained source](apk/2.26.37.71-status-observers/targets.json),
[independent raw selector bytes](status-observers-evidence.txt),
[80 callback declarations](status-observer-declarations.json).
Offsets below are method-relative bytes. Research only; no Rust or live change.

## Correcting a historical selector error

[Receipt state-only prose](receipt-state-only.md) previously named DLx selector
**5**. Its immutable state-post-hook.txt actually shows **const/16 0x10**, and
this export plus raw APK bytes independently confirm selector **16**. The
mistake was our prose, not an APK change. Selector 5 belongs to a call/audio
route and must not be followed as message-status persistence.

0Yq.A01 gets its A02 Handler, constructs DLx(message, reason, **16**, owner)
and posts it. DLx.<init> stores the reason in A00, message in A02, owner in A01
and selector in $t. DLx.run dispatches 16 to **16aa..16c6**, casts those fields
and calls **0Yq.A00(owner,message,reason)** without changing that integer.
The original 15A.A07 caller supplies **-1**; this is an observer reason, not
native status -1 or a server timestamp. Message status 20/21 from the
[send-job path](send-job-state.md) does not become the callback reason.

## Message and chat callbacks

0Yq.A00 first calls **0Ys.A0P(message,reason)**. A0P uses 0MN.A03 and constructs
FcA(message,reason, **1**) before passing it into 078.A00. FcA.<init> preserves
the reason/object and selector. FcA.COL's nonzero-selector path calls
**0Mv.BuN(message,reason)**. Its zero-selector branch instead calls GAC.Bnz
with a UserJid; that is not this message callback.

Next 0Yq.A00 requires a nonnull original key chat and calls
**0Yr.A0O(chat,false)**. A0O creates 3J4(selector **3**, chat, false) and calls
078.A00 with null mode. 3J4.COL selector 3 at **008e..00ac** calls
**0cQ.BfD(chat,false)**. The other constructor overload and selectors have
separate collection/chat/user callbacks; they are retained without equating
those callbacks to native status setters.

Finally 0Yq.A00 calls B2N.A00(message); a nonnull Fdp result selects EH5.A0L.
The descriptor's meaning and this optional delegate's effects remain untraced.
No receipt_user INSERT, native status setter or send stanza is constructed by
the selected 0Yq/FcA/3J4 dispatcher bodies. Indirect callback/delegate effects
are not thereby absent.

The message object is forwarded, without a shown clone or immutable status/time
snapshot. Consumers can read B3b later. Handler.post or callback dispatch is not
proof of a SQL commit, actual live invocation or observer completion order.

## Generic observer dispatch is not always synchronous

**Confirmed:** 078.A00 snapshots its A04 iterator into three collections by
07G and 0L6 marker tests. Non-marked entries enter the ordinary collection.
The 07G collection goes into one path; 0L6 into another (07G takes precedence
when both tests match). Selected paths invoke COL directly, while others queue
through 0GR/3Pf selector 25 or the A02 helper under mode/thread/property tests.
It also records selected collection counts through nullable 0Yu.

Complete marker meaning, thread predicate 0KP.A03, queued helper effects,
registration/eviction and effective properties remain **unknown**. Therefore
0MN.A03 or null mode cannot be relabelled a universal synchronous completion
barrier. No new actor ordering or retry behavior is inferred for Rust.

## Main message observer and declaration inventory

The concrete **0My.BuN** identifies itself with
MainMessageObserver/onMessageChanged unknown type. Its input integer dispatches
observer reasons, independently of the later B3b calls. Reason **-1** at
001e branches to **0538**; reason 24 shares that block through its switch.
The method is retained completely, including other message/media/UI branches.

At 0538..056e, B3b()==**5** plus flag 0x1000 invokes B12.A01. Then key.fromMe
controls further processing; reason 24 bypasses a later status filter. For -1,
the filter at 0582..059a accepts 1hg.A03's six recognized receipt statuses or
**13** before considering selected chat/user/addon/2qy paths. A later A03
check at 0642..0656 governs the selected D5m/Dhk/Elx delegate.

**Bounded consequence:** when those status checks see **20 or 21**, -1 skips
the selected user-metadata path and returns at the later six-status check.
It is not a global assertion that every observer is inert or that this mutable
object cannot have changed by callback execution. B12, 2qy and other transitive
delegate/flag authority remain unknown. The reason must not be confused with
native state or a receipt timestamp.

The [declaration inventory](status-observer-declarations.json) was generated
through existing apk-smali with empty class filter and method BuN. It contains
**80 exact-prototype physical declarations**: **43 return-only**, **one with
no bytecode (0Mv)** and **36 with bodies**. The representative 1yE return-only
body and main 0My body are exported. Each scanned block's SHA-256, instruction
count and three direct-body checks are retained for rechecking; all scanned
bodies lack the literal receipt_user, direct base 178.A0H invocation and direct
base A08 write. This inventory does not prove actual registration, complete
inheritance/reflection closure, indirect SQL absence or every setter effect.
Do not promote those negative flags into native receipt-row membership.

Reproduce the inventory scan with its recorded command; split output at
newline+# class: and hash each following UTF-8 block exactly, including its
trailing newline. New releases must be anchored separately before comparison;
rendered block hashes are structural evidence, not semantic equivalence.

## Remaining runtime boundary

**Unknown/unported:** full observer registration/delegates, transitive send and
receipt-user writes, complete native state progression and primary cleanup's
user-row intersection. This corrects the post-hook route and makes the remaining
callback frontier reproducible. Existing device rows, original send versions,
DeliveryFacts and replay reservations remain unchanged. No account was opened,
no message/receipt was sent and no historical marker was cleared.

Existing apk-smali, apk-research-bundle and apk-research-diff reproduce this
source. No new CLI command, flag or HTTP route was added; x-live remains false.

```sh
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-status-observers.json \
  --output docs/protocol/apk/2.26.37.71-status-observers --check
cargo run -p xtask -- apk-research-diff \
  docs/protocol/apk/2.26.37.71-status-observers \
  docs/protocol/apk/2.26.37.71-status-observers
```

## Validation

Observer evidence validation: exact 14-topic export (15 methods, 19 files,
216,841 bytes), cached self-diff, all 80 declaration/hash records, message/chat
callback prototypes and independent raw selector/payload checks passed. CLI
reference, API-symbol, formatting, whitespace and hygiene checks passed
(2,149 links/107 APK checksum bundles). Research/docs only; no fresh Rust
runtime suite, account operation or live invocation is claimed.
