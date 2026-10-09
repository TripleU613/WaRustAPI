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
Complete A07 and C8L semantics remain untraced.

The broadcast branch normalizes a user through 0Yo.A0C, loads an existing
individual message using the original ID/direction, and builds a separate CYV
for its primary device and original time/status. Missing messages log and skip;
there is no recreation or new request ID in this branch. Full broadcast
recipient expansion/mapping is unknown. 0DE.A0R's numeric type 3 is anchored
by the broadcast log; A0i is type 5, A0m admits types 1/2 while excluding
A0W's type 28. Complete public JID-class mappings remain separately untraced.

1ht.A03's special-message predicate checks 0hd.A02, message A0z,
1ib.A02 -> its private A00 user selection, message A02 and 1id.A00.
Its transitive message classification/selected user remains unknown; the
predicate is not permission to infer single-participant behavior from row count.

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
