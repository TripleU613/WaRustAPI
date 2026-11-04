# Server-sync lifecycle evidence

WhatsApp Android 2.26.43.72. APK SHA-256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Static only. See [findings and next prerequisites](../../server-sync-lifecycle.md).

Canonical complete REA method records (unmodified, original limitations retained):

- `applied-versions.json`: `ev_9dd7102331aa6c11fe3d0e0137df1c664ce0e66d8b6e1fb3c719b3457c06bd7d`; `X.C0l8.A04`; java, source complete.
- `commit-version.json`: `ev_a5b52c438cb51bd1413a757b5a30c7621c4dd97212e1a04505fdf585dc81449a`; `X.C0l8.A08`; java, source complete.
- `commit.json`: `ev_5ca8a58948d9cffcdbabd9b7532cf9ed741834df4421d03c47b61d788c31a5fd`; `X.C0mj.A00`; smali, source complete.
- `delivery-failure.json`: `ev_7b3f88d9dd8b4f8142f1fabec079f49a7e0bf874ff156eaef8cf1aa93e8d289b`; `X.FUc.BsS`; java, source complete.
- `dirty-collections.json`: `ev_1a86f51e732c133862e59287d1db430d82f7922bbf681e5b94bbc9b15ec24992`; `X.C0l8.A03`; java, source complete.
- `disabled.json`: `ev_c5f6a7d2f294e81ed188266892a62b0800443fd203dfe0f5b5f1f7568ee774df`; `X.C0aS.A0d`; java, source complete.
- `error.json`: `ev_ba97e05f6134ed980c4dedec8e2038a59edcc7dfcf94962fc36b8839c55e9922`; `X.FUc.Bvj`; java, source complete.
- `gate.json`: `ev_746b3a665459da09273829ba24c534409d709318ceb3ce07e0445ab86ccf9b57`; `X.C0aS.A0c`; java, source complete.
- `has-dirty.json`: `ev_96f963d4cccd022ef135f7098cdd248768cd27582b55ee1dc06138c801d0f421`; `X.C0g1.A0S`; java, source complete.
- `is-dirty.json`: `ev_4205ff04e559f69f4240a6b3c15114cdbb208ea68d96af4cfdcddc72c0c44f16`; `X.C0l8.A09`; java, source complete.
- `post-build-callbacks.json`: `ev_080e92f221091153ae1661fa2af6493dd41405eb565e9500e1b7a6f4e3752566`; `X.FJU.accept`; java, source complete.
- `release-flight.json`: `ev_127ecec96fedb6f3d14792188cc8347e12920de38ff400ebead950d60f9ababe`; `X.C0aS.A06`; java, source complete.
- `response-abort.json`: `ev_30191bc9a17ad408d466e36c10b6c7b3677d8ac91a2d32925277c11fa20fb7c0`; `com.whatsapp.consumer.companiondevice.iq.SyncResponseHandler.A02`; java, source complete.
- `response-apply.json`: `ev_7852fb5349336dfa6d564f90f73256e89501a4c91639d8f13051723079e36db8`; `com.whatsapp.consumer.companiondevice.iq.SyncResponseHandler.A04`; java, source complete.
- `response-complete.json`: `ev_e1564fb0ca08953d298199207876bc8a882620fe95da48d14098798a1a24bf67`; `com.whatsapp.consumer.companiondevice.iq.SyncResponseHandler.A01`; java, source complete.
- `response-coroutine.json`: `ev_3b22ad51bf436c1629bb351d63bbe6e5417b9f64cdc0163f6a416ba0afa3ef37`; `com.whatsapp.consumer.companiondevice.iq.SyncResponseHandler.A00`; java, source complete.
- `response-failure.json`: `ev_21b113cde42515078961b7504d8f856f593f7d0a661580e206558a19f3b1f907`; `com.whatsapp.consumer.companiondevice.iq.SyncResponseHandler.A09`; java, source complete.
- `response-release.json`: `ev_d0cdba69495efbdf4c124e40d8a35641063ffd019679d1b17a51e7bfb0be2d9e`; `com.whatsapp.consumer.companiondevice.iq.SyncResponseHandler.A05`; java, source complete.
- `response-verified-apply.json`: `ev_bdbbe822dbf9e326660455eea6fe36900bd682985f5b122054c44c507ecb7b73`; `com.whatsapp.consumer.companiondevice.iq.SyncResponseHandler.A07`; smali, source complete.
- `result.json`: `ev_ea4b2d4d6fe11b5689284897ed6a72019b026e56d178eeba2116fa5027a6f098`; `X.FUc.CK4`; java, source complete.
- `schedule.json`: `ev_290d4bbdca9d07a71789a8476815bc22490124d9f71cbb7742f82127f6bae21c`; `X.C0aS.A0P`; java, source complete.
- `start.json`: `ev_f246dc7a53b43b6d997f4bd631280476876095fa7efa00b4a74b2e7d3cf8dbf1`; `X.C0aS.A0Q`; java, source complete.
- `sync-disabled-preference.json`: `ev_1d60c57aec9a0c460c5d69cf31d29d698f2f2564507cb1394b3a730d67815b56`; `X.C0Yz.A09`; java, source complete.
- `version.json`: `ev_6bf63698a0d12a16390aa75529f774e1d3e080f9f97a99a97335101368cb7270`; `X.C0l8.A07`; java, source complete.
- `worker.json`: `ev_d4b81e19f10d5c00c4b2c4092d0100c78dfd23949112e898a11ee50c4d4ba85e`; `X.Dxc.run`; java, source complete.

