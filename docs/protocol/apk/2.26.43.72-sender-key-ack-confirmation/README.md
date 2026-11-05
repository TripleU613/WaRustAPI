# Sender-key ACK confirmation and reset — APK2.26.43.72

**Confirmed static**, checked DEX TXT, not runtime observations or REA Evidence
envelopes. APK SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Pinned artifact was reverified2026-11-04 in the preceding admission trace.
Commands: `target/debug/xtask apk-smali APK --class CLASS --method METHOD
--signatures --exceptions`; the store/future exports omit METHOD to retain
all methods. Exact anchor `sent_sender_key` has3 physical references; this
anchor census is not a complete SQL-use or runtime census.

## Request callback precedes send and blocks the job

SendE2EMessageJob.A06 at1d88..1dc6 takes Eng.A03's Eoh key, normalizes it
through0Sf.A02, creates FfK and registers it with0Th.A02. Dispatch through
0Sf.A05 occurs at1dd2..1dd8; FfK.get follows at1dd8..1de4. FfK.get waits
on a CountDownLatch. BsT stores the returned value and success flag before
countDown; BsR stores failure and get throws ExecutionException. Neither is
a socket-write completion promise. The job separately examines the returned
node's error attribute at1e26..1e3e.

0Th.A02 registers under an Eoh key in A04 while synchronized. An existing key
logs duplicate_ackable_stanza; the selected body subsequently puts the new
wrapper, so this is not evidence of rejecting duplicate registration. Native
key equality and handler6 matching are retained in the
[admission bundle](../2.26.43.72-distribution-ack-admission/README.md).
Concrete Eng key production and the complete out-of-band distribution caller
remain to trace; this job path does not alone establish their address rules.

## Normal successful ACK marks selected devices

A06's v6 indicates the returned node has no error. At1f6e a false v6 jumps
to2110, bypassing the normal marking block. The nonempty distribution target
set is intersected with the send's supplied collection at1f82..1f8c. Native
key existence is checked before marking: bucket mode groups devices by EHo.A00
and uses F24.A02/0YL.A01, then0j7.A05 with false at2090..209c; the unbucketed
branch checks F24.A01/0YL.A01 and invokes0i9.A0U at20fe..2104. Missing-key
branches log instead of marking. Bucket identity and all branch predicates
are not fully reconstructed here.

0i9.A0U's selected normal branch executes SQL setting sent_sender_key to1
(binding1 at013a..0146), clears sent_sender_key_bucket, and binds device, group
and participant-user row IDs. It expands alternate device
coverage under its model predicate, updates in-memory participant models and
marks/closes nested transaction scopes. This is not a claim of Rust PN/LID
alias parity or atomic disk-outcome recovery.

## Refutation: post-ACK A0X is a reset, not another marking function

A06 uses v56 to enter a one-time sender-key path (0afa..0b26, explicit
one-time-sender-key log). Its selection also depends on send flags, message
kind, recipient count and config descriptors EMu.A06/A03/A02; effective values
and every semantic predicate remain unknown. After ACK processing, v56 with a
nonempty EF8.A01 set reaches0i9.A0X at21fa..2204. A0X calls0j9.A01 with false.
That SQL binds sent_sender_key=0 at0066..0072, clears its bucket and updates
selected devices; the method also updates cached models.

Therefore blanket descriptions of all post-ACK U/V/X helpers as setting devices
to having a reusable key are too broad. This does not establish that reactions
use the one-time path or explain sent_add_on_sender_key. The local normal chain
confirmation must not be widened to all native job outcomes from log proximity.

## Pre-ACK confirmation wrapper origin now connected

0Th.A02 snapshots1iV.A0C and elapsedRealtime before constructing2Gc(selector1)
at007a..00a4. The constructor maps callback to A01, manager to A00, marker to
A02 and timestamp to A03. 2Gc.BsT first invokes its wrapped callback
(0008..0016), records elapsed duration, then calls1iV.A0J on a nonnull marker
(004a..006c). For this registered message callback, the marker's confirmation
authority is a matched ACK callback, not socket-write success. This closes the
previously untraced constructor origin in the prekey/pre-ACK notes.

Because the wrapped callback runs first, FfK's waiter may unblock before A0J
finishes or its scheduled retirement executes. Do not describe sender-key
updates, pre-ACK retirement and the job result as one atomic transaction.
Reconnect/generation and alternative selector0 behavior retain their own limits.

## Rust audit consequence / remaining work

**Confirmed local source:** outgoing_group's normal reservation records all
distribution targets under the final group wire ID via Prepared::await_ack_in.
Out-of-band direct nodes have separate IDs and only the connection-local
Waiting::Distribution queue; advancing that waiter does not itself call
confirm_ack_in. The generic outgoing::acknowledge function requires an original
outbox wire-ID match, so those direct IDs are not automatically independent
durable confirmation records. An interrupted sequence therefore does not
retain per-distribution acceptance through that ledger. The original group
intent/key reservation remains uncertain and is not replayed.

**Unknown / next port work:** identify native out-of-band distribution key
construction, accepted-address normalization and which native durable state
those particular ACKs update. Preserve original IDs, ciphertext and chain/epoch
bindings before any Rust durable per-distribution confirmation change. Do not
mark every recipient accepted after one distribution, transplant native resets
to ordinary chains, infer reaction parity, replay uncertain work or clear keys.
This evidence-only chunk changes no production behavior and sends no live traffic.

## Checksums

- [job.txt](job.txt): `572d6304696d7098436a3dfa5a588559879aa2f971072964af719509b84183a7`
- [store.txt](store.txt): `d760a5b9ebf164c5c5d16c614623aae5f098bae3d478c8ae72cde8dd5dc16f41`
- [normal.txt](normal.txt): `6ebb2e955fcf4fc1d31295140a359cf18443d7b383dbdd0ce3735bf360f9bea4`
- [addon.txt](addon.txt): `052b2da7decfbbbc38cfb6146f7fad81e5ec96cf813024b225b933c8501a2686`
- [future.txt](future.txt): `98ee2d6cf1d5b090cb16d55041d452b3fde6035cbe0b959a5cdf71a4f3861ebc`
- [registration.txt](registration.txt): `b5e4ea502916c5ebaab717b1b286129feed8a2e64a2114354ab26abc9800f511`
- [wrapper.txt](wrapper.txt): `3236c6386ccc2b03bda3973a0a0052b3199b22f541584b6655a0e129b6f2f488`
- [wrapper-init.txt](wrapper-init.txt): `3dd2809e29bed83711ad22bb7d8d90fb4b1d6947346da50afbdb9ea67bdaa019`
- [xrefs.txt](xrefs.txt): `58967d3eefd9f0be6e456181f955cddf14ce89a92c0928801ebcdf20d2bb0765`
