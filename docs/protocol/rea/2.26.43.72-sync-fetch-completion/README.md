# Fetch-result completion evidence

Static WhatsApp Android 2.26.43.72, APK SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No APK execution, download request or account operation occurred.
See [findings and partial port plan](../../app-state-fetch-completion.md).

Complete original REA envelopes are retained with raw results and limitations.
Parsed inventories do not establish code coverage. All method bodies below are
complete exports; Java is recovered code, not original source. F2T.A00 and
N2E.A04 include warning/control-flow limitations; checked DEX corroborates the
F2T parser. LP2 streaming behavior needs independent Rust fixtures before port.

| File | Method / class | Evidence ID | Representation |
| --- | --- | --- | --- |
| `authenticated-stream-class.json` | `X.LP2 (parsed members)` | `ev_fc741ff9bd74a6e70be0b215deefc6810721dd400d0303f744ff0c52cf01d72f` | inventory |
| `cipher.json` | `X.MTj.A02([B[BI)Ljavax/crypto/Cipher;` | `ev_0a8f7595e574bca414a7bcbdf7c096ce685d85fc59085435e6a40162a446424d` | java |
| `decrypt-stream-class.json` | `X.MuI (parsed members)` | `ev_4a986b1f9ed2944ba74f85440f8ea435ef6efd6dbfaef0c622641da9c67ae98d` | inventory |
| `decrypt-stream-open.json` | `X.MuI.CQc(LX/1gY;)Ljava/io/OutputStream;` | `ev_83a3fe1aeb32f7b0ac618318e78903d3e04dc215c5776dc5c4cc2cbbdd98a965` | java |
| `decrypt-stream-prepare.json` | `X.MuI.<init>(LX/0jS;LX/NbB;LX/MKa;Ljava/lang/String;Ljava/lang/String;Z)V` | `ev_2be0055bcf605cbce52cf1d459b4a35f19213834581ab47a18cf22cf9ac4615c` | java |
| `derive-key.json` | `X.Mta.AMt([B)LX/LvP;` | `ev_f48e1f415de6a3afdb90c9eefa9a53d0e7a10f0b031db69e98f74024a985fc16` | java |
| `download-base-class.json` | `X.LIp (parsed members)` | `ev_1ad03a2974373e340accabb8ca47d6ba0f3669d6211675666ef6cab96b9a3e67` | inventory |
| `download-class.json` | `X.C0lN (parsed members)` | `ev_28c61471f7a17db2fab09df0164748988f81d13456e30e9c86bdac07b691a1e4` | inventory |
| `download-extra-hook.json` | `X.LIp.A06(Ljava/io/File;)Ljava/lang/Integer;` | `ev_67501500c4f786f025f2e812781b96773f50e6bd575208e7203ad7a6cdf23707` | java |
| `download-job-class.json` | `X.N2E (parsed members)` | `ev_6d67dd9c719fc4c934dffd23f532c554a5318dbecb595d892bc3d2265d75f961` | inventory |
| `download-mutations.json` | `X.C0lN.A01(LX/Dg5;Ljava/lang/String;)LX/DXE;` | `ev_2c7e0496f0211ca0b0805abd5457c5ecc50b1e5a6686562725c4c3b289d89602` | java |
| `download-outcome.json` | `X.C0lN.A00(LX/Dxd;Ljava/lang/String;Z)V` | `ev_60a45dd3f6d9163539fc85696d8699cb411b210621226889d6322666bd0dd41a` | java |
| `download-prepare.json` | `X.Dxd.<init>(LX/0lN;LX/Dg5;Ljava/lang/String;)V` | `ev_106d1c40d41167eeb8fa42af97764ff7c7c3fa85d102acb931677da6f07da875` | java |
| `download-request.json` | `X.Dxd.A05()Ljava/lang/Object;` | `ev_589e120e1f9596ac0ddfd146811c49a8b4f7d6eb08e29331b3cdcb6e86d178aa` | java |
| `download-run.json` | `X.N2E.A04()LX/LqJ;` | `ev_924b6c5fdb2eff7bda9848066e47f76908fe5d3c5b152f8f04ee10d06fed0ded` | java |
| `download-snapshot.json` | `X.C0lN.A02(LX/Dg5;Ljava/lang/String;)LX/Dem;` | `ev_e11a68281b26c1d87e9f21d804f35c167c7b64e12d3237420353561fa2fc6f10` | java |
| `encrypted-transfer-class.json` | `X.MuB (parsed members)` | `ev_9c79efd64fe87330c7c0767edef0d5986d4453d6c7de6a5c0ef8f5b993011d23` | inventory |
| `hash-algorithm.json` | `X.Kls.A14()Ljava/security/MessageDigest;` | `ev_32d3f8549d50e3e18a23b7f6944df9adcb3d67aa3342ccc752b93db3d7d536c0` | java |
| `hash-check.json` | `X.MXA.A00(Ljava/lang/String;Ljava/security/MessageDigest;)I` | `ev_bc2253d545a98fd6902178cb2de34c0c512c737536eb65f9534c7cc0f344752c` | java |
| `hmac.json` | `X.DBh.A0k([B)Ljavax/crypto/Mac;` | `ev_9a9380f19408eca4bdebf823160f776f5d6bf554cb7948e6d4c1ef8b3e434d8b` | java |
| `kdf-class.json` | `X.C1oO (parsed members)` | `ev_ac6e116208adc57a9ff7c5cd5b5a798b5b0ba8a10e6ca616437b48abbb6a371b` | inventory |
| `kdf-expand.json` | `X.C1oO.A01([B[BI)[B` | `ev_91c82667e8d17fac5adf09c47988d3f99179190310246ad943deb632b5f590a2` | java |
| `kdf-extract-expand.json` | `X.C1oO.A00([B[BI)[B` | `ev_ef659668243cfd19e0989264d492ac5d926366b3ed9bd21aea7c9a49af023723` | java |
| `kdf-extract.json` | `X.C1oP.A00([B[B)[B` | `ev_e4c3dde7bc208e5601028092d2b2914f20ee82f09d22cae0b5c0ea9a5a7d273e` | java |
| `kdf-salted.json` | `X.C1oO.A02([B[B[BI)[B` | `ev_1c693dc119cd3a6870aacfe6214b54486f4343e27fdd22d1291abc9ae41d89ba` | java |
| `key-provider-class.json` | `X.Mta (parsed members)` | `ev_1f2124c9e7a14c7e524560d58e1ebfa2a62e2551a62a6eed88480f41f9a04c22` | inventory |
| `mac.json` | `X.MTj.A03([B[B)Ljavax/crypto/Mac;` | `ev_118d3389e5a0fdc13ff3e98ea31c8eb1bc7578bfdfeb9179d337d2fbf49727ea` | java |
| `response-body.json` | `X.F2T.A02(LX/0Tp;Ljava/lang/String;)V` | `ev_0b7dfe5a3f98b3fc723298bd3f798e41d4bf33a7acea3fd0cea9948b44d294ec` | java |
| `response-class.json` | `X.F2T (parsed members)` | `ev_db108df00391e576e2f16c32db8484abf641a35d596aca07c3a5b2ea477bba41` | inventory |
| `response-data.json` | `X.F2T.A00(LX/0Tp;)LX/EUl;` | `ev_52313ad1fcaf278f73c1ea4195b8b5d7ff94f8dc7ad507b0f0a6bfc1d012b7fb` | java |
| `response-empty-list.json` | `X.F2T.<clinit>()V` | `ev_a6d413c4d8abe9d1e2fc2b2ad808ec3a78293ddd7f88bd82bbffb506418a452b` | smali |
| `response-name.json` | `X.F2T.A01(LX/0Tp;)Ljava/lang/String;` | `ev_a60b12089f82b5f2b77b82773e689f07dc9c394621826209543f594fe8dbe8ca` | java |
| `response-parse.json` | `X.F2T.<init>(LX/0Tp;)V` | `ev_1a84773304ea019fe73a97ebf078ddbd3e1e7760dbb2d65285296b4e16d64c3d` | java |
| `split-key.json` | `X.MTj.A01([B)LX/LvP;` | `ev_6e7f944fde74e5c2ce27e58a13d6e26cc98c7ef04e802e8a194f76fd3818ac32` | java |
| `stream-close.json` | `X.LP2.close()V` | `ev_6f26ec8a74e8bcb93b2dcc6c1f1617ab14646115a49a32b0d5117b4f6d7b3acd` | java |
| `stream-prepare.json` | `X.LP2.<init>(LX/LvP;Ljava/io/OutputStream;J)V` | `ev_a85d726901e081c964f1581e16c5859c91c2bfb65db55490e2379f69b5349bae` | java |
| `stream-write.json` | `X.LP2.write([BII)V` | `ev_31e10d73369b94d32f1d951d1fcd11fe9beea6198739df5851394d4ff16139da` | java |
| `transfer-integrity.json` | `X.MuB.A00(LX/MuB;)Ljava/lang/Integer;` | `ev_084e72e93ace4bd36c8e7b8a5c27476cda74ed07fefb18203148d6a17336a453` | java |
| `transfer-prepare.json` | `X.MuB.<init>(LX/08Y;LX/08q;LX/0Ze;LX/K1R;LX/0lO;LX/0VM;LX/MOq;LX/LzD;LX/MOv;LX/NaF;Z)V` | `ev_4b9d907dd107cd2f9b5cdad56c8a7bbc361b7c6bf7f5f1b2e74dcd5f2c69fd3b` | java |
| `transfer-request-class.json` | `X.LzD (parsed members)` | `ev_a871d760ab3dd36c05e39ad451ea2c38125106e9388179c5e77e0816bc3321b1` | inventory |
| `transfer-request.json` | `X.LzD.<init>(LX/0jS;LX/NYv;LX/NbB;LX/MKa;Ljava/lang/Long;Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;Lkotlin/jvm/functions/Function1;ZZ)V` | `ev_f39ee3579c178ca6c7e3a4b0a440ec850e3e156eed29059dab944be6a210fab7` | java |
| `transfer-run.json` | `X.MuB.AOW()LX/LqJ;` | `ev_22a9cad18756df0838eef65817ad34c895e50dfb83cc7e3d0d5f8f8cd8ddca77` | java |
| `error-code-attribute.json` | `X.C0Tp.A05(Ljava/lang/String;I)I` | `ev_0284e0911087591771667deae4f0a1262470536763e259dc117357a2c4c94633` | java |

