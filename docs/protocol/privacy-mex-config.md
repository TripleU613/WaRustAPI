# Privacy MEX and configuration continuation

2026-11-04. **Confirmed static**, current WhatsApp Android **2.26.43.72**,
SHA-256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
[Evidence](rea/2.26.43.72-privacy-mex-config/README.md). This extends
[account privacy refresh](account-privacy-refresh.md); older unknowns in that
note describe the first bounded trace. No private state or live service was read.

## Persisted query and conversion

The APK's `assets/whatsapp-android-mex_client_persist_ids.json`, SHA-256
`a30a4b3ce55150747820bbf353e5d2e742fde19e2a11dd3e43b094d7088ae797`, contains:

| GetPrivacySettingsQuery field | Exact value |
| --- | --- |
| Server doc_id | `32774292262215379` |
| Client doc_id | `36022951673618922043902506414` |
| Operation-name hash | `3602295167` |
| Operation-text hash | `3618922043902506414` |
| Schema | `whatsapp_mex` |

The existing `apk-mex-queries` extractor retains the exact integer identifiers
as decimal strings in `persisted-query.txt`. The asset record is confirmed;
the current namespace-to-asset registration and outer dispatch remain separate
unclosed bindings. `74K` is only the JSON result model, with a constructor and
no document-ID getter. `0iT.<init>` stores namespace, call name and input rather
than supplying a hardcoded document ID.

`FxW.A0D` sets `jid` using `Fws.A0z`, which calls `Jid.getRawString()`.
Thus the previously traced variables are `query_input: [{jid: own LID raw
string, privacy_features: selected feature names}]`, not a peer or group query.
The own-LID prerequisite and list selection remain as documented in the first
trace. This is input structure, not proof of the whole transmitted envelope.

`7gM.<clinit>` and `83S.A01` establish exact server feature conversion:

| Feature | Local category |
| --- | --- |
| ABOUT | status |
| CALLADD | calladd |
| CHANNELCREATION | channelcreation |
| CHANNELVIEW | channelview |
| COVER_PHOTO | cover_photo |
| DEFENSE | defense |
| DEPENDENT_ACCOUNT_MESSAGES | dependentaccountmessages |
| GROUPADD | groupadd |
| GROUPCREATION | groupcreation |
| LAST | last |
| LINKED_PROFILES | linked_profiles |
| MESSAGES | messages |
| ONLINE | online |
| PIX | pix |
| PROFILE | profile |
| READRECEIPTS | readreceipts |
| STATUSVIEW | statusview |
| STICKERS | stickers |

