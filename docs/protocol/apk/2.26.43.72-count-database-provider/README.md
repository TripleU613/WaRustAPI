# Count queue database provider identity and memoized backend choice

**Confirmed static**, WhatsApp2.26.43.72, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Seventeen complete method exports from xtask apk-smali APK --class CLASS
--method METHOD --signatures --exceptions. File names identify each method in
sections below. DEX text, not REA Evidence IDs. Only specified supplier switch
branches are interpreted; other2Jd/1X1 branches are outside this audit.

## Both callers resolve the same provider ID

The retained [1ic.A0A transaction](../2.26.43.72-prekey-upload-replies/prekey-ack-txn-bulk-transaction.txt)
uses manager.A0G ->0Xp.A07. [Manager constructor](queue-manager-constructor.txt)
initializes A0G with00o.A01(2Jd(self,34)), and itsA01 with05d.A00(0xc3e).
[2Jd selector34](database-suppliers.txt),0a0c..0a2e, gets thatA01 provider,
casts0Xd and calls0Xd.A00. The static provider ID is not a new config default.

The retained [1ih.A04 retirement](../2.26.43.72-count-completion-retirement/disk-retirement.txt)
uses backend.A08 ->0Xp.A07. [Store constructor](stanza-store-constructor.txt)
initializes A08 with00o.A01(2Jd(self,32)), and itsA02 with05d.A00(0xc3e).
2Jd selector32 at0a7a..0a9c gets thatA02 provider and likewise calls0Xd.A00.
Thus the two suppliers share a binding ID and owner access, not merely a
coincidental return type.

## Handle-local caching versus shared binding identity

[05d.A00](provider-wrapper-factory.txt) wraps [00D.A00](provider-handle-factory.txt),
which creates a new05h handle per call. [05a.A03](provider-unbox.txt) delegates
to its handle.get. [05h.get](provider-handle-cache.txt) memoizes the result
of00D.A02 for that handle with a double-checked monitor. This local cache alone
would not prove that two handles get the same instance.

[00D.A02](provider-resolve.txt) resolves through [06o.A00](provider-root.txt)
and00D.A03. [A03](provider-global-cache.txt) checks a static ConcurrentHashMap
keyed by boxed provider ID, then under a provider lock checks again, constructs
through1Uw.A00 on a miss and stores a nonnull result. Existing nonnull entries
are returned unchanged. [07N.A00](provider-lock.txt) canonicalizes the lock by
provider ID using a static ConcurrentHashMap.putIfAbsent, so concurrent callers
use the same installed monitor within that lock-map lifetime. This supports the same0xc3e binding object for callers
within a stable cache lifetime. Provider invalidation/reset, mixed old/new
handle lifetimes and class-loader/process boundaries were not audited; no
permanent cross-generation singleton claim follows.

## The owner memoizes database selection

[0Xd constructor](database-owner-constructor.txt) sets itsA01 to
00o.A01(1X1(self,38)); [0Xd.A00](database-owner-access.txt) returnsA01.getValue.
[1X1 selector38](database-choice-supplier.txt),0260..0296, reads config0x646a:
false selects provider0xc2c, true selects0xc2d, resolved through00D.A02.
The exact provider implementations and effective config are not established.

[00o.A01](lazy-factory.txt) creates00q. [00q.getValue](lazy-value.txt) checks its
uninitialized sentinel, synchronizes on its lock, invokes the supplier once,
stores the result and clears the initializer. Therefore this owner's selected
0Xp is memoized; it does not re-read646a for every caller/access. This is not
authority to switch database implementations in an existing Rust store.

**Confirmed conditional identity:** when both callers resolve the same nonnull
0xc3e cache entry, they share its memoized0Xp. This closes the prior missing
provider-identity edge for that stable lifetime. It does not prove correct
behavior across provider reset, disk errors or process death.

## Acquisition still has two wrapper paths

[0Xp.A07](database-acquire.txt) obtains the helper read lock and0Gz/0H0 state.
When0H0.A03 is false it constructs0aq with this helper; true constructs0ar
with the same helper plus timing/state dependencies. The selected effective
branch, subclass overrides, actual SQLite connection/thread ownership and all
nested transaction effects remain open here. Do not promote shared helper
identity into proof of every transaction's atomicity or durable commit.

The earlier transaction bridge retains normal outer-completion callbacks, and
pre-ACK insertion in1ic.A0A receives its supplied0aq directly. Together these
are concrete inputs for a port, not a completed recovery implementation.
Rust runtime/network/deletion behavior remains unchanged. No live operation.
