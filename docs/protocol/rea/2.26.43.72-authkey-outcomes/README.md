# Auth-key request outcome and startup evidence

Static APK 2.26.43.72, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No execution or account traffic. Complete Evidence includes full fallback smali
for the startup method; Java availability is not asserted for that method.
[Findings and limits](../../authkey-outcomes.md).

| Record | Selected method/operation | Evidence ID |
| --- | --- | --- |
| [client-auth-token-callers.json](client-auth-token-callers.json) | `X.C1jA.A01` | `ev_9504198bee43cf55340a66372a590f9d113998700c8dc4cc460269a38279daf6` |
| [client-auth-token-pending.json](client-auth-token-pending.json) | `X.C1jA.A00` | `ev_fb70cab55421164242995932ed2516712f72a1bc49da7ee3eb8a6db530ca4477` |
| [client-auth-token-request.json](client-auth-token-request.json) | `X.C1jA.A01` | `ev_5649a3c2dd82159df26f4512b5b3e7a7e841dc463a3b0ef6103b3ee9a9d52bee` |
| [client-auth-token-send.json](client-auth-token-send.json) | `X.C1rj.A05` | `ev_16a3d0d37ff40d980ba00862d7582a73c096087fd7923b34edbf648866ef3a0a` |
| [client-auth-token-success.json](client-auth-token-success.json) | `X.FUT.CK4` | `ev_fa32b710d15f9f681708426e440b5e5a2d9e5aa244aee6579149f311b876f804` |
| [connection-startup.json](connection-startup.json) | `X.C0ZJ.A0z` | `ev_4fdf11deb3bd5ba549acb3df00df348eb1f79c1c129c10d414c6aac276c99d94` |
| [delivery-failure-runnable.json](delivery-failure-runnable.json) | `X.C8bj.run` | `ev_e03b630d55d6f5b5829a6c98f0b13bb672613fcd7f9e2ad7b08aca282fd79bbb` |
| [handler-dispatch.json](handler-dispatch.json) | `X.C0m2.handleMessage` | `ev_5f72999abb1c2187af11f8d1a0b17e3368c6bd5f622a6f5f14633c757ed44ad6` |
| [incoming-iq-callback.json](incoming-iq-callback.json) | `X.C0ln.A01` | `ev_445bcabed23b1091f4b30e63e2bd073f980299ad5b82df8787e84b425d8bf9c2` |
| [incoming-iq-dispatch.json](incoming-iq-dispatch.json) | `X.C0ln.A02` | `ev_80d4e071801b1f82db5ae86e1f0e005473793751804c719646752ef1d1af1c4c` |
| [incoming-iq-match.json](incoming-iq-match.json) | `X.C0ln.A0H` | `ev_5c63f04eb4867c65329178e3a6bcba3d331928a348aa4c2f59cf98e01eb9556e` |
| [incoming-message-dispatch.json](incoming-message-dispatch.json) | `X.C0ln.A07` | `ev_d23372c445548a1673206bcd127d5963898e8a5c86d9b26df789451aa48f76be` |
| [message-ack-timeout.json](message-ack-timeout.json) | `X.C0ln.A06` | `ev_c99c7cfcda25fb2a1714a3c338c50bf2a52ab10c8da9ab9e2200d1ed90cfa340` |
| [message-client-class.json](message-client-class.json) | `X.C0Sf.` | `ev_ab1a54b64720d4669645a7581c33e73529bf6fb6584c8b6d8836e74e9990e371` |
| [pending-failure.json](pending-failure.json) | `X.C0ln.A03` | `ev_611b9c6c803de26a637f25d0897907aefa14c771e207e4423d92fd9568f6c2e7` |
| [pending-registration.json](pending-registration.json) | `X.C0ln.A08` | `ev_b65440891883900d658024c5cf3f009a791df60e110091a9831017a20e30fa92` |
| [pending-router-class.json](pending-router-class.json) | `X.C0ln.` | `ev_4bc899896dd28b0797b8bec828e3382651fc763628eaba30578342e0deb9813b` |
| [pending-router-getter.json](pending-router-getter.json) | `X.C0Sf.A03` | `ev_84572837980b95453f23040a7235dd5b0a073452cad48b3e196fb08efa6d66ee` |
| [rotation-callers.json](rotation-callers.json) | `X.C19i.A00` | `ev_e7d6c495a4ed4bb58975400da28e3fa6152c365ddbb65920776a57b433314772` |
| [send-iq-entry.json](send-iq-entry.json) | `X.C0Sf.A0W` | `ev_59a2c625b792787b398f483a9afc792c46159c1a9dcd7788881d844d9e036eaa` |
| [send-iq-internal.json](send-iq-internal.json) | `X.C0Sf.A0A` | `ev_c8643951cfc21d3f1531b7067f6076ca33a6c4406d600c9745a38572e54b0aa7` |

Checked DEX exports and SHA-256:

- [client-auth-token-send-checked.txt](client-auth-token-send-checked.txt): `710a9c8fed4e121c73b5210d63d7ec0b14ebb5747cef76d7d4380dfcc9fb1b0c`
- [delivery-failure-checked.txt](delivery-failure-checked.txt): `7dd5e2a054b2132ffcd6ef497c3c50e18e14f6689db85674fab7e7dcaac3f27e`
- [pending-failure-checked.txt](pending-failure-checked.txt): `9ae3b22f41df5bcf12c7bda72207a35ae662f1dc77113a5df87ff2acd380d706`
- [pending-registration-checked.txt](pending-registration-checked.txt): `99acfbec072ba7b3e3cd1818627a04fa3664293655cc0a429564b416f57dfee8`
- [push-consumer-checked.txt](push-consumer-checked.txt): `df65a93664e6d6c9be99020e891993030df015fd0a2b230dc772c00ffed4d5ba`
- [startup-trigger-checked.txt](startup-trigger-checked.txt): `e75620ce701dd4a50da59adedbfe4f7686c08b6d34c24bea401bf45a2d8cbd69`

Reproduce with `xtask apk-smali APK --class CLASS --method METHOD --signatures
--exceptions`: `LX/0ln;` A08/A03; `LX/8bj;` run; `LX/0ZJ;` A0z;
`LX/F2Y;` A02; `LX/1rj;` A05. Outputs are complete selected methods, not a
bulk decompiled tree. Text offsets are method-relative bytes. No Rust rotation,
client-auth-token or callback-router ownership/parity record is claimed.
