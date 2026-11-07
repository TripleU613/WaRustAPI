# AEA account-type input consumers

**Confirmed selected static**, pinned APK2.26.43.72, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Existing xtask DEX fallback (Android REA provider unavailable); no new REA
Evidence ID or live observation. Excerpts are explicitly bounded, not complete
methods. PCs are bytes, exception ranges/branch offsets code units. Full unrelated
large methods stay outside Git; export hashes and exact ranges are retained.

## Exact consumer and literal censuses

[apk-callers ExG.A06](consumer-callers.txt) finds three records:19R.A09,
HXp.A00 and IC1.run. [Exact aea_account_type literal census](literal-refs.txt)
finds the same three methods. This is not a complete dynamic SQL/token/reflection
search and does not prove no differently constructed producer exists. It fixes
the input boundary behind the prior native function log
updateChatEncryptionStateFromUsyncAccountType; that name alone does not prove
this field belongs on every contact/device discovery request.

## Business notification profile

[19R.A09 excerpt](notification-excerpts.txt) retains0000..029e, including type
argument202 branch, profile child selection, selected early return and AEA block.
The stanza profile is read at0066; selected branch0230 requires that child,
extracts a UserJid and checks aea_account_type child/value. Only nonnull parsed
user and nonnull value reach ExG.A06 at0288. Missing field/value skips this
update. Other notification branches, ACK scheduling, strict type-number mapping
and complete business-profile effects are not reconstructed here. This is not
an evidence-backed rule to reject or ignore an otherwise supported notification.

## Profile-result application and business-profile worker

[HXp.A00 excerpts](profile-result-excerpts.txt) retain H6v.A0D/H3H source selection
and the selected account-type application. The H3H holder has UserJid and node;
`profile` is the selected child at19ca. The later block uses H6v.A09 when present,
otherwise H3H.A00 as the ExG recipient. It reads profile/aea_account_type/value
and invokes ExG.A06. Upstream H6v production, root reply correlation, intervening
control flow and complete aliases remain outside these bounded excerpts. Do not
infer an ordinary usync/list/user path from the helper name.

[IC1.run excerpts](profile-worker-excerpts.txt) retain dispatch and selected
1838..18c4 branch. This branch first calls BusinessProfileManager.A0G with Hj8
and Hyh.A0D user, then forwards optional node/aea_account_type/value to ExG.A06.
Missing node/child selects a null value and still calls ExG.A06, unlike19R's
nonnull-value guard. ExG maps null/other strings to NOT_SET under its own policy;
that is not the same as authenticated identity or receipt attestation. Constructor
producer, full reply parsing and later IC2 callback contents remain untraced.

All three reuse the previously retained [ExG.A06 policy](../2.26.43.72-aea-cooldown-policy/usync-account-type.txt),
including gate, peer exclusions, local cooldown suppression and classification
update. Earlier [Em2 receipt path](../2.26.43.72-retry-aea-admission/post-session-state-update.txt)
uses a different entrypoint and verified/derived E7b input; no string-only
business-profile result substitutes for that verification or identity adoption.

## Current Rust comparison and consequence

Confirmed source: transport/contacts.rs constructs contact/lid/disappearing_mode
queries; transport/devices.rs constructs devices discovery. Neither selects or
adopts aea_account_type. transport/notifications.rs business_ack uses the generic
base ACK path; it does not project these business-profile chat states. Relevant
Rust search was scoped to those modules and the exact aea_account_type field,
not a proof about every possible future or unsupported protocol.

This is a missing native business-profile/chat-classification capability, not a
newly proven malformed supported request or an ACK defect. No field is added to
ordinary contact/device queries, and no unsigned string is treated as receipt
AEA. Research only; runtime unchanged, receipt_keys_unsupported remains. No
key adoption, attestation generation, verification bypass, live account operation
or traffic. Next bind business-profile request/reply correlation, alias ownership
and chat-classification-to-network consumers before implementing this capability.

Reproduce exact apk-callers ExG.A06 and apk-xref aea_account_type; apk-smali with
signatures/exceptions for19R.A09,HXp.A00,IC1.run. All exception records from each
export remain in its excerpt; edges to omitted instructions are intentionally
unresolved. Manifest covers each retained file except itself.
