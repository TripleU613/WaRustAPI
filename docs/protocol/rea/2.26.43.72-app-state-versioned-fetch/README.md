# Version-aware fetch evidence

WhatsApp Android 2.26.43.72; APK SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Static only. Findings and validation:
[app-state-versioned-fetch.md](../../app-state-versioned-fetch.md).

Complete canonical REA records, original limitations retained:

- `empty-constructor.json`: `ev_891c28ffca870ac60927ff9e06e59ce31a7b2d382a3803522b23d954458b03f0`; `X.EdY.<init>`, overload0, complete java.
- `empty.json`: `ev_42eab65bf0d2db1d4cb899a587ed6085680dda5c05c7907b9227f1e766a538f8`; `X.EdY.A00`, overload0, complete java.
- `finish.json`: `ev_e59a97ae6713a17ca6fe236d880bc81ee05366092ef3b16267381fffc12b9226`; `X.EfK.A00`, overload0, complete java.
- `hook.json`: `ev_aa20a237b1870c25260cb3e35429755dbc5c1225e1f4f18d404d07c263cffa95`; `X.C0gG.A00`, overload0, complete java.
- `iq-attrs.json`: `ev_391952f0f041b6e85c6f7fdc062b9e0aa2c87adb666d50795074620befd35101`; `X.DBi.A0h`, overload0, complete java.
- `iq-root.json`: `ev_2c155894ba54d1bd225af39e2abd69fef234b90e1601355d5e783672926bc867`; `X.B22.A0K`, overload0, complete java.
- `long-encoding.json`: `ev_223421c228f2bcb70de660d777cd1b0ee2186a342fe3983ee739e267d6705c66`; `X.C0Tl.<init>`, overload3, complete java.
- `root.json`: `ev_a09626b6344141358cf09dbd3897445a4f25417eba57fd03291078df6bc744ad`; `X.EfK.<init>`, overload0, complete java.
- `version-attr.json`: `ev_e977a40cb4962a88f9c52e8b48dfbba32e534c8752140f5b159e11584d931ae1`; `X.B1y.A1I`, overload0, complete java.
- `wire.json`: `ev_7e387debc48a80dca38036ceeaca545cfeb850486d2ffe0cf500601713860c9f`; `X.EfK.A01`, overload0, complete java.

`root.json` constructor Java hides field initializers; the checked DEX
`version-fetch-wire.txt` confirms sync builder allocation and the initialized
maps/set. `0gG.A00` only invokes provider lookup and discards its return;
provider initialization side effects are not established by that body.
Prior exact held-version producer, empty-mutation worker and send callback
records remain in `../2.26.43.72-server-sync-lifecycle/`; they are not copied.

Checked DEX exports use repository tooling, exact current descriptors:

```text
target/debug/xtask apk-smali <apk> --class 'LX/EfK;' --signatures --exceptions
target/debug/xtask apk-smali <apk> --class 'LX/0gG;' --method A00 --signatures --exceptions
```

These are partial annotated instruction streams, not compilable smali.
Method offsets are bytes. APK is outside checkout under `android-lab/apks`.

## Text SHA-256

- `version-fetch-hook.txt`: `242618e52612b3fc71744ca6e14677f643272b81750d1dd641cc3828eaa5d566`
- `version-fetch-wire.txt`: `5ce4410729ae3d9726cbb1f654a97866157faf64d642a8f975f36733dedd4077`

Ownership maps request construction only; no notification scheduler completion is claimed.
