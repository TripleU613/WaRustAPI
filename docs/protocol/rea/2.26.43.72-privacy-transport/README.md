# Current privacy transport continuation

WhatsApp Android **2.26.43.72**, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Static only. [Findings and supported runtime branch](../../account-privacy-notifications.md).
Earlier request flags, category validators and pending-map semantics stay in
[privacy refresh](../2.26.43.72-account-privacy-refresh/README.md) and
[MEX/configuration](../2.26.43.72-privacy-mex-config/README.md).
These new records close transport/DI questions; they are not a runtime refresh
port. Their ownership sidecar remains empty. The supported non-modify category
port cites the earlier19N/0oe/8AX records.

## REA records

- `01-X-C2LE-A0L.json`: `ev_33b8d61f2f83ece883ea0fba968c7751d8e06e470e9a0857afe013db77074b44`; `X.C2LE.A0L`; java source complete.
- `02-X-C0iT-getCallName.json`: `ev_da8444c8d28031196f0d01d1b430e5a79684eedb6cac382fdd51bd6c5a27c338`; `X.C0iT.getCallName`; java source complete.
- `03-X-C0iT-getResolvedBuildConfigName.json`: `ev_13f12366b5130f427f76b10b33288385c2a9ef3b010d4961bbfc5b3b9a14c64e`; `X.C0iT.getResolvedBuildConfigName`; java source complete.
- `04-X-C0iT-getQueryParams.json`: `ev_4e9821a4bd916c1326cf262ab3ffbecfb5517ff68648fedf19c69a7b5e54db2d`; `X.C0iT.getQueryParams`; java source complete.
- `05-X-C0ig-A01.json`: `ev_14175006bf9a404f43c706a77b55d1e19eb0bbf2bc4a0f8d8541ece1158412b6`; `X.C0ig.A01`; java source complete.
- `06-X-C2LE-A0Q.json`: `ev_790acfc5d77dafb5106d7584529da10c87c01d553bbee61be8c758b132c5e8b6`; `X.C2LE.A0Q`; java source complete.
- `07-X-C0iq-ARv.json`: `ev_2a42e08409fb054ca06316616bde9ef97f6899d482ee51a45b85bb2cdf61cf98`; `X.C0iq.ARv`; java source complete.
- `08-X-C0iq-AQV.json`: `ev_451b0e55438bbe2f82c1185583a8639091c436f60e3a62b6828ca41b0079dd74`; `X.C0iq.AQV`; java source complete.
- `09-X-C0iq-A00.json`: `ev_846b6856f03596d2403c97de7d5ee91705791d216188b157e98ed5320068e5c7`; `X.C0iq.A00`; java source complete.
- `10-X-C0iq-init.json`: `ev_78ccd20ab6e8fd54422d68c1d56cefadbc1ed8c17297b8fce24bdc28ac73d4c8`; `X.C0iq.<init>`; java source complete.
- `11-X-C0iq-AQY.json`: `ev_844b7e3f45b193f711b70afd963270e474b80a3159cd1a763efc6aac95655b2a`; `X.C0iq.AQY`; java source complete.
- `12-X-C0j3-init.json`: `ev_73ebd33dac4a441e2e2cfd72db4733afd0fbb03c6719121983571d5aaa1dfe66`; `X.C0j3.<init>`; java source complete.
- `13-X-C0j2-A04.json`: `ev_0d81685799e66e8205a35f995371d518fc5498183f0ee8dc66aefff887913226`; `X.C0j2.A04`; java source complete.
- `14-X-C0j2-A03.json`: `ev_5959e480afbd611108706b819604d04e2278c4b21d86e09a6f144574a129df73`; `X.C0j2.A03`; java source complete.
- `15-X-C0mF-init.json`: `ev_6f903cc1efba2c12557ab7a1a13a5f362f6e0554b99d501249f66387351806d4`; `X.C0mF.<init>`; java source complete.
- `16-X-C0ih-A00.json`: `ev_0ac47aeb17d29c9f3632dfb0a5160c886f134adc49abb97715cb9f4cfbce03b3`; `X.C0ih.A00`; java source complete.
- `17-X-C0j3-A06.json`: `ev_376bcf84be66988437a34d2f8c9b4e6be15634e0736f5f2b3a210c7da5dfa5e1`; `X.C0j3.A06`; java source complete.
- `18-X-C0mK-init.json`: `ev_7ee09dfbf918dbcfc3ea101b33903e20de6e66d9190093a000543ba6b200932e`; `X.C0mK.<init>`; java source complete.
- `19-X-C0ik-A00.json`: `ev_3d1e86d222ec668a432a8df99f0c74382e49701b37367f052dfcc12a7b953b0a`; `X.C0ik.A00`; java source complete.
- `20-X-C0io-A02.json`: `ev_b3a285db9e8fe16b2980272d66889d9003e5545b2014f62b235e6681eb73ca3a`; `X.C0io.A02`; smali source complete.
- `21-X-C0mb-CK4.json`: `ev_a28483c7d9798d74e84195337cdc9965674907ab691026c5a90defc11c36eb9e`; `X.C0mb.CK4`; java source complete.
- `22-X-C0mI-init.json`: `ev_11ab239e98d4aefe28b0b081d5d4c2424900184e2886456b68b1c55341eda811`; `X.C0mI.<init>`; java source complete.
- `23-X-C0mO-clinit.json`: `ev_b197ab5d8acc9b2e2e549145eb3d683b27cc9202016e376b236cbd6578f215dd`; `X.C0mO.<clinit>`; smali source complete.
- `24-X-C0mb-init.json`: `ev_a17144d7f8a76399dca89eb154f18b4e337d707bc7239a6e900af0a0b32a96f9`; `X.C0mb.<init>`; java source complete.
- `25-X-C0mZ-BKn.json`: `ev_0f2f08590d17ac52bc8674cfbab50d709089379d546bfc4fcbb3b833ea7b8996`; `X.C0mZ.BKn`; java source complete.
- `26-X-C0mZ-BKo.json`: `ev_0f64189b43ca9d04b1da5eac7a8025e1dc67af8f0a2d57418fb6f6fc55d6314f`; `X.C0mZ.BKo`; java source complete.
- `27-X-C0ik-A01.json`: `ev_8c91b415cf2bfe22ca38c62001b7446d1ec9bc81d60964cc6038efb0757b298d`; `X.C0ik.A01`; java source complete.
- `28-X-C0md-CK4.json`: `ev_f9919fc97cc94b8c1dee5497cb6562c1faf997260cb3983f22297a84e03964fb`; `X.C0md.CK4`; java source complete.
- `29-X-C0is-A03.json`: `ev_a1a39d27d71ac4f8f67508e64d9f56c9f4b7a8e3c719e422d928a716db6254c8`; `X.C0is.A03`; java source complete.
- `30-X-C0iT-getTreeModelType.json`: `ev_79748a1c9d991c16db601d5ed3dd19ddad70ba8d71f1e17e8f0c7c10a4a8b50c`; `X.C0iT.getTreeModelType`; java source complete.
- `31-X-C0ig-init.json`: `ev_897d0c1d071758a620c193d4f8b6226e444f8ae545322aaae68256774cbdb6b4`; `X.C0ig.<init>`; java source complete.
- `32-X-C0md-A00.json`: `ev_eebf91228c29bc7318c98bec5a2ed49e8edf137d30b670ad3ce6c34b7d0a23a7`; `X.C0md.A00`; java source complete.
- `33-X-C0is-A04.json`: `ev_118ae79844f09f7b9f941fd9784cf99d4db7a4754a193b613d79d213387df2e2`; `X.C0is.A04`; java source complete.
- `34-X-C0is-A01.json`: `ev_e2467ad7c16082b441341612b6fe68c55f363ec441ecc7a62199b1ecde2de37e`; `X.C0is.A01`; java source complete.
- `35-X-C1Wy-invoke.json`: `ev_0ec26d3ca3a560cd6d4d59f71d25b88c637c5d9ef63435e8f6ef36b989bc92a8`; `X.C1Wy.invoke`; java source complete.
- `36-X-C0ii-A00.json`: `ev_57f6cd2f817fd9ee12ef39cae2fc1e79832a260fdf513a0226344149665947db`; `X.C0ii.A00`; java source complete.
- `37-X-C0ij-clinit.json`: `ev_a314f7f761e81d8086c351ec36ba1c1daba6693ceccc4a444566eba9e21332be`; `X.C0ij.<clinit>`; smali source complete.
- `38-X-C7Ou-init.json`: `ev_710042cea3e19746612d592ef773e2be4213b7a9bd76e4c46a0dc2b9490641b5`; `X.C7Ou.<init>`; java source complete.
- `39-X-C7jy-A00.json`: `ev_855fd4df2d56d36bbc723284e02dcb217ed051b041b0d4ae4bc5c447eec5d6e7`; `X.C7jy.A00`; java source complete.
- `40-X-C0is-A00.json`: `ev_9950beb89a04b967a9ed4746c7528b7e3a1a46ad8bb565811eacccc1286bb36b`; `X.C0is.A00`; java source complete.
- `41-X-C0im-CRh.json`: `ev_9367b4a9c224fec466f4649d94efd2c76fc72d867bcfb10a73d0d0d32edfe25a`; `X.C0im.CRh`; java source complete.
- `42-X-C0im-init.json`: `ev_8bd0bddf701006ac7b37dd63da49b5afcc068171ad02bb7930c92b4e9ec5680c`; `X.C0im.<init>`; java source complete.
- `43-X-C0is-A02.json`: `ev_d372f6a87b8b9ba5ff3a69c0097c5ce0100918625355f47b7d6299df9d4ecfa0`; `X.C0is.A02`; java source complete.
- `iq-success-model.json`: `ev_b0c1fea77a647ce3bcc01cc1d8078775102d5df6ab089cbf5b027f6af5fa390d`; `X.C7Ou.<init>`; java source complete.
- `namespace-registration-callers.json`: `ev_a94d72c21a090c7e4e9de22cc235b19b05dc35f4022faa51b37a082ee77822eb`; `X.C0ik.A01`.

