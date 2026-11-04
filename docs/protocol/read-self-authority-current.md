# Current read-self authority and wire

2026-11-04. **Confirmed static** WhatsApp Android **2.26.43.72**, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
[Complete selected records and checked DEX](rea/2.26.43.72-read-self-authority/README.md).
This extends [current outgoing selection](read-receipt-policy-current.md).
No receipt code or private account state changed in this chunk.

## Eligibility is loaded Me plus an active own-device store

Current `0jD.A02` uses interface `09F.BUx`. Both concrete implementations
identified in the [server-role audit](rea/2.26.43.72-server-sync-role-recovery/README.md)
are now closed for this method: checked `09G.BUx` directly forwards to
`09O.BUx`, which calls `A07` then tests **A0H != null**. A0H's type is
`0E2`. This is loaded-Me presence, not the distinct `registration_device_id>0`
companion-mode predicate BU7/BU8. REA's no-body display for09G is superseded
by the physical DEX forwarder, not interpreted as missing implementation.

`09O.A07` schedules `1Wk` discriminator1 through its09a load coordinator when
not already loaded. `1Wk.call` reads `<filesDir>/me` via `0D0.A01`, handles
identity metadata, assigns through synchronized `09O.A00` and finishes load
coordination. A00 updates the Me object/PN/LID providers and derives own device
JIDs; invalid identities clear through A01. Full file parser, key binding,
registration/pairing ingress and invalidation are **unknown** here. A configured
phone number or current account registration enum is not substituted for this
native persisted object.

When BUx is false,0jD.A02 returns an empty set. Otherwise it obtains
`1nG.A08().keySet()`. A08 iterates `1nH.A06`, excluding values for which
`F0I.A04` is true. The new current checked loader closes the field identity:

- `1nH.A07` obtains column `logout_time` at0074..0080, reads that long at
  0372..037e into v74, then supplies it as the **third long** toF0I at03f4..03fa.
- `F0I.<init>` assigns that third long toA02 at002e..0036. The first two longs
  are last_active and login_time; they are not interchangeable with logout.
- `F0I.A04` compares A02 tozero and calls2L9.A1N, now traced as `i>0`.
  Thus positive logout time excludes the entry; zero/negative pass this filter.

The SQL producer1nN.<clinit> builds `SELECT <1nO column array> FROM devices`;
no WHERE/ORDER BY is appended there. Invalid DeviceJid parsing skips a row,
the immutable map is cached, and skipped-row count is retained. The complete
column array and cursor wrapper are not interpreted as an independent
authorization check.

Previously retained current1nH.A06 establishes own-LID-to-own-PN conversion:
preserve the device number and metadata, copy other entries first, and let an
existing PN record win. Conversion is not discovery of another linked device.
This set supplies a nonempty **eligibility test** in0xW.A06/A09. The selected
read job/writer does not enumerate it into one outbound receipt per device.

## Current writer preserves read-self and original route

The job's currentA0L dispatches Message selector89 containingEXu. The existing
full checked writer0v0.A00 has switch00f0 case89/419 ->1f18; it calls
`F3e.A07(EXu)` at1f50..1f56 and writes that same node through1rj.A04 at
2006..2010. The participant diagnostic between these calls does not strip or
replace fields. This reuses retained whole-method evidence rather than another
full decompile.

EXu stores key/chat/id, selected type, optional participant, remote sender,
recipient, class, peripheral origin, extra IDs and server-store time.
`F3e.A07` passes type unchanged throughA00 with mode0. A00 callsExf.A05,
then emits receipt/to/id, nonnull type, participant only when nonnull and
different fromto, nonnull recipient, positive sts, and optional class. Remaining
IDs become list/item/@id in order throughA0A; a single ID has no list.

For a plain PhoneUserJid (current getType0) or0Dx logical-user JID (getType18),
no participant and no remote sender, the route helper returns that chat as to
with no participant. A selected read-self therefore has the ordinary shape:

```xml
<receipt to="original-chat" id="original-message-id" type="read-self"/>
```

It is not automatically addressed to our own primary JID. The eligible own set
is not copied into destination attributes by this selected writer.

CheckedExf.A05 preserves the full route alternatives:

| Condition | to | participant candidate |
| --- | --- | --- |
| A0m(chat) or A0P(chat) | chat | remote sender if supplied, otherwise supplied participant |
| Otherwise A0P(participant) | supplied participant | remote sender if supplied, otherwise original chat |
| Neither special branch | remote sender if supplied, otherwise chat | supplied participant |

A0m is A0l or type28; A0P tests types3/5/6. Human-domain names and all producer
authority for these specialized cases remain **unknown**. The Java display
loses the swap in the second branch; checked0024..0030 proves it. The final
builder omits participant when equal toto. Recipient/class/sts need their own
trusted producer inputs; they must not be invented from the ordinary example.

`Eqi.A00` can add meta fields, but the selected job passes mode0 and the
peripheral allowlist is exactly null/inactive/read (`Eqi.<clinit>` plus6oe.A1b).
Consequently **read-self gets no mode/peripheral meta child** through this
path, even when a peripheral string exists. This does not suppress unrelated
extra-ID list or class attributes.

## Own-store mutations and remaining ingress authority

The exact1nH.A01 cache-field inventory contains11 physical references: loader,
two0WB cleanup paths,1nG removal/addition,1nH removal, andG0j.A03. These are
structural references, not a proof of every dynamic write.

Current1nG.A0F replaces a row in devices fromF0I, including logout_time,
ADV index, login/active times and metadata, then invalidates raw and converted
caches and runs additional device-state side effects.1nG.A00 distinguishes
deletion from marking devices logged out with current time, protects the own
primary against removal and invalidates both caches.1nH.A01's guarded removal
rechecks metadata before deleting, rather than treating every stale snapshot
as removal authority. Their complete downstream callbacks remain **unknown**.

Two current static callers add devices: FeN.run discriminator14 supplies a
pairing-contextF0I throughCompanionDeviceManager; Fei.run discriminator43
constructsF0I from a device/key-index entry with zero timing defaults before
calling1nG.A0F. Their product entry-point, pairing cryptographic validation,
account-sync response correlation and complete ordering still require tracing.
Zero-valued timing fields in this constructor are native producer behavior,
not permission to manufacture a device in Rust.

## Rust inventory is not this authority

**Confirmed repository mismatch:** transport/devices.rs stores contact
DeviceList and signed ADV/peer-pin provenance. `request_sync` explicitly refuses
own-account identity and directs it to account sync. Its ingestion always adds
the primary owner, even if the server supplies no companion device. Therefore
`!DeviceList.devices.is_empty()` would enable read-self incorrectly. Its
companions_current flag validates a contact list against a current peer pin;
it carries no own-device logout_time store or native pairing/deletion lifecycle.
Identity application also defers own identities. Those verified contact rows
must not be reinterpreted as an eligible native own-companion set.

No existing supported Rust owner exposes the required loaded-Me plus current
active-own-device eligibility. This does not prove the private account has zero
companions; **unknown** is retained. The [automatic normal-read negative guard](read-receipt-policy-current.md)
therefore keeps disabled-preference work pending rather than guessing read-self.

Next exact work: authenticated creation/reconciliation of the own devices map
from19N devices ->0Vy workers and their request callbacks; FeN14 pairing-context
validation; Fei43 input-map authority; remaining0WB/G0j cleanup writers; Me file
parser/creation and account isolation. Only after that authority is represented
durably should a read-self intent be selected/reserved. Full contact/force/status
policy and native scheduling remain separate obligations. No live operations,
account/device invention or receipt transmission occurred.
