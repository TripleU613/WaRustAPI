# Current account privacy refresh evidence

Target: WhatsApp Android **2.26.43.72**, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Static only. Findings, implementation boundary and next selectors:
[account-privacy-refresh.md](../../account-privacy-refresh.md).
The existing incoming `19N.A09` envelope and checked first-child/modify branch
are retained in the sibling
[dispatch evidence](../2.26.43.72-notification-sync-obligations/README.md),
not copied here. No Rust behavior or wire ACK was added.

## Canonical REA envelopes

The JSON records below preserve original source, method/class selectors,
provider identity, engine markers and limitations. A complete source record
is not proof of every runtime branch or semantic correctness of Java recovery.
Two large synthetic dispatchers fall back to complete smali; only the documented
privacy discriminators were interpreted. Unrelated branches remain untraced.
The ownership sidecar maps only category admission/adoption records to the bounded readreceipts notification port. Refresh selection, other categories and the full manager remain unported.

- `category-validator.json`: `ev_f94bd3345710b2912f69fd153bd1c22b5e8b99b357ce11d853dc051d269b6a01`; `X.C8AX.A04`; `(Ljava/lang/String;IZZ)Z`; java source complete.
- `consumer-members.json`: `ev_9fe41160e7aaf3dae916adc6d86d7dae10ac41ac559384ab4da0877816f9f456`; `X.C6pB`.
- `mex-feature-list.json`: `ev_056b8ef6e30bbee6bd99aaed1a558b382bdde0c5f0e80739cbae994f804d22ab`; `com.whatsapp.privacy.MexPrivacySettingsHandler.<clinit>`; `()V`; smali source complete.
- `mex-feature-selection.json`: `ev_fad7f6a44587d9d6bfe855a32501d0828699c12bf18e60e65c68e8c5ab5aaa75`; `com.whatsapp.privacy.MexPrivacySettingsHandler.A02`; `(LX/0cf;)Ljava/lang/Object;`; java source complete.
- `mex-launch-coroutine.json`: `ev_8c91028ce2cd1855957896db69feb48e6fec98de18bd0ce5eb4ce174f3044d5d`; `X.C8io.invokeSuspend`; `(Ljava/lang/Object;)Ljava/lang/Object;`; java source complete.
- `mex-launch-members.json`: `ev_d13a7e8c2930acb40b46a368ba9f7a291312dcb7444874f4eff2e3c94c91d0fe`; `X.C8io`.
- `mex-members.json`: `ev_06cadc52893f10cb9f211640979690a83b84b81ae16d9881512002e9910230d6`; `com.whatsapp.privacy.MexPrivacySettingsHandler`.
- `mex-reply-consumer.json`: `ev_11c6a8fc3afa7cd86f6c92fb0ba5e1c70f572d9f35e24d0dc1462b1d92ea9659`; `com.whatsapp.privacy.MexPrivacySettingsHandler.A01`; `(LX/6pB;LX/0cf;)Ljava/lang/Object;`; java source complete.
- `mex-request-context.json`: `ev_8e9671091e533d485298a55bd7ee94608df96b8b576b6e11a7c49c75e01535cf`; `com.whatsapp.privacy.MexPrivacySettingsHandler.A00`; `(Lcom/whatsapp/privacy/MexPrivacySettingsHandler;Ljava/util/List;LX/0cf;)Ljava/lang/Object;`; java source complete.
- `mex-request-shadow-coroutine.json`: `ev_bdde8a76f58e40a44bef4640f63a398da08e883dd24ea09c713b166ee9e58db5`; `X.C8ir.invokeSuspend`; `(Ljava/lang/Object;)Ljava/lang/Object;`; smali source complete.
- `privacy-map-adoption.json`: `ev_31b02933f04201b47e2debccb4de7c4e4c6ae9bfccfa5a8eff03d73b65285e5c`; `X.C0oe.A0R`; `(Ljava/util/Map;)V`; java source complete.
- `privacy-value-boolean.json`: `ev_88268c9766914b533ff8364759d79ecfd8149ce93c157811f3d6cece78a6e2d6`; `X.C8AX.A03`; `(Ljava/lang/String;)Z`; java source complete.
- `privacy-value-code.json`: `ev_1331422a765ac3ca85950a0bac2f996de3009a95affe137a0b4476406fd41d5f`; `X.C8AX.A00`; `(Ljava/lang/String;)I`; java source complete.
- `refresh-admission.json`: `ev_0b150ace0d5e5181730b1722d655e8ae19177fd15c3d82e310c02bb9ef82d5b2`; `X.C1sv.A00`; `(LX/1sv;IJZ)Z`; java source complete.
- `refresh-completion.json`: `ev_902e18261f738e382bd5fde11cf897fcc83998cbc6d3b87e1ef78edfc041b06f`; `X.AnonymousClass278.A00`; `(I)V`; java source complete.
- `refresh-error-consumer.json`: `ev_c5e705ab2bd9c3e3fc5f0f95d74c8c7b1d34451e36b8ae8e7ad086c775680abf`; `X.C6pB.A00`; `(Ljava/lang/Integer;)V`; java source complete.
- `refresh-manager.json`: `ev_02b7acf93f40485579545932d69c0ad3f264bb17412c940795359bcd66380ccc`; `X.C1sv.A01`; `(LX/1sx;ZZZ)V`; java source complete.
- `refresh-protocol-selector.json`: `ev_217c243d72c8d029c40d0e9e98228f2d5ada8779986d77250708ebc50c6933b4`; `X.C0oe.A0M`; `(LX/278;)V`; java source complete.
- `refresh-success-consumer.json`: `ev_3da5f8b16606bb759e81528b476f6972132035abf85ecae68a9a54a2465ac65d`; `X.C6pB.A01`; `(Ljava/util/Map;)V`; java source complete.
- `request-flags-builder.json`: `ev_dfb191a9412c92fe0fb6f4d5b3b34b2ffb57c0444f990532a95d7cab7611c5e4`; `X.C2D6.A00`; `()LX/1sx;`; java source complete.
- `request-flags-constructor.json`: `ev_44b4ccef964b68fb291369699f21bfa39bea1818036abf732684dd4d80cc3b61`; `X.C1sx.<init>`; `(ZZZZZZZZZ)V`; java source complete.
- `request-members.json`: `ev_6de4d70660cfebb6146ee6721b23eaca75b79d9a4c2b1c937a5f5512c3253aec`; `X.C1sx`.
- `smax-callback-members.json`: `ev_81d3ef6041508e5374b4b4e55480de6a6a6df26e7b4748ecf93dc52e38724325`; `X.C8R5`.
- `smax-category-converter.json`: `ev_ebccf574cc6b934cc36133f8eede570c1a56f9ea2f888df66e4c400c45223f4a`; `X.C8db.invoke`; `(Ljava/lang/Object;)Ljava/lang/Object;`; java source complete.
- `smax-category-index.json`: `ev_957cecfce2e67d9915a58312d5bf20e72ce06ccfad19aad51d14ee8f5c2371c3`; `X.C8dV.invoke`; `(Ljava/lang/Object;)Ljava/lang/Object;`; smali source complete.
- `smax-delivery-failure.json`: `ev_8a996dd45764caf98c1361771d0af837ad8c7f88a96847fb1e331253204f7afe`; `X.C8R5.BsS`; `(Ljava/lang/String;)V`; java source complete.
- `smax-error.json`: `ev_00d8b4863360c83142e1ab2dd35d5d90c2df653deed71ffc2e3eb6a755d38b2c`; `X.C8R5.Bvj`; `(LX/0Tp;Ljava/lang/String;)V`; java source complete.
- `smax-request.json`: `ev_c6873dbbef21370f6112fd73625b116699b0884216c5f7efc7bfbc22add23ee5`; `X.C0oe.A02`; `(LX/278;LX/0iX;Z)V`; java source complete.
- `smax-success.json`: `ev_0a3f44335e51f3311f34ca3fd77c36d8a5c92ccd1e4e73b7d634e69e08d9541a`; `X.C8R5.CK4`; `(LX/0Tp;Ljava/lang/String;)V`; java source complete.

