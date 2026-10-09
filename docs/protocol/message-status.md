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
key/flag conditions; that task's complete effects are **untraced** here.
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
