# MobileConfig query setup failures and conditional metadata

**Confirmed selected static evidence**, APK 2.26.43.72 code 264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reconstructed arm64 libmobileconfig-jni.so: 852440 bytes, SHA256
`5c50f623849463f4b71b662dbceabeef1a5f4dc36f56f0154c9048ef0d09255c`.
Only static reads occur; no library/APK execution or account operation.

## Earlier reasons for an empty query-parameter vector

The [requester dispatch](../2.26.43.72-mobileconfig-requester-dispatch/README.md)
calls 0xac09c with output vector, requester and request-data pointers. The producer
checks the requester string at +0x28 using the libc++ short/long string length
helper 0xca4a8. Zero length constructs optional diagnostic logging with
BadRequesterSetup / FBMobileConfigRequester::getQueryParams / empty deviceId,
then helper 0xcade4 zeroes all three output-vector words and returns to cleanup.
The earlier requester construction copied this string from updater+0xf8. Its
upstream source and relationship to account/device identity remain untraced.
Do not invent a device identifier to satisfy this check.

With a nonempty deviceId, the producer first tests the request-data string at
+0x08. A nonempty string takes the query_hash route and bypasses the schema-empty
check. For an empty string it obtains two containers via 0xaa704 and 0xaa89c.
When request-data mode word +0x9c is not 1 and both containers' +0x20 words are
zero, it logs the same setup category with no parameter schemas available,
zeroes the output vector and returns. Otherwise processing continues. The
getters read configuration+0x258 fields +0x18/+0x20 and can take deferred-refresh
branches; the refresh algorithm and precise container types are still unresolved.
No parameter-schema replacement or new account state is justified by this trace.

**Caller interaction matters:** a mode other than 1 with an empty returned vector
is locally completed as false/Empty query params without calling the fetcher.
Mode 1 bypasses that caller refusal. Therefore the producer's empty-device result
alone does **not** prove the caller can never fetch with an empty vector. These
are two separate gates; mode 1's semantic name and intended runtime conditions
remain unproven. Diagnostics are local setup evidence, not server responses.

## Selected metadata uses distinct values and presence rules

The producer appends key/value string pairs to its native vector. The pair
helpers create a key string followed by the value string at +0x18; elements are
0x30 bytes. Retained outlined helpers make string-copy/move operations explicit.
These fields are SDK logical inputs before the Java map/route conversion, not a
complete HTTP or MEX envelope.

| Field | Selected source and gate |
| --- | --- |
| query_hash | Nonempty request-data string +0x08 is supplied to append helper 0x7ccac at 0xac188–0xac194. Other schema-derived branches also append this key; their algorithms remain open. |
| request_data_query_hash | Request-data byte +0x29 equals 1: getter 0xaa8f8 supplies the value to pair helper 0xae37c. This getter can refresh deferred state; no independent nonempty-value guard is shown on this selected append path. |
| global_value_hash | Copies request-data string +0xa8; appends through 0xae330 only if the copied string is nonempty (0xac630–0xac6e0). |
| ep_refresh_id | Signed request-data int +0xc0 is at least 1; std::to_string(int) yields decimal value, appended using key 0x1a716 (0xac6e4–0xac70c). Zero/negative skip this selected append. |
| one_query_hash | Configuration string +0x160 is nonempty; copied value goes to 0xae2e4. This is a distinct key from query_hash. |
| bool_opt_policy | At 0xacb0c, raw configuration byte +0x194 is ORed with 2 when byte +0x208 is nonzero, then converted by to_string(long) and appended through 0xae2b0. Do not normalize the first raw byte to a boolean. |

Further selected conditions: requester byte +0x54 equals 1 enables a
param_list_checksum append on the nonempty-query-string path and the inspected
schema path. The helper 0xad9a0 combines container-derived checksums (optionally
XORing another) and formats an unsigned decimal result; the underlying checksum
algorithm is untraced. In the inspected schema path, request-data byte +0x49
equals 1 appends with_names with literal true using 0xae3c8. These conditions are
not evidence that every field is universally mandatory or independently usable.

The earlier [Java MEX route](../2.26.43.72-mobileconfig-fetch-route/README.md)
consumes selected map fields, so do not assume that every native vector entry
becomes a top-level MEX variable. The previously retained explicit-null queries
field likewise does not identify this vector as the JSON queries array.

## Evidence and reproduction

```
python3 docs/protocol/apk/2.26.43.72-mobileconfig-query-setup/inspect-query-setup.py ../android-lab/output/metrics-native/reconstructed-reproduction/libmobileconfig-jni.so > /tmp/mc-query-setup.json
cmp /tmp/mc-query-setup.json docs/protocol/apk/2.26.43.72-mobileconfig-query-setup/setup.json
```

The inspector reuses pinned ELF/member validation, checks selected strings and
BL targets, and emits the bounded findings. query-producer.txt retains the
selected 0xac09c..0xad9a0 function region, including later branches and exception
cleanup for future reuse; this report does not claim every retained instruction
has been semantically modeled. Other files isolate pair/checksum/getter and
outlined-helper evidence. These are LLVM static disassemblies; nearest-exported
symbol headings are not local function identities.

rea-query-entry.json retains complete REA Evidence
`ev_a28567f5395738933f7b307a27c796f05a3c0e93d291c503ccba25120fd6574a`:
`e scr.color=0; pd 70 @ 0xac09c`, exit 0, no output truncation, unchanged artifact
hash, with an entrypoint warning. Seventy instructions is an explicitly bounded
entry window. Register-relative address annotations are not runtime observations.

**Still unknown:** mode/flag field producers, deviceId provenance, schema query
serialization, underlying hashes/checksums, additional vector entries and their
route consumption, runtime fetcher installation, response parsing, persistence,
retry policy and cache activation. No Rust runtime behavior changes or guarantee
of server acceptance follows. Next trace the concrete schema query/hash producers
and the request-data constructor rather than guessing required wire values.

Follow-up: [schema fragments and ordered checksum](../2.26.43.72-mobileconfig-schema-fragments/README.md)
now derives the selected fragment-token rules and param_list_checksum recurrence.
Real table/selection producers and other hash algorithms remain untraced.
