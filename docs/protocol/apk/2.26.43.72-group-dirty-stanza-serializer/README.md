# Current dirty-group stanza serializer

**Confirmed static**, pinned WhatsApp2.26.43.72/code264307238, APK SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Baseline `2f40e622`. Offline xtask fallback, not REA-provider or runtime evidence.
The [known provider failure](../2.26.43.72-retry-key-envelope-bytes/rea-provider-attempt.json)
remains separate. No Rust runtime, account or network operation changed.

## Lazy operation resolves to a concrete query

The preceding [8UL construction](../2.26.43.72-group-dirty-worker-dispatch/stanza-operation.txt)
installs8cm selector10. The [bounded invoke excerpt](lazy-selector-excerpt.txt)
binds that selector to070a: it maps each typed1bc group to Dyu selector11 and
constructs BVu selector4 from held IQ ID, context and converted child list.
All omitted switch branches are explicitly excluded, and the complete source
hash/length is checked by reproduction.

The [complete selected BVu constructor](query-constructor.txt), selector4 at015a,
creates an iq builder through [B1x.A0c](iq-builder.txt). It supplies to=7Ks.A00,
whose [server domain](server-domain.txt) is g.us, and xmlns=w:g2. The
[B23 helper](iq-attributes.txt) adds type=get and the supplied id. A query child
receives context from the held string and one of four listed values:
get_participating_groups_paginated, inactive_group_migration,
per_group_dirty_recovery, per_group_dirty_recovery_truncatable.
The current dirty worker supplies the last of these, as retained previously.

[Dyu selector11](group-element-constructor.txt) creates group with typed jid
using [B1y.A1F](jid-attribute.txt). [B21.A1C](add-group-child.txt) reads each
Dyu's stored node and appends it to the query. BVu stores the resulting root;
[its A00](query-accessor.txt) returns that node for selector4, closing the worker's
BVu.A00 ->0Sf.A0W handoff. No application-level payload/identity is fabricated.

This establishes the selected semantic shape:

```xml
<iq to="g.us" xmlns="w:g2" type="get" id="supplied-id">
  <query context="per_group_dirty_recovery_truncatable">
    <group jid="typed-group@g.us"/>
  </query>
</iq>
```

The example is schematic, not captured wire bytes or a valid literal group ID.
Attribute insertion/encoded byte order, actual ID value and server acceptance
are not proved by a semantic-node comparison.

## Generated diagnostics must not become invented hard gates

The ID helper calls [0Tk.A06](id-string-check.txt) with0..2^53-1 and false.
That helper checks Java String.length, **not** numeric conversion of the ID.
Null follows its diagnostic/false path; other out-of-length paths call00L.A0C
and return true if that call returns. Do not impose a numeric-ID policy from
those integer constants.

The child list helper [0Tk.A07](children-check.txt) receives1..10000 in this
constructor. Null calls00L.A0C and returns false; out-of-range nonnull sizes call
that diagnostic and return true if it returns. This alone does not prove a
hard refusal at1/10000 or guarantee acceptance: effective00L assertion behavior
remains untraced. [Optional context](optional-context.txt) omits null and otherwise
calls [A07](enum-attribute.txt), which delegates enum checking to A00 then writes
the attribute. Effective failure behavior of that enum checker is not newly
traced; the selected worker supplies an explicitly listed valid context.

A bounded lookup of Dyv.A00 found a different sibling iterator helper; B21.A1C
instead accesses Dyu.A00 directly. That unrelated helper is not retained as this
child path and no broader absence claim follows.

## Rust comparison and limits

Current groups.refresh_node / iq already construct the same selected fields:
iq/get, g.us, w:g2, supplied ID, query context and group@jid children. No missing
field mismatch was found in this semantic subset, so no wire change is made.
Rust's local256-group bound, generic/legacy group admission, queue ownership,
callback response handling, effective native flags and full round/retry lifecycle
remain distinct. The native worker typed-construction gate is documented in the
preceding bundle; this serializer trace does not erase that compatibility gap or
justify rewriting historical work. No MEX branch or native-wide parity claim.

Reproduce13 retained outputs (complete selected methods plus labelled excerpt):

```sh
python3 docs/protocol/apk/2.26.43.72-group-dirty-stanza-serializer/reproduce.py ../android-lab/apks/WhatsApp.1.apk
```

Every selector verifies its complete export hash/length before deterministic
method/excerpt selection and byte comparison. No raw APK, bulk class tree or
unrelated constructor overload collection is committed.

Validation: all13 retained outputs reproduced byte-for-byte with full source
hash/length checks. APK coverage, formatting, regenerated CLI reference,
OpenAPI library-symbol test, whitespace and staged repository hygiene passed.
No full runtime suite was rerun for this research-only chunk.
