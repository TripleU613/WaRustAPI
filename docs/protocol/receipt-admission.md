# Receipt admission and separate state/timestamp paths

2026-10-09. **Confirmed static**, APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[13-topic recipe](apk/recipes/2.26.37.71-receipt-admission.json),
[complete methods](apk/2.26.37.71-receipt-admission/targets.json),
[raw status ordering fixture](receipt-status-order-evidence.txt).
This traces 15A.A02 selection toward the device/user writers. Full native
account predicates, broadcast recovery and post-write effects remain unported.

## Admission before splitting

**Confirmed:** 15A.A02 enforces at most 300 input values with an assertion;
this is a native local collection bound, not a server request/rate limit.
Each input must satisfy 1hg.A03's six-status predicate. It chooses original
UserJid when provided, otherwise device.userJid, then derives chat predicates
0DE.A0m/A0i/A0R and a 1ht.A03(message) && 0DE.A0l(user) condition.
Nonpositive CYV timestamps log and skip the input (014e..0192). This is
separate from parsing a recognized direct receipt with an absent `t`.

The message-key A02 boolean and 1gd.A0d participate in the ownership check.
A value failing that check takes a special path gated by 08Z.BO1(device),
Cju.A01(device) and property object AzZ.A04. Foreign-device results can be
skipped under that property; effective property/account values remain unknown.
Do not substitute a fabricated provenance or unconditional true predicate.
1gd.A0d checks the message-key boolean, and for message type 0x40 consults
Bps.A00 via BO0, then tests message A0y. Those full message-type/field meanings
are not inferred merely from the helper's name. Cju.A01 tests equality of
userJid with its static 0hh constant; it is not a general companion predicate.

The ordinary branch rejects B3b()==6 with `invalid message`. Status 15, or
status 5 with BO0(selected user) true, goes to the device-only pool with
`statusunchanged`. Other values use the chat-dependent state comparison below.
BO0/BO1 implementations and complete own-user mode authority remain unknown
in this trace; the existing saved-account authority must remain intact.

## Ordering is not numeric status comparison

**Confirmed:** 1hg.<clinit> installs this ordered array:

`[14, 0, 1, 2, 20, 21, 3, 4, 15, 5, 11, 12, 13, 8, 18, 17, 16, 9, 10, 7, 6, 22]`.

The [raw fixture](receipt-status-order-evidence.txt) independently retains
all 22 entries at classes2.dex 0x0d8d08; apk-smali's current 64-byte preview
truncates the last six, so those values were read from the exact APK payload.
1hg.A00(current,new) searches indices and returns -1 for forward movement,
1 for backward movement, 0 for equal known ranks. A04 returns true when that
result is nonnegative: **equal is included**, despite `statusdowngrade` logs.
Unrecognized status behavior must be checked against the retained loop rather
than inferred from this known-value description.

At 02a0..0300, eligible ordinary individual messages with a nonadvancing
status are kept in the device-only pool. Forward inputs use the individual
message pool. The multipart path similarly loads per-user caches via
13v.A00 -> 8G8.A03, gets the selected user's 7lk.A00 aggregate status and
routes nonadvancing inputs to the device-only pool (06da..076a). Inputs
without a prior cache/user row go to the multipart update pool.

**Confirmed consequence:** 07b4..07ba passes the device-only pool to 0Z4.A06.
Thus an equal/lower status can still update an earlier device timestamp, while
not advancing per-user/message state through this path. An implementation that
returns early on a duplicate/lower status and discards all timestamp handling
would miss this split. Conversely these branches do not authorize promoting
Rust's independent raw observation facts into a native aggregate state.

## Distinct pools and writer path

The individual-message pool updates message status before adding values to the
first manager pool at 061e..0624. It contains ordinary individual advancement
as well as a resolved broadcast individual copy; it is not exclusively broadcast.
The multipart update pool becomes the second manager pool. Before persistence,
13v's A03 key-set field and message A0l filter both collections at 08f6..099c. The
selected lists are concatenated and passed to 0Z4.A06 at 0a00..0a06, then to
the [per-user writer](receipt-user.md) already traced. Full origin/lifecycle
of 13v.A03 and A0l semantics remain unknown; no deletion/tombstone meaning
is invented here.

