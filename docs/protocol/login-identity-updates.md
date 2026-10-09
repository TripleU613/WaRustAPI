# Partial authenticated login identity updates

2026-10-09. **Confirmed local bug/fix:** `connection::connect` replaced the
whole `login-identity` checkpoint whenever either success attribute `pn` or
`lid` parsed. A PN-only reply therefore erased an already known LID; a LID-only
reply erased the stored PN. Present malformed values were also treated as absent.
This is our persistence behavior, not an observed server requirement or refusal.

**Implemented/offline tested:** login-result persistence now updates only
supplied identity fields in the existing write transaction. Absent fields and
unrelated checkpoint members remain unchanged. With neither field supplied,
the checkpoint is not rewritten. Newly supplied users must have the expected
PN/LID namespace, a nonempty user, agent zero and plain/AD encoding. Device-bearing
forms normalize to their user, consistent with the existing own-LID lookup.
A newly supplied PN can still initialize a missing AccountRecord.own_jid;
known account PN is not replaced by this patch. No identity is invented.

Malformed present fields return fixed redacted diagnostics naming `success pn`
or `success lid`. An existing nonobject record or malformed nonnull saved member
refuses a requested identity update rather than being silently repaired. This
validation does not run when neither identity field is supplied. A valid newly
supplied value retains the prior replacement behavior; this patch is not a new
policy for legitimate server-provided own-LID changes.

The login checkpoint, authenticated time and confirmed clock sample share the
same transaction. A refused update rolls all of them back, preserving the prior
`outcome_unknown` reservation. The real connect path already preserves the raw
authenticated reply before that transaction. It returns the fixed local error;
that error is not recoverable by automatic reconnection. No reservation is reset,
no login payload is replayed, and no account/Signal/Noise key rotates.

The checkpoint `at` records the latest identity-field update, not a new observation
time for every retained field. No per-field provenance or historical migration
is claimed. Missing historical fields remain missing; malformed history requires
inspection rather than automatic repair.

## Evidence boundary and reproduction

**Observed previously:** successful authenticated sessions carried `pn` and `lid`;
see [transport evidence](messaging-transport.md). This change does not claim a
new live partial success or malformed server response.

**Confirmed static context:** APK 2.26.37.71 SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`,
[stored own-identity evidence](own-identity.md): 08i.A01 keeps a typed PN from Me,
while A02 stores a typed LID user/device separately and its shown other-input
branches do not replace those fields. This context supports distinguishing own
namespaces; it does **not** establish identical native login partial-update,
exception or malformed-history handling. Those are **unknown**. No new APK source
was discovered or native requirement inferred in this implementation.

The failing-before partial-success regression showed `(Some(PN), None)` after
PN-only success despite the earlier saved LID. It now covers PN-only, LID-only
with device encoding, absent-both input, own-LID inspection and encrypted reopen.
Negative tests cover empty/server-only/wrong-namespace/agent-bearing/nontext
input, malformed saved objects/members, unchanged account/checkpoint/pin/session/
outbox rows and event head, and preserved uncertain login status after reopen.
All data is synthetic; no real account or network operation ran.

```sh
cargo test -p whatsapp-api --all-features login_identity
```

Existing `listen` performs this login persistence; `peer-keys`, `peer-identity`
and `clock-inspect` remain redacted offline inspection paths. No new command,
flag or HTTP route is supplied. Fresh live partial-login behavior and complete
native own-identity lifecycle remain **unverified/unported**.

Validation: 840 workspace/all-feature tests passed. Strict all-target Clippy,
warnings-denied Rustdoc, default CLI tests (25 plus API-symbol), regenerated
CLI reference, formatting and whitespace checks passed. Repository hygiene
checked 2,182 local links and 108 existing APK bundles. No fresh live test.
