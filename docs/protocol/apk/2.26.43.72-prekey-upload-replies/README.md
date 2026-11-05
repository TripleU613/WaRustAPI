# Prekey upload reply callbacks — APK 2.26.43.72

**Confirmed static**, checked DEX exports, not observed runtime or REA Evidence
envelopes. APK SHA-256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`
reverified 2026-11-04. Commands: `xtask apk-xref APK --anchor count_low` and
`xtask apk-smali APK --class CLASS --method METHOD --signatures --exceptions`.
Only selected methods are retained; no original Java source is claimed.

## Findings

- `count_low` has five references: 140.CK4 twice, 141.CK4 twice, 1ru.A02 once.
  The latter was not traced here, so this is not exhaustive replenishment policy.
- **141.CK4**, 000c–005c: optional first `list` and `pq_list` selected through
  0Tp.A0F; each `count_low` is compared with literal `true` using equalsIgnoreCase.
  Absent child/attribute feeds null and therefore false. **140.CK4**, 0044–0094,
  uses the same two optional children and comparisons. This supports existing
  Rust legacy count_low semantics; it does not justify requiring a list child
  as proof of acceptance or inventing an error if that child is absent.
- **0Tp.A0F** walks direct children in array order and returns the first exact
  TextUtils.equals name match; it does not enforce uniqueness or choose the last.
- **141.CK4**, 008e–00cc, snapshots pending arrays/callback, nulls the held fields
  under a monitor and sets state 12. Missing arrays become empty. It reports
  success then schedules Fd9 with selector 1 and both count-low booleans.
  **Fd9.<init>** binds the selector and booleans. **Fd9.run** selector nonzero
  calls 13z.A0X/A0W for nonempty arrays, then EcZ.A00 with success=true and both
  flags. Exact durable effects of 13z methods and downstream EcZ remain untraced.
- **141.Bvj** calls 1ud.A01, defaults numeric code to zero/text to null when
  absent, clears pending arrays/callback under a monitor, sets state 12, logs
  and calls EcZ.A00 with success=false and false count-low flags. No automatic
  retry can be inferred without tracing that callback. 140.Bvj is retained for
  comparison; its full state/error policy is not claimed equivalent to 141.
- **1ud.A01** iterates error children and returns the first with a present code,
  optional text and code transformed by 0HF.A00(code,0). No-code children are
  skipped. Exact numeric parser behavior was not traced. Rust's first-error,
  eight-character diagnostic retention is not native integer/text parity; it
  currently does not determine prekey acceptance or scheduling.

## Remaining work

Upload producer-to-callback ownership, 1ru.A02, EcZ implementations, durable
13z effects, migrated/PQ strategy, sender authorization and full count-trigger
ACK/upload ordering remain open. Rust reads only legacy count_low for automatic
follow-up; both native callbacks also expose a PQ flag. No new PQ behavior is
implemented from this trace. Current callback evidence supersedes assumptions
based solely on the older Bor name without renaming that older-build evidence.
No production behavior or live account operation changed.

## Retained exports and SHA-256

- [countlow-xref.txt](countlow-xref.txt): `3816accadee0a20d3c9513d1acdaea2eac408c123d63e297af91b9aeceb309c2`
- [140-result.txt](140-result.txt): `e13fb15e44b89a43635950288596c5549a3ed87f408ec1176e50eafd936cff95`
- [141-result.txt](141-result.txt): `9fb486a75e2a54c11cac00f524484aa03531f0f16309b2ed8dc1dbbbe67fe218`
- [140-error.txt](140-error.txt): `42ddcaa42056a136a67efeb5d79342b882b6a9cc27e2019a732cdead58ef3040`
- [141-error.txt](141-error.txt): `4e30cb42de156f9fa53d37da475d6bf8fc68ba0b95b444bd578c97e0700133f3`
- [result-worker.txt](result-worker.txt): `007e77b74284d2b63c1e986218d312a4c967f07e600203dfa714f07df18f4929`
- [error-parser.txt](error-parser.txt): `1be0e3bd36e81549ffcb65c4e65a02c4ee84656cf2cb81dcdbd59f06a56b6923`
- [child-selector.txt](child-selector.txt): `d0d1534f2ccbf7046482d845157b888f83e86e8cb2bc779074725b95f3bfecb6`
- [worker-constructor.txt](worker-constructor.txt): `bf7d5e615f6347d88359f879cb4f7d2d4a9067ceecdd77927b04ae2abb1c711a`

## Store effects and completion policy follow-up (2026-11-04)

**Confirmed static**, same pinned APK as above. This resolves the previous
13z storage-call boundary for the selected 141/Fd9 result worker, not all upload
producer paths or cross-family atomicity.

- Fd9's nonzero selector calls 13z.A0X for the legacy array and 13z.A0W for
  Kyber. Each facade enters/exits a 0Wk.A04 scope and delegates to 0WJ.A0x/A0w.
  Those methods convert each ETs.A01 ID byte array through 1nr.A00; legacy
  invokes 0Yb.A03, Kyber invokes 0WJ.A0v. ID-decoder internals remain untraced.
- **0Yb.A03**, 0018–002a derives a seconds timestamp through 08q.A00 / 1000.
  Updates are grouped in at most 200 IDs (0036–0042): prekeys rows selected by
  prekey_id IN parameters get sent_to_server=true and upload_timestamp.
  0152–01be inserts a prekey_uploads timestamp record and calls 1hk.A00/close.
- **0WJ.A0v**, 0028–003a similarly derives seconds, updates kyber_prekeys in
  at most 200-ID groups with the same fields, then inserts prekey_uploads with
  key_type=1 (015c–01d6) and finishes its transaction scope. Full SQLite
  wrapper semantics are not rederived here. These two storage operations run
  sequentially before EcZ.A00 in Fd9; they are not proven one cross-family
  atomic transaction. Rust's single-store settlement transaction is local policy.
- Normal return from both mark-sent calls precedes the completion invocation;
  Fd9 contains no catch around those calls. A thrown storage exception therefore
  does not reach that success callback along this method. Full executor exception
  recovery and process-crash semantics remain open.
- **EcZ.A00** success=true (0018–00ca) calls 13z.A08 and either finalizes when
  both count-low flags are false or queues Fcd when either is true (031e–0344).
  Fcd's eventual work and callback construction/ownership remain untraced.
- Failure selects **EMZ.A02** through A17 at 00d0–00e8. With that flag false,
  406 logs regeneration and schedules Fea selector13; 500<=code<600 sets a
  flag and uses 0Vf.A01()*1000 for delayed Fd5 scheduling; other codes stop.
  This establishes branches, not Fea's actual key mutation or complete delay law.
- With that flag true, 406 calls 13z.A02(406): null stops, nonnull queues Fec
  selector22 with that delay. 500<=code<600 uses the same nullable helper, then
  records the last code, calls A08 and schedules Fd5 when a delay exists; other
  codes stop/reset. The flag's source/value, A02 retry bounds, scheduler units
  beyond the explicit multiply, and runnable behavior are still untraced.

**Rust implication / unknown:** ordinary successful legacy/KEM mark-sent before
follow-up is supported. Native failure behavior is not universally stop/no-retry;
Rust currently stops without reproducing these gated branches. Do not label that
full parity or derive a key-regeneration/reset implementation from log text.
Next trace targets: EMZ.A02 definition/value, 13z.A02 delay/bounds, Fcd/Fd5/Fec/Fea
selected branches and upload producer-to-EcZ captured state. No runtime change
or live traffic occurred. These are checked DEX TXT exports, not REA envelopes.

Additional export hashes:

- [store-facade-legacy.txt](store-facade-legacy.txt): `9251adb6cb74c2ad3fce155aa4668725acf88ef1b1ca1923afd46b1a66e3863e`
- [store-facade-kyber.txt](store-facade-kyber.txt): `4b0cc689b5b159c5634788515038bfcad71bb87357ae399a0e2d63e0276a6214`
- [store-coordinator-legacy.txt](store-coordinator-legacy.txt): `5794f7f6e7952da319b888bacc565ede21846222b97006203d3594de2fd4ba66`
- [store-coordinator-kyber.txt](store-coordinator-kyber.txt): `e9518ad96af75fad91dfb5a7353f5d9af350d3d7ee078688c7dc40639324df9b`
- [legacy-upload-write.txt](legacy-upload-write.txt): `089a5706fbf848ae3fbb5fcc028e26043a315d6007903978076a015df5d0589a`
- [kyber-upload-write.txt](kyber-upload-write.txt): `a129382aa4da80fc00b371b799864ba27a011f769fdb32163981c387671828ec`
- [completion-policy.txt](completion-policy.txt): `0241b2d1b32f46a863b4673debfbd8526b55b6c27c2eab431bebedee2cf17160`

## Retry limits and delay inputs (2026-11-04)

**Confirmed static**, pinned 43.72 as above; no runtime configuration observed.

- **EMZ.<clinit>**, 003a–004c constructs A02 with key **0x8b60** and false
  passed through 2L6.A0s to 00f.<init>(id,false,false). This is a declared
  descriptor input/default, not proof 00E.A17 returns false for an account.
- **13z.A02**, 0002–0022 returns null/reset via A09 unless code=406 or
  500<=code<600. The accepted branch sets A06=true under a monitor.
- If **A05=true**, it reads A00's old value, decrements it, then requires the
  old value >=1 (0034–0044). Therefore old=1 can still produce a delay with
  remaining=0; old=0 decrements to -1 then enters cleanup, not another retry.
- If **A05=false**, it reads 0Vf.A00, the current invocation counter, and
  refuses/cleans up at counter>=11 (0046–005e). Do not call this a universal
  eleven-attempt limit: counter initialization, other consumers and mode changes
  also matter. Cleanup clears A06/A05/A00/A03 and resets the backoff object.
- For code **503 or 406** with A05=true, delay is A0a(**0x374b**) *1000
  (007c–00ae). It does not invoke 0Vf.A01 in this branch. Other eligible cases
  invoke 0Vf.A01()*1000. No range/default for that configured integer is proven.
- **0Vf.A00** reads its invocation counter. **A01** increments it, then returns
  the prior A00 sequence field; when prior<=cap it advances the two sequence
  fields by addition. **A02** resets counter and previous to zero, next to seed.
  The constructor asserts positive seed/cap. **13z.<init>**, 017a–0190 binds
  seed=10 and cap=0x262 (610). This is a Fibonacci-style sequence, not an
  exponential or jittered law; the condition is not a hard min(value,610).
  A03 can reset/advance to a requested invocation index and is retained too.
- **Inferred from those operations**, starting from reset, successive A01 return
  values are 0,10,10,20,30,50,80,130,210,340,550 before the ordinary counter
  limit blocks another A02-mediated retry. These are helper values before the
  explicit *1000 in callers, not observed server wait requirements or proof of
  the number of sends, and the shared counter may already have advanced.
- Bounded A05 write inventory identifies 13z.A02, 14o.BLn (two writes) and
  FDh.CbF. Those producers and their A00 budgets are not yet traced; the mode
  must not be enabled by guesswork. Scheduler/worker behavior, A09 reset,
  runtime config value and flag evaluation remain separate questions.

This narrows the earlier completion-policy unknowns without implementing a
retry scheduler, regenerating keys, changing local failure settlement or sending
traffic. Exports below are DEX text, not REA Evidence envelopes.

Additional export hashes:

- [retry-delay.txt](retry-delay.txt): `1aa3144ac0d7aa1a6b2693b5c2b5edfa63a78c23fe46cb63430947cf98c6cbf1`
- [policy-flags.txt](policy-flags.txt): `6868dd5daf0c484f086fdfb58d4ad6b450c5e3e9bb0485f989cbff52ba040deb`
- [backoff.txt](backoff.txt): `1fe16d57e0cbacb6707834a4af954e76f64f78658cf9ef133e04ff3801352aeb`
- [limited-mode-writers.txt](limited-mode-writers.txt): `3e2e8081d2a7fb5008e9695646b48180315f07fba0bd135956541b6b065413b3`
- [flag-constructor.txt](flag-constructor.txt): `95640509d28e00f7f00fffc5d569127c738d6d03594a8292e9679b977ca52717`
- [manager-constructor.txt](manager-constructor.txt): `b9f2b70458589561ca4509b4270257d398506c3a23518e374500a2a3a955bb96`
