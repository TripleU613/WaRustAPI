# MobileConfig null/list variables on the selected JSON route

**Confirmed selected static**, APK2.26.43.72 code264307238, SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Static exports and existing pinned platform evidence; no REA runtime envelope,
observed wire request or APK execution.

## Null setters have different intermediate shapes

The [fetch route](../2.26.43.72-mobileconfig-fetch-route/README.md) invokes
GraphQlCallInput.A0B("queries",null),A0A("query_hash",null) and
A0A("query_string",null). They cannot all be modeled by a generic omit-null rule.
A0B first callsNha.A07, which obtains a fresh/pooledNhb, activates it if new and
attaches that child under the name throughA0A. It then callsGraphQlCallInput.A04;
that method returns immediately for a null or empty input list, leaving the
attached list node. Nhb.<init> initializes a fresh emptyArrayList; the selected
fresh-node construction therefore creates queries as an empty
intermediate list, not null or a missing key. Pool/refcount correctness remains
an independent lifecycle obligation, not proven for arbitrary stale objects.

The string setters callNha.A00, which appends key and value even when the value
is null. Existing complete [object/map lifetime evidence](../2.26.43.72-blocklist-mex-values-lifetime/README.md)
already proves direct null retention and last-write wins in the exported map.
Reuse that source rather than repeating its general analysis. Its
GraphQlCallInput.A00 conversion turns an emptyNhb into an emptyArrayList; stored
null scalars remain null. A03/privateA01 builds sortedTreeMaps with every named
entry. This is confirmed intermediate object/map behavior, not yet socket bytes.

6oj.A0P creates0i1 and attaches the nonnull generated input underinput.
0i1.A00's null guard applies to the entire input object, which is present here;
it does not omit individual nested null fields. Its primitive wrappers' separate
null-skip rule must not be substituted for direct input setters.

## Platform-qualified JSON prediction

The existing [0mF constructor](../2.26.43.72-mex-server-jid-equality/envelope-request.txt)
passes query.getQueryParams().Axc() intoJSONObject(Map), wraps it undervariables
withqueryId and serializes. The retained
[unmodified Android16 JSON source](../2.26.43.72-mex-null-serialization/README.md)
recursively wraps Collection asJSONArray and null map values asJSONObject.NULL.
That constructor behavior differs fromJSONObject.put(name,null), which removes
an entry. Under this pinned platform implementation, this selected JSON route
therefore predicts:

- queries: []
- query_hash: null
- query_string: null

predicted-shape.json combines this inference with the preceding confirmed setter
inputs, using a clearly synthetic debug string. It is not a ready-to-send request,
production fallback profile, Android capture, effective unit/stack choice or proof
of byte ordering/escaping. Actual endpoint parameters and their producer gates
must still come from valid SDK state; no debug ID or client hash is invented.

**Unknown:** which platform implementation executes on a particular handset,
Pando's serialization, full socket envelope, cache/SDK hydration and live server
acceptance. The0iq/0j3 Pando gate must be retained; this evidence does not authorize
assuming all MobileConfig fetches use0mF. Exact mobileconfig wire behavior remains
platform/branch-qualified rather than universal.

## Reproduction and impact

Exports use apk-smali --signatures --exceptions forGraphQlCallInput.A0B/A04,
Nha.A07/A0A/A01,Nhb.<init>,6oj.A0P and0i1.A00. Reuse the prior full direct setters/converters,
Axc export and0mF constructor linked above. No new platform download or modified
Android serializer is required; prior pinned source checksums remain authoritative.

This closes the selected list-versus-null ambiguity for a future request port.
Rust still has no MobileConfig fetch/hydration adapter. No sender, request,
persistence mutation, identity reset or live account operation was enabled.
