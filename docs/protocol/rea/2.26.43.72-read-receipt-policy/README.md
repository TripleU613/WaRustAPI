# Current outgoing read policy evidence

WhatsApp Android2.26.43.72, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Static only. [Findings, negative guard and unported obligations](../../read-receipt-policy-current.md).
Earlier37.71 records remain separate. The ownership sidecar maps only the
necessary negative privacy condition to receipts.rs; no full native policy or
read-self coverage is claimed.

## Canonical REA records

- `X-C09S-A1H.json`: `ev_c5f70c13404de64cc1a4529a8dc484d5d461d5968b11fdc4586d3f579fb94ffc`; `X.C09S.A1H`; java source complete.
- `X-C0Dn-A0V.json`: `ev_fec0a308f33c762a9019b4081f3bc703a65cd331786497004e4b169cb36839ef`; `X.C0Dn.A0V`; java source complete.
- `X-C0No-A06.json`: `ev_c67974c591f30deacd00bb948d39c44647d34f2279ed2cb919d7cd4ea0119667`; `X.C0No.A06`; java source complete.
- `X-C0jD-A02.json`: `ev_ec6cbd2bdf455eb70631f32f09ec23d56e126b2adc83c6459533e3a0bc8a80f5`; `X.C0jD.A02`; java source complete.
- `classifier-A0a.json`: `ev_fd9ca5335758a4bfdd9c1497c0e2a40b8b002a8be34c12e145e807f15ccd9e7d`; `X.C0Dn.A0a`; java source complete.
- `classifier-A0g.json`: `ev_35c7cade2cade59f65d7704dad845dec0ba136b8029d4e4b729562d9f2a84cb5`; `X.C0Dn.A0g`; java source complete.
- `classifier-A0l.json`: `ev_9b636e8b8b1cdfacb77aa5ae6306f5d845bf18921938fb71e53603b9bc305159`; `X.C0Dn.A0l`; java source complete.
- `job-cancel.json`: `ev_ec68617ca28ccaaadbac634bc5d89a18380a63450c9a89332aadd3327fc03027`; `com.whatsapp.messaging.receipts.jobqueue.job.SendReadReceiptJob.A0K`; java source complete.
- `job-constructor-callers.json`: `ev_33a1406948f86eb05aa22e056f77c6633ee82984a1da47c202f0a26a12e77ea3`; `com.whatsapp.messaging.receipts.jobqueue.job.SendReadReceiptJob.<init>`.
- `job-execute.json`: `ev_10fe01200659e5d6ca1d179134c7bfc63004be079abc8a8e8d3a48b14502e419`; `com.whatsapp.messaging.receipts.jobqueue.job.SendReadReceiptJob.A0L`; java source complete.
- `job-members.json`: `ev_7f8a576c8a8942a7fd32e7fa857db1c26598f72a69fd402b7112d80ef7834fa5`; `com.whatsapp.messaging.receipts.jobqueue.job.SendReadReceiptJob.`.
- `job-policy-binding.json`: `ev_3561513d69ec4780ab01e1dd446201d608f3f831fbf3c2e3ba5faf00c7d2c28c`; `com.whatsapp.messaging.receipts.jobqueue.job.SendReadReceiptJob.Cf0`; java source complete.
- `job-start.json`: `ev_672da25cbc1601a961840155904ef7952bdffd07fe9c2bd9d3c36dc64655a080`; `com.whatsapp.messaging.receipts.jobqueue.job.SendReadReceiptJob.A0J`; java source complete.
- `linked-active.json`: `ev_ca18e1deb3007e923894601f90ae156f6f931bb5e12e33e55f4a48af47edf589`; `X.C1nG.A08`; java source complete.
- `linked-loaded-gate.json`: `ev_9983a3b78af86f631017ca5853bb93110ad534ed34e1ea34c0d05e38d975097b`; `X.C09F.BUx`; java source complete.
- `linked-own-namespace-view.json`: `ev_b841a5c20d4a946da5464db0d68abf9062227bc53b3fad3a0326dc39d54ad1c5`; `X.C1nH.A06`; java source complete.
- `linked-owner-members.json`: `ev_e14a119d662c04138586667fce7eab29d93365cac6d6f79dc80449af6069b648`; `X.C1nG.`.
- `policy-A00.json`: `ev_2a1abaf00efdc6983f7855c4c7ba47abb0d3efc1ea094d8a0fd70f9025ef81e8`; `X.C0xW.A00`; smali source complete.
- `policy-A01.json`: `ev_8430334b1a811a3d2cd66b001fc8ddbb548e2e5b981d99c6f9c05bd1751da447`; `X.C0xW.A01`; java source complete.
- `policy-A04.json`: `ev_605f6b433b6120ded750410cb52cb55aa287f9a9d4bef69c5fa0d043bae714a0`; `X.C0xW.A04`; java source complete.
- `policy-A05.json`: `ev_f3b239e806e675bd63a031c1b03485eded385338b348d5fb0e2a23904f98924f`; `X.C0xW.A05`; java source complete.
- `policy-A06.json`: `ev_7aaf3580c73ed1bdfb1b7fe6807d963f7494f88693141218fae4cfcbf56456c2`; `X.C0xW.A06`; java source complete.
- `policy-A07.json`: `ev_c6142700911628650ef579c96bef17d9f5d80ac2b9d542599311ab4cd1639220`; `X.C0xW.A07`; java source complete.
- `policy-A08.json`: `ev_22ebb840e91b0e4d6968b3298e7ca7e2be148b5789f9c4d30e36d27df8e6a1de`; `X.C0xW.A08`; java source complete.
- `policy-A09.json`: `ev_7e4737642717c72b694cad95049c070c071133cd5c07e07594386c9f6ad78666`; `X.C0xW.A09`; java source complete.
- `policy-members.json`: `ev_9b0ff679e9ce79ad7cd2b0fa4d7d12774b61b232156519c0db1a698bc0d2404b`; `X.C0xW.`.
- `producer-A0E.json`: `ev_d3e92290483c280a12e5e05c6a9073293b2a8f533094883798e7cb8759bf8189`; `X.C0x0.A0E`; java source complete.
- `producer-A0H.json`: `ev_c2464a863df1354d7679ac9a1837861453efd2fd8cffcaa506e9e1301128b491`; `X.C0x0.A0H`; java source complete.
- `producer-A0P.json`: `ev_50ca6c1bd749c5e68b279541317554738d9799e28d12e43561e9ee3127ecf6ac`; `X.C0x0.A0P`; java source complete.
- `producer-A0V.json`: `ev_a12f77fa55ccd94f7037f54a57dff1245b7fa43d5847e7e062e528b207e5a312`; `X.C0x0.A0V`; java source complete.
- `producer-A0d.json`: `ev_bcd8e0114fe86910bf3a9f1e84d46c2c36797211786a8c2e43f9630f796f646e`; `X.C0x0.A0d`; java source complete.

