# Current outgoing edit-field propagation

APK 2.26.43.72, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
[Interpretation, comparison-build predicate and limits](../../own-unkeep-edit.md).

The three JSON files are complete REA method Evidence. `send-job-send.txt` is
complete structural disassembly from the repository's checked `xtask` decoder,
not REA pseudocode. It was produced without executing the APK:

```sh
target/debug/xtask apk-smali /absolute/path/to/WhatsApp.1.apk \
  --class 'Lcom/whatsapp/jobqueue/job/SendE2EMessageJob;' --method A06 \
  --signatures --exceptions > send-job-send.txt
```

Exact source file SHA-256:
`572d6304696d7098436a3dfa5a588559879aa2f971072964af719509b84183a7`.
Offsets in that file are method-relative **bytes**, distinct from REA's
code-unit invocation offsets.

The REA Java rendering of A06 was partial: its warnings consumed the 1 MiB
source limit (19,843,363 bytes reported total). It was not treated as complete
proof, and is not duplicated here. Its partial Evidence ID is
`ev_367f1300b412de13de9de26e6af629f7c7ece4a27037edb901b940f202ddaeee`.
Use the complete checked disassembly for the source path above. Inventory,
metadata and this bounded propagation trace do not establish whole-method
semantic interpretation or runtime execution.
