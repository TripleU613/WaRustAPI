# Current server-sync role and recovery evidence

WhatsApp Android 2.26.43.72. APK SHA-256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Static only. [Findings and bounded gaps](../../server-sync-role-recovery.md).

Canonical REA records retain full original outputs and limitations:

- `account-init.json`: `ev_2e3a97b672fe0d88b67fcb0e0b193607bfbcec8866340076e6232797a6ee26df`; `X.C09O.A07`; body `available`, java.
- `change-number-completion.json`: `ev_634fa23357c958216331af7ebfbb13613f9575a50b595c22002a41705b7f7758`; `X.C13N.A0K`; body `available`, java.
- `companion-state-setter.json`: `ev_4d4c08e9c3fcd56ab80c3f1e69e9375941fd5e669789d1ec5ad87666ea8edd15`; `X.C09T.A00`; body `available`, java.
- `db-reset-provider-gap.json`: `ev_3157ed9cc6cbd1c36facd4d75ac95241fd418dfea78ad55a65522ce216f5feb4`; `X.C28m.A0G`; body `not_available`, java.
- `device-index.json`: `ev_3f48ac7483d40a51ba60f9d5ec364ca020a9972a5f0f8fa197ec785b05a0df52`; `X.C09O.Auf`; body `available`, java.
- `fatal.json`: `ev_c4f784710cc9f23764f7b5c492c4abcd8b4cb691bfdec54b6afda37178ddec1d`; `com.whatsapp.consumer.companiondevice.iq.SyncResponseHandler.A08`; body `available`, java.
- `predicate.json`: `ev_ff37fd6ee96e3134ce4528760a6bda35fb67c35742750a06eb8859dcb4ffd304`; `X.C09O.BU8`; body `available`, java.
- `preference-reset.json`: `ev_701f092ba3e58a00837e42b14943065bc0c31f95a3b73b8d28fdcaf4b644171a`; `X.C0Yz.<clinit>`; body `available`, smali.
- `recovery-begin.json`: `ev_e93a66dbd313bd1fc2f5d0238b9b5f0a57af567c45bd113756c451f857e566fa`; `X.C0Yz.A03`; body `available`, java.
- `recovery-change.json`: `ev_ece13378ca849e4fbd7b1bd4ae2cbd5598f39ec7612aa64f742c433405847d6a`; `X.C0lJ.A03`; body `available`, java.
- `recovery-entry.json`: `ev_793ddc6c744aa5c35a3bd7ad273f83fb3291fefce7d829c1867cd25896b11297`; `X.C0lJ.A01`; body `available`, java.
- `recovery-logout.json`: `ev_da3bbdbf20d1945cb42f02d54d3bea5d41ccf59ae7c3976068784cdd5bf4a600`; `X.C0lJ.Bsp`; body `available`, java.
- `recovery-ready.json`: `ev_27de7253695d1128ae500670467f2734bd548c9e10c0fa6b16bbe5b5fbbd2a29`; `X.C0lJ.A04`; body `available`, java.
- `recovery-result.json`: `ev_cbbbbd0466aaf344ef8145ae3eb5fc81214ebd8fddf6e1e85456b4e5bc03c31d`; `X.C0lJ.CK4`; body `available`, java.
- `recovery-set.json`: `ev_41684b84c447a0a496323a1fbf9e9e20c02034f960260bb2c6c4cb295c216fc9`; `X.C0Yz.A05`; body `available`, java.
- `registration-device-reset.json`: `ev_532f03d1d42d2c1bfa91fcdb6bb5bdc251b88f48394c52db85347ec5a0178a2c`; `X.C09O.A01`; body `available`, java.
- `reregister-trigger.json`: `ev_b55d2fb17ed40ab8f6ed6d032331deaffcf2352194f418087cb8745e91078817`; `X.C13N.A05`; body `available`, java.
- `reset-preferences.json`: `ev_6b8ddcc4cd3ba6fe785f778cf809c710855d316478e998b6f576f94ca5457a92`; `X.FDp.CKt`; body `available`, java.
- `x--c09-g.json`: `ev_2cce8f138f2ae32f9e499a736bba5a35fb93b3afd0babd3db67fa41973aecab7`; `X.C09G.BU7`; body `not_available`, java.
- `x--c09-o.json`: `ev_f5b0f60455316f08f82ac95216f3eb7ccecaea8893abb8b0e2fcf9727c9a7727`; `X.C09O.BU7`; body `available`, java.

Unavailable provider bodies in `x--c09-g.json` and `db-reset-provider-gap.json`
are explicit negative findings; their real DEX bodies are independently retained.
`companion-state-setter.json` refutes mapping09T.A00 to device-index writes.
Original REA method identity remains unchanged despite clearer local filenames.

## Reproduction and limits

APK path outside checkout: `android-lab/apks/WhatsApp.1.apk`.
Existing `target/debug/xtask apk-smali <apk> --class <exact class> --signatures
--exceptions` exports are named by their subject. Filtered exports:

- role-predicate: `LX/09O; --method BU`; proxy: `LX/09G; --method BU7`;
  role-device-index: `LX/09O; --method Auf`.
- recovery-preferences: all `LX/0Yz;`; recovery-handler: all `LX/0lJ;`.
- reregister-trigger: `LX/13N; --method A05`; change-number-trigger:
  `LX/13N; --method A0K`; registration-task: all `LX/Fel;` (constructor binds
  discriminator, full run retains the referenced7/8 branches).
- recovery-delete-action: `LX/Dp8;`; register-recovery: `LX/28m; --method A0G`
  (**DB reset**, not fresh registration); downgrade-recovery: `LX/1sj;
  --method onDowngrade`; reset-callback: `LX/FDp; --method CKt`.

