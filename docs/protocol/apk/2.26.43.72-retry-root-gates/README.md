# Retry-root removal gates — APK 2.26.43.72

**Confirmed static, incomplete applicability.** APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
`root-normalization-smali.txt` is the complete checked extractor output for
`LX/FTr;->ATZ(LX/EoG;LX/Enu;LX/Ewq;)V`, registers49. It supplements the earlier
[JADX export and unresolved policy](../../retry-enc-count.md#retry-root-normalization-remains-unresolved).
No APK, account or network operation was executed. This checked DEX export is
local evidence, not a REA Evidence envelope.

The map-removal block1442..1478 removes `t`, `participant`, `recipient`,
`recipient_pn` and `peer_recipient_username` from Ewq.A0E. Checked instructions
confirm immediate gates including Enu.A0B (loaded into v20 at009c..00a4), a
nonzero v21, positive v17 (Ewq.A05 loaded0090..0098) and nonnull v9 (EiB at this
point). Numerous earlier message/classifier branches also precede the block.
These are not five unconditional removals for every retry.

The v21 gate, complete EiB producer, applicable message/address classifiers and
proven correspondence to Rust's supported direct saved/fresh-session paths are
**unknown** in this bounded trace. The whole method is retained so those gates
can be followed without relying on duplicated JADX source. No root attribute is
removed in Rust on the basis of this incomplete policy. Existing refusal of
retry counts3/4 and unsupported group/broadcast retries remains unchanged.

Reproduce using the pinned APK outside Git:

```sh
target/debug/xtask apk-smali /path/to/WhatsApp.1.apk \
  --class 'LX/FTr;' --method ATZ --signatures --exceptions
```

Stdout must match `root-normalization-smali.txt`. One exact method is present;
its full signature is in the retained output. `checksums.json` covers this README
and export. No complete root-normalization compatibility is claimed.
