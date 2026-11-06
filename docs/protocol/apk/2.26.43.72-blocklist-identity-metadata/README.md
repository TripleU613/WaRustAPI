# Blocklist typed identity and metadata selection

**Confirmed selected static**, WhatsApp2.26.43.72 build264307238,
SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No runtime/device/account or server observation. Complete selected DEX exports
plus [complete REA helper Evidence](../../rea/2.26.43.72-blocklist-identity-metadata/README.md).
The [retained full parser](../2.26.43.72-blocklist-snapshot-children/parser.txt)
is reused rather than duplicated.

## Identity fallback is typed and nullable

0Tp.A0A first uses the attribute's stored Jid A01; otherwise it passes the
stored string via0Tp.A0M into0Dd.A02. A0M returns A03 when the attribute exists,
or the supplied default when absent; this does not prove every constructor's
stored-string materialization policy.0Dd.A02 returns null for null/empty input
or a caught parse failure fromA00 (catch-all only around the parse call).
The full A00 grammar/subclass dispatch is not newly traced here.

A0A logs a non-compliant parsed/stored JID but continues to conversion. Its
Class.cast result is returned; only the selected ClassCastException catch
returns null. The separate DeviceJid conversion branch does not apply to the
UserJid/PhoneUserJid requests in6qD.A02. Do not infer that non-compliance itself
throws, that every Throwable is caught here, or that Rust's broader parsed Jid
variants match Java's requested class.

6qD.A02 tries UserJid jid first, then PhoneUserJid pn_jid only when null. If
both are null,0144..014e logs/skips that item and continues. Rust currently
returns a content-free projection error for invalid/missing/non-user identity;
its pn_jid fallback also accepts any supported user-like Jid, not a separately
proven PhoneUserJid class. These are distinct gaps. Admission must not be
relaxed by assuming Rust Jid::parse reproduces Android class/grammar behavior.

## Metadata is a priority branch, not three independent optional fields

0Dn.A0Z tests getType()==18, null false.0Dx.getType confirms18 and extends0Dw;
PhoneUserJid.getType is0 and extendsUserJid. The focused0Dw.getType lookup has
no method body; that negative does not mean it lacks an inherited type method.
No exhaustive census of all type18 subclasses is claimed.

In the retained6qD.A02, all metadata is gated by A0Z at00b8..00c4:

- Non-type18 jumps directly to membership Map.put at00e8.
- Type18 reads display_name. A nonnull value takes the display-name branch:
  nonempty writes its map; empty writes nothing. Both then go to membership.
- Only null display_name reaches00fc. A nonnull typed PhoneUserJid pn_jid
  appends the LID/active association and goes to membership.
- Only absent/null typed pn_jid reaches012c. A nonnull username (including an
  empty string) overwrites its map. Missing username leaves prior map data.

The action==block and active==true Booleans were computed before those
branches. Metadata/association handling is not conditioned on block==true.
Membership always uses its separate last-write Boolean map. Thus a last-item
membership correction does not establish last-row metadata semantics: an
absent/empty later name does not universally delete an earlier map value,
and a present name can suppress PN/username extraction in this parser.

Rust's current unified BlockedRow snapshots accept display_name/username for
ordinary phone JIDs and replace metadata on the last block row. The parser
trace establishes a mismatch in intermediate metadata selection, but durable
native metadata adoption/retention and alias side effects still need tracing
before changing the persisted row policy. No guessed field merge, alias
installation, snapshot skip, sender or automatic request is introduced.

## Reproduction

From repository root with the pinned APK and built xtask:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
OUT=/tmp/blocklist-identity-metadata-repro
mkdir -p "$OUT"
sha256sum "$APK"
target/debug/xtask apk-smali "$APK" --class 'LX/0Dn;' --method A0Z --signatures --exceptions > "$OUT/lid-predicate.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0Tp;' --method A0A --signatures --exceptions > "$OUT/typed-attribute.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0Dx;' --method getType --signatures --exceptions > "$OUT/lid-type.txt"
target/debug/xtask apk-smali "$APK" --class 'Lcom/whatsapp/infra/core/jid/PhoneUserJid;' --method getType --signatures --exceptions > "$OUT/phone-type.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0Tp;' --method A0M --signatures --exceptions > "$OUT/string-attribute.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0Dd;' --method A02 --signatures --exceptions > "$OUT/parse-null.txt"
target/debug/xtask apk-smali "$APK" --class 'LX/0Dw;' --method getType --signatures --exceptions
```

checksums.json binds every other bundle file. Reproduction of static exports
is not execution of the APK, protocol acceptance or proof of zero restrictions.
