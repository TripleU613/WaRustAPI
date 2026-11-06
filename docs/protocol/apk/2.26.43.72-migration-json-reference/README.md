# Migration-shaped JSON: controlled Android reference oracle

## Authorities and scope

APK binding is confirmed selected static for WhatsApp2.26.43.72 build264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
The [existing shared envelope](../2.26.43.72-mex-server-jid-equality/README.md)
already traces0mF: query parameters.Axc() Map -> JSONObject(Map), wrapped under
queryId/variables, toString ->0mK. Fresh0mF/0mK constructor exports match those
retained methods. This is reuse of an existing serializer finding, not a new
claim that this APK ships org.json. A focused constructor-body lookup again
returned no org/json/JSONObject bodies; runtime class identity remains separate.

The prior [static platform null inspection](../2.26.43.72-mex-null-serialization/README.md)
is reused. This bundle adds controlled compilation/execution of that pinned
public JSON implementation with migration-shaped synthetic maps. It executes
no WhatsApp class, Android device, account or request. Runtime provenance is
ordinary Java tooling, not REA runtime Evidence or live server acceptance.

AOSP reference: android-16.0.0_r1, libcore commit
`fff4fcc0cf7f080cf2511cbb57561482b13b218f`.
[JSONObject primary source](https://android.googlesource.com/platform/libcore/+/fff4fcc0cf7f080cf2511cbb57561482b13b218f/json/src/main/java/org/json/JSONObject.java).
Map conversion wraps null with the JSON null sentinel; nested maps and
collections are recursively wrapped. This differs from put(name,Java-null).
That qualified source result was already retained; the new oracle verifies its
selected consequences under a desktop JDK, not every OEM/runtime implementation.

## Controlled execution

sources.json pins bytes/SHA256/URLs for six unmodified Apache2.0 AOSP JSON files.
The existing JSONObject/JSONArray/JSONStringer files are reused when no public
cache copy exists; three additional source files can be fetched with --fetch.
No upstream algorithm source is rewritten or replaced. Annotation-only stubs
supply Android/libcore annotations so the same algorithms compile on the
installed JDK21. Stubs do not implement JSON behavior. Compilation/classes use
a temporary directory that is removed on completion.

ReferenceJson.java supplies four synthetic inputs and asserts explicit field
presence/null, Boolean types, conditional dirty_ack absence, string/UTF8
roundtrip and list shape. expected.json retains complete emitted JSON strings
and UTF8 byte counts. queryId is cross-checked against the separately pinned
MigrateBlocklistLid release export. The all-null and empty cases test serializer
behavior; they do not assert that the native builder emits those inputs or the
server admits them. Input-policy/provider authority is not synthesized.

Cases: null_fields; escaped_username_dirty; boolean_identifier; empty_blocklist.
Quotes, backslash, newline and a non-ASCII character are synthetic test data.
No account identifier, real list/hash or user content is used.

The APK's0Tj.A05 converts body String through08Q.A05, whose initializer isUTF-8;
complete selected exports here bind this bridge. The
[existing request guard trace](../2.26.43.72-mex-request-envelope/README.md)
already establishes Java UTF16-length checks and assertion calls, not numeric
IQ-ID parsing or universally safe false-return omission. Fresh trace/string
exports match those retained files. No guard limits or current Rust group
construction changed.

## Reproduction

From repository root with installed JDK21. The --fetch command reads only pinned
public source URLs and writes the explicit public cache; the second is offline:

```sh
JAVA_HOME=/home/tripleu/.local/share/mise/installs/java/temurin-21.0.13+9.0.LTS \
python3 docs/protocol/apk/2.26.43.72-migration-json-reference/reference_oracle.py \
  --sources /tmp/migration-json-public-cache --fetch \
  --check docs/protocol/apk/2.26.43.72-migration-json-reference/expected.json

JAVA_HOME=/home/tripleu/.local/share/mise/installs/java/temurin-21.0.13+9.0.LTS \
python3 docs/protocol/apk/2.26.43.72-migration-json-reference/reference_oracle.py \
  --sources /tmp/migration-json-public-cache \
  --check docs/protocol/apk/2.26.43.72-migration-json-reference/expected.json

target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/0Tj;' --method A05 --signatures --exceptions > /tmp/migration-text-encoding.txt
target/debug/xtask apk-smali ../android-lab/apks/WhatsApp.1.apk --class 'LX/08Q;' --method '<clinit>' --signatures --exceptions > /tmp/migration-charset.txt
```

Without --check the oracle prints the report for review. Each cached source is
verified before compilation; --fetch is required for any missing public source.
checksums.json binds every other bundle file. Source metadata is separate from
APK provenance, and no duplicate REA envelope/source library copy is added.

## Remaining boundaries

Combining prior migration input/map evidence, this shared non-Pando wrapper and
the controlled reference predicts explicit nested JSON nulls under the pinned
platform behavior. Pando serialization, actual device/OEM library selection,
identifier policy/authority, writer/account generation and complete migration
adoption remain unverified. No dedicated Rust migration sender, automatic fetch,
retry, cancellation or live operation is introduced. Existing already-inspected
cache/history issues are not repaired by these serialization fixtures.