A separate normal state-only pool at 07f0..0882 maps original receipt status
8 -> state argument 10, 13 -> 16, and 17 -> 17; other values use -1 and skip
that A07 call. C8L.A00 and a 0Fs row comparison can suppress the call. These
state arguments are not the receipt_user SQL column selectors (5/8/13).
They must not be conflated with outgoing wire `type` or cached timestamp slots.
A07 admission/persistence is now traced in [state-only updates](receipt-state-only.md);
The immediate [C8L/chat-row suppression boundary](receipt-state-suppression.md)
is now traced; the full chat read-state lifecycle remains unknown.

The broadcast branch normalizes a user through 0Yo.A0C, loads an existing
individual message using the original ID/direction, and builds a separate CYV
for its primary device and original time/status. Missing messages log and skip;
there is no recreation or new request ID in this branch. Full broadcast
recipient expansion/mapping is unknown. 0DE.A0R's numeric type 3 is anchored
by the broadcast log; A0i is type 5, A0m admits types 1/2 while excluding
A0W's type 28. Complete public JID-class mappings remain separately untraced.

The subsequent [receipt route trace](receipt-route.md) retains 1ht.A03's
complete helper chain: A0z bypasses metadata selection only, and 1ht.A00
selects the first qualifying DLg user rather than counting recipients. Message
flag/descriptor producer authority remains unknown; neither predicate nor row
count proves single-participant behavior or receipt_user existence.

**Unknown/unported:** effective property and own-device predicates, special
message/broadcast/counterpart authority, manager exclusion lifecycle, remaining
post-write callbacks and state projection, all send-type seeding and complete
mutable cleanup/fetch/final resend recovery. Native nonpositive-time admission
is documented without deleting historical raw Rust receipt facts. No runtime
transport, saved account, keys, IDs or waits changed; no live operation ran.

Reproduce the bundle with the linked recipe, --output and --check. Recheck the
raw fixture at its APK entry/offset separately. For new releases re-anchor into
a separate generation and use apk-research-diff; rendered/static changes alone
do not establish live protocol parity. No CLI/API capability was introduced.

Receipt-admission validation: 13 source topics / 18 files / 121,518 bytes
reproduced exactly from APK 2.26.37.71. Cached self-diff verified 13 unchanged
topics and four metadata artifacts; raw 96-byte/22-entry status-order evidence
independently matches classes2.dex at its recorded offset. Repository hygiene
checked 1,393 local links and 33 APK bundles / 8,507,021 bytes; diff hygiene
passed. Research/docs only; no fresh runtime tests or live account operation
is claimed.

Follow-up: [receipt-manager exclusions](receipt-manager.md) identifies A03's
constructor and revoke-path producer, successful-close registration and the
message A0l deletion/copy context. The earlier device-only pool precedes this
manager exclusion. Complete other flag writers/reload and revoke admission
remain unknown; no blanket receipt rejection is inferred.

Follow-up: [stored own identity](own-identity.md) resolves BO0/BO1's proxy/direct
implementations and PN/LID sources. Effective account contents, remaining property
selection, complete per-user projection and own-device lifecycle remain separate
unknowns; predicates do not consult arbitrary directory aliases.

Follow-up: [ownership short circuit and flag provenance](receipt-ownership.md)
establishes property 35516's true/true baked defaults and the ten physical flag
writers. 15A skips the helper/foreign property when key.A02 is true; requiring
A0y authority for every outgoing receipt would add a guard absent in this branch.
Effective configuration and later pool/write admission remain separate unknowns.

Follow-up: [message status storage and blank device entry](message-status.md)
identifies A08, downgrade logging followed by assignment, zero reset and the
two status-6 overrides. Exact field writes and blank-receipt fallback are
retained; ordinary outgoing initialization and user-row seeding remain unknown.

Follow-up: [receipt key direction and state-only updates](receipt-state-only.md)
traces parser direction, A07 admission, played eligibility and the core update
result. Self receipt status does not by itself determine key direction, and an
A07 true return does not prove every downstream database update succeeded.