`apk-callers` inputs are exact owner/name pairs visible in the report headers
or filenames:0Yz.A05 (set),0Yz.A03 (begin),0lJ.A01 (mark),0aS.A0R (delete).
`apk-xref` anchors are `syncd_dirty`, `companion_syncd_dirty`, and
`registration_device_id`. `apk-field-refs` keyset uses owner `LX/0Yz;`,
field `A03`, type `Ljava/util/Set;`.

`apk-virtual-overrides` for09F.BU7 finds only the interface declaration because
that command follows superclass ancestry. The separate09G.BU7 run finds its
forwarder; neither is an interface implementation inventory. The retained direct
implementor list was produced by reading each APK classes*.dex header's type
and class-def tables, following every class_def.interfaces_off type_list, and
selecting lists containing exact `LX/09F;` (all DEX members, no class-name guess).
It yielded09G and09O;09G superclass ancestry had no other declaring BU7 body.
This is static declared membership only, not runtime dispatch proof.

Preceding lifecycle evidence already retains `0aS.A0c/A0d`, `0g1.A0S`,
`SyncResponseHandler` checked DEX and constructors; those are linked, not copied.
Ownership stays empty because these dependencies are not a completed Rust port.

Additional bounded searches retained: `server-sync-recovery-entry-worker.txt`
(`FdE.run`) reaches mark-dirty reason10, and
`server-sync-delete-register-entry.txt` (`LEs.A0m`) reaches reason5 after a
message-store initialization path when `A0c` permits. Their full callers/gates
were not reconstructed; neither establishes fresh-registration defaults.
The generic `Fel` constructor callers are retained to bind the13N.A05/A0K
workers; unrelated branches do not establish additional behavior.

## Text SHA-256

- `server-sync-change-number-trigger.txt`: `8d65a183697efd54532354de43ec3400f315c052c6a3c8e892fba1ff38ae7bae`
- `server-sync-delete-callers.txt`: `cc32f78c958c709d62840f83398972dd1e917619afc1e9785912a10f87c6040b`
- `server-sync-downgrade-recovery.txt`: `05bdcd0dcc4d148525d2b76cf43bccff4e159bbadab7cc1787a367de16f5b9ab`
- `server-sync-mark-recovery-callers.txt`: `b466e37e4e14bb599609bccf3ea8aeb8a7e30b6b103c69e9938de95426358277`
- `server-sync-recovery-begin-callers.txt`: `fef84aa8f76843dc21bfda7673f9b0c4ceb3054795a32a82d009a9193668214a`
- `server-sync-recovery-delete-action.txt`: `ea07cd9d147853a1c3a23ab6d522f5670bbe41eccd230564ca98fd60899b89a5`
- `server-sync-recovery-handler.txt`: `224e236ebe09a4dac73a18c7c9f1e8c348375363faec3792034d349836e3ad60`
- `server-sync-recovery-keyset-refs.txt`: `77220a73ff5f620fa995969c3dc158b92137e0cee00c5d1e2d289a34a8e37512`
- `server-sync-recovery-preferences.txt`: `3b0599c1aa1cb59071a6757bb6c5a7f50e965081486087ebb83d47505bee6daf`
- `server-sync-recovery-reset-callback.txt`: `9d4b79ac4863feab5b2df73f338af7fb883eb6a1c746622993be3cc465cb753e`
- `server-sync-recovery-set-callers.txt`: `54ca248744a6282a4fa7c87ea89bef41082d4603eb6d37bc6570f3a4e90a1dea`
- `server-sync-recovery-xrefs.txt`: `f6388a80fa9dfbcc23ca112c93ca4c646ac9fdbf9025245b2b39ee4133c1de37`
- `server-sync-register-recovery.txt`: `81e6830698762bc2061e8318730fde5074f21fee63ccb3ff7d6341aada70e38a`
- `server-sync-registration-task.txt`: `95ae460a6305959f4846ffa3b9fa53c0d2a676979f1dfe7d6545d4b87c30a5d9`
- `server-sync-reregister-trigger.txt`: `67c618aa1f2c33502b42b5c9ff4244ddb2cd48d6b1ab1e92c9378daf756bb5a4`
- `server-sync-role-abstract-overrides.txt`: `bc38a0af251b1cbeabb6f3d41756de2901450b79b2428a266edf8d978e5ebc1f`
- `server-sync-role-device-id-xrefs.txt`: `334bded25538651f4b2fd8efc8fde6d79e095c891519397267372428f867f160`
- `server-sync-role-device-index.txt`: `f32109f60aedab5b509279116f92b92171769ecdacd9c96e48d352e4c6624083`
- `server-sync-role-implementors.txt`: `82e4f7221409d2e523a514a8e75347244e892c61feb061c6e582e976ccbb8832`
- `server-sync-role-overrides.txt`: `a32f271fd90623cb415562188f5eb9194a20ccf554f1f37e81fd9bef4bdaca2f`
- `server-sync-role-predicate.txt`: `ed831a112d8b4832e410b1e8f0e7c69a2d1dfff8eef9c27cd597288a805782a4`
- `server-sync-role-proxy.txt`: `2911a1fded8827167b57a9200ca1bb0c2869a64fb11b410a2c949f4f9a4d22f3`
- `server-sync-recovery-entry-worker.txt`: `aead51c3ff5faa9e337e897bd6c54499eb28d53cff6f74820a9925d6ecb57f7f`
- `server-sync-delete-register-entry.txt`: `bc0ee5b19a175d6cc3b525cfd8760f0149e4db67452518b41d052532341223a5`
- `server-sync-delete-worker-callers.txt`: `df35b0cd7d38b9595bb611da9acc21328822dc03ab86479e8e80cc5b7fb30614`
