# Current read-self authority and writer evidence

WhatsApp Android **2.26.43.72**, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Static only; no Rust receipt behavior changes in this evidence chunk.
[Findings and next authority boundary](../../read-self-authority-current.md).
Ownership stays empty because no read-self capability is implemented.
Earlier job selection and own namespace views stay in
[read policy](../2.26.43.72-read-receipt-policy/README.md).

## Canonical REA envelopes

- `add-own-device-callers.json`: `ev_2c26098a31ef9d3b47c5e6219b001a005a8a033d99cf96db53df0fa39fdc3791`; `X.C1nG.A0F`.
- `add-own-device.json`: `ev_844170c8caa5cae8479909599d212143f86fb2ff14f7614473531356385141fd`; `X.C1nG.A0F`; java source complete.
- `device-metadata-constructor.json`: `ev_d7e9d192aa682881abf02f2f2649746452efa5e9a79ad5f6ea39dac45673b746`; `X.F0I.<init>`; java source complete.
- `logout-positive-helper.json`: `ev_5f8bae8c2019b303c9d465a6f45622a266daf51153ce7a3b4ecd46660fcd5f53`; `X.C2L9.A1N`; java source complete.
- `me-assignment.json`: `ev_f169ff8cc6bb1b5ce844a03853933bc54b84672a6ce15f4b6c099816841b956a`; `X.C09O.A00`; java source complete.
- `me-forwarder-rea.json`: `ev_6ece6153dcb8431303c3f6bcc55d9105fe4510b3296308048b334f728c6240cb`; `X.C09G.BUx`; java source complete.
- `me-gate.json`: `ev_6feb71a582acaab10968c505549d71b1a8ae6a42d75fc6a6ca091c158bf3e390`; `X.C09O.BUx`; java source complete.
- `me-load-gate.json`: `ev_935ba7124d79fc1699dfeff26c248fe457fe089bfb01347df1b4262f1f2089e8`; `X.C09O.A07`; java source complete.
- `me-load-worker.json`: `ev_cdac7b5f25498154f9e33b311ad48e0f8fbe517e2e0e6306cab0d02960a99360`; `X.C1Wk.call`; smali source complete.
- `me-loader-members.json`: `ev_432d64a7a37b0cf18c1add17ac87ea5a584350154e99bb3da4c78168487203fc`; `X.C1Wk.`.
- `me-members.json`: `ev_1bb38a7c20ce5d952a7eedeaa265a986718312e4030463e7ca2ddcc68633de8d`; `X.C09O.`.
- `metadata-add-worker.json`: `ev_8220244f1dc6dc1826f0e16148953a683eab74e2a796f10c61ad3b85f5a2ad02`; `X.Fei.run`; smali source complete.
- `own-device-loader.json`: `ev_52e917d541c6255ac6a47359c8d3455d106204f83ef36505efdaf880ebcfaa54`; `X.C1nH.A07`; java source complete.
- `own-device-query.json`: `ev_332911d58b225a84ec7710225519b15bf7e958e4791db3fbe1cfd60214c59358`; `X.C1nN.<clinit>`; smali source complete.
- `pairing-add-worker.json`: `ev_0b709602f94ec6fbde014ea004bad763d469d0a9fe0191335ffe789a2bc2d6ee`; `X.FeN.run`; java source complete.
- `receipt-extra-ids.json`: `ev_af792a10e37e6c5fd29726ea3b4e1511b313e515cdbc38595eb1e9cfbc4a1481`; `X.F3e.A0A`; java source complete.
- `receipt-meta-builder.json`: `ev_6c87e89acdd7cf1e0cdad04dc75c5c47c8a1ce427595aca207036cfa14810288`; `X.Eqi.A00`; java source complete.
- `receipt-meta-set-helper.json`: `ev_e1207763f611db464cac58f0599eed1b33893a52a82fb728c8f519bbe28fc1ac`; `X.C6oe.A1b`; java source complete.
- `receipt-meta-type-sets.json`: `ev_a00d3bcfd376aab1e515d3d44aa532fba0561006fb416eb983d9f2f251fbc496`; `X.Eqi.<clinit>`; smali source complete.
- `receipt-model-constructor.json`: `ev_c6693fdda06a4bce21468cbcbb7cab6c14ef07a1b7b9390809b877a12649ff0f`; `X.EXu.<init>`; java source complete.
- `receipt-model-members.json`: `ev_c1057cc06fb1ef8615eb4199b7f5c9eb3bd08a8b88ba1df6ff2f2822c44cd9f6`; `X.EXu.`.
- `receipt-model-references.json`: `ev_1410a2efb084900f4b35e5c67261b6afc6a457f338b99c722b68e1b6af6fe9bd`; `X.EXu.`.
- `receipt-writer-adapter.json`: `ev_4f06fc6a1caa7037cdae743813723e96983c84e742bc09f751c529fcce8fd8a1`; `X.F3e.A07`; java source complete.
- `receipt-writer-builder.json`: `ev_d7b93f2e3fafd262b6263c7de23a9b6ea5159a6f141fa264226ef173106c525c`; `X.F3e.A00`; java source complete.
- `receipt-writer-members.json`: `ev_5e71ed7fd1125ea06c697b929bcd0fe249d595d33a1f31b78b84092169ca38d7`; `X.F3e.`.
- `remove-device-store.json`: `ev_86380a7c6f79826bee1d43b111af2c9039cec1a157340100e659c76df3b31f1a`; `X.C1nH.A01`; java source complete.
- `remove-own-devices.json`: `ev_8ac7bdf5284f7c787454a90180fa7a77e2907b41f9936cdf3aac98222a643437`; `X.C1nG.A00`; java source complete.
- `route-group-classifier.json`: `ev_ab64e51334d1389a5a8a87ab89905424bc819d05225ba7f1d257785b6e350ead`; `X.C0Dn.A0m`; java source complete.
- `route-special-classifier.json`: `ev_a477221d28a65b275ca4769c3c7ab315e03f325d187159cd7cef945d125c7e53`; `X.C0Dn.A0P`; java source complete.