UNSET/ALL/DEPENDENT_ACCOUNT_CALLING/PHONE_NUMBER_SHARING/**STATUS** have no
category conversion in this current method. `STATUS` must not be substituted
for `ABOUT`; the first is omitted and the second maps to `status`.

`7gK.<clinit>` and `83S.A00` establish settings conversion:

| Setting | Value |
| --- | --- |
| ALL | all |
| ALLOWLIST | contact_allowlist |
| DENYLIST, MYCONTACTSEXCEPT | contact_blacklist |
| KNOWN | known |
| MATCH_LAST_SEEN | match_last_seen |
| MYCONTACTS | contacts |
| NONE | none |
| OFF | off |
| ON_STANDARD | on_standard |
| WITH_PIN | with_pin |

Unrecognized values return null and the previously traced coroutine omits
them. Enum-to-category conversion does not replace the later category validator.

## Effective integer configuration has two authorities

`00E.A0a(23463)` -> `A02(00G.A03,this,null,23463)` -> `A06` passes:
legacy integer lambda `1XK` discriminator 1, MobileConfig result lambda `1XN`
discriminator 0 and MobileConfig direct lambda `1XO` discriminator 1.
`00G.A03` has `A01=false`, so its legacy cache is `00E.A0P`, not `A07`.

The checked `A06` branch requires nonnull lookup provider `A0N`, nonnull
MobileConfig context `A0M`, `A13()` and ready atomic `A08` before selecting
the MobileConfig value as its result. `A13()` is exactly
`A0E && A0b && !A0S.get()`. Otherwise the legacy lambda supplies the result.
When `A0E && A0G` is true, comparison can evaluate both sources; it does not
mean the compared legacy value becomes the selected MobileConfig result.
Caching controlled by `A0c` and `A0d` also occurs in the selected SDK branch.

`1XN/1XO` first resolve `0A7.B8Q(property)`; a zero specifier yields integer
zero (the result wrapper labels it INVALID_SPECIFIER_MISSING_IN_WA_LOOKUP_MAP).
A nonzero specifier calls `MobileConfigUnsafeContext.Arl` and casts long to
Java int. This is not equivalent to reading the raw AB-property reply.

The legacy integer lambda goes through `00E.A01`: check cache, synchronize and
check again, register exposure, obtain the compiled integer default from `A1B`,
read the selected preferences' decimal property key, update cache and exposure
state. If neither compiled nor supplied default exists, it throws rather than
silently inventing zero. `08Y.A1B` returns its `A02` map; `08Y.<init>` builds
that map including `08Y.A0I`, which explicitly inserts **23463 -> 0**. This
closes the compiled legacy default for the `ab-props` subclass, not this
account's effective value.

Checked `00E.<init>` establishes migration fields: only the `08Y` subclass
loads 23048 into `A0F`; `A0E` becomes true for that subclass when 23048 is true
or integer 27668 is positive. Other subclasses get false for these fields in
this constructor. `A0b` reads 24896, `A0G` reads 22646, `A0S` reads 25403 and
ready `A08` starts false. The Java recovery obscures the `08Y` condition; use
`config-constructor-dex.txt` 01da..0224. Later readiness/migration mutations
are **unknown**, so constructor defaults are not a permanent mode assertion.

`00D.A02(66)` -> `A03` checks the global DI cache then calls
`1Uw.A00(66,context,...)`. The concrete case-66 producer is now **confirmed** as 08Y in the
[token MobileConfig follow-up](apk/2.26.43.72-token-mobileconfig-binding/README.md). `08Y` explicitly supplies preference name `ab-props`
and the MobileConfig dependencies. This establishes a real configuration owner,
but not that Rust can infer the selected source from an absent local record.

## Category validation and implementation boundary

The retained checked `8AX.A04` confirms the value restrictions after `A00`
has rejected unknown strings. `readreceipts` reaches the general path: all,
contacts, none, known, off and on_standard codes pass; contact_blacklist,
contact_allowlist, match_last_seen and with_pin fail for this category.
Its Boolean conversion is independently true only for all/on_standard.
Do not turn contacts/known/off into unsupported true values simply because
their category admission succeeded.

Other categories have separate rules: stickers admits contacts/none/allowlist;
online admits all/match_last_seen; messages/dependentaccountmessages admit
all/contacts; defense/channelcreation admit off/on_standard; statusview admits
contacts/none/blacklist/allowlist; channelview admits all/none/with_pin.
Groupcreation's special restriction and groupadd allowlist depend on account
Booleans. General-category admission is followed by category-to-local-key
mapping and side effects, so this helper alone does not authorize durable
adoption of arbitrary category strings. Existing pending-value reconciliation
is documented in the first trace and must survive a future port.

**Unknown implementation dependencies:** effective configuration hydration and
precedence in Rust; MobileConfig specifier 23463 and
ready-state lifecycle; current MEX namespace/outer transport; own-user model
and collection coercion details; full account-category persistence/observers;
pending setters and retry/dirty lifecycle. `1az.A05/A0C` narrows the optional
STATUSVIEW gate to dependent-account/age-experience predicates and property
34229, but their transitive predicates remain unclosed.

No unconditional SMAX refresh or notification ACK was implemented: a compiled
legacy default does not justify overriding a hydrated SDK mode, and no existing
Rust privacy checkpoint owns the complete reply's effects. The next useful
port needs an explicit configuration owner and category state, not an
unreferenced request-builder utility. This chunk changes evidence only; it does
not claim protocol equivalence, live acceptance or any account-restriction
causation. No live requests, state resets or fabricated provenance occurred.

## Token MobileConfig binding and one ready transition — 2026-11-05

**Confirmed static, 2.26.43.72:** concrete 0A8.B8Q indexes 0El's table.
865/909/996/997 all have nonzero SDK specifiers; missing raw AB rows do not
imply an unmapped/zero SDK property. ID66 binds08Y, ID252 binds0A8. A retained
0A8.BPZ branch sets08Y's A08 ready true and resets its SDK cache viaA0t.
One producer is closed, not the complete initialization or override lifecycle.
[Exact specifiers, full methods, derivation and binary-asset boundary](apk/2.26.43.72-token-mobileconfig-binding/README.md).
Effective account integers, parameter-map defaults and configured token policy
remain unported. No send policy changed and no live account traffic occurred.

## Privacy setting mutation input and response rules (2026-11-06)

**Confirmed selected static:** [MEX setter, coroutine bindings, input builder,
response enum readers and persisted IDs](apk/2.26.43.72-privacy-mex-set/README.md)
close0oe.A0P's selected MEX handoff through8ho5 and8gk1. The operation is
SetPrivacySettingMutation, namespace whatsapp-android-mex, server doc_id
26887749497493182. Selected input nests feature/setting Enum.name values under
setting; the corrective pair is READRECEIPTS/NONE. No own-JID field is built in
this selected input; complete SDK authentication/envelope is separate authority.
Do not reuse GetPrivacySettingsQuery's own-LID/query_input structure.

A successful nonnull xwa2_privacy_feature_update model uses translated response
feature/setting when both enum objects/translations are nonnull. Either enum
object null selects the original requested pair; this is not a partial merge.
Present untranslatable enums fail, as do missing model/success=false branches.
The enum reader uses JSONObject.isNull then optString/Locale.US uppercase and
Enum.valueOf with unrecognized defaults; null and unknown-present values are
not interchangeable. Complete framework coercion remains outside this trace.

Normal result maps reach6pB.A01 ->0oe.A0R; selected failures call6pB.A00(null),
not a synthetic classic numeric error. This setter's false continuation flag
still leaves pending desired ownership on failure. Full effective transport
mode, SDK final wire/deadline/refusal policy and durable recovery remain open.
No typed Rust setter, arbitrary MEX-action pending guard or correction request
is introduced. Exact large persisted IDs remain strings; no new REA record.

Validation: README commands reproduce nine exports and three extraction
outputs byte-for-byte; selected branch targets, full bundle checksums and
pinned APK digest verify. Enum field bindings reuse retained complete REA
initializers without duplicate records. Formatting, OpenAPI library test,
APK coverage freshness and diff whitespace pass; hygiene checks440 bundles.
No Rust source changed, so workspace tests/Clippy were not rerun for this
evidence-only chunk. No live account or APK runtime operation.
