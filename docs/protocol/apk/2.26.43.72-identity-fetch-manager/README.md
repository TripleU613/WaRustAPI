# Identity notification fetch manager, APK 2.26.43.72

Confirmed static DEX relationships for pinned APK SHA-256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`, code264307238. No target execution, live account traffic or REA Evidence envelope claim. Focused xtask fallback continues the already diagnosed unavailable Android provider; no repeated setup. Every txt is a complete selected method, with full DEX signatures and exceptions. checksums.json binds all other files.

## Reproduce

From repo root with `APK=../android-lab/apks/WhatsApp.1.apk`:

```sh
target/debug/xtask apk-smali "$APK" --class 'LX/14t;' --method A0A --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/14t;' --method A07 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/14t;' --method A06 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/14t;' --method A00 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/14t;' --method A05 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/14t;' --method A02 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Fy;' --method A0W --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/Ex8;' --method A06 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/Ex8;' --method A02 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Dn;' --method A0k --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Dn;' --method A0S --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/F3k;' --method A06 --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/EoY;' --method '<init>' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/EoY;' --method equals --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/EoY;' --method hashCode --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/E7N;' --method '<clinit>' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/EHs;' --method A00 --signatures --exceptions
```

## Call flags, not inferred policy

Entry is [Fc1.run case15](../2.26.43.72-identity-ack-dispatch/identity-worker.txt)0db6..0ea4. After nonnull held-key lookup it calls14t.A0A([device],8,true). A0A synchronizes and delegates toA07 with null context and integer0/0, forwarding true. A07 sets null scope, callsA06; A06 forwards toA00 with final boolean false. Conditional second call goes directly toA07 with the last boolean true. A00 log strings bind boolean33=fromIdentityChange,35=isPqSessionUpgrade. A05 log and forwarding bind boolean34=fetchPqPrekey.

| Worker call | Context / effective scope | Internal reason | fromIdentityChange | fetchPqPrekey | isPqSessionUpgrade |
| --- | --- | --- | --- | --- | --- |
| First A0A | null / E78.A02 | 8 | true | false | false |
| Conditional A07 | null / E78.A02 | 8 | true | true | false |

Reason8 is manager/job metadata, not a new IQ wire attribute. Thus the second call is **not** a PQ session upgrade and neither notification call enters A00's early skip for upgrade=true plus manager backoff flag. This does not exempt them from other downstream guards/backoff.

## Secondary gates

0Fy.A0W requires its selected map to contain the user, nonnull0i4 fromA0G, trueA0d, flag0x480b and nonnull0i4.A16. The map/cache meaning, A0d and A16 authority are not established here. It is not demonstrated to be an own-account test. Ex8.A06 requires one of0Dn.A0k/A0S plusEx8.A02. The category predicates accept numeric getType sets {0,11,18,22,26,7} and {17,24,25,19,23,27}, respectively; subtype-name mapping is not inferred. Ex8.A02 requires A03 plus flag0x5e60. Existing [Ex8 policy trace](../2.26.43.72-sender-key-session-requirement/pq-policy.txt) retains deeper A03/A04/key-readiness code; effective runtime values and metadata gates are not observed. No flags or capabilities are fabricated in Rust.

## Admission and in-flight scope

A00 uses default E78.A02 when scope is null. It examines A0B entries against SystemClock.uptimeMillis: remove only when storedA03+60000 < now (equal is retained), at0092..00f6. This is cleanup during another manager invocation, not an autonomous retry timer, durable send reconciliation or proof of safe replay. Completion/removal on other paths and map owner lifetime are not traced here.

For non-upgrade calls, A00 selects E7N.A03 when fetchPQ=false or A02 when true at0214..0222, then F3k.A06 builds EoY using scope, mode, user category and device. E7N initializer constructs distinct REGULAR/PQ enum instances. EHs.A00 supplies the user-category discriminator1/2/3/default0 from three JID predicates; their complete subtype mapping is outside scope. EoY constructor/equality/hash include user, that discriminator, device, E7N mode and E78 scope. A00 checks A0B membership before adding a device to new work at0236..026e; identity-change devices are separately collected at026e..0278. Therefore regular/PQ requests for the same device do **not** coalesce into the same manager-map key. This refutes a single-device-only coalescing assumption; later job-queue deduplication is not established.

A00 also filters14t.A02 and has guarded address normalization. A02 checks EqK.A00 and0x808c; A05 describes that exclusion as unexpected TEE bot. EqK/category mapping and normalization behavior are not generalized to supported Rust PN/LID addresses here. A05 returns when its input list is empty, then re-filters excluded devices, preserves the identity subset, and constructs BulkGetPreKeyJob with both PQ-fetch and upgrade flags. Large-list batching/config0x399 is present in the complete method but is outside this single-device audit. Job persistence, deduplication, retry policy and actual wire sends remain separate.

## Rust boundary

Rust's identity-notification path still emits its guarded ordinary key read and does not fabricate the conditional PQ request, metadata, feature flags or a 60-second resend. Its durable uncertainty and original request/adoption authority remain distinct from the native in-flight map. This research adds no automatic request, replay or new wire parameter. Current flags/scope are established; full native policy, effective gates and live behavior remain incomplete.
