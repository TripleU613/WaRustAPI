# Device synchronization before native resend selection

2026-10-09. **Confirmed static**, APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The [recipe](apk/recipes/2.26.37.71-sync-resend.json) preserves the native
job, all-device key-fetch response, original device set and target filters.
[Version comparison](receipt-version.md) is only one stage of this job.
[Original-send snapshots](send-version-snapshot.md) preserve one Rust input;
they do not yet preserve the native per-device receipt lifecycle.

## Job inputs and synchronization branch

**Confirmed:** SyncDeviceAndResendMessageJob's constructor accepts an original
message key, an array of user addresses, start/expiration milliseconds
and a Boolean. It preserves the original message ID/chat and both times, builds
an exact user set and saves raw user strings. These are user recipients, not a
list of device addresses. A0G first checks BNm login and a positive expiration
against 08B.A00; expiration strictly before the current value logs and returns.
An equal time reaches execution (005e uses if-lez on current-minus-expiration).
Neither clock adjustment nor the producer of the constructor Boolean is newly
established here.

A00 resolves the saved original through COS.A00 and returns null for missing
messages or the shown disabled mapped-status case. It also has native message
conversion/update branches, whose complete semantics are unknown. A0G reads
again after synchronization: a message disappearing while waiting is not recreated.

A0G 00dc..0124 chooses between two synchronization paths using 0DE.A0Z,
0Fs.A0Y, 0DE.A0i/1iZ and 0bA.A0k. True routing guards jump to 04f4, the
0ss.A09/0xX.A0M user-sync path; they are not all hard rejection predicates.
Otherwise 0124..0182 copies the users and removes 08Z.Apx's own user. An empty
result reports reason 8 and ends the run. The log mentions a self device, but
the actual collection and removal are user addresses. Do not turn this into an
inferred per-device exclusion policy.

0182..01ae obtains Can.A04's Boolean and calls Cio.A02(users,false) for the
missing-LID preparation; its caught exception is logged and the fetch path
continues. Effective routing/mapping/PQ inputs and complete Cio behavior remain
unknown. This is not authority to manufacture LIDs or silently merge namespaces.

## The all-device key-fetch request

**Confirmed:** 01ec..0286 iterates remaining users. For each, 0bN.A0B yields
known device addresses. It reads Cfc.A01/CNd.A00/BYp.remoteRegistrationId_; only
positive values enter that user's nested device-to-registration map. The outer
map contains the user even when no device has a positive registration ID.
A local missing registration is omitted, not replaced by an invented value.

02da..04c2 constructs this distinct IQ shape:

```text
iq id=<generated> xmlns=encrypt type=get to=<1S7.A00>
  key_fetch [pqsupport=true] [context_jid=<original chat>]
    user jid=<actual user>
      device id=<actual device number>
        registration <four binary bytes>
```

The child is literally `key_fetch`, not `key`. Each registration is the existing
session's positive remote ID encoded by 1Lz.A03 as four bytes, most significant
first; it is not decimal text. User children can have an empty device list.
The Boolean from Can.A04 controls pqsupport. context_jid additionally requires
property 0x6530, a nonnull chat and the shown 0DE type checks. Their active
values are unknown. The 0h8.A0O call carries request kind 0x15a and timeout
0xfa00 (64,000 milliseconds), then waits on the callback future. That native
timeout is not a newly authorized Rust retry or server-directed cooldown.

At 04d2..04f2, the original job user set is passed to 101.A03 with integer 3.
The all-device callback Boolean or alternate 1hI.A01 result controls progression
at 0526. The transitive observer meaning of 101.A03 is not newly established.

## Result completion is not per-device usability

**Confirmed:** DFC.C8f reads `list`, user `jid`, user `error/code`, and nested
device `id`. User errors enter a separate map. A device error logs its code/text
and skips that device. Non-error devices supply required identity/registration/
signed key, optional device-identity/one-time key/PQ key, and a one-byte type
(default 5 when type is absent; wrong present length throws). Cai objects carry
these raw inputs. Native trust/session validation happens later; parsing bytes
is not validation of their signature or origin.

The device list is sorted using DPh selector 0 through DO5 selector 8. DPh's
branch subtracts numeric device IDs, and its B7l catch returns equal. The wrapper
and complete malformed-sort behavior are not newly traced; do not invent a
strict total-order error policy from this comparison alone.

At 0358..039e, a nonempty Cai list queues DNs selector 47 through 0kY.A01.
DNs's constructor assigns arguments 1/3/2 to A00/A01/A02, preserving the
owner/list/future ordering through A00's factory. DNs.run's case table selects
08ce; it calls 14J.A04(list,0), then completes the
future with 1xh.A1A (true). 14J's traced A04 uses CPB admission and 0jc.A0L
session installation; a nonzero installation status logs an error and still
continues its normal processing. Full CPB/native Signal/ADV authority is unknown.
Thus future completion must not be equated with every requested device having
a valid newly installed session.

