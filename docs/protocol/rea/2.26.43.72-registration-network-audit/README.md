# Registration network audit evidence

APK 2.26.43.72, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Static REA/JADX evidence; no APK execution or live request.

- [class.json](class.json): complete parsed `AutoconfUseCase` metadata.
- [verify-response.json](verify-response.json): `AutoconfUseCase.A00`, overload0.
  Coroutine Java has duplicated/inconsistent control flow; do not infer status
  enum semantics from this rendering.
- [autoconf-usecase.txt](autoconf-usecase.txt): complete checked DEX output for
  the same class. SHA-256 `785eb90fcc5d85332719dae637d0219b68cab97c79a27432c791927b6c114c81`.
  A00 0080–0098 tests contact-point presence; 0198–01cc prepares and calls the
  downstream verification repository, with `autoconf` at 01b0.
- [endpoint-anchor-xrefs.txt](endpoint-anchor-xrefs.txt): exact-anchor results,
  not a complete endpoint inventory. SHA-256
  `302f0b1b235d160864c2e1dc63a11d2fa260a015e9a8c28953d6a37e3e6365fb`.

Reproduce with `xtask apk-xref APK --anchor reg_onboard_abprop --anchor
pre_pn_client_log --anchor client_log --anchor autoconf` and `xtask apk-smali APK
--class 'Lcom/whatsapp/registration/app/usecase/AutoconfUseCase;' --signatures
--exceptions`. Read the [audit and limits](../../registration-network-audit.md).
No Rust equivalence ownership record is claimed by these research leads.

- [method-classifier.json](method-classifier.json): complete `Hfo.A01` method
  classifier, not a runtime eligibility gate.
- [request-code-repository.json](request-code-repository.json): complete current
  `RequestCodeRepository.A00` Evidence; Java duplication warnings retained.
- [request-code-repository.txt](request-code-repository.txt): independently checked
  complete A00 instructions, SHA-256
  `b2022ce61f87962ac3c6fcf4f0647e2a79e5d6c3cffa3f6625dc6a4001d89f53`.
  Reproduce `xtask apk-smali APK --class
  'Lcom/whatsapp/registration/ui/task/RequestCodeRepository;' --method A00
  --signatures --exceptions`.

- [helper-A0P.json](helper-A0P.json): complete current `Hfq.A0P` REA record;
  [checked instructions](helper-A0P.txt), SHA-256
  `618bb2bbdfe92f00daf15665d5625dddc9673cb11aa490ed2e70ff5f7d623b4a`.

- [helper-A0S.json](helper-A0S.json): complete current `Hfq.A0S` REA record;
  [checked instructions](helper-A0S.txt), SHA-256
  `7e9f8deff3974f231115ce292d9e6978bbbc2c3206206fd2c6c81fa97043b21f`.

- [helper-A0U.json](helper-A0U.json): complete current `Hfq.A0U` REA record;
  [checked instructions](helper-A0U.txt), SHA-256
  `4170b0c3611e085539ff5496c2bb003659b1ef8329877bacad9ed56e715539e6`.

- [helper-A0M.json](helper-A0M.json): complete current `Hfq.A0M` REA record;
  [checked instructions](helper-A0M.txt), SHA-256
  `e651641c2cf9a3d5057b87b99dac913208ca091098db2f1e41b18cb359c855ae`.

Reproduce helpers with `xtask apk-smali APK --class 'LX/Hfq;' --method A0P
--signatures --exceptions`, substituting each method name.

Additional complete helper records (static producer leads; no Rust parity claim):
- [Hfq.A0O](helper-A0O.json)
- [Hfq.A0R](helper-A0R.json)
- [Hfq.A0T](helper-A0T.json)
- [Hfq.A0X](helper-A0X.json)
- [Hfq.A0Q](helper-A0Q.json)
- [Hfq.A0Y](helper-A0Y.json)
- [Hfq.A0W](helper-A0W.json)
- [Hfq.A0Z](helper-A0Z.json)
- [Hfq.A0V](helper-A0V.json)
- [Hfq.A0a](helper-A0a.json)
- [Hfq.A0b](helper-A0b.json)
- [Hfq.A0c](helper-A0c.json)
