# Retry base-key recipient normalization — APK 2.26.43.72

**Confirmed selected static.** Version code264307238, APK SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`. Thirteen complete method exports include exception tables; two bounded runnable-arm excerpts retain source byte lengths/hashes and explicitly omit other instructions/exception tables. One direct caller inventory is retained. Repository xtask exports are not REA Evidence. No Android execution, live database, effective remote flags or server acceptance was observed.

## Original versus normalized recipient

The prior [base-key save/comparison](../2.26.43.72-retry-message-base-key/README.md) and [explicit deletion](../2.26.43.72-retry-base-key-schema/README.md) call `0YG.A02`. Its [complete body](normalize-address.txt) first requires EoY.A01 ==0. It asks [A01](address-phone.txt) to resolve EoY.A04 as PhoneUserJid. Nonzero address type, an unresolved phone or a missing later mapping returns the original EoY; there is no blanket textual PN-to-LID rewrite.

A01 explicitly excludes `0` and `Server`, uses PhoneUserJid.Companion.A03, reports invalid parsing, then returns null when [0x9.A03](special-phone.txt) is true. That predicate covers0xC or [the phone predicate](special-phone-kind.txt), which checks [the selected numeric ranges](special-user.txt): 13135550000..13135559999 or13165550000..13165550099. Numeric parser internals are outside this trace; the comparisons/ranges are literal DEX facts, not permission to invent identifiers.

A02 also contains a special-address/feature0x536a branch through [0Dn.A08](special-conversion.txt), which looks up7m0.A00 and constructs0xC when a map value exists. However, A01 filters the same special predicate before returning its phone. This selected direct call chain therefore does not establish execution of that later branch. Treat it as retained code with an apparent earlier exclusion, not evidence that all bot retries use it or that its feature is enabled.

The ordinary mapping uses [0Ws.A0E](stored-mapping.txt). It returns null while the message store is not ready, uses09F.BVC/Auj for its selected own-account case, otherwise resolves a positive phone row ID, consults a synchronized cache and falls back to0Wy.A01 plus cache update. An absent mapped ID or negative mapped ID returns null. Selected class reconstruction is0Dw; the downstream database query, complete mapping provenance and cache recovery are not traced. This is account-user mapping, not proof that every result is a particular namespace.

If no mapping is returned, [0YG.A00](missing-mapping.txt) optionally schedules a missing-account diagnostic using feature0x2176, then A02 returns the unchanged original address. A constructed IllegalStateException is passed into the runnable; this method does not throw it. The runnable's diagnostics and effective flag remain untraced.

For a mapped user, A02 calls DeviceJid.Companion.A02 with original EoY.A00 (device) and [F3k.A06](address-build.txt) with original EoY.A02/A03. F3k builds a new EoY with mapped user text, derived address type, same device and both retained dimensions. [EHs.A00](address-type.txt) derives address integers1/2/3 from getType18/22/(26 or27), otherwise0; complete predicates are [A0Z](type-A0Z.txt), [A0W](type-A0W.txt), [A0O](type-A0O.txt). These integers are address categories, not cryptographic algorithm IDs. Do not reduce a base-key key to message ID plus bare user, discard the device/extra dimensions or normalize nonzero original types unconditionally.

## Direct single-delete callers and lifecycle limits

[Direct caller inventory](delete-callers.txt) finds two calls to0WJ.A0r in this APK. [Fei.run selector15](delete-trigger-i-excerpt.txt) obtains the coordinator from F2A.A0O and supplies EoY plus1dW. [Feq.run selector33](delete-trigger-q-excerpt.txt) obtains it from Ens.A06 and supplies EoY plus F1A.A01. These selected arms establish delegation, not the scheduler, user-visible trigger or a time-to-live. No timestamp test appears in either arm. A bounded probe for an assumed F2A.A0r method found zero callers; it did not identify the actual producer and is not evidence that cleanup is unused.

These findings refine recipient identity and missing-mapping policy but do not close base-key TTL, bulk deletion triggers, full migration, all caller conditions or transaction/crash semantics. Rust count2 base-key persistence and higher-count policy remain unported; counts above two remain refused. No runtime code, outgoing field, retry count, database migration or account operation changed.

Reproduce: `python3 docs/protocol/apk/2.26.43.72-retry-base-key-addresses/reproduce.py ../android-lab/apks/WhatsApp.1.apk`. The script checks the APK hash and every retained export, including complete-source identity before excerpting. `checksums.json` covers every bundle file except itself.

Selected status-state and VoIP producers for these direct deletion arms are now retained in [the producer follow-up](../2.26.43.72-retry-base-key-delete-triggers/README.md); global TTL and ordinary-chat/bulk cleanup remain unproved.
