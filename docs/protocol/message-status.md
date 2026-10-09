# Message status storage and blank device receipt entry

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Eleven-topic recipe](apk/recipes/2.26.37.71-message-status.json),
[complete methods/signatures/catches](apk/2.26.37.71-message-status/targets.json),
[exact field writers and physical setter invocation inventory](message-status-evidence.txt).
This identifies concrete state sources and the blank-device-receipt entry, not
complete outgoing message initialization or user-row seeding. Offsets in prose
are method-relative bytes; inventory offsets are APK-entry DEX bytes.

## Storage is distinct from receipt advancement

**Confirmed:** 178.B3b() returns field A08:int directly. 178.A0H(int) compares
current/requested values through 1hg.A00; when the comparison is positive it
logs FMessage/setStatus/statusDowngrade. Its branch at 000c targets 006e:
000c + 2*0x31 = 006e. Both logged and unlogged paths then assign the requested
integer to A08 and return. It does not throw or refuse a downgrade, preserve the
old value, validate a receipt status or perform a SQL write.

This differs from [15A's receipt selection](receipt-admission.md), which sends
nonadvancing receipts to the device-only pool before calling the state setter
for advancing inputs. A universally monotonic Rust setter would change behavior;
a setter alone would also lose the caller's receipt admission gates. Equal rank
skips the log but still stores. Unknown-rank handling belongs to the retained
1hg.A00 comparison, not a numeric greater-than shortcut.

**Confirmed:** A0E resets A08 to zero directly. Bn0.A0H and Bn1.A0H override
the setter and ignore their supplied integer, invoking the superclass with **6**.
The [ownership constructor](apk/2.26.37.71-receipt-ownership/message-flag-default.txt)
contains no explicit A08 initialization. **Inferred:** the new object's field
starts at the platform zero default; this does not prove the state of a fully
prepared/sent/restored outgoing message. The mirrored-message A0H(4) calls in
[flag provenance](receipt-ownership.md) must not be substituted for a universal
outgoing initial state. Other constructors/producers remain untraced.

## Explicit writes and restoration

**Confirmed:** the all-DEX writes-only scan finds five physical A08 writer
methods: 178.A0E, 178.A0H, 7z7.A06, D29.Byz and DGG.Bqh. The setter invocation
scan finds **111 physical calls** with declaring class 178 and method A0H;
several share a caller, so this is not 111 distinct workflows. Reflection and
other declaring-class virtual calls are outside that scan. The two overrides
above show why the physical invocation inventory is not full virtual dispatch.

7z7.A06(178,8BW) obtains 7z7.A00(170)'s integer and writes it at 02b8..02bc.
The subsequent [FStatus enum trace](status-enum.md) identifies the decoder's
context-dependent enum mapping. Full stored-row authority remains **unknown**;
this is not a demonstrated SQL row decoder or receipt-time-derived state. D29.Byz(CXl,178,BZY) handles status-mention fields
and, after 7xQ.A03 plus key.A02 checks, directly writes **5** at 00ea..00ee.
Full 7xQ classification and serialized-message trust remain untraced. These
writes bypass the logging setter; there is no shared implicit monotonic guard.

## Normal blank receipt entry is a device path

**Confirmed:** DGG.Bqh(178,35X) obtains target devices through 15x.A07/A09,
checks message row/type/status, and refuses some paths with the exact
ReceiptsProcessor/addmsg not adding blank receipts / no target devices logs.
Its 1im/pre-populated-cache path can schedule DNt discriminator 6 under additional
key/flag conditions; its selected device expansion is now followed in
[the deferred trace](blank-receipt-deferred.md), without establishing user-row
seeding or complete target authority.
Otherwise it calls 0Z4.A08(message,targets) at 010a..0110. If that returns false,
it logs error adding blank receipts and writes **20** at 0124..012c, then builds
Cnn and passes it to 0zO. Full fallback scheduling/persistence behavior remains
unknown; do not classify 20 as a WhatsApp response or generic resend permission.

0Z4.A00 selects its injected 0Z5 for 1im, 0ZE for 1iZ, otherwise ordinary 0Z6.
0Z4.A08 first calls that store's A0C(message,targets); false returns directly.
The ordinary 0Z6 A0C body scan has no match (retained in the inventory);
its inherited 0Z5 A0C delegates to the already traced
[0Z5.A0D pending insert](apk/2.26.37.71-receipt-version/pending-receipt-insert.txt)
with the final Boolean **false**. It catches SQLiteConstraintException, logs
and returns false; it has no retry loop. A0D's nullable per-user version/binding
callbacks are [separately retained](receipt-version.md). Thus this blank entry
connects to device pending membership, not an observed receipt_user timestamp.

The remainder of 0Z4.A08 conditionally builds user_lid_row_id records in
receipt_coex/message_add_on_receipt_coex after specific device, account and chat
gates, with its own transactions/catches retained. These tables are distinct
from receipt_user. **Unknown:** effective coexistence gates, all subtype
classification/target producers and transitive helper effects. The shown path
is not proof of global absence of normal-send user-row seeding. Do not invent
receipt_user membership from blank device targets or a coexistence row.

## Rust boundary and next trace

The existing Rust immutable original targets/versions and mutable exact-device
membership are separate from native message A08 and receipt_user rows. No saved
state is reconstructed or migrated from raw DeliveryFacts by this research.
The [cursor loader trace](message-loader.md) now identifies status/time reads
and one media preparation source. Full ordinary outgoing initialization, live
user-writer admission and counterpart authority remain unfinished.
Those determine which positive receipt creates a real user row and therefore
qualifies the primary identity-cleanup intersection.

No Rust runtime, account, key, request ID or server wait changed. No message,
receipt, database reset or network operation ran. Existing apk-smali,
apk-callers, apk-field-refs and apk-research-bundle reproduce this work. The
research API remains CLI-only, x-live=false. Recipe selectors need reviewed
new-release relocation; no complete remap or semantic parity is claimed.

Message-status validation: eleven topics / 16 files / 79,539 bytes reproduce
exactly from APK 37.71. Cached self-diff verifies eleven unchanged topics and
four metadata artifacts, not cross-release semantic parity. CLI-reference
regeneration, OpenAPI/library-symbol checks, formatting, whitespace and hygiene
passed. Hygiene checked 1,499 local links and 40 bundles / 9,411,474 bytes.
Caller/writer inventories and the no-matching ordinary override body result
remain separately attributed. No fresh workspace/runtime or live test is claimed.


## Ordinary text construction and excluded setter candidates (2026-10-09)

**Confirmed static:** same 2.26.37.71 APK/hash as above. The
[17-topic recipe](apk/recipes/2.26.37.71-text-status-initialization.json) and
[snapshot](apk/2.26.37.71-text-status-initialization/targets.json) preserve the
ordinary text chain and examined alternative setter candidates. This narrows
outgoing initialization, not native receipt_user insertion or a Rust cleanup
implementation. Offsets below are method-relative bytes.

### Ordinary user-input text chain

B1l.A00 identifies itself with
`UserActionsTextMessageSending/createFMessageTextFromUserInputs`. It optionally
normalizes the supplied chat through 0Yo.A0D (`0006..002e`), builds preview
callbacks and calls 1Ze.A00 at `00b6..00bc` or `024a..0250`. Both timestamps
come from **08B.A00**, the NTP-preferred getter; the token-job A04 getter must
not be substituted. Complete effective chat/type/preparation gates remain
unknown. No live text, key or request ID was generated for this trace.

1Ze.A00 calls 0od.A03(chat,true) and constructs 1he with key/string/time
(`0010..0026`), optionally applies a copied list with 1ib.A04, invokes its
supplied APu callback (`003a..0044`) and performs 1Zg quoted postprocessing
(`0044..004e`). It contains no explicit status setter. 1he's string constructor
calls its key/time constructor, which calls 178's key/type/time constructor
with type **0**. The explicit 1he constructor writes affect its own A04/A00/
A05/A03/A02 fields, not inherited message status A08; A00=-1 is not A08=-1.
The inherited constructor was preserved by the earlier ownership bundle.

7gT.A00 selects Fex when its 8BJ preview input is absent; Fex.APu is exactly
return-void. With preview input it selects 8GL.APu, which writes preview fields
and invokes additional preview helpers; those transitive effects are not
claimed completely traced. 1Zg.A00 returns immediately on null quoted input
(`0000 -> 023c`). Nonnull quoted handling clones separate message objects and
attaches them, with helper/serialization side effects retained but not fully
interpreted here.

**Inferred bounded initial value:** at the immediate plain-text construction
boundary, absent optional list/callback/quoted effects, A08 remains the platform
zero default. This differs from the previously traced media factory's explicit
A0H(1). It is not proof that every prepared B1l result or fully queued/sent text
retains zero: 1ib, preview/transitive callbacks and B1l's later metadata helpers
remain separate authority. Do not rewrite Rust send reservation states from
this native integer or create receipt_user rows from a constructed message.

### Separate workflows, not universal initial-state evidence

The exact 1Ze.A00 physical caller inventory contains 13 invokes (B1l has two).
Other callers include question composer, click/run/composition flows and 1Zb;
this inventory does not prove every caller's subtype, timing or send behavior.
Reproduce with `apk-callers --class 'LX/1Ze;' --method A00`.

1Zb.Ag9's two accepted input subtypes return through a shared **A0H(6)** and
flag write (`005a..0072`). One creates 1he through 1Ze; another uses 1Zh.A00.
1Zh explicitly calls A0H(0) after either EIH or 72l construction. Thus an outer
workflow can replace a factory's initial state; 6 is not an ordinary text
factory default. Complete 1j2 subtype semantics remain unknown.

The earlier setter inventory also led to seven scoped exclusions, now retained
as full bodies rather than repeatedly searched as ordinary send factories:

| Method | Confirmed reason it does not establish ordinary text initialization |
| --- | --- |
| 0VA.A00 | Cursor-based message restoration, including status loading/setter; not a new text constructor. |
| 0Z0.A09 | Cursor restoration of a 1iZ add-on's status/server timestamp and metadata. |
| 0ZG.A06 | Poll delivered-snapshot restoration; resets a selected vote message then invokes status 4. |
| 0zH.A09 | Local read-state update selects 16 or 17, then persists through 0zc. |
| 12y.A00 | Received content builds 1il; its conditional status-4 branch depends on direction and A01 policy. |
| 1fm.A04 | Orphan/add-on resolution; selected outgoing-key reconstruction calls status 4 and writes A0y. |
| 17J.A01 | Add-on error/bounce path; the selected setter receives the caller's status argument. |

0zC.A09 is an additional examined settings-preprocessing path. Its explicit
status **13** at `0422..0428` follows account/device/settings predicates and
then A0Q; it is not an unconditional initial outgoing state. The large selected
method contains ephemeral pre/postprocessing beyond the interpreted branch;
retention is not a complete semantic port. No row deletion, reservation replay
or blanket status assignment follows from any of these exclusions.

**Remaining authority:** actual live per-user receipt creation still needs its
status/ownership/target admission, native message preparation/load context and
canonical scope. Rust raw DeliveryFacts, immutable send versions and mutable
device targets cannot be relabelled receipt_user membership from this trace.
No runtime/store/API route or live operation changed in this source chunk.

**Text-source verification:** exact re-export verifies 17 topics / 22 files /
296,181 bytes; cached self-diff reports 17 unchanged topics and four unchanged
artifacts. CLI-reference regeneration, API-symbol integration, diff whitespace
and repository hygiene pass (1,823 local links; 76 checksum bundles). Rust is
unchanged, so the preceding 821-test/Clippy/Rustdoc run is not counted as a new
runtime validation here. Private scans stay local; examined findings, negative
results and bounded selectors are retained in this document/recipe.

## Blank-receipt failure logging continuation (2026-10-09)

**Confirmed static, APK 2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
[four-topic recipe](apk/recipes/2.26.37.71-blank-receipt-failure-log.json) and
[checked snapshot](apk/2.26.37.71-blank-receipt-failure-log/targets.json).
Offsets are method-relative bytes. This closes the previously untraced
Cnn/0zO call boundary; it does not establish ordinary outgoing preparation.

The retained DGG.Bqh failure branch (0124..015a in
[original source](apk/2.26.37.71-message-status/blank-receipt-message-write.txt))
writes message A08=20, constructs Cnn(message,null), sets builder A05=3 and
A0G=true, then calls Cnn.A00. Cnn's constructor stores the message/entity
references and defaults A05=1; the caller overrides that builder value.
These are separate fields: builder 3 is not native message status 3.

Cnn.A02 copies its fields into a new CgL value. The checked register flow
passes A05 as the first integer constructor argument, and A0G as the first
Boolean argument. Cnn.A00 builds that value and directly invokes 0zO.A01;
there is no intervening send-job enqueue in those two methods.

0zO.A01 identifies its work with `MessageLogging/buildMessageSend` strings,
builds Bh0 and other event objects, and submits them through 0BW.CGQ.
For example, 21aa..21b8 submits the populated Bh0; 3030..3036 submits
a separate 2U5 event. Its terminal-failure diagnostic at 023a..0272 is a
conditional branch, not proof that every Cnn call is terminal.

**Negative result / unknown:** the complete selected 0zO.A01 body contains
no direct 178.A0H call, direct 178.A08 field access, or receipt_user literal.
Together with the event-producing call chain, this refutes using this boundary
as evidence of another message-status setter or ordinary send scheduling.
It does not prove absence of transitive helper side effects: numerous provider
and helper calls remain uninterpreted. Full event-field semantics, queue/store
atomicity, native per-user receipt membership and primary cleanup remain
**unknown/unported**. No Rust runtime, account state or live request changed.

Verification: the four-topic, nine-file, 213,751-byte export reproduces from
the pinned APK; cached self-diff checks identity, not a second release.
CLI-reference and API-symbol checks, formatting and repository hygiene are
run for this source-only chunk; no fresh full runtime suite is claimed.

Follow-up: [text store submission](text-store-submission.md) connects B1l's
construction result to the selector-4 asynchronous store handoff. Scheduling
reason integers are not status setters; complete A09/send transition remains
unknown. Payment, status ACK and unsent-recovery candidates are retained as
separate contexts rather than universal text initialization.
