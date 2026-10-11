# Stored encryption attestation state and admission properties

2026-10-10 follow-up: [primary attestation admission](primary-attestation-admission.md)
supersedes blanket primary attestation presence deferrals below. Structurally
valid changed-key candidates follow the native E2EE fallback; verified same-key
hybrid/non-E2EE candidates may refresh without an override child. Failed same-key
metadata and companion attestation remain deferred. Native state persistence
and non-E2EE routing remain unported; earlier validation below is historical.

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
This extends [signature verification](attestation-verification.md) and
[session preview](session-refresh-preview.md). It establishes native storage
and baked property inputs, not the effective state of the retained Rust account.
No account, transport, admission or acknowledgment behavior changes here.

## Schema is NOT_SET by default, not E2EE

[1A2.AR4](apk/2.26.37.71-attestation-state/identity-schema.txt),
`013e..0162`, builds the identities column `account_encryption_attestation_type`
with A06=true and A01="0". [0Km.<init>](apk/2.26.37.71-attestation-state/column-descriptor.txt)
translates those members to `NOT NULL` (`0064..008e`) and `DEFAULT 0`
(`00b8..00f4`). [0Kl.A00](apk/2.26.37.71-attestation-state/column-builder.txt)
resets builder members after each column, so the preceding column's settings
are not the source of this default. The table is identities (`019a..01a4`).