`response-verified-apply.json` and `commit.json` use complete smali fallback after
JADX rejected its Java recovery. `worker.json`, `start.json`, and other recovered
Java contain warnings. Keep them with the checked DEX, not as compilable originals.
REA `FJU.invoke` lookup failed because its callback is named `accept`; metadata
listed `accept`, which was inspected instead. This failure does not imply a missing
callback. Existing type map/notification handler records are referenced from the
preceding notification-sync-obligations directory and not duplicated.

Each checked text was generated with existing repository tooling:

```text
target/debug/xtask apk-smali <outside-checkout WhatsApp.1.apk> --class <exact descriptor> --signatures --exceptions
```

Exact descriptors, in file order below: `LX/0mj;`, `LX/0aS;`, `LX/FUc;`,
`Lcom/whatsapp/consumer/companiondevice/iq/SyncResponseHandler;`, `LX/FeN;`,
`LX/0l8;`, `LX/Dxc;`. `server-sync-commit.txt` additionally filters `--method A00`;
`server-sync-result-worker.txt` filters `--method run`. Offsets are method-relative
bytes, not DEX code units. Text is partial annotated bytecode, not compilable smali.
The method bodies include real DEX signatures and checked exception ranges.

## Text SHA-256

- `server-sync-commit.txt`: `62c1a0e0be99b1e645f22f654439ac358006da3429dcfcfa7fbc5b80161ee5a9`
- `server-sync-manager.txt`: `e307709740154375ef6c83101a6003b7e411dcb64f07bc94b87bddfb6dafacfd`
- `server-sync-request-handler.txt`: `14604786b10e374c1a060198bfed43cbafa9759e9d4f788cbee0f9e972e74c3b`
- `server-sync-response.txt`: `f7460c967c0bd2ecab0ebfd8d977c2ed241381bcd6a99399fa63c71cc35dacdb`
- `server-sync-result-worker.txt`: `2465677468b85f3871b0fee262845a2f6eb3affc02c33d7a610a7c1bdceaee16`
- `server-sync-version-store.txt`: `b82406b7e4ffb7be60e6f307486a594206438f33b5ae03365338e35578cea6ac`
- `server-sync-worker.txt`: `f3d87d115ea66f14892ecbba2e9a2b8e303344fe1088a881fed373c91a01aa80`

Ownership is intentionally empty: traced dependencies are not completed Rust ports.
