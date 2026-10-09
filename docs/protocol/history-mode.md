# Stored companion mode behind history admission

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Eight-topic recipe](apk/recipes/2.26.37.71-history-mode.json),
[complete methods and exception tables](apk/2.26.37.71-history-mode/targets.json),
[preference anchor and setter caller inventory](history-mode-evidence.txt).
Offsets are method-relative bytes unless the inventory labels DEX byte offsets.
This narrows the predicate boundary in [history queue staging](history-queue.md).

## Reuse the established own-user authority

**Confirmed earlier:** [stored own identity](own-identity.md) already traced
08i.BO0 and its 08a proxy. The predicate compares the input with stored own
PhoneUserJid or LidUserJid after their load/initialization. That earlier source
is unchanged; the history queue document's untraced BO0 wording meant the
immediate queue trace, not an absence of repository evidence. Full Me/prefs
import authority and repairs remain unfinished.

The no-body lookup of 08l.BO0/BMv was a wrong implementation anchor, recorded
in the inventory so it is not mistaken for proof these capabilities are absent.
No global interface implementation closure is claimed by the new mode trace.

## Mode is the saved registration device ID

**Confirmed:** 08i.BMv calls BMw(false). 08a.BMv directly calls the same
08i.BMw(false), rather than supplying a different mode policy. BMw first
calls A09, then its false-argument branch reads SharedPreferences
`registration_device_id` with default 0 and returns whether the signed int
is greater than zero (000e..002e). This path does not use the AtomicInteger.
08i.Apt independently reads the same preference with default 0 and returns
the integer without a positivity conversion.

The true-argument BMw branch uses 08m.A02's AtomicInteger. If its current
value is Integer.MIN_VALUE, it reads the preference, attempts
compareAndSet(MIN_VALUE, value), then reads the atomic again before the same
positivity test. 08m's constructor initializes that exact sentinel.
It is a cache variant; the history BMv call passes false. No negative/zero
value is transformed into a companion device ID.

08i.BNm calls A09 and tests whether A0F Me is nonnull. Its 08a proxy delegates
to that method. It is Me presence, not a signed-bundle check or confirmation
that the history database is ready. The queue caller separately consults
0Gc.A0C. Full load ordering and database readiness semantics remain unknown.

## State writer and bounded producer trace

**Confirmed:** 08m.A00 writes registration_device_id with putInt/apply,
then assigns the supplied integer to the atomic cache. It performs no local
range validation or cryptographic verification. Its separate A01 writes
`companion_registration_state` with commit; the Boolean result is not inspected
in the shown method. These are separate fields, not one interchangeable status.

The exact A00 caller scan finds three physical invokes: twice in 08i.A01 and
once in 11J.BFC. The previously retained
[08i.A01 source](apk/2.26.37.71-own-identity/set-own-pn.txt) passes constant 0
when the stored PN is absent or parsing fails. A valid PN instead constructs
its DeviceJid using the saved Apt() value. It does not derive a positive
registration device number from the PN's digits.

11J.BFC's retained companion-registration completion path calls
Jid.getDevice() on its selected JID, then A00 at 1210..1222. Adjacent branches
store platform and ADV identity data; the full handler is retained but its
selected JID producer, signed identity verification, event type admission,
registration state transitions and side-effect ordering are **unknown** here.
The presence of that setter is not authority to invent a device number or
consider a companion paired.

The exact registration_device_id anchor scan finds seven constant references
in six physical methods (BMw has two). GEu.A02, 0g1.A0b and 1LA.AAT are
recorded but untraced; the scan is not a complete preference-write inventory
for computed keys or other indirections. No preference, identity or mode was
changed during this analysis.

## Port consequence and reproduction

**Inferred:** own-user membership, positive stored device ID, Me presence,
registration state and cryptographic linked-device trust are distinct inputs.
The history queue's mode gate can be reproduced only from actual saved account
provenance; a mode flag alone cannot grant the other inputs. Complete companion
registration, peer-envelope authentication and history-media validation remain
unfinished, as does actual receipt_user admission for primary cleanup.
No Rust runtime, reservation, request ID or live request changed.

```sh
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-history-mode.json \
  --output docs/protocol/apk/2.26.37.71-history-mode --check
cargo run -p xtask -- apk-research-diff \
  docs/protocol/apk/2.26.37.71-history-mode \
  docs/protocol/apk/2.26.37.71-history-mode
```

Static artifact checks do not establish successful pairing or live sync parity.