- `phone-jid-type.json`: `ev_0f0d6a4d6a6df414785730e46d23673709b8f16d15d85acc0cd0104aff69b067`; `com.whatsapp.infra.core.jid.PhoneUserJid.getType`; java source complete.

- `logical-jid-type.json`: `ev_6f9777270f55f633528dd71a6c4abf043db29c8abddd38927b7fe2009323cd10`; `X.C0Dx.getType`; java source complete.

## Checked structural exports

`apk-smali <current-apk> --class <exact-descriptor> --method <exact-name>
--signatures --exceptions` was used for09G.BUx,1nH.A07,F0I.<init>,Exf.A05,
F3e.A07 andF3e.A00. These are annotated instruction streams, not compilable
smali. Field inventory uses`apk-field-refs --class LX/1nH; --field A01
--type Lcom/google/common/collect/ImmutableMap;`.

- `device-metadata-fields-dex.txt`: SHA-256 `f4796e3e7f39ff8721130bf828c0730ce21d6c6b48500805119f063a2941e3f5`.
- `me-forwarder-dex.txt`: SHA-256 `be8fbedddadb409fec1fde38cd12a3df39673021b8eb2851b4e75d8e20d3e487`.
- `own-device-loader-dex.txt`: SHA-256 `8c138a45af348294be49520c6f318b973df20b584055b22dc3ff6d2c937ba648`.
- `own-map-field-references.txt`: SHA-256 `e93198efc195f90ae28b77b8690761a081e2dfc6ccefca89fe2eee312648613e`.
- `receipt-route-dex.txt`: SHA-256 `4e3cc5148ca3fecce731b30a810ae7dbc65e5bf454d2af541dbf12a0ad1bfd87`.
- `receipt-writer-adapter-dex.txt`: SHA-256 `594464fb14a5cf4cfebdd93da90aea52b73547966092177ca876d817a7e795c3`.
- `receipt-writer-builder-dex.txt`: SHA-256 `804f09526237df22cf244c2fe847ba35865b640de4c0b3945ca3f182750dad7f`.

The existing full current writer dispatch is reused, not re-decompiled:
`../2.26.43.72-server-pong/writer-dispatch-checked.txt`, method0v0.A00,
switch00f0 case89/419 target1f18; F3e.A07 invocation1f50..1f56 and write
1rj.A04 at2006..2010. Full source and evidence envelope remain in that bundle.

Negative selectors:09G.BUx displays no provider body despite a concrete DEX
forwarder. 1Wk is Callable.call, not Runnable.run or Function.invoke; those
failed selectors did not establish missing code. Fei's full method is smali
fallback; only discriminator43 /target5bc is interpreted. FeN's full Java
method has recovery warnings; only discriminator14's add-device call is used.
Unrelated branches are retained but remain uninterpreted.

Complete own-state ingress, pairing cryptographic authority, cache callbacks,
remaining field writers and runtime account state remain unknown. The11 field
references are structural inventory, not proof of every dynamic write.
