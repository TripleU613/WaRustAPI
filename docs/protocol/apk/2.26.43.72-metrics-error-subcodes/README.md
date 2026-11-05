# Metrics failure subcodes — APK 2.26.43.72

**Confirmed static / bounded native branch execution.** Pinned APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`;
reconstructed ARM64 libwhatsapp.so SHA256
`be34e7520883f6fcedbdae0ed0167a1d84a7cdc5bbe63576a4115f89d3b782f5`.
[Reconstruction identity](../2.26.43.72-superpack-reconstructed-members/README.md).
No app, network request, real response, account or durable storage was executed.
Local Rizin exports and Unicorn fixtures are not REA Evidence envelopes.

## Two distinct inputs reach the same numeric classification

The complete [response worker](../2.26.43.72-metrics-native-transport-responses/response-worker-assembly.txt)
0x728e18..0x729410 checks ordinary success and the distinct401 branch first,
subject to the earlier error/endpoint fallback described in the
[transport trace](../2.26.43.72-metrics-native-transport-responses/README.md).
For subsequent failures, **manager byte+0x40 exactly1** selects this path:

- Require nonnull MCFError; copy its failure-reason string.
- Find constant string **`Error Content = `**, object0xc3b720. If present,
  take the suffix from the end of the matched prefix through the string's end.
  It does not search for a matching closing brace or trim a trailing diagnostic.
- Deserialize that suffix through MCIJSONCreateDeserializedObjectFromString;
  cast root to dictionary, look up **`error`** (object0xc3b6c0), cast that to
  dictionary, look up **`error_subcode`** (object0xc3b6f0), cast to number.
- Extract a32-bit integer via0x71dc40, which initializes a stack value to0,
  calls MCFNumberGetValue with format0, then returns the low32-bit value.
  The helper does not branch on the imported conversion's return status.

The earlier fallback checks byte+0x40 **bit0**, while the branch above checks
**exactly1**. Those conditions must not be collapsed for unproven flag values.
String objects are resolved through their+0x28 relocated character pointers,
not guessed from nearby text. `inspect-subcodes.py` retains their exact addresses.

For the other selected branch, complete
[body parser0x72943c..0x729540](../2.26.43.72-metrics-native-transport-responses/regular-error-parser-assembly.txt)
receives the response-data object directly, deserializes through
MCIJSONCreateDeserializedObject, and uses the same dictionary/key/number chain.
The worker also invokes this parser before its early error/endpoint fallback.
Thus a parser side effect can precede trying another endpoint.

## Recognized codes change metrics credentials, not account identity

Both paths use32-bit arithmetic to recognize exactly these extracted values:

| Extracted subcode | Local diagnostic result | Selected credential action |
| --- | --- | --- |
| 3067002 | 2 | release current; promote spare or request renewal |
| 3067003 | 3 | release current; promote spare or request renewal |
| 3067004 | 4 | release current; promote spare or request renewal |

Current credential is manager+0x78; spare is+0x80. The recognized branch calls
MCFRelease(current), then if spare exists stores it as current and zeros spare.
If no spare exists, it zeros current and calls renewal scheduler0x728b24.
That scheduler has its own provider/pending-state gates; a call is not proof a
renewal request was sent. See the
[credential lifecycle](../2.26.43.72-metrics-native-credential-lifecycle/README.md).

Other extracted values and malformed/missing dictionary fields map to local5
without those credential-slot changes. In the failure-reason path, missing
error/reason/prefix maps to6; malformed JSON/schema after finding the prefix
maps to5. The body parser returns6 for absent response data,5 for malformed or
unrecognized data. These numbers are **internal diagnostics**, not HTTP statuses,
regular task outcomes, ban codes or proved server policy names. No semantic
names such as expired/revoked are assigned to the three subcodes here.

In the failure-reason branch, after classification and any credential change,
the worker checks remaining endpoints: if present, it calls the sender again;
otherwise it invokes optional callback-table slot+0x20 and calls0x728050(raw1).
A recognized subcode alone does not take the401 task-removal branch. This is a
metrics-specific path, not authority to retry other API requests or replace
account identity. No Rust production metrics sender exists in the bounded
source search recorded in the [regular IQ trace](../2.26.43.72-metrics-regular-iq/README.md).
No Rust behavior is changed or fabricated telemetry introduced.

## Reproduction and limits

Run from this directory, with the reconstructed library path:

```sh
python inspect-subcodes.py /path/to/libwhatsapp.so
python check-subcodes.py /path/to/libwhatsapp.so
```

The second command requires Unicorn2.1.4. Compare stdout byte-for-byte with
`subcode-boundaries.json` and `subcode-results.json` respectively. The checker
maps pinned PT_LOAD segments and relocations, then executes the complete
selected native body parser and its actual argument/number helpers. Controlled
adapters supply JSON objects, casts, dictionaries, numbers and release behavior;
the renewal scheduler is intercepted and counted. It does **not** execute the
JSON library, actual refcounts/destruction, renewal, HTTP, persistence or clocks.

Twenty-four parser cases cover absent body/root/error/subcode and eight numeric
values, each with/without a spare credential. They assert exact diagnostic,
current/spare slots, current release and renewal-call count. Seven additional
cases execute the worker's integer-classification slice0x729140..0x729154 with
w0 supplied and w22 initialized5. That slice test does not execute its preceding
failure-reason extraction or following endpoint/credential behavior. The latter
remain static branch conclusions, with full worker bytes retained above.
Malformed nonnull wrong-type objects and imported number-conversion failures
are not exercised. The32-bit extraction contract does not establish full JSON
number coercion/range behavior.

**Inferred:** these are explicit metrics credential invalidation responses at
this local boundary. **Observed live:** none. **Unknown:** authoritative server
meanings, frequency/mandatory telemetry, callback generations, exact renewal
outcomes, complete endpoint retry policy, durable buffer removal/recovery and
whether any of this affects restrictions. No zero-ban guarantee follows.
