# App-state sync trigger and flight evidence

WhatsApp Android 2.26.43.72, APK SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Static only; no packet, account or APK execution. See
[scheduler mapping and flight boundary](../../server-sync-trigger-flight.md).

New canonical complete REA records, original limitations retained:

- `manager-deferred-state.json`: `ev_b3fc802b6b928b77b65497998208fde141bf426456f81d8637835fb25e76b552`; `X.C0aS.A0b`; source complete.
- `request-context-clear.json`: `ev_9a1e5d210c9f94f809c347b999794cd5aadec2d8fe5609ae032200873f29dea2`; `X.FUc.A00`; source complete.
- `retry-delay.json`: `ev_ff4a8c90e27ea8b7f8b840b572aae5c11d258a7b5a9ce9ec9306b4d7afa7c1cb`; `X.C1C8.A01`; source complete.
- `send-with-callback.json`: `ev_fc0d9a3b6ff44f4af5926b569cad1b63b7cb7f2e010bdb47c11712caa61267a2`; `X.C0Sf.A0R`; source complete.

Existing exact-current records are reused, not copied:

- `../2.26.43.72-server-sync-lifecycle/`:0aS.A0Q/A06/A0P/A0c/A0d,
  Dxc.run,FJU.accept,FUc.BsS/CK4/Bvj,SyncResponseHandler.A00/A01/A02/A08.
- `../2.26.43.72-app-state-versioned-fetch/`:EfK.A01/A00 + emptyEdY result.
- `../2.26.43.72-authkey-outcomes/`:0ln.A08 timer registration,0m2 handler,
  0ln.A03 pending failure removal,8bj.run delivery callback and0Sf.A0W/A0A.

Checked exports use existingxtask, pinned APK outside checkout:

- `apk-field-refs --class 'LX/0aS;' --field A02 --type Z`: all8 direct field
  instructions, two writer methods. No constructor write: Java boolean default.
- `apk-callers`:0aS.A0P (37),A0Q (7),A06 (5),<init> (1);EfK.A00 (2),
  <init> (2);FUc.<init> (1);Dzb.<init> (37 across overloads, not just digest);
  0ln.A03 (2). Full descriptors/callsite offsets appear in outputs.
- `apk-xref --anchor 'w:sync:app:state'`:4 exact string refs; string refs alone
  do not establish concurrency ownership.
- `apk-smali --class 'LX/Feb;' --signatures --exceptions`:full4 methods,
  including discriminator4 delivery worker and helper binding.
- `apk-smali --class 'LX/Dzb;' --signatures --exceptions`:full29 methods,
  constructor(String,String,int) emits coex digest for itsnon28 branch; other
  constructors/branches are unrelated and not claimed ported.

Structural text is partial annotated bytecode, not compilable smali. Method
positions are byte offsets. No owner scope, lifetime or runtime outcome is
inferred from a string hit alone. Ownership is empty: this is research for a
future flight lifecycle, not a completed scheduler or generic IQ lock.

## Text SHA-256

- `sync-trigger-builder-callers.txt`: `deb362d5e7a0ddfeb0d144e2425e023fe41b874edae093937c43f4ddb6082d08`
- `sync-trigger-delivery-worker-full.txt`: `d24bd6284822910818d722b6a03d19cf3b0f8fb846ac8ba3a36f1dfae70396f6`
- `sync-trigger-flight-clear-callers.txt`: `600b6bdbcc1be7f1921eb30eb58be63fde3a575f54062acdc4161739d70ce4e6`
- `sync-trigger-flight-refs.txt`: `ce3024aa73cdbbdcc044a340f2984cccd94ee7cb88fec5ece761a5a4cfd69df1`
- `sync-trigger-force-callers.txt`: `0cef5de3d11987c9cb8f8aeaab0413a48930b23c22f8cef1ec80597cc6d53249`
- `sync-trigger-iq-failure-callers.txt`: `a2c916dbe77d4d390407bf89480d7bacc7705ed4bb2954e232edc47e604cb638`
- `sync-trigger-manager-owner-callers.txt`: `b97c26b9f2c9ed4dac2a8646ce02d5e4d446e2a8857c4260554424074512fff1`
- `sync-trigger-namespace-xrefs.txt`: `0beaf0f524e2843dddb396aa1646c2cd4e92266513a4eb2afa1ff1b10a07fcdc`
- `sync-trigger-other-query-builder.txt`: `deed68ec241507b30772f05ca76adafb64f7d1fece43d3012a735f6b2a64295a`
- `sync-trigger-other-query-callers.txt`: `19912dfb31d7861b86aac18d54a1b479c526bbbaad4a3c220f422df0d8203e86`
- `sync-trigger-request-builder-constructors.txt`: `f733364b2dcc283e97bc9de36e1dc6ad4ba3662c2a769eda1e95e9182278b68e`
- `sync-trigger-request-owner-callers.txt`: `3ef49c110652629e5c989774472fb2e82149504600a2f77fe54b8bbf568428e3`
- `sync-trigger-start-callers.txt`: `180a6d2586a25fb9c0a5f10bcb92a9f99bf7d1ba53f7c2e886169f888ac583de`
