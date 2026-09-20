# APK research helpers

Small offline helpers used on the full Dalvik disassembly of the comparison APK
(`./target/release/xtask apk-smali ~/Downloads/WhatsApp.apk --class "" > dump.txt`,
about 570 MB for build 2.26.37.71). They never touch the network or the lab data.

- `dumpm.py DUMP ANCHOR [ANCHOR...] [--max-methods N] [--max-lines N] [--full]`
  prints every method whose body contains a `const-string` equal to an anchor,
  in a compact view (strings, calls, constants). Use it to find the builder of a
  stanza from an attribute or namespace string; remember that names in the
  binary token dictionary (`crates/whatsapp-api/fixtures/tokens/web-md-v3.json`)
  may still appear as string constants in builders, but not always.
- `desc.py DUMP [CLASS...] [--anchor FIELD_NAME...]` parses protobuf-lite
  `dynamicMethod` descriptors (RawMessageInfo strings) into `(field name, number,
  type)` triples. Field names are consumed in declaration order; for very large
  messages the name alignment can drift after fields whose names are not string
  constants, so cross-check numbers against live captures or the public schema.

Evidence produced with these tools is written up in `docs/protocol/*.md` with
the class names of the build it came from.
