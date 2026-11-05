# Old Signal counter and retry eligibility — APK 2.26.43.72

**Confirmed static:** version code264307238, APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Checked-DEX exports, not REA Evidence envelopes. No APK execution or live traffic.

F22.A02 at069e..06c0 constructs and throws ED4 with “Received message with old
counter: ” and counter values. Ordinary message decryptor0WJ.A0P independently
constructs the same ED4 at0186..01bc. Its ED4 catch handler at code-unit0134
(byte0268) loads -1001 at026a. The previously retained
[EtY/F2q classification](../2.26.43.72-outgoing-retry-error-map/README.md)
excludes -1001 from selected crypto retry eligibility. The prekey decryptor
0WJ.A0O also maps ED4 to -1001 in that prior bundle.

**Inferred compatibility mapping:** libsignal DuplicatedMessage carries the
corresponding consumed/unavailable old-counter condition. Rust now retains it
as SignalOldMessageCounter instead of merging it into generic direct-message
failure. Direct inbox and pairwise SKDM group boundaries return Unsupported:
continue receiving, preserve capture, no automatic retry and no delivery ACK.
Authenticated durable same-ID duplicates still return Duplicate and retain
existing confirmation behavior. This does not classify an old counter as an
authenticated stored message.

**Unknown:** complete native old-counter ACK/admission policy, sender-key
primitive equivalence, other failure classes and every producer. Constructor
inventories EDQ/EDR/EDY are retained as bounded exploration only: their call
sites alone do not establish semantics. F22 also contains future-counter, MAC
and uninitialized-session messages; these exports do not implement all their
producer mappings. Sender-key error formatting remains a separate uncorrected
redaction/classification gap in signal/group.rs; this change only preserves
the direct decryptor's typed condition, including pairwise SKDM processing.

Reproduce each export with target/debug/xtask apk-smali APK --class 'LX/F22;'
--method A02 --signatures --exceptions, or class LX/0WJ; method A0P.
Constructor inventories: apk-callers APK --class 'LX/ED4;' --method '<init>'
(and EDQ/EDR/EDY). Exports retain complete selected method/exception output.

Offline regression reuses consumed ciphertext under a fresh stanza ID after
real libsignal decryption, including reopened storage, and verifies Unsupported
with unchanged session, identity pin and authenticated event records. A typed
error regression checks redacted diagnostics. Full native parity and server
acceptance remain unverified.

Validation:1327 workspace/all-feature tests passed; formatting, strict workspace
all-target/all-feature Clippy, CLI reference regeneration, APK coverage and
research checksum/link hygiene passed. Offline only.