The [C0F values and C9D decoder](session-refresh-preview.md#native-stored-admission-state)
map native stored 0 to NOT_SET, 1 to E2EE, 2 to HYBRID_E2EE and 3 to NON_E2EE.
These remain distinct from the attestation protobuf's 0/1/2 account types.
**Unknown:** this descriptor is not a traced ALTER/migration of every historical
SQL database. A historical Rust identity with no attestation field does not
inherit a native schema default or establish which native migration ran.

## Different producers must remain different

**Confirmed static:** [0kc.A0G](apk/2.26.37.71-attestation-state/identity-save.txt)
explicitly supplies the caller's C0F.intValue (`00ba..00d0`) with address,
identity public key and timestamp. It calls 0IY.A08 (`00dc..00e2`), whose
[implementation](apk/2.26.37.71-attestation-state/sql-replace.txt) delegates to
SQLite `replaceOrThrow` (`0012..0018`). This is not a state-preserving update
that automatically keeps the previous attestation column.

[0kc.A0F](apk/2.26.37.71-attestation-state/state-update.txt) instead updates
only the state column for exact recipient_id/recipient_type/device_id
(`0016..004a`). The returned update count is not used here. After closing the
SQL handle (`004a..0050`), a non-NOT_SET state with a resolvable UserJid produces
9qn(state, singleton user) through 0ka.A01 (`0050..0092`). A notification here
alone is therefore not proof a matching SQL row existed. Observer consumers,
cache effects and complete transaction ownership remain **unknown**.

The [exact caller inventory](attestation-state-references.txt) finds two callers
of A0F: 0jc.A0L (bundle/session processing) and DEC.A00 (gossip updating).
The [existing bundle trace](session-refresh-preview.md) calls A0F only with a
nonnull admission result and enabled property 0x6f6f. A mathematically verified
attestation is not by itself that admission result.

[DEC.A00](apk/2.26.37.71-attestation-state/gossip-state-update.txt) rejects own
users, requires property 0x6f6f and ignores incoming NOT_SET (`0000..0060`).
It distinguishes absent versus present ByteString paths, acquires an address
lock, loads saved primary identity/state and can schedule a key fetch with reason
0x11 (`01ca..01d4`). A path with different identity bytes invokes A0G
(`01d6..0216`); a state-only path reaches A0F (`0258..026a`).
**Unknown:** upstream gossip authentication, complete mismatch branches and
observer lifecycle are not ported. These calls do not authorize accepting
arbitrary incoming XML attributes as authenticated stored attestation state.

[D0O.C5T](apk/2.26.37.71-attestation-state/own-state-update.txt) is a distinct
own-account callback, not a schema migration. Conditional ByJ outcomes select
HYBRID_E2EE or E2EE (`0032..0078`, `0078..00ac`), update recipient -1/type 0/
device 0 (`00ac..00fc`), then notify available own PN/LID users after close.
Its complete request/result authority remains **unknown**. The earlier local
scratch filename called this a migration; inspecting the method refuted that.

## Actual primary backfill

**Confirmed static:** [0kc.A07](apk/2.26.37.71-attestation-state/primary-backfill.txt)
updates state to E2EE where state=NOT_SET AND device_id=0 (`001a..0070`),
returning the SQL update count. It does not update all companions or prove all
accounts are E2EE. There are two physical calls, both in
[8oO.A0F](apk/2.26.37.71-attestation-state/backfill-task.txt).

That task selects hosted-account JIDs from user_device_info joined to jid where
account_encryption_type=1 (`0010..001e`), forms primary addresses and reads
already-HYBRID identities. With no devices to fetch it calls the bulk update and
returns true (`0182..01ae`). Otherwise it runs the fetch/wait path and rechecks
HYBRID membership (`02b8..031c`). When required accounts remain missing it
records failure and returns false without the bulk update (`031c..03cc`);
when none remain it removes the callback and performs A07 (`03cc..03fe`).
Full task scheduling, callbacks and historical completion remain **unknown**.
The producer demonstrates why default NOT_SET and migrated E2EE are different
states; neither should be invented for an old Rust pin.

## Baked false is not an effective false gate

**Confirmed static:** [00D.A10](apk/2.26.37.71-attestation-state/property-read.txt)
passes register zero to a **Boolean reference** argument of A0E. That value is
null, not Boolean.FALSE. [A0E](apk/2.26.37.71-attestation-state/property-dispatch.txt)
constructs 1Y1 discriminator 2 and delegates to A07. The
[function](apk/2.26.37.71-attestation-state/property-function.txt) delegates that
case to [A0D](apk/2.26.37.71-attestation-state/boolean-fallback.txt).
A0D first reads an existing value, otherwise gets the baked map entry, uses it
as SharedPreferences.getBoolean's default, and caches the returned value
(`0004..003c`, `0064..00fc`). Unknown fields with no explicit fallback throw;
null at the wrapper is not a blanket false default.

[07t.A0C](apk/2.26.37.71-attestation-state/admission-baked-defaults.txt) boxes
false into v1 at `0000..0018`, with no later write to that register before the
two relevant puts. It puts **0x7497=29847 → false** (`439c..43ae`) and
**0x6f6f=28527 → false** (`43ae..43c0`).
[07t.<init>](apk/2.26.37.71-attestation-state/property-initialization.txt)
passes builder v14 to A0C (`0100..0106`) and builds that builder into A00
(`9f2c..9f42`). [07t.A15](apk/2.26.37.71-attestation-state/concrete-default-map.txt)
returns A00; the base [00D.A15](apk/2.26.37.71-attestation-state/abstract-default-map.txt)
is abstract/native in the dump. Exact field-write inventory finds only this
constructor write; a whole-class scan found each property constant once.

**Unknown:** effective per-account values. [00D.A07](apk/2.26.37.71-attestation-state/property-precedence.txt)
includes cached reads and MobileConfig delegate paths (09T/09U), as well as
fallback invocation (`0192..01e0`, `0224..028a`). Delegate/native internals,
precedence under every runtime state and the retained account's assignments
are not established. Returned AB property rows are not equivalent to these
effective MobileConfig values. No user switches to suppress these gates are added.

## Reproduce and compare

The media-free checksum bundle retains 19 complete method topics, including
register flow, full signatures and exception tables. The largest bodies are
07t's constructor and Boolean default builder; retaining them preserves the
map construction/overwrite evidence for another APK release.

```sh
target/debug/xtask apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-attestation-state.json \
  --output docs/protocol/apk/2.26.37.71-attestation-state --check
target/debug/xtask apk-research-diff \
  docs/protocol/apk/2.26.37.71-attestation-state \
  docs/protocol/apk/2.26.37.71-attestation-state
```

A self-diff verifies reproduction, not equivalence with a new APK. Re-anchor
new obfuscated classes and produce a new hash-pinned recipe before comparison.
The inventories above are reproducible with apk-xref for the column string,
apk-callers for 0kc.A0F/A07, and apk-field-refs for 07t.A00 with --writes-only.
A lookup of 1Y1.A00 found no body; its actual function is named invoke.

## Validation

Validation: the 19-topic / 24-file / 1,266,707-byte bundle reproduces
exactly; cached self-diff verifies all topics and four metadata artifacts
unchanged. CLI-reference regeneration, OpenAPI/library-symbol checks,
formatting, whitespace and hygiene passed (2,372 links / 117 bundles).
Research/docs only: no fresh workspace runtime or live test is claimed.
