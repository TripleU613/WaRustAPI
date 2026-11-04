# Current own-companion ingress and account authority

2026-11-04. **Confirmed static** unless labelled unknown, WhatsApp Android
**2.26.43.72**, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
[Retained evidence](rea/2.26.43.72-own-companion-ingress/README.md).
This continues [read-self eligibility](read-self-authority-current.md).
No Rust receipt capability, account/device state or live traffic changed.

## Device advertisement is not primary pairing completion

Current19N.A09 handles account_sync/devices. Its A02 first requires a nonempty
device list and compares saved `adv_timestamp_sec` with key-index-list/@ts;
saved <= supplied is admitted. Missing key-index-list supplies zero here.
This comparison is not itself a signature check. 6qD.A01 parses required typed
DeviceJid `jid`, optional typed `lid`, and key-index default0 into Ek2 entries.
Duplicate-key handling belongs to the immutable builder, not a guessed overwrite.

The non-modify online branch schedules FeS1 -> FeS0. It builds device/index
maps from the parsed entries and calls0Vy.A0a(map,true,true). Modify/offline
also schedules a device refresh and can run Fee10 -> Fee9, which checks the
already pending pairing context against advertised JID or supplied LID before
calling A0W. The Java FeS export has recovery warnings; checked DEX is retained.
Coex-token behavior is not claimed from an apparent empty recovered monitor.

0Vy.A0a normalizes own aliases through A07 before comparison, asserting equal
ADV indexes when PN/LID entries collapse to one device. A07 rewrites only when
09F.BVC says the user equals own PN or own LID, preserving device number;
migration state chooses the preferred namespace. Non-own input remains unchanged.
The active pending pair is excluded from the ordinary removal set.

It compares advertised membership/indexes with active own-store metadata. Missing
or changed entries take removal handling; additions exclude the current own PN
and LID device and obey the namespace migration gate. The crucial role split is:

| New advertised entry | Native work |
| --- | --- |
| Matches the saved pending pairing device, with completion flag enabled | A0W(saved Eo9) |
| Unknown and BU7 companion mode is true | Fei43 -> construct F0I ->1nG.A0F |
| Unknown and primary mode | A0Y(device,`unknown_companion`,true,false) |

Thus a primary client must not turn a raw server-assigned device/index pair
into an active trusted companion. The companion-role branch is a different
authority path. Its zero timing defaults are native metadata construction,
not authorization for Rust to fabricate a device or assume its role.

Unknown-companion A0Y normally enters a logout/removal operation, with local
per-device suppression and an exception while a qualifying hosted pair is in
progress. It is not an adoption fallback. Complete logout query construction,
error callback and suppression branches remain **unknown** here; no such request
was sent or implemented. The notification's full outer sender/owning-account
parser and transport admission remain separate unresolved authorities.

## Primary pending context has request-generation provenance

F1G.A00 builds pairing identity/index material from saved EXq and local account
keys. It signs device details with the local account identity and computes the
pairing HMAC using the saved pairing secret. Optional hosted, subscription,
authentication-proof and challenge branches exist; their effective flags and
complete input-validation authority remain **unknown**. No challenge response,
key or telemetry was manufactured.

Fdy's nonzero discriminator consumes those prepared values. It refuses an absent
or inactive FUY request. It builds an IQ/set to server with xmlns=md and a
pair-device child carrying prepared ref/pub-key/device-identity/key-index-list
and optional protocol fields. It registers **that FUY** with generated ID,
request discriminator219 and timeout literal32000 through0Sf.A0Q. Complete
central request-manager correlation is not inferred solely from the callback.

FUY.CK4's success branch requires type=result, a device child and a parsed device
JID. It parses optional companion properties, takes the ADV index from its saved
Dfj and creates F0I with local current active/login times and logout_time=0.
It records the received device separately, then schedules FeN12. A received
device JID is an intermediate result, not active-own-store authority.

The FeN12 branch, retained in the preceding bundle, requires FUY.A0N=true,
the prepared EXq and saved identity Ee1. It creates Eo9 and commits a positive
prepared pairing timestamp via10V.A06, then schedules Fc2(29). F1G.A02/A05
set an old request's A0N=false and cancel its timer before replacement. Fdy
checks this before dispatch; FeN12 checks again before context creation. These
are explicit native generation checks. They must not be replaced by “some
pairing is pending” in a future port.

Fc2(29), checked target10f4 onward, sets0Vy.A00=Eo9 only when no context exists,
sets pending-login flagA0b and starts its timeout: positive property15632 seconds,
otherwise60 seconds. A conflicting pending context is diagnosed, not overwritten.
The account-device advertisement can complete only this saved context/alias,
not supply a replacement identity for it.

