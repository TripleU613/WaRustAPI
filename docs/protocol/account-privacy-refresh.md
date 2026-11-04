# Account privacy refresh obligation

2026-11-04. **Confirmed static** unless marked otherwise, WhatsApp Android
**2.26.43.72**, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
[Retained evidence](rea/2.26.43.72-account-privacy-refresh/README.md).
This continues [notification dispatch](notification-sync-obligations.md), rather
than treating an `account_sync` envelope as permission for a generic ACK.
Earlier [37.71 read-setting evidence](read-setting-sync.md) remains separate.

## Modify means refresh, with a protocol choice

`19N.A09` reads the first child. For child `privacy` and exact
`action=modify`, checked DEX 09ae..09d2 sets **field** `2D6.A05=true`;
`A05` is not a callable method. The shared continuation at 0e76..0e86 builds
`1sx` through `2D6.A00`, then calls `1sv.A01(request,false,true,false)`.
`2D6.A00` and `1sx.<init>` map this flag to `1sx.A06`, the privacy request.
Sibling flags remain false. This is a refresh, not the privacy setter.

`1sv.A01` uses request kind 3 for privacy. Its last Boolean controls the
60,000 ms per-kind suppression in `1sv.A00`; this notification supplies false,
so that guard does not suppress this refresh. The middle Boolean is true:
the manager writes `account_sync_privacy_num_retries=3` for a fresh request
(checked 0344..0394), records kind 3's local timestamp and calls
`0oe.A0M(callback)` at 0578..0590. Three is saved retry bookkeeping, not
proof of three immediate transmissions or a server wait. The full retry
scheduler remains **unknown** here. The first Boolean false means this callback
does not request aggregate `account_sync` dirty-bit completion.

Current `0oe.A0M` reads property **23463** (`0x5ba7`) through `00E.A0a`.
Checked DEX establishes these branches:

| Effective property value | Selected work |
| --- | --- |
| 1 | Launch MEX via `8io.invokeSuspend`, discriminator 18 |
| 2 | Launch shadow comparison via `8ir.invokeSuspend`, discriminator 7, and dispatch SMAX with a completion object |
| Any other integer | Dispatch SMAX through `0oe.A02(callback,null,false)` |

**Unknown:** this account's effective property and its configuration precedence.
The Rust AB-property inspection is explicitly evidence-only, with MobileConfig
precedence/application unfinished. It has no effective 23463 selector. Neither
an absent property nor the existence of the SMAX branch proves which protocol
the account should use. No property default was inferred.

## SMAX construction and reply processing

`0oe.A02` builds the following with generated ID, server JID `0mL.A00`,
request code **70**, long argument zero, callback `8R5` holding `6pB`:

```xml
<iq to="s.whatsapp.net" id="generated" xmlns="privacy" type="get">
  <privacy/>
</iq>
```

The destination was separately established in the
[current active-IQ trace](rea/2.26.43.72-active-iq-audit/README.md).
The long zero does not establish an unlimited timeout or authorize replay.

`8R5.CK4` requires the reply's **first child** to be `privacy`; it does not
search later siblings to substitute for an invalid first child. It traverses
that child's children in index order through `8dV.invoke` discriminator 16,
then `8db.invoke` discriminator 17. Each selected child must be `category`.
Missing `name` or `value` yields no pair. `value=error` instead requires a first
child `error` and uses its optional `code` as the value, omitting the pair if
the code is absent. Pair collection produces a map. Full collection-helper
duplicate and exception behavior was not independently re-traced.

Checked `8R5.Bvj` 0014..005a takes the first `error` child with a present
`code`, attempts integer conversion, calls `6pB.A00` once and returns; if none
has a code it calls with null. The Java display incorrectly suggests a second
fallthrough call. `BsS` reports delivery failure only to the optional shadow
completion. Neither error path supplies an adoption map or calls the success
completion. No retry duration is established.

`6pB.A01(map)` calls `278.A00(3)` when that callback exists **before**
`0oe.A0R(map)`, then optionally signals the shadow completion. `278.A00(3)`
decrements its outstanding count and removes `account_sync_privacy_num_retries`.
Its aggregate dirty-bit callback depends on its separate Boolean; the modify
path supplies false. Success completion therefore does not prove every returned
category was accepted, and it is not the notification wire ACK.

