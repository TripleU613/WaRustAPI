# Auth-key storage and connection lifecycle evidence

Static APK **2.26.43.72**, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No execution, key replacement or server request. Full REA envelopes are retained
including provider limitations and coroutine/decompiler warnings where present.
[Findings and boundaries](../../authkey-lifecycle.md).

| Artifact | Operation / method | Evidence ID |
| --- | --- | --- |
| [active-key-getter.json](active-key-getter.json) | `X.AnonymousClass144.A0E` | `ev_bedbe93d0b6b62f81374ef9e54068f85209a8abd2979e7a3ce5c068c3bf2ca97` |
| [active-key-load.json](active-key-load.json) | `X.AnonymousClass144.A00` | `ev_de095770980f1066f93c1714921389b456a7aa8e5538a83438ee1826c5c8d7c0` |
| [active-key-references.json](active-key-references.json) | `trace_android_references` | `ev_3f354dddc9d6d6cd99e4bbb0b89f498426e6d0e04ae95749aaa1707d41d632ad` |
| [active-wrapper-getter.json](active-wrapper-getter.json) | `X.AnonymousClass144.A0D` | `ev_b6118d9e311cb19261baa7a9c9b7afea064e0789c256c1c408f3586be4e03ecc` |
| [active-wrapper-references.json](active-wrapper-references.json) | `trace_android_references` | `ev_9c4badeb1ae9ce292bdb0bdfc2d8ec4905434cbc091bb08d1b6ae1f93b7213ad` |
| [auth-store-class.json](auth-store-class.json) | `inspect_android_class` | `ev_b1ec97aa9b0c85db1a6ed607deeddced9726891689204482c4fa3c2d612de372` |
| [companion-key-use.json](companion-key-use.json) | `X.C0Xl.A07` | `ev_c877a2725477d8fb758ee3a3184e09b82cedbc4bbdbe83057695f1539a237d08` |
| [connection-key-load.json](connection-key-load.json) | `X.C1jv.A0C` | `ev_f03607ec1959dd965248306844ea7b0707038a6dbb64f2a52216e22c37a10ced` |
| [generate-keypair.json](generate-keypair.json) | `X.C1nq.A00` | `ev_c95afa240d0debb66b4fac17ae7790e100fbe664dc3cf737f5f79a5d48f1d361` |
| [invalidate-blacknoise.json](invalidate-blacknoise.json) | `X.AnonymousClass145.A04` | `ev_1630d62d666186ad9389b9f20d167fc16ac42770294388eb2ae0beb4a4daf5c7` |
| [persist-password-key.json](persist-password-key.json) | `X.AnonymousClass144.A06` | `ev_2a779cabb94ca6a3fc03d14152d739d9c935ab517b47b3a664a94c08c4141207` |
| [replace-password-key.json](replace-password-key.json) | `X.AnonymousClass144.A0B` | `ev_0f4f62393ec091c1ba85442aab9facf8e887c5bd8e5af62ae03a43c2319cfddd` |
| [search-auth-store.json](search-auth-store.json) | `search_android_classes` | `ev_f5367b77bbcc2d4a16bb92772f45cd436206308da26e88a11438757812ecafab` |
| [search-blacknoise.json](search-blacknoise.json) | `search_android_classes` | `ev_f1706a367add6b687d9ed69dfeee208c696fa82bb96cca5ca8651e7c401a20cf` |
| [search-keypair.json](search-keypair.json) | `search_android_classes` | `ev_d25912b8d7a5df10a60ab29f5d49b6b156b5d7cb89e8de00cbc90c38d8db01da` |
| [serialize-keypair.json](serialize-keypair.json) | `X.C1nq.A02` | `ev_18b7feedd685370bc6cc13e35349aba21f7445222f6edced8c9116c3953c6767` |

Independent checked DEX reproduction:

- [password-replace-checked.txt](password-replace-checked.txt), SHA-256 `2b3c173d9f40d8fd800dfabd7c366e3a0dac1aa0c920a2225c79bc67e7ab4304`;
  `xtask apk-smali APK --class 'LX/144;' --method A0B --signatures --exceptions`.
- [connection-load-checked.txt](connection-load-checked.txt), SHA-256 `fd5a72216165c4b69b89ffb01eaf6e57b6daf696f29d6226c609303074122f38`;
  `xtask apk-smali APK --class 'LX/1jv;' --method A0C --signatures --exceptions`.

No Rust ownership sidecar is claimed: these are prerequisites for a future
server-requested rotation lifecycle, not an implemented rotation capability.