## Checked DEX exports

These are annotated instruction streams, not compilable smali. Generated with
`target/debug/xtask apk-smali <current-apk> --class '<exact descriptor>'`
plus `--signatures --exceptions`; method filters below. Offsets are method-relative
bytes; branch operands use DEX code units. Checked exports resolve important
Java-display loop/return ambiguities.

- `privacy-map-adoption-dex.txt`: class `LX/0oe;`, `--method A0R`; SHA-256 `cc6e3659db1bdf70253bd3032059d4319bcc4412cf3767a7703a3c7d5502c970`.
- `refresh-manager-dex.txt`: class `LX/1sv;`, `--method A01`; SHA-256 `5ffa8ee91630ea77f5a5d80e1dc9a0725fe3b61e0014533d789b1d70535fd292`.
- `refresh-protocol-selector-dex.txt`: class `LX/0oe;`, `--method A0M`; SHA-256 `eb3c49c92f4b68496b761e976aba2e30d0bf0808164853252ec77b02329a5b51`.
- `smax-callback-dex.txt`: class `LX/8R5;`, all five declared methods; SHA-256 `3dd159a948c72d46728f89c7e7a170f256b67f47b43ff8e23af717ec27277a91`.

## Negative results and exact continuation

- `X.C2D6.A05` is not a method; parsed membership contains constructor and
  `A00` only. `A05` is the Boolean builder field. Its `A00` and `1sx.<init>`
  establish the privacy flag mapping without guessing a missing callable.
- The incorrect guessed package `com.whatsapp.privacy.protocol.xmpp` did not
  resolve. The existing full class inventory resolves the exact class
  `com.whatsapp.privacy.MexPrivacySettingsHandler`, whose records are retained.
- Current SMAX `8R5.Bvj` returns after the first present code, contradicting
  Java's displayed extra fallthrough callback; current `0oe.A0R` preserves a
  conflicting pending requested value, contradicting displayed fallthrough.
- No effective property 23463 mapping exists in Rust. Encrypted action replies
  may still retain raw returned values; selected-property inspection is not
  configuration precedence/application. No new inspection field was added.
- Next exact selectors: configuration resolver `00E.A0a` for 23463 and its
  precedence; MEX query model `74K`, input `FxW.A0D`, request `8ir.invokeSuspend`
  discriminator 6, converters `83S.A00/A01`; conditional `1az.A05/A0C` for
  STATUSVIEW; `1sv`/`278` retry/dirty completion scheduler; `0oe.A0Q` observers.
  These are unresolved work, not claims that their full behavior is absent.

No device, live request, effective private-account property or privacy setting
was observed. No secret-bearing account data is included.
