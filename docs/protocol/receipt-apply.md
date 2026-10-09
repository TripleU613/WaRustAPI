# Incoming receipt application handoff

2026-10-09. **Confirmed static**, APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The [eight-topic recipe](apk/recipes/2.26.37.71-receipt-apply.json) preserves
[full selected methods and signatures](apk/2.26.37.71-receipt-apply/targets.json).
This extends [receipt parsing](direct-receipts.md) toward the native per-device
row lifecycle; it does not add a Rust cache or change receipt-send reservations.

**Confirmed:** 14e.A04 calls Cmk.A03 to construct DdM, then calls
Cmn.A04(DdM,int,long,long) with 0, 0x1388 and 0 respectively. Those two long
arguments are separate from the event's timestamp. Their complete scheduling
meaning is not inferred here; 5,000 must not be copied as a receipt timestamp.

DDH.CFC(index) returns A02 (actual DeviceJid); CeD(index) returns A01 (long);
Ccf(index) returns A00 (status); BSJ(index) reads the indexed A07 message key.
These method bodies independently confirm the field roles already retained in
the DDH constructor. They do not merge actual device, selected user and chat.

Cmn.A04 reads CFC/CeD/BSJ/Ccf and constructs CZ4 items. Its large method also
contains orphaned-receipt storage and deferral paths; their complete admission,
clock, mapping, expiry and recovery semantics remain **unknown/unported**.
In particular, following one interface read is not full dispatcher parity.

**Confirmed next writer boundary:** private Cmn.A00(List) reads each CZ4's
status A00, message A04, device A02, user A03 and time A01. Under 1hg.A03's
status predicate, it skips B3b()==0x15 (the log calls this permanent send
failure), otherwise passes these five values to
CNn.A00(DeviceJid,UserJid,178,int,long)->CYV at 00ce..00d4. The predicate and
CNn/CYV row policy remain untraced here. Other statuses use a separate 15A.A07
path; do not map all statuses to the same device delivery-timestamp update.
Cmn.A02 is an auxiliary collection builder involving 1hg.A03, a non-1iZ guard
and message/0lp pairs; it is not the device row writer itself.

**Unknown:** CNn's insert/update/cache branches, how read/played affect delivery
rows, timestamp units/defaults at the parser, canonical namespace authority and
identity-cleanup interaction. These are the next required trace before porting
a mutable receipt ledger. Current immutable original-device snapshots and
existing independent delivery facts are preserved without inventing these rules.

Reproduce using apk-research-bundle with the linked recipe and --check against
its committed directory. For another APK, export a new anchored recipe/bundle
and use apk-research-diff. No account operation or network request was performed.

Receipt-application validation: eight source topics / 13 files / 112,789 bytes
reproduced exactly from the pinned comparison APK. Cached self-diff verified
eight unchanged topics and four unchanged metadata artifacts. Repository
hygiene checked 1,336 local links and 27 APK bundles / 7,921,661 bytes; diff
hygiene passed. This follow-up changes research/docs only; the preceding
cdca7ad runtime chunk passed all 798 workspace tests and strict lint/doc/CLI
checks. No new runtime implementation or live receipt application is claimed.