## Reused committed evidence

No duplicate exports were requested for `FUc.CK4/Bvj`,
`SyncResponseHandler.A04/A07/A09/A01/A02/A08`, `0mj.A00`, `0l8.A06/A08`
or `FeN.run`: see [the lifecycle bundle](../2.26.43.72-server-sync-lifecycle/README.md).
Its complete checked `server-sync-result-worker.txt` has discriminator40 at
`0c98..1418`; the incoming-apply transaction is in `server-sync-response.txt`.
The `A01` checked DEX is authoritative where recovered Java has an empty branch.
The [trigger/flight bundle](../2.26.43.72-sync-trigger-flight/README.md) retains
request context detach and timeout details.

## Checked bytecode

Generated with the existing `xtask apk-smali` against the exact APK, with
`--signatures --exceptions`. Source selectors:

- `download-checked.txt`: `LX/Dxd;`, all methods.
- `download-base-checked.txt`: `LX/LIp;`, all methods.
- `response-checked.txt`: `LX/F2T;`, all methods.
- `media-domains-checked.txt`: `LX/0jS;`, `<clinit>`.
- `media-domain-constructor-checked.txt`: `LX/0jS;`, `<init>`.

Offsets are method-relative bytes, not DEX code units. These are annotated
structural disassemblies, not compilable smali. Their SHA-256 values:

- `download-base-checked.txt`: `6f8e626aff863892a4dcca38343d45d01d53550713f2b3c23b874bfb1f3d6616`.
- `download-checked.txt`: `1337c35eb69c7d8473e57eef095fe428c09365460d2af5de79c9c0e7c2c5395f`.
- `media-domain-constructor-checked.txt`: `ee4bf7e68d368b942c961e99e4f874debde1e72b99200453a90bb78d3bd4d78a`.
- `media-domains-checked.txt`: `2b7bb90d1caee46c3bfab201a98c101e6eeeaa2e994b31aeede4006cd036d784`.
- `response-checked.txt`: `0092cab1614bdd434a574abf24434dd7fd9ebec37e0adbccf5d586fd77d4eb25`.

Four explicit ownership entries bind `response-parse.json` (category precedence
only), `response-name.json`, `response-body.json` and
`error-code-attribute.json` to `transport/app_state_sync/response.rs`. Their
scope is the supported structural classifier only: opaque data is not protobuf
validated, downloaded, MAC verified or promoted. In particular `F2T.A00` remains
unowned because its full protobuf/error semantics are not ported. Ownership
records are source citations, not method-completion or interoperability counts.