A0W performs duplicate-registration handling, cancels its timer, records native
registration timing and schedules FeN14. That worker writes own-device metadata
through1nG.A0F, then schedules FcP10. FcP's selected smali target1040 takes
identity **from Eo9.A02**, asserts09F.BRq for the device, and calls0WJ.A0J.
Current09O.BRq is exactly: nonnull device, BVC(user), and device number !=0.
BVC compares to current own PN or LID. A raw server device/index pair does not
supply this saved Signal identity.

FcP10 then clears pending-login and starts FeN16 critical bootstrap. The prior
31-record read-self bundle retains metadata writing and active-set filtering;
its scope is distinct from complete identity/session/bootstrap readiness.
0WJ.A0J has transaction and identity-change handling, but its recovered Java
contains invalid-looking exceptional flow. Its full semantics and conflict
admission remain **unknown**; the call is not permission to replace a held pin.

FDV.onSuccess later clears the pairing context and registered-device marker,
records verification scheduling and notifies bootstrap observers. Full failure,
timeout and cancellation cleanup remains **unknown**. Device assignment,
metadata registration, Signal identity persistence and bootstrap completion
are separate milestones, as already established for the historical37.71 path.

## Removal, namespace migration and cache lifecycle

0Vy.A0U -> A0V -> FcO7 calls1nG.A00 with its removal mode. The preceding current
trace established that this method distinguishes deleting metadata from setting
positive logout_time, protects own primary and invalidates raw/converted caches.
The local Boolean is derived from role and whether all active entries disappear;
checked refresh-reconcile DEX is authoritative where Java loses its assignment.
Full cleanup side effects remain unported.

The two0WB cache writers previously left unclassified are now identified:
**A0E migrates own-device rows to PN; A0F migrates them to LID.** They are not
logout/reset APIs. Both update/delete namespace-collision rows transactionally
and invalidate1nH.A01/A02 after success. Their complete row-selection and collision
grammar remain **unknown**. The prior note's “cleanup paths” wording is narrowed
by these checked methods, without treating a namespace migration as logout.

G0j.A03's account-data cleanup deletes the companion database through its wrapper,
invalidates both maps and removes logged-out-companion display preferences. Its
caller authority/order relative to Me, account switch and other stores remains
**unknown**. Full account separation cannot be inferred from one cache clear.

## Me is persisted identity data, not standalone authentication proof

0D0.A01 reads a selected directory's serialized `me`, supporting current Me and
legacy MeOldFormat. It reconstructs0E2 from country/national-number fields and
typed PN/LID; malformed/missing/unexpected serialization returns null or fails
through the retained parser. It does not cryptographically authenticate a file
merely because parsing succeeds. The selected directory/account is part of the
authority boundary.

09O.CrQ preserves a missing LID from that account's saved self_lid when available,
writes changed Me through0Cz.A00, persists self_lid, then updates memory viaA05.
0Cz writes the selected Me serialization; A05 updates providers/observers.
09O.ChR fills an absent LID but does not replace an existing one in this path.
09O.A01 clears Me, own PN/LID device views and registration_device_id.

Positive loaded-Me presence has several different origins:

- Primary registration13N.A06 constructs it from saved registration preferences,
  requiring a parsed PN. A0C/A0M can install it in memory for pre-restore/passive
  connection; A0K change-number completion persists it. The preference writers'
  full verified-server-response chain remains **unknown** in this chunk.
- Companion107.BLn handles pair-device/pair-success and later registration
  completion. Its checked body contains pairing-state gates, HMAC/account-signature
  validation, signed-device identity persistence, registration_device_id and
  AKT ->CrQ. The complete producer-to-completion binding and all crypto helper
  semantics remain **unknown**; a no-body REA display is not absence of code.
- AccountSwitcher0Qq.A0J reads an inactive account's own directory and may fill
  its absent LID from that account's saved preferences. It is not the current
  account's identity by default.
- Mrw.A02 export can **temporarily set Me** from saved-before-logout identity,
  then clear it again. BUx true alone therefore does not prove a live authenticated
  session, current registration completion or authorization to send read-self.

These negative boundaries matter for Rust: its account registration enum or a
parsed PN/LID cannot simply stand in for every native Me use. Any future own
inventory must bind to the owning encrypted AccountId, authenticated session and
actual pairing generation; transient export or another account's metadata must
not activate read-self eligibility.

## Port boundary and exact remaining work

Rust still has no equivalent verified own-device lifecycle. The contact
DeviceList remains prohibited as a proxy, including its always-present primary.
No read-self send was enabled. The existing disabled-preference automatic intent
continues pending with its reason.

Next gates: complete outer account-sync source/account parsing and timestamp
authority; request-ID/source correlation for pairing result; QR/ref/secret/remote
identity validation before EXq; exact primary Me preference creation after a
verified response; account-switch/logout generation invalidation; transactional
own metadata/identity/session/bootstrap rollback and retry policy. Native copied
device numbers, timestamps, signatures and account identities must come from
their proved owners rather than synthetic records. These remain required parts
of the full objective, not optional tests or an ACK-only substitute.
