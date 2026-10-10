# Companion addition coordinator, cache worker and callbacks

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Recipe](apk/recipes/2.26.37.71-companion-addition-observers.json) and
[source topics](apk/2.26.37.71-companion-addition-observers/targets.json)
retain 16 topics / 21 files / 270,236 bytes, with complete selected methods,
signatures and exception tables.
[Exact-name scans](companion-addition-observer-scans.json) retain declaration
and constructor-caller inventories and complete local-output digests.
These claims apply to the
[returning companion directory branch](companion-directory-addition.md),
not every identity change or native observer invocation.

## Coordinator: use actual caller inputs

**Confirmed:** 11T.A0D at 09ba–09e0 calls 1IW.A05 with old devices,
added devices, empty removed devices, the coordinator, selected user,
false and false. Static A05's incoming registers are v12=old, v13=added,
v14=removed, v15=coordinator, v16=user, v17/v18=Booleans.

The empty removed set skips 0022–005c, including the DMh removal job.
Nonempty added enters 0122 from the branch at 0064:
`0x0064 + 0x005f * 2 = 0x0122`. Whatever 1h6.A02 returns, false v17
bypasses the following local-message path and reaches 022c. For a remote
user BO0=false jumps from 0238 to 0272 and returns. Thus this call does
not enqueue the traced type-0x47 local-message objects, inspect own hosted
removed devices, or run the removal worker. 1h6.A02 itself is not expanded
here; bypassing subsequent work is not proof every preceding getter is pure.
The effective gates and behavior for different Boolean/own/removal inputs
remain **unknown here**, not disabled by a Rust option.

## Membership worker invalidates one map entry

**Confirmed:** after the nested membership transaction's A00 call,
11T.A0D creates DNd(B7j, selected user, 19) and registers it with
0ZN.A05 at 0994–09ae. The constructor maps these arguments to A00,
A01 and $t. 0ZN.A05 asserts SQLite inTransaction(), wraps the Runnable
in Cpf discriminator 1 and passes it to 0Gy.A00 with the supplied object.
Full outer-transaction execution/rollback callback ordering is **unknown**
from these methods alone; registration is not proof the callback ran live.

DNd.run's top switch maps discriminator 19 to 134c. That branch enters a
monitor, removes the supplied user from B7j.A07:B7k.A00, exits and returns
at 1374, with an exception handler that exits and rethrows. It performs no
pin, ratchet or sender-key-store operation in that branch. Other discriminator
branches in this large retained method must not be attributed to addition.

B7k initializes A00 as a HashMap. B7j.A00 reads a cached ImmutableMap
for the user; on a miss it selects device_jid_row_id/key_index from user_device
where user_jid_row_id matches. B7j.A01 likewise reuses cached entries and
queries missing users, then fills the per-user cache. **Inferred purpose:**
removing the entry makes future readers reload committed membership/index
rows, rather than reusing the old map. This is not an identity reset.

**Confirmed Rust comparison:** devices::device_list_in reads the checkpoint
through the supplied ReadTx every time, with no equivalent separate B7k map.
companion_admission commits membership in the same transaction as session and
application state. No new Rust cache invalidation is needed for this specific
map; this does not establish equivalence of every native cache.

## Typed event and concrete callback declarations

**Confirmed:** JhE.A0L dispatches Kx8(user, added, removed) when either set
is nonempty. Kx8 stores these as A00/A01/A02; COL casts the observer to LZ8
and invokes CBu(user, added, removed).

The exact-name scan across LX/ classes found two CBu declarations: abstract
LZ8 and concrete D0B. The exact D0B constructor caller scan found CaG's
constructor and IdentityVerificationActivity's constructor. CaG passes
0; the activity passes 1. D0B's constructor preserves that discriminator.

D0B.CBu discriminator 0 iterates **removed** devices and invokes
CaG.A0I:Dd3.BZO. An addition with empty removed returns without BZO.
Discriminator 1 calls the activity's A12 when either set is nonempty.
A12 checks own-user or the activity's contact equality before scheduling
L5A discriminator 47 on the UI thread. Complete runnable effects are
**unknown here**; scheduling UI work is not evidence of a wire request,
identity rotation or new cryptographic authority.

**Unknown:** effective registration/lifetime of these objects, differently
named or inherited/native/dynamically loaded behavior, complete UI runnable
semantics and universal observer coverage. The exact scans bound the finding;
they do not prove that no other implementation can exist at runtime.

## Validation boundary

No account/network operation or Rust runtime behavior changed. Use the existing
apk-research-bundle command with the linked recipe and --check to reproduce
this export; apk-research-diff compares its cached topics across exports.
A self-diff validates retained structural equality, not another release's
protocol semantics. Remaining native group/counterpart limits are recorded in
[the group-state trace](companion-addition-group-state.md).

Validation: exact regeneration verified 16 topics / 21 files / 270,236 bytes;
cached self-diff found 16 unchanged topics and four metadata artifacts.
CLI-reference regeneration, OpenAPI/library-symbol checks, formatting,
whitespace and repository hygiene passed (2,473 local links, 127 bundles /
26,797,855 bytes). No new workspace runtime test or live account operation
is claimed for this static-only chunk.