## MEX and shadow continuation

`8io` discriminator 18 calls `MexPrivacySettingsHandler.A01`, which obtains
the result through `A02` and calls the same `6pB.A01(map)` on success; failure
calls `6pB.A00(null)`.

`MexPrivacySettingsHandler.A00` requires an own LID before dispatch. Missing
own LID returns failure (`myLid is null`); it does not fall back to SMAX in this
method. `A02` starts from its static 17-feature list and conditionally appends
`STATUSVIEW` through an account-state gate. The static list is retained in
`mex-feature-list.json`; the gate's complete semantics remain **unknown**.

`8ir.invokeSuspend` discriminator 6 constructs a `query_input` list containing
own-user input and `privacy_features`, with query name `GetPrivacySettingsQuery`
and client string `whatsapp-android-mex`. It reads `xwa2_fetch_wa_users`, selects
the first user, checks model discriminator `0x2179873b`, and traverses
`privacy_settings.settings`. Feature and setting enums map through `83S.A01`
and `83S.A00`; missing/unmapped values are omitted. A missing selected settings
object returns failure, not a synthetic empty success. The exact document ID,
own-user input serialization, full enum conversion and complete outer MEX
transport binding remain **unknown** in this bounded trace.

Shadow discriminator 7 starts the MEX read and waits up to local **20,000 ms**
for the SMAX completion before reporting comparison via
`MexPrivacySettingsHandler.A03`. The retained body includes later comparison
branches; their full interpretation remains **unknown**. This timeout is not
a server-directed retry delay. MEX comparison is not evidence that an arbitrary
second mutation is safe.

## Adoption and the Rust boundary

`0oe.A0R` validates value codes through `8AX.A00` and category/account gates
through `8AX.A04` before dispatch. Checked DEX 00b4..00ca establishes that a
conflicting pending requested value skips the entry **without removing the
pending request**. A matching pending value is removed before dispatch; an
entry with no pending request is also eligible. Invalid/unsupported values
take 02bc..02f6, log and remove that category's pending request. The Java display
loses these important loop branches.

For `readreceipts`, 0238..0276 writes/applies `read_receipts_enabled` using
`8AX.A03`. `all` and `on_standard` produce true, `none` produces false after
validation. Other categories have their own state and observer side effects.
This is the category-map path, not the separate read-self local-off consistency
rule documented for build 37.71. Do not infer a setting from absent data or
reuse the local-off rule for this map.

**Confirmed repository negative:** searches of the current Rust sources for
`privacy`, `readreceipts`, `read_receipts_enabled` and preference/checkpoint
ownership found no account-privacy refresh/adoption checkpoint. Existing privacy
code covers trusted-contact tokens and per-chat limit-sharing controls;
`receipts::queue_read` explicitly documents that full APK preference policy is
unported. `transport::abprops` retains selected property evidence but does not
provide an effective 23463 value. A raw capture is not an applied privacy state.

**Inferred implementation consequence:** a refresh-only SMAX request followed
by an ACK would be incomplete when MEX/shadow is selected and would have no
faithful state consumer. No Rust receive behavior or ACK was changed in this
chunk. Unknown account-sync siblings remain captured and unacknowledged; no
generic notification ACK was introduced. No account state, live request,
telemetry/provenance, identity or server cooldown changed. Static omissions do
not establish ban causation or promise freedom from account restrictions.

The next implementation gates are concrete: establish configuration ownership
for property 23463; finish the exact MEX request/enum conversion when selected;
define durable privacy values and pending-request reconciliation; port the
supported child's processing before the existing base ACK; and test persisted
work before ACK, redelivery deduplication, malformed/unsupported child refusal,
correlated success/error adoption, and uncertain transmission without replay.
Those are dependencies of this obligation, not a reduction to an ACK-only port.

The [MEX/configuration continuation](privacy-mex-config.md) closes query IDs,
feature/value conversion and compiled legacy-default selection. It also refines
readreceipts validator admission beyond all/on_standard/none; effective config
hydration, outer MEX dispatch and durable adoption still remain prerequisites.