## Checked exports and limits

`apk-smali <current-apk> --class LX/1Uw; --method A00/A01
--signatures --exceptions` produces the DI router and case66 binding.
Namespace static initialization uses class LX/0ik; method <clinit>.
These are annotated DEX instruction streams, not compilable smali.

- `di-case66-dex.txt`: SHA-256 `7c697c7e3e1c45ccef430c1ca605953a5a17fc0bab323e1a2e42bbe9d00df77e`.
- `di-router-dex.txt`: SHA-256 `f852095f305051f8344d087a0efeb3f6742c3538d9cdde11fe11f74e2f7dda7e`.
- `namespace-init-dex.txt`: SHA-256 `8c1c9f4f5b9ed19f6f6d671658dcf2d630018f1d98cd4165dce46a2897d3c626`.

Negative selectors: 0j3.A04 is inherited from0j2, not a declared method;
0mZ is a typed callback and has noCK4 (0mb/0md ownCK4). T4f has noCtT;
0is.A02 invokesCtW. Constructor7Ou overload0 is an unrelated notification
model; the actual IQ constructor is overload1, DEX(0Tp,0mK)V. Both observations
are retained so future traces do not reuse an alias or overload by guess.

No private state or live protocol response was inspected. Native Argo decoding,
full schema validation and callback exception semantics remain unported.