When no Cai objects were produced, DFC directly completes true at 03e2..03f4,
then iterates/logs the separate user-error map. Missing list, no devices, skipped
device errors or only user errors can therefore complete without a usable bundle.
Conversely BmU and BjL complete false using 1xh.A19 for IQ error/delivery failure.
The two Boolean helpers were read independently. No raw server error text
should be echoed by a Rust inspection path.

## Candidate subtraction and final filtering

**Confirmed:** after success and reloading the message, A0G 0544..0550 reads the
original receipt-device set through 0Z4.A04. For ordinary message storage,
0Z6.A0O resolves the message and calls 0Z5.A05; missing messages give an empty
LinkedHashSet. A05 copies the key set of the message's 7hf receipt cache. There
is no timestamp test in A05 itself. This is receipt-row membership, not only
our delivered/read/played facts; the transitive cache load remains untraced.
The native set is the receipt cache at job execution, not proven immutable
initial targets. [Identity cleanup](identity-receipts.md) can delete selected
receipt-device rows and invalidate that cache; its complete effect on this
subtraction still needs a coordinated receipt-lifecycle port.

For 1ii messages A0G calls 15x.A08. Otherwise it requires the outgoing Boolean
and raw message A0C long equal to zero before 15x.A04. The name/meaning of that
long is not inferred merely from its zero check. A null or empty candidate set
returns. 05da removes all original receipt devices from the candidate set,
then exits if nothing remains. Only this difference reaches primary-version
comparison. A changed current version alone does not select original devices
for another transmission.

15x.A04 has separate message/chat paths: 1Yx.A04 selects an alternate target,
0c3 starts from prior receipt devices and expands their user device lists,
0DE.A0l routes through A06, and the remaining path selects own-device lists
through CPQ and removes BLM-matching devices. A08 adds its own inbound/outbound
and message guards plus conditional PN/LID map key/value selection. The complete
mapping/recipient gate implementations are unknown; exact raw branches are
preserved rather than treating any cached directory alias as authority.

After version filtering, 0802 calls 15x.A09 on the selected set. A09 can mutate
it again: its group addressing-mode branch removes unsupported JID types;
a migration branch retainAlls resolvable map keys; its later branch applies a
DPG selector 30 filter. Those predicates' complete semantics remain unknown.
The earlier version equality result is not the final wire target set.

Only then 087a seeds receipt rows with A09 and 0880..08d6 creates BpI(original
message), CMQ/Ca1, explicit targets, original start/expiry and two false Booleans,
and submits through D5K.A00. The job awaits that separate sender future.
Reusing a message object is not proof of ciphertext reuse, new wire ID,
ratchet advancement or recovery behavior; these sender consumers are untraced.
A0H examines job requirements; A0I returns true for the supplied exception.
That native job retry decision is not a reason to relax Rust uncertain-send guards.

## Rust boundary and reproducibility

**Confirmed local audit:** searching current API/CLI source for `key_fetch`
found no implementation. devices::after_primary_identity_change_in schedules
existing device synchronization; it does not implement the above fetch/job.
Current send snapshots contain exact user namespaces, distinct device counts
and nullable versions, but no original per-device receipt membership. Counts
cannot reproduce removeAll. Native receipt insertion/removal, cache loading,
original-device provenance, authoritative fresh target selection and final sender
recovery still need ports. Historical snapshots are never backfilled or relabeled.

No new request, automatic resend, pin replacement, receipt deletion or account
migration is enabled by this research. A usable Rust path must validate signed
bundles and authenticated device admission, preserve the original send and
per-device ledger, apply message/expiry/version/namespace policies, and reserve
its distinct transmission atomically without replaying uncertain attempts.

Reproduce with existing CLI tools:

```sh
cargo run --locked -p xtask -- apk-research-bundle \
  ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-sync-resend.json \
  --output docs/protocol/apk/2.26.37.71-sync-resend --check
```

Use a new separate recipe/export for a different APK, then apk-research-diff on
the two cached bundles. A self-diff verifies the tool plumbing, not another
release's equivalence or a live worker. This chunk performs static analysis only.

Sync/resend research validation: all 22 source topics / 27 files / 272,318
bytes reproduced from the exact comparison APK. Cached self-diff verified
22 unchanged topics and four unchanged metadata artifacts in 0.031055 seconds;
this is plumbing verification, not cross-release equivalence. All 65 xtask
tests, CLI-reference regeneration, API/library-symbol check and formatting
passed. Repository hygiene checked 1,324 local links and 26 APK bundles /
7,808,872 bytes. Runtime changes are comments only; no account operation,
key-fetch query or live resend was performed.
