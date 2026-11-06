# UsyncQuery persisted server ID, APK43.72

Confirmed static WhatsApp2.26.43.72 code264307238 SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`. Supported xtask DEX fallback continues diagnosed unavailable Android provider. Full selected classes/methods and exact packaged JSON asset retained; no execution/live traffic or REA Evidence envelope claim. checksums.json binds every other file.

## Reproduce

From repo root:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-smali "$APK" --class 'LX/0iT;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0ih;' --method 'A00' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0ik;' --method 'A00' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/5vM;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0ik;' --method 'A01' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0ig;' --method '<init>' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/1Wy;' --method 'invoke' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0ij;' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0ii;' --method 'A00' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0im;' --method 'CRh' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0im;' --method '<init>' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0in;' --signatures --exceptions
target/debug/xtask apk-field-refs "$APK" --class 'LX/0ik;' --field A00 --type 'Ljava/util/concurrent/ConcurrentHashMap;'
target/debug/xtask apk-callers "$APK" --class 'LX/0ik;' --method A01
python3 - <<'PYTHON'
import hashlib, json, zipfile
p='../android-lab/apks/WhatsApp.1.apk'
assert hashlib.sha256(open(p,'rb').read()).hexdigest()=='887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e'
with zipfile.ZipFile(p) as z:
    raw=z.read('assets/whatsapp-android-mex_client_persist_ids.json')
    print(hashlib.sha256(raw).hexdigest())
    row=json.loads(raw)['UsyncQuery']
    print(json.dumps(row,indent=2))
    assert str(row['doc_id'])=='38635300486116501'
PYTHON
```

## Producer and resolver binding

The prior HSo constructor supplies callName UsyncQuery and buildConfig whatsapp-android-mex.0iT stores these inA05/A04 and returns them through getCallName/getResolvedBuildConfigName. The earlier0j2.A03 calls0ih.A00 with those values and refuses empty/null resolved IDs.0ih resolves config through0ik and calls CRh(callName), catchingException to return null.

0ig initialization registers0ij.A03 through1Wy selector22. Complete0ij bindsA03 to whatsapp-android-mex and whatsapp-android-mex_client_persist_ids.json.1Wy selector22 reaches0666, calls0ii.A00(A03).0ii constructs/caches0im with that asset path;0im's JSON lazyA02 uses0in, which opens the packaged asset and constructs JSONObject.0im.CRh reads getJSONObject(callName).getString("doc_id"), returning empty on JSON/IO error. The separate client ID getters are not selected by this CRh path.

The registry API registers only when a key is absent, and its literal resolver fallback5vM returns null fromCRh. The physical inventories contain12 registration call sites and3 registry-field references; alternate startup order/object aliasing and runtime overrides remain untraced. The selected static normal registration chain is established, not an observation of runtime registry state.

## Exact packaged identity

persisted-ids.json is the exact raw asset. selected-entry.json records its digest and the UsyncQuery entry:

- server doc_id:38635300486116501
- client_doc_id:407762649418300062941108847678
- operation_name_hash:4077626494
- operation_text_hash:18300062941108847678
- schema:whatsapp_mex

The server ID is the selected non-Pando CRh value used by the shared queryId/query_id/trace chain. The longer client ID must not be substituted. Preserve decimal digits without IEEE754 rounding: this server ID exceeds2^53; Python extraction retains an integer and separately records its exact string. The sibling Argo/flatbuffer assets were inventoried but not decoded, and are not JSON. Their identifiers and effective Pando selection remain separate.

## Port boundary

This closes the packaged UsyncQuery server ID and selected normal resolver chain for this pinned APK. It does not establish server acceptance, current remote configuration, full user/protocol variables, account gates, Pando ID selection or authenticated mapping adoption. No Rust request/API, automatic preflight, ID rotation or live operation was added. Do not reuse this ID as a timeless constant or confuse it with a group-info query ID.