- `linked-value-filter.json`: `ev_ebd500d24316d88424498d6402b6514e16ffa11b90a808df22c8cdc6df0b1e60`; `X.F0I.A04`; java source complete.

- `receipt-route.json`: `ev_ad9396490d5c20c252d4a686df32048cf5ec0285550345ebee2efc2019d18bd6`; `X.Exf.A05`; java source complete.

## Checked DEX and limits

`apk-smali <current-apk> --class LX/0xW; --method A05/A09
--signatures --exceptions` produces annotated instruction streams with full
signatures/exception tables. These are not compilable smali. A05 proves the
current read/read-self branches; A09 resolves Java's misleading local flow and
invalid-ID apparent fallthrough.

- `job-admission-dex.txt`: SHA-256 `7a66b3701c72c834a1dc03991fa4b918bfbebd0ba4e8e2d545d10502a42bbe58`.
- `outgoing-type-dex.txt`: SHA-256 `5aa019dac006d66ff4966ab3dab1f9c27969a667d27cd5fe90159e38f267215f`.

Negative selectors: old job methodA0G is not reused as current run method;
A0J is only start bookkeeping, A0L is execution. 09F.BUx has no provider body
and09F has no constructor; neither proves absence. Guessed1nI.A06 does not
resolve; actual1nG fieldA07 is1nH, whoseA06 is retained. Current09F concrete
BUx implementors09G/09O are exact next selectors (identified by independent
DEX interface scan in the server-role audit), not interpreted as registration
mode from the similarly named BU7 gate.

Own-companion fieldA02 positive filtering, complete current loader binding,
classification, route construction and native writer remain open. No private
account preference, companion set, device or live service was inspected.
