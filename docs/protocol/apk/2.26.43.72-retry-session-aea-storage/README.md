# Retry session processing and identity AEA storage

Pinned WhatsApp APK **2.26.43.72**, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
**Confirmed selected static**, not observed execution or server acceptance.
Existing xtask DEX fallback (Android REA provider unavailable) exported full
signatures and exception tables. No new REA Evidence ID. Instruction PCs below
are bytes; branch/exception offsets are code units (multiply by two).

## Session processing already writes AEA before returning

[0WJ.A0K](session-processing.txt) is the method called by the retained
[163.A02 caller](../2.26.43.72-retry-receipt-keys/key-processing-policy.txt).
Its parameters are E7b state, EoY address, ordinary/signed/PQ ETs descriptors,
identity bytes, registration bytes and identity type byte. It resolves the session
address and locks FjI at0014..002e. Under A0I policy it reads existing remote
identity; type/bytes mismatch can reach A0E or A0q cleanup at0088..00a6, before
new key parsing or signature checking. These predicates and cleanup side effects
are not fully traced; do not describe refusal as universally mutation-free.

It prefixes the supplied type byte to identity, ordinary and signed public key
bytes before0Yu.A00 parsing. Ordinary key is nullable (ID=-1); signed key is used
as a required descriptor. Selected EDQ decoding handlers log and leave nullable
key objects; they do not themselves return a final success. Registration uses
1nr.A01(bytes,0), key IDs use1nr.A00. PQ present with null signature logs and
selects absent Fqk wrappers; valid wrapper construction uses KEMPublicKey(bytes).
Do not infer current wire-validation policy from these lower-layer fallbacks.

At0302..0324, feature0x5f3f chooses Ecx.A00 with or without the F22.A06 monitor.
Only after that returns does0324..0360 examine nonnull E7b and gate0x6f6f,
resolve an identities address through0YT.A03, and call0YT.A00 then0YT.A05.
**The integer update count from0YT.A00 is discarded.** A zero updated-row count
is not turned into a nonzero session result here. This is an earlier AEA write
than the separate caller's later Em2.A02/ExG/DDs update traced previously.

The result accumulator starts0. Selected catches around session builder and
AEA-write operations map EDQ to **-1002** (037c),2xu to **-1010** (0376), and
other Exception to logged **-1000** (0368..0386). FjI closes before returning.
Earlier preparation is outside that selected exception-to-status region and
can propagate. Unknown Error/Throwable paths close/rethrow, not success.
Do not claim nonzero means rollback or zero means durable end-to-end success.

## Builder validation and write ordering

[Ecx.A00](session-builder.txt) invokes identity BZ3 first (return value is not
consumed here). It checks the signed-key signature via EMC/1oA over Fbl.A00()
bytes when a signed key exists, and checks a supplied PQ signature over the
KEMPublicKey.A00 bytes. Invalid signatures throw EDQ before the selected final
session write. Missing signed key also throws EDQ at0510. This is distinct from
the AEA domain `06 07 || original attestation payload` in the prior bundle.

The builder loads a session, may archive its in-memory current state, and derives
classic or hybrid keys with labels WhisperText or
WhisperText_X25519_SHA-256_CRYSTALS-KYBER-1024. At04e8 it calls0Yh.CrT to store
the session, then at04ee calls0Yi.Cbo for identity. Thus even before the identity
AEA update there are sequential session/identity calls. The concrete store
implementations, enclosing transactions, alias mapping and crash recovery remain
untraced; no atomicity or compensating rollback is established by this ordering.

## Identity-table AEA storage boundary

[Selected complete0YT methods](identity-attestation-storage.txt):

- A00 writes E7b.intValue to identities.account_encryption_attestation_type using
  recipient_id, recipient_type and device_id predicates; it returns the update
  count. It opens/closes a writable handle, with no explicit transaction begin
  visible in this method. An enclosing transaction is not ruled out.
- A02 selects that column using the same three address dimensions. No row yields
  E7b.A05 (NOT_SET); a stored integer is passed through EHp.A00, whose complete
  unknown-value policy is not established here.
- A03 applies0YG identities alias resolution and can return null for unresolved
  A01==0. A0K skips the selected AEA write on that null path.
- A05 skips NOT_SET; for a mapped DeviceJid/userJid it reaches0YQ.A00 notification
  behavior. Full consumer/cache effects remain unknown.

## Rust boundary and next proof

Research only: Rust receipt_keys_unsupported remains unchanged. No carried-key
adoption, attestation generation, live traffic or account operations. Local
peer_verification must not substitute for these native identity/AEA stores.
Next trace concrete CrT/Cbo ownership and transactions, cleanup/reset predicates,
0YT database schema/defaults/migrations and ExG/DDs state synchronization before
implementing receipt-key adoption. A boolean success or one combined state flag
would lose distinctions now confirmed by the retained method chain.

Reproduce with `target/debug/xtask apk-smali APK --class 'LX/0WJ;' --method A0K
--signatures --exceptions`, similarly Ecx.A00 and0YT.A00/A02/A03/A05.
Checksums cover each retained file except the manifest itself.
