# Auth-key rotation triggers and candidate ownership, APK43.72

**Confirmed static:** WhatsApp2.26.43.72 code264307238 SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`. Complete selected xtask DEX exports and direct-reference inventories through supported fallback; previously diagnosed Android provider remains unavailable. No execution, account operation or REA Evidence envelope claim. checksums.json binds every other file. Only selected connection/callback blocks are interpreted; full exports preserve their control flow.

## Three direct triggers

apk-callers finds three physical19i.A00 callers:19h.A09 (notification),0ZJ.A0z (connection), andFUT.CK4. This is an exact direct-call inventory, not runtime reachability or proof no reflection/native dispatch exists. [Notification and gate behavior](../2.26.43.72-authkey-rotation/README.md) remains authoritative: every trigger re-enters the same remaining-attempt/last-success/last-failure checks, rather than bypassing them.

0ZJ.A0z at0b0c invokes1rj.A05 and saves its Boolean. It calls1jA.A00, then calls19i.A00 at0b3e only when that saved Boolean is true.1rj.A05 is not a pure predicate: it reads pref_client_auth_token, requires a nonempty string, Base64-decodes with flag3, builds ib/cat with those bytes, invokes0Tk.A04 with1..1024 bounds, then1r2.Cyu(node,1), and returns true. Absent/empty/null-decoded paths return false. A bounds-helper call is not proof normal false omission or server acceptance. Decode errors are not caught in this exported body. This selected connection sequence is therefore saved-token send -> pending-token processing -> conditional rotation trigger, not an unconditional rotation on every connection. Complete0ZJ lifecycle and early path admission are not reconstructed.

1jA.A00 takes a pending string with AtomicReference.getAndSet(null), then callsA01 only when non-null. A01 stores it back if0AI.A0O is false; otherwise allocates an IQ ID, buildsDzb selector28 andFUT selector1, then calls0Sf.A0Q with command314 and32000 delay. Request constructor shape is not interpreted here; do not invent a full token-refresh wire recipe from its type name.

FUT.CK4's nonzero branch is the one bound by selector1. It validates a typed response throughB1w/8B6 andCgu/Cgr, extractsBVZ.A00 byte array, Base64-encodes flag3 and stores pref_client_auth_token through2L8.A1U. It then invokes19i.A00 at00a2 before0Sf.A08 over0TV.A03's map. The preference helper's commit/apply and complete typed-response conditions remain untraced, so storing is not claimed to prove crash durability or acceptance of arbitrary token bytes. The zero branch is unrelated and not interpreted.

## Candidate field census

apk-field-refs for19i.A00:1nq finds four instructions, all in19i.A00: three reads and one write. The [retained request body](../2.26.43.72-authkey-rotation/request.txt) assigns a fresh1nq only when null; later invocations reuse the existing candidate, allocate another IQ ID and re-evaluate timing gates. The19i constructor assigns only its store and sender dependencies; no candidate load appears there. No direct candidate-clear/persist instruction is identified in this exact field census. This supports in-process reuse, not crash-safe publication ownership. Generic serialization/reflection/native access, dependency reconstruction and other key-store fields remain outside the search. Do not declare candidate recovery absent globally or replay an uncertain publication merely because this field is not durable.

The prior callback trace shows delivery failure logs only; error and failed replacement update failure time/decrement. This caller inventory locates subsequent entry points rather than a self-scheduled timer. No independent retry timer appears in these selected trigger methods, but full scheduler and all connection events remain untraced. Existing source cannot establish a universal retransmission interval or restart recovery policy.

## Port boundary

Rust's missing rotation lifecycle needs coordination with client-auth-token send/refresh and durable old/candidate key ownership, not just a notification ACK. Android storage format does not need copying into Rust. This chunk establishes trigger order and bounded candidate reuse while keeping network uncertainty and crash recovery explicit. No Rust runtime change, token fabrication, key rotation or live account operation.

From repository root:
```sh
APK=../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-callers "$APK" --class 'LX/19i;' --method A00
target/debug/xtask apk-field-refs "$APK" --class 'LX/19i;' --field A00 --type 'LX/1nq;'
target/debug/xtask apk-smali "$APK" --class 'LX/19i;' --method '<init>' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0ZJ;' --method A0z --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/FUT;' --method CK4 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1rj;' --method A05 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1jA;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1jA;' --method A01 --signatures --exceptions
```

## Reconciliation with earlier evidence (2026-11-06)

The earlier [canonical outcome audit](../../authkey-outcomes.md) already closes
selected token writer, correlated reply, readiness/membership, nonce failure and
held-request dispatch details. The unknowns above describe this export's scope,
not a claim those blocks were never traced. In particular use the retained
[reply converter](../2.26.43.72-client-auth-token-reply/README.md) and
[failure callback](../2.26.43.72-client-auth-token-failure/README.md) rather than
re-exporting them. Global generation/restart ownership remains unresolved.
The [current requirements table](../../authkey-lifecycle.md#current-implementation-boundary-reconciled-2026-11-06)
is the next-work index. No new native behavior is inferred by this correction.
