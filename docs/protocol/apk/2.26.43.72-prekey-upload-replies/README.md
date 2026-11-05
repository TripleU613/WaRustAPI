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

## Limited-mode producer boundary (2026-11-04)

**Confirmed static**, same pinned 43.72. This follow-up examines the A05 write
inventory, not every branch of the large retained 14o.BLn method.

- **FDh.CbF**, 005a–0066 reads allowance through A0a(**0x3741**). It also reads
  0mi.A0E and config **0x6f43**, builds FDj with the allowance/flags, and
  registers it via 13z.A0T. These inputs do not prove current account values.
- 00aa–00ba tests **13z.A0I Optional.isPresent**; only its present branch reaches
  00ca–00da, which under a monitor sets **A05=true**, **A00=allowance**, and
  **A03=the derived boolean**. The absent branch jumps to 011a, bypassing those
  writes. Therefore this is not unconditional upload/startup retry initialization.
- The present branch evaluates **EMZ.A01**, and its two visible continuations
  invoke Optional.get then construct/throw a NullPointerException labelled
  **deleteVNameCert** (00e0–0114). Do not interpret the label as a successfully
  executed deletion operation or assume these paths complete normally. Which
  product/flavor supplies the optional dependency remains unknown.
- The absent branch resolves 13z again, unregisters FDj through A0U, and resumes
  a result/callback path. FDj's independent event behavior and the wider lifecycle
  of A0T/A0U remain untraced; the CbF write alone does not prove normal retry mode.
- **14o.BLn** selector **77** (switch target0184), reads needsMoreLegacyPrekeys /
  needsMorePQPrekeys and clears pending arrays/state. At 01fc–0208 it sets
  A05=false, A00=0 and A03=false before further scheduling. This is a reset,
  not another allowance initializer.
- Selector **78** (target00ea), later failure path with **EMZ.A02 false**, has
  separate legacy handling: for 503 while A05=true it reads delay config
  **0x374b** and decrements the old allowance (0364–039c). Old<1 clears
  A06/A05/A00/A03 and cancels the scheduler path; otherwise delay scheduling
  proceeds. Thus 13z.A02 is not the only consumer of the limited allowance.
  Other failure branches and scheduled runnable effects are not fully audited.

**Bounded conclusion:** within the prior A05 field-write inventory, FDh.CbF is
its explicit true writer, while 14o and 13z clear it. Its conditional/exception
paths prevent treating this as a ready-to-port generic retry mode. Runtime
optional provider, config values/default registry, FDj callbacks and account
teardown still require tracing. No key reset, certificate deletion, retry
scheduler or runtime behavior was implemented. No live traffic occurred.

Additional export hashes (full methods retained, findings bounded above):

- [mode-worker.txt](mode-worker.txt): `d9824a2cb728862a92d87e2902c29da4f52bfae785b3bc081ecf994fe87e109f`
- [mode-callback.txt](mode-callback.txt): `dd9fec9235588808024e9e1b0a39ce1dab2c16bb9c345b9367d3e1e612e8a8f1`

## Successful continuation selects ADD (2026-11-04)

**Confirmed static**, same pinned 43.72. Findings are confined to the continuation
entry, generation dispatcher and selected request/callback construction blocks;
the full A0B method is retained for subsequent branch-by-branch review.

- EcZ success with either needs-more flag queues **Fcd** (031e–0342 in the
  retained completion-policy method). Fcd.<init> binds the reason and booleans;
  **Fcd.run** forwards them to **13z.A0B(manager,reason,legacy,pq,false,flag)**.
  Its fourth boolean (useSet) is explicitly false, not inherited from the upload
  that produced the result. This closes the prior Fcd entry boundary.
- **13z.A0B**, 0000–0012 first calls **A0D(legacy,pq)**. A0D conditionally calls
  0WJ.A0k for legacy and A0j for PQ generation under 0Wk scope. Generation
  internals/amounts were not retraced here; these are ensure-if-needed dispatches,
  not unconditional key rotation.
- A0B uses config **0x52d1** for requested PQ count only when pq=true; otherwise
  zero (branch0a46). It creates a fresh EcZ completion carrying flags/reason.
- useSet=false branches to **06d8**, the ADD path. No legacy request and
  pqCount<=0 logs no prekeys requested and clears the upload guard; ADD mode
  marks A0V=2 and uses **13z.A0O -> 141** as IQ callback.
- With legacy requested, **0712–0742** reads **EMZ.A03** through A0e and clamps
  the batch limit to max(1,value). The prior EMZ initializer binds key **0x74cb**
  and declared default **200**. Actual runtime value is not established. It
  obtains legacy candidates through **0Yj.A02(limit)**. When legacy=false it
  creates an empty array; PQ candidates are separately limited to min(requested,
  available). Full selection SQL/eligibility remains an open boundary.
- Empty combined candidates do not send. With candidates, **0836–0864** adds
  **<op mode="add">**. The selected tail emits encrypt/set IQ to the existing
  server JID, including legacy list and/or pq_list. This does not by itself
  establish every ADD child/attribute order or ID/signature producer.
- **0a00–0a18** freezes legacy/PQ arrays and this EcZ in 141 under its monitor;
  **0a1a–0a3c** calls 0Sf.A0W(callback141,node,id,logicalType0x1da,delay0).
  The zero delay is a shared-dispatch argument, not proof of unlimited feature
  timeout or durable replay permission. SET path separately uses callback140
  and logicalType0x1d9 (068a–06ce).

**Confirmed Rust gap:** PendingUpload follow-up currently reuses the older
37.71 full SET constructor/batch policy (812 cap) via maybe_upload. Current
43.72 success continuation explicitly selects ADD with a configured legacy
batch limit, so these cannot be called equivalent. No speculative op insertion
or automatic policy switch is made in this evidence-only change. Before porting,
trace ADD candidate SQL, exact full child/attribute shape, generation parameters,
initial-vs-continuation admission and persisted batch compatibility, including
PQ material and retained uncertain reservations. Native producer/callback linkage
is now partially closed; all native upload paths remain far from fully audited.
No production behavior or live traffic changed.

Additional export hashes:

- [success-continuation.txt](success-continuation.txt): `f75f193a076c53cba5d4973a48bf8c6f7722a270c98a1242dc439f57c70a14d2`
- [success-constructor.txt](success-constructor.txt): `a024ab523e19a6507217f516a60a7962520146c9dc1fdd1b9d6fa52656f35577`
- [continue-manager.txt](continue-manager.txt): `0914da61d2b4bbd955720c126a5b2d1b327442456c1ec96d2df4e3dab5dca7ce`
- [generation-dispatch.txt](generation-dispatch.txt): `9f699fab4c95a2370b4087a06ec87b9668ec5dffc06bf04500e7cc2101e0f727`

## ADD selection and exact child layout (2026-11-04)

**Confirmed static**, same pinned 43.72; no runtime or wire capture.

**0Yj.A02 -> 0Yb.A01:** the latter queries exactly
`SELECT prekey_id, record FROM prekeys WHERE sent_to_server = 0 AND direct_distribution = 0 LIMIT ?`
with the supplied limit as its single parameter. There is no ORDER BY in that
SQL; do not infer native ascending-ID order from the returned array. 0Yj parses
each saved Ddm through EEz/ESl/Fbl, removes the first serialized public-key byte
via arraycopy, and constructs ETs(id,value,null). Caught IOException logs/skips
that row; other exception classes are not caught by this method. Rust's stricter
corrupt-row refusal remains local policy, not native skip equivalence.

**1nr.A04** outputs a 3-byte big-endian low-24-bit ID: shifts16,8,0 into the
array. This matches the existing legacy wire ID width; it does not independently
establish generation/wraparound policy.

**13z.A0B ADD branch**, retained in continue-manager.txt:

- 0836–0864: first child `op` with sole attribute `mode=add` and no payload.
- 0864–08e8: omit legacy `list` when its array is empty; otherwise each ordered
  `key` contains `id` bytes then `value` bytes, no signature. Keys follow the
  selected array order, not a newly sorted wire list.
- 08e8–08ee and 0972–09fc: omit `pq_list` when empty; otherwise each `key`
  contains `id`, `value`, `signature` from ETs fields, in that order.
- 08ee–096e: parent attributes are **id, xmlns=encrypt, type=set, to=0mL.A00**
  in that order, followed by the assembled children. 0Sf.A0G supplies id before
  node construction (0810–0824). ADD builds no identity, registration, type-byte,
  skey or pq_last_resort_key child in this branch. Those SET-only fields must
  not be copied into ADD merely because the outer IQ type is also set.
- 0a00–0a3c freezes exact selected arrays in callback141 before handoff. Empty
  combined arrays take the no-send/guard-clear path at079a–07ac.

**Porting readiness:** ADD layout and legacy selection filter are now closed at
this source boundary. Remaining inputs before replacing Rust continuation are
current generation parameters/availability, supplied batching-config provenance,
PQ selection/admission and durable mode/limit records that never rewrite old
SET reservations. This trace does not supply a runtime remote-config value and
is not justification to change initial upload policy together with continuation.
No production behavior or live traffic changed.

Additional export hashes:

- [add-selection.txt](add-selection.txt): `8638c79ff033e0c1ee2ca0ff30848be4a095d7a57139d613fca15f39fec055ff`
- [add-sql.txt](add-sql.txt): `f28abbda7c8aea030aecacdd3c062f870882137653de8e49be9544ec9ba1fc15`
- [add-id-encoding.txt](add-id-encoding.txt): `56c82fe43c2043ccfc64976d164df83b9f415f23dbb9eaf51a01780395316fdc`

## Legacy generation loop and cursor convention (2026-11-04)

**Confirmed static**, same pinned 43.72. This exposes an additional Rust parity
gap; it does not migrate existing keys or change runtime generation.

- **0WJ.A0k** enters 0Wk scope and calls **0Yl.A07**. The latter obtains the
  unsent count through **0Yb.A00**, whose SQL excludes sent_to_server and
  direct_distribution rows. Count>=812 returns without generation.
- **0Yl.A07**, 0046–0058 computes remaining deficit then a chunk of min(deficit,
  50). Crucially, its final **01ec goto/16 -0xd3** targets **0046** (DEX branch
  delta is in code units: 0x1ec - 2*0xd3 = 0x46). This is not a return after
  one chunk. The remaining deficit is reduced by the preceding chunk and each
  next chunk rereads next_prekey_id. Normal execution fills the initial deficit
  to 812 in chunks of at most 50. With initial count0 this implies 16*50+12,
  **17** chunks, not 50 keys total. This is static control-flow inference,
  not observed execution or a claim about concurrent writers/failures.
- **00c0–00da**: for saved native cursor N and chunk offset i, ID is
  **((N-1+i) % 0xfffffe)+1** using Java signed int arithmetic. This differs
  from Rust's local zero-based cursor formula ((next+i)%modulus)+1.
- **01d4–01ec** computes saved next as **((N+chunkCount)%0xfffffe)+1**, then
  calls **0WJ.A0u(records,next)** before looping. These exact expressions are
  retained rather than silently normalized. Cursor range/default and overflow
  preconditions still need the identity-row getter/initializer trace.
- **0WJ.A0u**, 0044–00dc saves each generated record with sent_to_server=false
  and direct_distribution=false; 00dc–00ee forwards the supplied next integer
  unchanged to 0YT.A06 for next_prekey_id, within its transaction scope. The
  SQL wrapper's insert collision semantics and complete identity binding remain
  untraced; no overwrite permission is inferred.
- **Rust gap:** prekeys.generate currently generates only min(deficit,50) per
  call and advances a local zero-based checkpoint. Existing encrypted stores
  therefore do not contain native cursor semantics. Changing the meaning of
  that checkpoint in place would reinterpret history and risk collisions.
  A current-build port needs explicit generation strategy/version ownership,
  preserved old uncertain batches/keys, and collision refusal retained.

This supersedes any assumption that a 50-key Rust call establishes current-build
generation parity. The older 37.71 documents/source comments keep their original
build attribution; no full older-build retrace was performed here. ADD generation
amount is now traced; native initial cursor/provider and PQ generation/admission,
configuration and durable migration still remain open. No live traffic.

Additional export hashes:

- [ensure-legacy.txt](ensure-legacy.txt): `9cf364892571b10a35a1677020aca7b066c536b5bc65bf6737312487ae1bbc04`
- [generate-legacy.txt](generate-legacy.txt): `553e858708b1432e1bfa05a17cf52a915a9ec460638a3010b20356f148ff6908`
- [unsent-count.txt](unsent-count.txt): `4bd4b68b7677ee496bc17fc7202fa900053e803c7c0148e5b15770476cb69fe4`
- [generation-save.txt](generation-save.txt): `9ab858339a3ff714a9f8c59c8a1c8cc8252013096cc96023da4e23814031a82a`

## Native cursor ownership and creation seed (2026-11-04)

**Confirmed static**, same pinned 43.72; no existing store was reset or migrated.

- **0YT.A01** binds the generator's supplied identity query to self tuple
  **recipient_id=-1, recipient_type=0, device_id=0**. It reads the requested
  column through Cursor.getInt and returns that saved value; no in-method
  normalization/default is applied. Missing self row throws SQLiteException
  **Missing entry for self in identities table**. It does not generate a new
  cursor or treat the missing row as zero.
- **0YT.A06** writes the supplied integer to the supplied column using that
  same self tuple. It does not normalize the cursor either. Updated-row count
  is not checked in this wrapper; full SQLite binding/lifecycle remains open.
- The exact **next_prekey_id** xref has six references: generator0Yl.A07,
  saver0WJ.A0u, creator0Yg.A02, JniBridge.jnidispatchIO, 1j8.ATc and
  28o.onUpgrade. The JNI/migration paths were not audited here, so this is not
  a claim every cursor writer follows creation semantics.
- **0Yg.A02**, 00d6–00f4, seeds next_prekey_id with **0Yg.A00()+1**; a separate
  invocation similarly seeds next_kyber_prekey_id (00f4–0112). It inserts the
  identity row through SQLiteDatabase.insertOrThrow at012a–0132. This is an
  identity-creation branch, not evidence of ordinary per-login reseeding.
- **0Yg.A00** uses SecureRandom.getInstance(SHA1PRNG).nextInt(0xfffffe),
  returning an integer in [0,0xfffffe). Adding1 yields cursor **1..0xfffffe**.
  Algorithm-unavailable throws AssertionError. This establishes the range and
  independence of the two draws, not observed randomness or runtime provider.
  An earlier 0x7ffffffe draw in A02 serves another field and is not this seed.

**Compatibility conclusion:** native positive cursor N means a first generated
ID of N (per the preceding 0Yl formula); Rust's saved zero-based cursor c means
first ID c+1 modulo its existing convention. Do not simply mark the existing
NEXT_ID_KEY as native or replace it with a random seed. An explicit versioned
strategy can retain old record/batch IDs and carry a separately owned native
cursor, with collision refusal and no automatic account identity rotation.
Exact migration policy is not implemented by this research. Native schema/JNI
imports and signed/PQ bootstrap remain open. No production change or live traffic.

Additional export hashes:

- [cursor-A01.txt](cursor-A01.txt): `87ab111326c68d4f3711df4f2b88273957a3049479a774916e200a8d07a3014e`
- [cursor-A06.txt](cursor-A06.txt): `eff4cb8c800e2b50602cf8522be64f1d93e820fea39c3863360c46135c2d42e1`
- [cursor-xref.txt](cursor-xref.txt): `f3780f5da2bd5b2d70040a912386290d00b59ac222218508fa0fa05df0036c32`
- [cursor-initialization.txt](cursor-initialization.txt): `fd9e6ca9522e1702ee489bf0f26e83ce287ff786886080afe2b5545392e2b3ac`
- [cursor-random.txt](cursor-random.txt): `f33beedb4a9b05bab2ccc69b56d4fdc9484dd492ddb8378fb15ba647e7e1c2a4`

## ADD ordinary-PQ selection and generation dispatch (2026-11-04)

**Confirmed static**, pinned 43.72 as above; not runtime observation.

- **0WJ.A1A**, 0010–0038, reads **0x52d1**, clamps its retrieval limit to
  max(1,value), and calls **0Yd.A02**. That SQL selects kyber_prekeys with
  **sent_to_server=0 AND direct_distribution=0 AND last_resort_key=0 LIMIT ?**,
  with no ORDER BY. It therefore selects ordinary PQ keys, not last-resort.
  The enclosing ADD builder separately applies min(requested,available), and
  only invokes this retrieval when its requested PQ count is positive.
- A1A parses each row through **EqE.A01 -> 0Yj.A00** to ETs; caught Exception
  logs/skips that row. Conversion/signature producer internals are not retraced
  here; a malformed row is not proof of different algorithm negotiation.
- **0WJ.A0j -> 0Yl.A06** ensures ordinary PQ generation. A06 counts the same
  unsent/non-direct/non-last-resort population. Target is **0Yj.A00**, supplied
  as the final integer constructor parameter (0Yj.<init>0052–0056), not a literal
  812 in this method. The provider/default of that constructor argument remains
  untraced. Count>=target skips generation; otherwise deficit=target-count.
- A06 checks native **WhatsAppLibLoader.BbM**. Missing library logs/reports and
  skips; UnsatisfiedLinkError around generation is caught/reported. This is
  actual gating, not permission to silently omit required PQ fields elsewhere.
- It obtains identity through **0Yj.Anw**, reads positive native
  next_kyber_prekey_id, and generates min(deficit,50) records per chunk through
  **EqE.A00(identity,KEMKeyType.A02,id)**. Exact enum/algorithm and signature
  implementation are not established by the enum field name alone.
- ID expression is **((cursor-1+i)%0xfffffe)+1**, next cursor is
  **((cursor+chunkCount)%0xfffffe)+1**. It saves records through
  **0Yd.A01(...,null,records,0)** and next cursor through 0YT.A06, then reduces
  deficit (0294–029e). If positive, the backward branch returns to018e for
  another <=50 chunk; otherwise it returns. No last-resort-generation call
  occurs in this selected ensure method. Full save-flag semantics remain open.

**Porting consequence:** ADD can carry ordinary PQ without last-resort material;
SET and ADD must not share an unconditional last-resort prerequisite. This does
not establish native target value, algorithm negotiation or signature parity,
and does not implement a Rust policy change. Remaining concrete inputs are PQ
target DI provider, EqE/0Yj conversion signing path and current-build initial
upload admission/config scope. Existing uncertain batch mode and IDs must stay
unchanged during any port. No production change or live traffic.

Additional export hashes:

- [pq-A0j.txt](pq-A0j.txt): `acd09a632b5d10d1d8e1a32961e2bbc6b8ff44b0377756ad9f33612b3393bffc`
- [pq-A1A.txt](pq-A1A.txt): `29845c40d2ad7529466c29b4d4917e17f008af8658a7a10c8e74058468743543`
- [pq-generate.txt](pq-generate.txt): `8396a8f8a77ba85979664a3e67321dc4298eef056bbf367821e384f6a3289e97`
- [pq-select-sql.txt](pq-select-sql.txt): `513ae1e2069257144d2505856e5015a21d511c9c91184745e24be337283c294d`
- [pq-target.txt](pq-target.txt): `f7a384316b86f180773159089b55439ca42e4a4685b4d0cbd3860992bfbcd5b0`

## PQ signing and wire-byte provenance (2026-11-04)

**Confirmed static**, same pinned 43.72. This closes Java-side provenance, not
the native KEM algorithm implementation or runtime byte lengths.

- **KEMKeyType.<clinit>**, 002c–0044 maps **A02** to **ML_KEM_1024**, ordinal2,
  constructor code10. The other entries are ML_KEM_512/code6 and ML_KEM_768/code7.
  These constructor codes are not automatically wire-prefix bytes.
- **EqE.A00**, 0000–0024 calls **KEMKeyPair.generateNative(type)**, reads its
  **publicKey.A00** byte array, and passes that entire array as message to
  **EMC.A00 -> 1oA.A03(identityPrivate,publicBytes)**. No ID concatenation,
  prehash, type-byte removal or domain-prefix construction appears in this method.
  It stores the same public array and generated signature in Dfd along with
  private bytes, ID and System.currentTimeMillis timestamp.
- **1oA.A03** requires a32-byte private key, requests64 random bytes, and delegates
  **calculateSignature(random,private,message)** to its **1oB** provider.
  The concrete provider/native signature primitive remains outside this trace;
  Java input provenance is established, not full cryptographic equivalence.
- **0Yj.A00** reconstructs a KEMKeyPair from saved public/private bytes, reads
  publicKey.A00 and the saved Dfd signature, then constructs ETs with3-byte ID.
  Unlike the legacy converter it contains no arraycopy stripping a type byte.
  ADD passes those ETs bytes directly to value/signature (previous layout trace).
- **KEMPublicKey.<init>** simply retains the supplied array. **generateNative**
  has no DEX body (native); access$generateNative delegates to it. Therefore
  this Java trace alone does **not** establish public byte length, native secret
  serialization, wire prefix0x44 or the mathematical KEM implementation.

**Bounded Rust comparison:** prekeys.kem_upload_children signs row.public and
uses the same bytes as value; this matches the observed Java dataflow shape.
The local signal/kem generator produces1568-byte ML-KEM-1024 public bytes with
its expanded secret representation, supported by separately retained
[native vector work](../../native-kem-vectors.md). No byte-prefix change is
justified solely from KEMKeyType's numeric code. Current ADD/generation policy
and provider/target parity remain unfinished; there is no new wire-format or
signature patch in this research chunk. No live traffic.

Additional export hashes:

- [pq-signing.txt](pq-signing.txt): `647531382bc55b0502ebdaf8e038ddea897eb87075c870707c5c45f090bf5837`
- [pq-wire-conversion.txt](pq-wire-conversion.txt): `1c536b4ca6fa1618063c6eacde419cf4abae03f5c35dc25be84fc0c10bf48154`
- [pq-type.txt](pq-type.txt): `bd6d690f47528dbcd31ad7dc0eb064ac7f4fa3650ae523f40b143ec130673515`
- [pq-sign-function.txt](pq-sign-function.txt): `e7d48499f1fac433f1e5f8c507506aea186e7a776b87b35478ae6e82338aa2c2`
- [pq-native-wrapper.txt](pq-native-wrapper.txt): `18cd6927f4618721fb4ff5a2ad719c30efd31dcb37510a75ebb46bf5507e1d41`
- [pq-public-wrapper.txt](pq-public-wrapper.txt): `d821fe2b941dcbf2546032811b92660885151060224c142b161989751fd9fb89`

## PQ target provider and declared defaults (2026-11-04)

**Confirmed static**, same pinned 43.72, reusing an existing matching-build REA
record rather than duplicating its method export.

- Exact 0Yj.<init> caller search finds **0WJ.<init>**. At **0170–017c** it
  reads integer property **0x52d0 =21200** through 00E.A0a; the result flows
  unchanged in v36 to **0Yj.<init>** at0260–0266, which stores its final argument
  in A00 (prior pq-target export). This closes the generation-target DI chain.
- The matching APK default registry **08Y.A0Q** contains **21200 ->200** and
  **21201 ->10**. Existing REA evidence
  [ev_135fccfdf04c5e17931490728a40bf7deb18d2732c6418d1a17bf53553c6fc38](../../rea/2.26.43.72-privacy-mex-config/27-X-C08Y-A0Q.json)
  has subject SHA256 matching this pinned APK. Its method body was inspected
  directly for both entries. No new REA envelope or fabricated coverage added.
- Therefore ordinary PQ generation target is property21200 (declared default200),
  while requested/retrieval batch size is property21201/0x52d1 (declared default10).
  These are distinct controls, not a shared10-key inventory target or812 legacy
  target. Runtime overrides remain possible; no live effective value was read.
- 0Yj captures generation target at coordinator construction. Its per-call batch
  selection reads current config through A1A/A0B. Constructor capture does not
  prove automatic reconstruction after config changes; that lifecycle is open.

**Rust comparison:** kem_prekeys.upload_rows_in currently fills only its selected
ordinary batch count, default10, alongside last-resort generation. That older SET
strategy does not reproduce current native ordinary inventory target200 or ADD's
non-last-resort replenishment. The two controls need separate versioned runtime
handling rather than changing a single default from10 to200 (which would also
change transmitted batch size). Existing rows/uncertain batches remain untouched.
This evidence closes target source/default while leaving effective config and
current-build initial/continuation admission for the implementation stage.
No production change or live traffic.

Additional export hashes:

- [pq-target-callers.txt](pq-target-callers.txt): `3eb0c339e9913def990e195ea39cae3670de6b2ee3c9da9b3b6d99ccec5d258c`
- [pq-target-provider.txt](pq-target-provider.txt): `f224b42c70d8419ab934ca54b030bd811f1b3bac9677e950737c0b7e6007bbcb`

## SET/ADD entry admission and migrated state (2026-11-04)

**Confirmed static**, pinned 43.72. This closes manager-level entry predicates;
upstream registration/startup/notification routing is not yet fully traced.

- Exact **13z.A0B** callers are Fcd.run (known successful ADD continuation) and
  **Fd5.run**. The latter passes stored reason plus four booleans unchanged.
  Its constructor's extra integer **$t** is stored but **not read by run**;
  do not invent dispatch semantics from that integer alone.
- Fd5 constructor callers: **13z.A0R**, **13z.A0V**, **EcZ.A00**, **Fec.run**.
  This is an exact concrete-call inventory, not proof of all reflection/native
  entry paths. The Fec method remains untraced in this chunk.
- **13z.A0R(reason)** first requires **Ex8.A04**, then shared A0E guard with
  null token/false override. It queues Fd5 with **legacy=false,pq=true,useSet=true,
  flag=true**. This is a PQ-only SET entry, not unconditional generic startup.
  A0R's upstream callers remain to be traced.
- **13z.A0V(legacy,pq)** rejects bothfalse, calls A0E(null,reason2,false), then
  requires **Ex8.A03** (logged isPQMigrated). Non-migrated branch logs unexpected
  and clears guard through A07. Migrated branch ensures the requested key
  families, checks available legacy candidates through A1B when requested, and
  queues Fd5 with **legacy=available,pq=input,useSet=false,flag=false**. Thus
  this manager's low-count entry is ADD and depends on migrated state.
- **Ex8.A04** reads Boolean property **0x52ce=21198**. **Ex8.A03** first requires
  A04, then returns **0WJ.A0z**. That method executes
  `SELECT EXISTS(SELECT 1 FROM kyber_prekeys WHERE last_resort_key = 1 AND sent_to_server = 1)`.
  Migrated here therefore means the gate plus any sent last-resort row, not
  merely the presence of any PQ key or an assumed account-wide marker. An empty
  cursor result logs query failure and returns false; wider DB errors propagate.
- **13z.A0E** checks transport0AI.A06, an inflight predicate A0Y(uptimeMillis),
  and, absent caller override, blocks when A06 backoff flag plus config0x47af.
  Accepted path records uptime through A0C and stores reason/token metadata.
  A0Y/A0C expiry law, encoded02X return identities, full backoff lifecycle and
  upstream ready/account binding remain open. Rust durable uncertain reservation
  guarantees are not replaced by native in-memory timeout behavior.

**Porting impact:** a future ADD implementation can bind mode to successful
continuation and the migrated low-count scope, while retaining distinct PQ SET
entry. It must not infer that all count notifications or all initial uploads
use ADD, nor infer migration from unsent PQ existence. Rust currently lacks this
current-build manager strategy. This is evidence, not an implementation or
permission to mark keys sent locally to enter that scope. No production change
or live traffic; existing account state and uncertain requests remain intact.

Additional export hashes:

- [upload-entry-callers.txt](upload-entry-callers.txt): `2114d4fe78b4ef265744b48cefa63e561f70ec44efc5269243a14a50b037cfe4`
- [upload-entry-worker.txt](upload-entry-worker.txt): `78462946e9fc7f74847c8884795f9f40712429543615113b9b5e160ddbebc61e`
- [entry-constructor-callers.txt](entry-constructor-callers.txt): `b5eacfc74d62ff3187bb0e590478c680292f94a5aeee009959e6e2b41130872a`
- [entry-constructor.txt](entry-constructor.txt): `c77822f0bd8f78bedef59ec6615e4e2758b419593e6ba11a0687389a1c97aedd`
- [entry-A0R.txt](entry-A0R.txt): `301db8f27f46668ac076d05bcd60fc69151cf499172796100e4901c41a452e34`
- [entry-A0V.txt](entry-A0V.txt): `b97439a348d2761e5c5015af2aa64987e05b58e2015d0a105344af8262bb749e`
- [upload-guard.txt](upload-guard.txt): `a3c3d5b7c5b621302453b6182fed43388ed005790a0bc3e80d240bde6c8b3a72`
- [migrated-predicate.txt](migrated-predicate.txt): `df4a05e923ac0cc11fb2a20f1d473f1b1b840d7d0bdda8a1522b074411322a5a`
- [set-admission.txt](set-admission.txt): `e58a2f2a9c7035d5d1f3246592541f3a90411141dd19ea3441fac3acb9ca1920`
- [migration-state.txt](migration-state.txt): `ddaaa0effc16926ccff56b9c6bdaf28557b70ea58f835e0adaa58199c8b9cb2a`

## Notification and daily PQ entry routing (2026-11-04)

**Confirmed static**, pinned 43.72. This adds upstream context; startup Fea
branches and all cron/job lifecycle are not yet fully audited.

- Exact A0R callers are **FKS.onRandomizedDailyCronNoMessageStore** and
  **Fea.run**. Exact A0V callers are **15v.A02** (two invokes) and **Fc8.run**.
  Fc8's default selector branch simply forwards stored two booleans; which
  producer constructs that selector remains open.
- **15v.A02** is the count worker. It combines **Ex8.A03** migration with
  **Ex8.A04** gate21198 and **EMv.A00** before selecting branches. EMv.<clinit>
  registers A00 as **0x70d7**, declared defaultfalse. Current runtime value and
  descriptor evaluator behavior were not observed.
- Its labelled migrated/PQ-upload-enabled branch at01d6 passes **legacy-present**
  and **PQ Integer nonnull** to A0V, not a numeric low-count threshold. Zero or
  negative nonnull PQ counts still mean presence at this call site. The labelled
  PQ-low-count-disabled branch at01a0 invokes A0V(true,false) only if legacy is
  present; PQ-only skips. Raw02X branch encoding is retained rather than turning
  every obfuscated constant name into a guessed enum ordinal.
- The non-migrated branch logs useSET. With legacy-present it checks A10 unsent
  availability, calls A0k if none, then A0S(reason2). PQ-only logs no action.
  A0S's request/legacy upload path remains the next trace boundary; this method
  does not send every count notification through modern A0B/ADD.
- **0232–0244** calls stanzaMetadata.A01 for ACK after the worker's upload
  decision, or logs missing metadata. This is call order, not a guaranteed wire
  order relative to an asynchronously scheduled upload. Rust currently ACKs
  before its generation/upload decision; that known ordering gap remains.
- **FKS daily cron**: gate-off branch detects existing Kyber state through A0y
  and schedules DeleteKyberPreKeysJob (selected job body not traced). Gate-on
  branch checks A0Y(inflight) and migrated predicate; when not migrated, it
  ensures ordinary PQ through A0j and invokes A0e (last-resort getter/generator
  internals not retraced here), then **A0R(reason10)**. This links daily missing-
  last-resort migration to the guarded PQ-only SET entry, not all initial uploads.
  No deletion/reset/scheduler behavior was implemented in Rust from this trace.

**Implementation consequence:** account migration/gate plus notification policy
choose legacy SET versus ADD, with distinct daily PQ setup. Rust's unified
maybe_upload currently omits those branch distinctions and PQ-only notification
handling. This adds required porting inputs; it is not evidence to blindly
switch all notifications to ADD. Upstream raw notification dispatch/parser,
A0S/Fea startup paths, EMv effective values, non-migrated signed-key bootstrap
and native ACK durability remain open. No production change or live traffic.

Additional export hashes:

- [upstream-A0R.txt](upstream-A0R.txt): `c0fa2c4d26e454dfda84c9e7cf366a18ac6277228a29debc601fabe97dffea14`
- [upstream-A0V.txt](upstream-A0V.txt): `936bfa2b80c65347c535bb15e88634d2faae937c1d1dccb42381d376c0b1522c`
- [count-worker-routing.txt](count-worker-routing.txt): `520aaa0e19c522f1d23a388d9d787a5aa30770015e977845d300f833a8c2edd1`
- [daily-set-entry.txt](daily-set-entry.txt): `9eeabb113a9f5b16bed4acd708fa5db476e1c3af6e98f42a1f9ce137218c2184`
- [count-async-routing.txt](count-async-routing.txt): `c79e8b7caa754200672f727530cca078ebd2dfe2b60cdd10553e1c4b0b97d33c`
- [count-policy-flag.txt](count-policy-flag.txt): `ee6707c2b42e9f43caafecbc55c4c82a83e1ff9aa628cd42df1e042d9a407279`

## Legacy SET snapshot and message85 handoff (2026-11-04)

**Confirmed static**, pinned43.72. This closes the previously open A0S entry
through its descriptor handoff, not the final message85 writer.

- **13z.A0S(reason)** calls A0E(null,reason,false). On admission it calls
  A03(callback=null,manager,token=null,other=null,true,reason==1). It is distinct
  from A0B's callback140 SET and callback141 ADD constructors.
- **13z.A03** checks09F.BU7 and optional transition callback handling, records
  A0W/A0b and sets A0V=1. Inside 0Wk scope it snapshots registration via0YT.A08,
  identity public bytes via0WJ.A19, legacy unsent array viaA1B, signed key viaA0c.
  Full getters/selection limits and transition callback semantics remain open.
- It reads Ex8.A04/21198. If gate or its final reason-dependent boolean is true,
  it fetches an unsent last-resort throughA0b; if absent, it can log skipPQ.
  With last-resort present it obtains ordinary PQ throughA1A, invoking ensure
  A0j first only when the final boolean is true. Thus this builder has an explicit
  caller-driven override/ensure distinction; do not reinterpret it as changing
  the server gate or enabling arbitrary PQ upload. Reason1 upstream remains open.
- It encodes registration via1nr.A03 and schedules **Fdv** carrying those frozen
  values/arrays/signatures and its boolean. No node bytes are built in A03 itself.
- **Fdv.run** calls Optional.A01 and00L.A01 before proceeding; their meanings
  are not retraced here. On normal return, the carried boolean conditionally
  calls13z.A09, then a monitor sets A04=true and saves legacy/PQ arrays and
  last-resort in13z.A08/A07/A02. These are in-memory reservations, not durable
  encrypted-store guarantees.
- It builds **EXb** from signed/last-resort keys, strings, identity/registration
  and arrays, then **Message.obtain(...,what=0,arg1=0x55,arg2=0,obj=EXb)**.
  It invokes **0Sf.A04(message,null,transport,null,delay0,false)**. Final writer
  layout and legacy reply-event routing remain untraced in this follow-up.

**Porting impact:** a complete current-build strategy has both legacy message85
SET and modern A0B SET/ADD paths. The older Rust full SET layout resembles the
legacy path, but this does not prove all flags, prerequisites or callbacks match.
Current ADD port cannot replace the legacy branch indiscriminately. Exact getter
selection, reason1 source, assertion/provider behavior and final writer are the
remaining concrete legacy-path questions. No runtime change or live traffic.

Additional export hashes:

- [legacy-set-entry.txt](legacy-set-entry.txt): `e7ac4860e525bb00efc0e3f89b0a372d5fc1fcb868b825c08fde0524f8e4a6f6`
- [legacy-set-builder.txt](legacy-set-builder.txt): `b1a950feb704b17cff5442f53d9bd985af30284de905cc7b4811d91b1cd94c00`
- [legacy-set-handoff.txt](legacy-set-handoff.txt): `8f76d7f0c73bfe56c5cead182b7d427daad8ae280653be97a39c62fdde64a835`

## Legacy writer shape and success-event path (2026-11-04)

**Confirmed static**, pinned43.72, reusing the already retained full
[0v0.A00 writer](../../rea/2.26.43.72-reachout-state/reachout-iq-write-admission.txt)
(SHA256 `6ec286521e703b2e16b3ff057c27e744fe5835683b8edd815cfeefec12c32d3d`).
No duplicate huge writer export or new REA coverage is added.

**Correction to preceding handoff note:** Fdv obtains Message with **what=0,
arg1=85,arg2=0**, not what85. Writer0v0.A00 reads **arg1** at00ce and its
sparse switch selects case85 ->12be. The corrected argument mapping is reflected
above; raw exports were unchanged. Success event likewise uses what0,arg1=77.

- **12be–1334** reads EXb, allocates an ID using incremented1rj.A00 converted
  by Integer.toHexString, and stores **290(owner,registrationBytes)** in
  **1rj.A03** map under that ID. This is separate from the shared0ln IQ callback
  registry; its complete matching/removal/error lifecycle remains untraced.
- **1342–13a6** builds legacy key{id,value}. **13a6–1460** prepends op(mode=set)
  when property0x5bf2 is true OR both reason/coex-token strings are present;
  only the paired-string case includes reason and coex_token child. Current
  runtime flag and token-authority policy are not established by this writer.
- **1460–1528** appends identity, registration, type byte05, list and
  skey{id,value,signature}. **1528–1608** conditionally appends ordinary
  pq_list and last-resort key with{id,value,signature}. **1608–1684** constructs
  outer attributes id,xmlns=encrypt,type=set,to=0mL.A00 in that order.
  **1684–168e** calls1r0.Cyt(node), followed by write/set-pre-key log routing.
  This closes the selected legacy field-shape boundary, not full wire output.
- **290** extends **1ru** and captures owner/registration in its constructor.
  Inherited **1ru.A02** reaches its290 branch0340–039e, reads optional first
  list/pq_list count_low using case-insensitive true, then calls1lZ.A1Q(legacy,pq).
  The rest of this shared parser is retained but not fully audited here.
- **1lZ.A1Q -> A0c** constructs **what0,arg1=77,arg2=0**, setting bundle
  needsMoreLegacyPrekeys/needsMorePQPrekeys only when at least one is true,
  and submits through1lW.CPm. This matches14o.BLn selector77 and its flag reads;
  concrete event-consumer dispatch and legacy error-event path remain open.

**Rust comparison / remaining work:** broad full-SET fields match the legacy
writer; op gating/coexistence authority, hex-ID allocator/lifetime, callback/error
routing and initial reason ownership remain incomplete. Modern ADD remains a
different request mode and continuation policy. No production change or live
traffic. Do not claim shared0ln callback behavior alone proves legacy routing.

Additional export hashes:

- [descriptor-callers.txt](descriptor-callers.txt): `3633e9230cfceaf14ea81f4d470d9ad9c2626c67f6c5ece65d720f409c89b8df`
- [set-writer-xref.txt](set-writer-xref.txt): `baef2d62d1b34d9cb7c37a692fbb0e9903f3b7d38301f85e9b7297688bf83431`
- [legacy-reply-callback.txt](legacy-reply-callback.txt): `2beed47b26ceb88566226ffa2f2fd38a05aece9e0a9a7c4299731e691a006d16`
- [legacy-reply-parser.txt](legacy-reply-parser.txt): `b5f8f8f5f62bb9bc5cefdaabe45cbb926572de3f5bb376d1668cb296ba682fa6`
- [legacy-success-event.txt](legacy-success-event.txt): `22e2d9a34c65835642f3fd1cd5474e4f8d87535455b668fc8fcc4019aeb534d5`
- [legacy-success-message.txt](legacy-success-message.txt): `795ee5f04496745f054701f806be8842561a5511435f806094bb26d7685dd1f4`

## Legacy reply removal and error-event path (2026-11-04)

**Confirmed static**, pinned43.72. Findings concern the selected result/error
branches, not every unrelated branch of the retained dispatcher method.

- Exact1ru.A02 caller search reaches **1rA.A03**. Its type=result branch
  **0040–0078** requires ID, removes the ID from **1rD.A00** map, invokes
  found1ru.A02, then forwards the node/ID through1lZ.A1E. This establishes
  removal-before-legacy-result callback in that path; shared0ln pending state
  is not interchangeable with this map. Writer-map to reader-map constructor
  alias and all teardown/error paths remain to be bound explicitly.
- The exacttype=error branch **007a–00ee** removes the map entry; for **290**
  it calls the already retained **1ud.A01** coded-error extractor, defaults
  missing pair to numeric0/textnull, then **1lZ.A0r(code,text)**. This is a
  special node-aware path, not generic1ru.A01 integer callback handling.
- **1lZ.A0r -> A0X** places errorCode as int and errorText as string in a Bundle,
  constructs **Message(what=0,arg1=78,arg2=0,obj=bundle)**, then submits through
  **1lW.CPm**. This matches14o.BLn selector78 and its retained retry/error policy.
  Concrete CPm listener routing remains open; matching code numbers alone do
  not establish complete event lifecycle or durable callback ownership.
- **1ru.A00** is a no-op; **1ru.A01(int)** handles other subclasses, with no290
  branch. Negative A03 method lookup has no body. These bounded results refute
  treating any of those methods as the selected legacy prekey error parser.
- The dispatcher distinguishes exact result/error before these map removals;
  other types reach other request routing. This independently supports Rust's
  prekey reply-type admission correction for the legacy path as well as shared
  0ln evidence. It does not prove native retention after every malformed input,
  sender validation or complete result/error body validation.

**Rust impact:** legacy error extraction has numeric/text handling and manager
error scheduling, while Rust retains a bounded code string and stops. The gap
remains explicit; this research does not add reset, automatic retry or key
regeneration. Missing/error-text grammar is not invented from generic APIs.
No production change or live traffic.

Additional export hashes:

- [legacy-error-A00.txt](legacy-error-A00.txt): `4a4185b8e8d740b22891dd1b0a1a9ff7f6fb44c4cf5c156ac10afedade8c893c`
- [legacy-error-A01.txt](legacy-error-A01.txt): `1701b58f618861abc925c5fbb8d852cd259a85b37ae3953311d63972f2e07c73`
- [legacy-error-A03.txt](legacy-error-A03.txt): `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- [legacy-reply-dispatch-callers.txt](legacy-reply-dispatch-callers.txt): `59c087aeb48377679276c9229cfc5790903bd2881a30eccc431b6525c722e91e`
- [legacy-reply-dispatch.txt](legacy-reply-dispatch.txt): `28e18ba55fd1c87e6b8974285009ec71c52304bbcb3de606da8ff2259279c638`
- [legacy-error-event.txt](legacy-error-event.txt): `d2a45174e7aecf68cb3559783edf89311c26017a7782cb7cbfdc648bc3c2e034`
- [legacy-error-message.txt](legacy-error-message.txt): `f3f20a338e90987fcb9a45b08c286bd8f125733943ca9be82d98eef5b2fcb5ee`

## Legacy writer/reader map alias (2026-11-04)

**Confirmed static**, pinned43.72. Full constructor/selected transport method
exports retained; unrelated A0z connection branches are not claimed audited.

- **0ZJ.A0z**, **0916–0920** allocates one **ConcurrentHashMap** in v13.
  In the normal non-mocked branch, **0962–0968** passes v13 to **0ZJ.A0S**
  while **0a1a–0a24** passes the same v13 into **1rj.<init>**. No reassignment
  to v13 occurs between those calls on this selected normal path. Mock-provider
  present branch at0920–0944 throws createMockedReaderWriter-labelled exception;
  it is not evidence of another normally functioning alias.
- **1rj.<init>005a–005e** retains the incoming map in **A03**. The legacy writer
  uses that exact field for its generated-ID/290 entry (prior writer trace).
- **0ZJ.A0S001a–0024** passes its supplied map unchanged into **1r7**.
  **1r7.<init>000a–000e** retains it in A03. A0S invokes each configured
  1r8 factory'sAL1 with that same context and registers returned parser.
- **1rC.AL1** reads context.A03 and passes it into **1rD.<init>**; that
  constructor stores the reference in **A00**, the map removed by1rA.A03's
  result/error branches. Thus **when this factory is used**, writer1rj.A03 and
  reader1rD.A00 are aliases of that per-setup ConcurrentHashMap.
- Exact constructor caller searches bind1rj to0ZJ.A0z and1rD to1rC.AL1.
  1rA base constructor initializes dependencies; it does not allocate the reply
  map. No global shared-IQ-map equivalence is inferred from inheritance.

**Closed boundary / remaining proof:** the concrete constructor reference flow
is now proven for the normal setup path. Configured factory-set membership and
registration dispatch, CPm listener binding, teardown/reconnect reuse and reader
map cleanup are not proven by these constructors alone. Allocation per A0z
invocation is not a durable account/request ownership guarantee. This advances
prior alias unknowns without claiming the complete lifecycle is closed.
No runtime change, new retry or live traffic.

Additional export hashes:

- [map-1rA.txt](map-1rA.txt): `3fc8e1a224a9d35647fd9325d4d962054fb24d3e97f87afc6c84d194d6282df7`
- [map-1rD.txt](map-1rD.txt): `89d549c450413c115d87f48910692a098374df40646ac9f57bfaa67e3926be3d`
- [map-1rj.txt](map-1rj.txt): `aeac1dc101f1c77ec63588d3f0375f2eee8f81feff44e9ebabbfebcdf90db401`
- [map-callers-1rD.txt](map-callers-1rD.txt): `b62f4368c9154a5e687bcd91105c5b1e942d86d0a04d6de44af5889e093ca5e5`
- [map-callers-1rj.txt](map-callers-1rj.txt): `2dac608722772aa6d7a2e8c7973fd287569a68e05059ecfaeab3ae68e2fe9080`
- [map-reader-factory.txt](map-reader-factory.txt): `ce27aabbce0f341b19a7cde62e688dcbe6fe0252a6926638dd1d6e48e2b4fe5f`
- [map-transport-binding.txt](map-transport-binding.txt): `e75620ce701dd4a50da59adedbfe4f7686c08b6d34c24bea401bf45a2d8cbd69`
- [map-reader-context.txt](map-reader-context.txt): `96e64014c4f4238bedb03128387ba114741e636618245ba4c95d343a974e73bb`
- [map-context-constructor.txt](map-context-constructor.txt): `573053bd91fffb64ba2f2b03728fedd870e1dd8adc1dc58d552ed30aa606eaee`

## Legacy IQ handler registration boundary (2026-11-04)

**Confirmed static**, pinned 2.26.43.72, same APK SHA-256 as this bundle.
Checked DEX exports below are disassemblies, not REA Evidence envelopes.

- **1rA.A02, 00da–00ee** recognizes a `1rD` instance and returns the
  single stanza tag `iq`. Exact `1rD.A02` body search finds none; the
  inherited method supplies this tag. Other subclasses have different tags.
- **0ZJ.A19, 0000–003e** obtains every tag through virtual `A02`, checks
  `Map.containsKey`, and inserts `(tag, handler)` only if absent. A duplicate
  throws `IllegalStateException` with the retained addStanzaHandler diagnostic;
  registration does not silently replace an earlier handler. Together with
  the earlier A0S loop, this establishes how a returned `1rD` would be
  registered under `iq`, not which configured factories are present.
- Full **1rC** export has a trivial Object constructor, `AL0` returning null,
  and the previously traced `AL1` constructing `1rD` from the shared context.
  Exact `1rC.<init>` caller search returns **zero direct invoke callers**.
  This is a bounded negative: it does not rule out reflective construction,
  generated provider configuration or other type-based registration. Factory
  set membership remains **unknown**, and absence from the runtime set is
  not inferred.

**Remaining:** prove A1H factory membership and actual reader dispatch; concrete
CPm listener routing, teardown/reconnect cleanup and durable ownership are still
open. No production change, automatic retry or live traffic.

Additional export hashes:

- [registration-factory-class.txt](registration-factory-class.txt): `b3d365c9a2aa701dc2b53386ffffc18a3b6a4ef43cbf30d33a7370d8e9e24024`
- [registration-parser-register.txt](registration-parser-register.txt): `5acde23d6f188328557a56d9593cfc9bfa191da4960a5f2f1fe2a6012b3bca89`
- [registration-factory-callers.txt](registration-factory-callers.txt): `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- [registration-parser-tags.txt](registration-parser-tags.txt): `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- [registration-parser-tags-base.txt](registration-parser-tags-base.txt): `f1a9e0f0cfd210e978f8fc8f86167bf812a5196456de4c30939a2065a3954648`

## Legacy concrete listener handoff (2026-11-04)

**Confirmed static**, pinned 2.26.43.72 and this bundle's APK SHA-256.
These checked DEX exports are not REA Evidence envelopes. Only selected
branches of the full handler exports were inspected.

- Normal **0ZJ.A0z 04e4–0500** constructs `1lY(this)` and passes it to
  `1lZ(08q,1lW)`. **1lZ.<init>0006–000a** stores that listener in A01,
  closing the concrete CPm receiver for this setup path. The exact 1lZ
  constructor caller search returns A0z.
- **1lY.CPm** reads the selector through **1uW.A00**, which returns
  `Message.arg1` unchanged. It calls **0ln.A0G(selector)**. When true it
  copies the Message with `Message.obtain(original)` and calls **0ln.A07**;
  otherwise it sets the original `what=0` and queues it on its own Handler.
- **0ln.A0G**: when A0Q is true, explicit selector checks include 77 and78
  (`0x4d`, `0x4e`, 0150–0168), both returning true. When A0Q is false the
  SparseIntArray/list membership branch decides instead; no unconditional
  routing claim is made for that configuration.
- **0ln.A07** sets the copied event's `what=1`, retains the selector in
  arg1 and queues on its A0E/0m2 Handler. **0m2.handleMessage** case what1
  begins at021c and reads arg1. Its selected 0348–036e branch checks an
  Optional and delegates to 0m2.A00/0ln.A00; their manager mapping is still
  **unknown** here. This is asynchronous handoff, not proof of completion.
- The other CPm route reaches **1lY.handleMessage what0,013c–0146**, which
  forwards to **0ZJ.A0s**. Its subsequent listener mapping remains untraced.
- `apk-virtual-overrides` on 1lW.CPm reports only the abstract declaration:
  the tool explicitly covers superclass ancestry, not interface dispatch.
  The concrete receiver above is proven by construction, not by treating
  that bounded tool result as an exhaustive implementation search.

**Remaining:** binding either handoff to 14o.BLn's77/78 manager branches,
A0Q/runtime factory configuration, reader dispatch and teardown are open.
Prior matching event numbers alone remain insufficient. No Rust behavior
change, retry policy change or live traffic.

Additional export hashes:

- [listener-overrides.txt](listener-overrides.txt): `a97aec7bdcd4e99504d352f3d15a20ca8b015956bddfb7054ff1479e8c982242`
- [listener-ctor.txt](listener-ctor.txt): `b146a6e9aabec78478a940be6a5df3e20ddf7e1a4de895e1358ce68d5b2f43aa`
- [listener-callers.txt](listener-callers.txt): `cbbd9919b37933a4d3f1f018147f5501ca4d391bcf989b132053761ec280e126`
- [listener-forward.txt](listener-forward.txt): `eb2d4f590bd95fbd6cc2ac5a1eec93dc693270d899bc032d729d65a6ec05ee08`
- [listener-classify.txt](listener-classify.txt): `98886db83a86a8ddd55b360ab8de162d4782ca574921f766115d13564fdfb9ad`
- [listener-handle.txt](listener-handle.txt): `ce5e0de9b6d49a0808374951e0dd1fa1323f1ea86fcc3dcedfef8664c002524b`
- [listener-selector.txt](listener-selector.txt): `ad3a61d949a35c31be5a9712e3ccca005469657bc4fc9746360efc5cfbaedc89`
- [listener-dispatch.txt](listener-dispatch.txt): `adea7a52ca16dee2b65b2eb01528b78ff20290ffb85829cde2c4304984cb140f`
- [listener-manager-handler.txt](listener-manager-handler.txt): `4c8a65dfb8084a0e069c95bf190c90cf19c93a0dd6c6c362f198bacae16bc94f`

## Legacy enabled-route manager binding (2026-11-04)

**Confirmed static, conditional route**, pinned 2.26.43.72, same APK hash as
this bundle. Checked DEX exports below are not REA Evidence envelopes. Only
the selected routes in large dispatcher/provider methods were inspected.

- **0ln.A00** with A0Q true compares selectors77/78 at0238–0248. Both
  branch to0502 (023c +2*0x163;0244 +2*0x15f), loading **SQ0.A0s**.
  The goto at0506 reaches0028: resolve provider, cast0Vw, call
  **BLn(original copied Message, selector)**. This binds both numbers to
  the same configured handler slot rather than merely matching names.
- **SQ0.<init>01e6–01f6** installs **00D.A00(0x1a4f)** in A0s.
  00D.A00 constructs05h with the integer;05h.<init> retains it.
  **05h.get** lazily caches **00D.A02(id)**, which invokes00D.A03 using
  the application scope. A03 returns an existing cached object or calls
  **1Uw.A00(id,scope,application)** before caching the result.
- For id **0x1a4f=6735**, **1Uw.A00** selects group0 (`id>>14 &0x7f`)
  ->A01. A01 masks0x3fff and selects the range above0x193b through0x1a54
  ->A0H. **A0H** packed-switch case6735 targets0c46, constructs **14o**,
  and returns it at0c50. Exact14o constructor caller is this A0H.
  Thus this enabled route resolves **14o.BLn**, whose77/78 bodies were
  retained earlier in mode-callback.txt.
- **0m2.A00** only unwraps supported1k2/1ua/1ub/1uc objects or returns
  null; it is not an event-number-to-prekey-manager converter. The prior
  selected handler region's Optional check does not supply the binding;
  actual dispatch is0ln.A00 (retained handleMessage at07f8).

**Boundary:** this closes the selected A0Q-enabled provider route, not runtime
A0Q truth, the alternate sparse/list route or0ZJ.A0s forwarding. Provider
cache/scoping synchronization is visible but no runtime substitution is
observed. Factory membership, complete reader dispatch and teardown remain
open. No production change, automatic retry or live traffic.

Additional export hashes:

- [manager-event-convert.txt](manager-event-convert.txt): `9db573779e431a639b15e8f49a26766048900fa543797078b28555fbdb20c7d3`
- [manager-event-dispatch.txt](manager-event-dispatch.txt): `e7d9526bf0da728750f11810f974ac6d31f8c856425eb29224dfe691f128a77c`
- [manager-providers.txt](manager-providers.txt): `abebad6e4d22114f5ea6d7b03bd7b1422df1922c51f57f72b4ceae82eb945e1a`
- [manager-constructor-callers.txt](manager-constructor-callers.txt): `e9b291aa666423d1c37b74c6ac1fca7d0d141e0492697a497a912f1349e156f7`
- [manager-provider-create.txt](manager-provider-create.txt): `f09f0f8fc6dff6d8b8f4046bf29e8043fd75b3d62c062976226feddfc0275e9d`
- [manager-provider-wrapper.txt](manager-provider-wrapper.txt): `05ef4106d14c39c4c5450ec6f1711550676dca2555af38455d30ace66a4a217c`
- [manager-provider-get.txt](manager-provider-get.txt): `74797a5d573d18c5f5c6c6bee0876dbe60726fb2132abfed7189045f7438ce75`
- [manager-provider-resolve.txt](manager-provider-resolve.txt): `61ca1ae2033c7f40604569ffa0ea628b4aa09c36bba7a9826086157e739f557a`
- [manager-provider-scope.txt](manager-provider-scope.txt): `c341b1ddb648a4214d7424c37940e96fb49defa16f827a93a33e935d7ceded9a`
- [manager-provider-router.txt](manager-provider-router.txt): `f852095f305051f8344d087a0efeb3f6742c3538d9cdde11fe11f74e2f7dda7e`
- [manager-provider-ctor.txt](manager-provider-ctor.txt): `54e4e9380731428ec7f6f8152419cfb60f33ebaa8ef817548a35473dc854fb02`
- [manager-provider-subrouter.txt](manager-provider-subrouter.txt): `7c697c7e3e1c45ccef430c1ca605953a5a17fc0bab323e1a2e42bbe9d00df77e`

## Legacy fallback event route (2026-11-04)

**Confirmed static, conditional**, pinned2.26.43.72, same APK hash as this
bundle. DEX exports are not REA Evidence envelopes. Large constructor/handler
exports were inspected only for the selected paths described below.

- Earlier **1lY.CPm** false-A0G branch queues what0 on1lY, then
  **0ZJ.A0s ->A0q**. A0q handles selectors4/87 specially; for77/78 it
  copies the event and calls **0ZJ.A1B.CPn** (0016–0028).
- 0ZJ constructor overloads preserve incoming0YS; full constructor
  **0442–0446** stores it in A1B. **0ZH.A00** passes the argument through.
  Its exact caller **1WY.run,0032–0052** supplies **0YB.A13:0YU** while
  creating/storing A1C/0ZJ for the selected cold_start worker path.
- **0YU.CPn** sets what5 and queues the event. **0YU.handleMessage**
  case5 starts012e, obtains DEe and checks arg1. The selected chain checks
  5,12,64,158,237,20,21,271,272; **77/78 match none**. The final if-ne at
  **019c** with delta-0xc4 targets **0014**, the return-void. Therefore
  this selected fallback does **not** deliver these upload events to14o.
  It is not an alternative equivalent prekey completion route.
- This does not establish that77/78 ever take fallback in normal operation:
  enabled A0Q explicitly admits them and disabled A0Q depends on registered
  sparse/list membership. Those configuration/registration proofs remain open.
- 14o constructor is retained: A07 is a13z dependency, not a reader-generation
  token. No stale-reader check is visible in the specific forwarding chain
  above; this bounded observation is **not** proof of absent reconnect
  protection elsewhere or permission to weaken Rust generation checks.

**Impact:** prior alternate-route unknown is narrowed: selected cold-start
fallback discards77/78, while the enabled route is proven to14o. Actual
configuration, factory membership, teardown and stale-reader lifecycle remain
unverified. No production change or live traffic.

Additional export hashes:

- [alternate-forward.txt](alternate-forward.txt): `6bbd1f87763a2a684d21e34e843e3cec2526c28d073af93270fdcbe10ec83d46`
- [alternate-dispatch.txt](alternate-dispatch.txt): `2e1144001ae26240ac67d9692605314132e2b210b329f374aed8d41ca21936a9`
- [alternate-owner.txt](alternate-owner.txt): `622190e42b4fe2423d486e4899254d2b0a5fa671f1b285915738989e7380f691`
- [alternate-owner-callers.txt](alternate-owner-callers.txt): `3ccfe91dc1691d39ab84b21908a213771461e4a28db5894f7ebf944b83889cf2`
- [alternate-create.txt](alternate-create.txt): `e367156a6db3d4d1b85d0866b859db438014345b4920e11775c82fd6c7e3b105`
- [alternate-create-callers.txt](alternate-create-callers.txt): `41857dd037ef4cf577a4884adcdc07712ce85e2dbdca2c2c1bb1685334b09628`
- [alternate-worker.txt](alternate-worker.txt): `2e9500fd9d09839d5f1ed752229f8053e7906006f5f54fed5e4dd09c107ca00a`
- [alternate-listener.txt](alternate-listener.txt): `1d83f6a5ed8e06ff127e171fa8826537728a756404f46bc95ad60995f4ffb669`
- [alternate-handler.txt](alternate-handler.txt): `098d0fcc38eb07bd64d3ac1baa91fa3ea8825cf5ac0cb23d8a4bdb1c22cc30f0`
- [alternate-manager-ctor.txt](alternate-manager-ctor.txt): `fcbf8475a7e6e43caf6401e1535978929baa8751abc4a06e72809b1248e013a7`

## Configured legacy IQ factory membership (2026-11-04)

**Confirmed static**, pinned2.26.43.72, same APK SHA-256 as this bundle.
Checked DEX exports are not REA Evidence envelopes. Full generated provider
and array methods are retained; only selected bindings are claimed audited.

- **0ZJ.<init>046a–047a** loads **00T.A05(0x1d2a)** into A1H.
  A05 ->A04 resolves through1Uw.A00. Retained dispatcher group0/A01
  sends id7466 (0x1d2a) to **1Uw.A0K** (range0x1c87..0x1d9f).
- A0K case7466 at0602 loads **07q.A68** and branches095a to construct
  **07r(scope,array)**. **07q.<clinit>871a–8796** constructs the14-element
  A68 array, including **0x7e=126**. Other entries are125,127..129,3011..3019.
  Initial constants at0000–0028 identify index registers. No protocol meaning
  is inferred for the other factories.
- **07r** retains that array in A04 and allocates matching cache length.
  Its iterator is08x; **08x.next0052–0068** fetches the current array ID
  and invokes07r.A02's05f provider. That provider is **1Vo(selector1)**;
  **1Vo.get000c–0020** forwards ID/scope unchanged to1Uw.A00.
- Binding126 follows group0/A01 case126 at04a2 to **08o.A0Q**. This
  provider **allocates1rC**, invokes **Object.<init>** directly, and returns
  it. This explains why the earlier exact1rC.<init> search had zero calls:
  it did not cover this allocation with an inlined superclass constructor.
  Earlier bounded negative is retained, but factory absence is refuted.
- Together with retainedA0S iteration,1rC.AL1 and duplicate-safeA19, this
  proves the configured set contains the factory creating1rD under`iq`
  with the shared writer/reader reply map on normal setup. Actual runtime
  execution and complete reader dispatch/lifecycle remain unobserved.
- **0ln.<init>01fa–020a** captures **A14(0x4c7a)** in A0Q. Thus the
  earlier enabled-route condition is a configuration read at construction,
  not a fixed literal true. Effective runtime property value and disabled
  sparse/list registration remain unknown.

No production change, retries or live traffic. The remaining lifecycle proof
includes teardown, stale readers and durable account/request ownership.

Additional export hashes:

- [factory-set-wrapper.txt](factory-set-wrapper.txt): `9c2c1936da9a9b6a8cb8c9c77e5d24baca8eb79eaf51f2d30725e00acc1a100a`
- [factory-set-resolve.txt](factory-set-resolve.txt): `fc9a20b159051d888307fd8622f4a206ce3e7dcb4ffdb210593e424f08f7f914`
- [factory-set-create.txt](factory-set-create.txt): `a4814ee6c3aafb2162aa89dc21c79db9961de3cd2499419c1529afe00ed22612`
- [factory-set-arrays.txt](factory-set-arrays.txt): `d79dc1090c0d0fcf19386a9d65244c704d0e9ccc686f99faaed9f8fa54b68009`
- [factory-set-iterator.txt](factory-set-iterator.txt): `37d135b0965da70876335e74fb47e7dec25a8b08b6d3077c91b34ca136b8a15b`
- [factory-set-next.txt](factory-set-next.txt): `b1bd321d92bfa406594e2084cf7b715282677dfc814d2c57a1ef60f5914aed94`
- [factory-singleton.txt](factory-singleton.txt): `37c7cf53dfbae900c0adac85e549972e1478381014e88ac7bf9390d6164f702d`
- [factory-set-container.txt](factory-set-container.txt): `f6887ecf7c8fc4daa6b380dfb63f9318189fe946ae2102de18371d90c6a3c23c`
- [factory-set-get.txt](factory-set-get.txt): `6c47f1491e121445026eff9320f7ded1ecaf4d497033d9b496f4aa16ac453bed`
- [factory-routing-ctor.txt](factory-routing-ctor.txt): `ae4ff1844574b41ccb13bfb9415f46b570458ea9a535915b102c81cd972f97e2`

## Disabled-flag registry route (2026-11-04)

**Confirmed static after successful registry initialization**, pinned2.26.43.72,
same APK hash as this bundle. DEX exports are not REA Evidence envelopes.

- Retained0ln constructor013e–015a stores **00D.A05(0x1d2d=7469)** set
  in A0K. Existing1Uw group0/A01/A0K case7469 loads **07q.A80** at061a.
  Retained07q initializer **aa96–aaa2** inserts **0x1a4f=6735** at index34;
  A80 is assigned at ac22. Binding6735 constructs14o (prior provider trace).
- **00D.A05** resolves throughA02, then switches a07r container's provider
  to00D.A05/05g when A06 is true. **00D.<clinit>** constructs05g; its
  **get(id)** delegates unchanged to00D.A02, retaining application-scope
  resolution. This is distinct from the prior00T set wrapper.
- **14o.Amf** returns74,75,76,**77,78**,82,83,84 from its little-endian
  int payload. **0ln.A05** with A0Q false, under A0P monitor and once-only
  A01 guard, iteratesA0K, adds each handler toA0G and maps every Amf
  selector to its list index in A02. Duplicate selectors throw; initialization
  recordsA01=true only after completing the loop. A0Q true skips this registry.
- **0ZJ.onLooperPrepared0062–0078** resolves0ln and invokesA05. Exact
  callers also includeFfx.invoke, not traced here. Once this initialization
  succeeds,77/78 have valid registry membership: retainedA0G admits them;
  retained0ln.A00 disabled branch invokes the indexed14o.BLn.

**Correction to scope of earlier fallback finding:** the fallback handler
does ignore77/78 if reached, but normal successfully initialized disabled-flag
registration supplies a route through0ln, just as the enabled direct provider
does. Do not report ordinary disabled configuration as necessarily losing
upload completion. Failure/early-event behavior, effective runtime flags,
reader dispatch and reconnect/teardown still need separate evidence.
No production change or live traffic.

Additional export hashes:

- [registry-index-refs.txt](registry-index-refs.txt): `1cc4199c50472a1f30a18708131c9a12dd924baeccb727e876c1447e77a0e8ca`
- [registry-set-wrapper.txt](registry-set-wrapper.txt): `ab81e1f0c13f081ddd7511615b1a7fe2b9b60115f4c28bb7a845725f8a618deb`
- [registry-populate.txt](registry-populate.txt): `a0d6f1c1dac7349a6440de38a839c7115b17a0b1712108d313306849377a4f84`
- [registry-selectors.txt](registry-selectors.txt): `5c1c4110dc3537e38e85ce797f8ef495af653ef18334f5c6c8074abe1bf19324`
- [registry-populate-callers.txt](registry-populate-callers.txt): `ca6b5d893938ef33f254074cb68650691d30fc65b6d068139f04de0e53b3f196`
- [registry-provider-init.txt](registry-provider-init.txt): `8d048311d43487bfce78c3d3ed5385cdf89de5e966d1a00e62bce23558f0b596`
- [registry-startup.txt](registry-startup.txt): `72ae356f4478cdfe0a5d276a6921549c115f9747c456d31737786f5c114eab03`
- [registry-provider-get.txt](registry-provider-get.txt): `9cdcd72ef69009750d2c0fb5fe488fe74c6dbb4d68663ca069fe40a5388806f3`

## Rust reply field separation (2026-11-04)

**Confirmed static / implemented offline.** Retained43.72 `1rA.A03`
result/error branches invoke separate callbacks: result ->`1ru.A02` reads
optional list/pq_list flags, error ->`1ud.A01` extracts error metadata.
See the earlier legacy-reply-removal-and-error-event-path section and its
checksummed exports; no new APK/runtime evidence is claimed.

Rust previously parsed legacy count_low on either reply type and saved an
error child's code even on successful results. `PendingUpload::settle` now
parses count_low only for result and code only for error. Exact IQ/id/type
admission and transactional key/batch settlement remain in force.

Regression `upload_reply_fields_belong_to_the_selected_callback_after_reopen`
uses synthetic result/error replies each carrying both list(count_low=TrUe)
and error(code=503). It verifies selected outcome and persisted batch after
reopen: successful result has count_low=true/code absent; refusal has
count_low=false/code503. This tests stored semantics, not live acceptance.

**Remaining:** numeric native error extraction (including first coded child),
PQ follow-up flags, ADD strategy and gated retry are not ported by this fix.
The historical stored batch representation is not rewritten. No live traffic.

## Rust coded-error selection and numeric conversion (2026-11-04)

**Confirmed static / implemented offline**, pinned2.26.43.72, same APK
SHA-256 as this bundle. Retained1ud.A01 skips error children whose code
is absent, then returns the first present code transformed by0HF.A00(code,0).
The newly retained **0HF.A00** checks TextUtils.isEmpty, returns the default
for empty input, otherwise calls Integer.parseInt and catches
NumberFormatException to return the default. No trim or search for a later
valid code is performed. This checked DEX export is not a REA envelope.

Rust settlement now selects the first error child with textual code, parses
with the existing Java decimal helper and i32 bounds, and records canonical
decimal text. Missing/malformed/out-of-range codes become
`Some("0")` on refusals; successful results retain no code. The string field
is retained for storage compatibility; historical batches are not rewritten.
This replaces first-error-only selection and arbitrary eight-character
truncation, which could lose a later code or truncate signed i32 values.

Regression `upload_refusal_selects_first_coded_error_and_preserves_java_integer`
settles synthetic refusals after a code-less child, reopens the store and
checks canonical signed limits, plus/leading zeros, fullwidth digits, missing,
empty, whitespace, malformed and overflow inputs. A later406 cannot replace
a present malformed first code. Native text metadata remains unported, and
no numeric retry or regeneration policy is activated by this correction.
No live traffic.

- [error-number-parser.txt](error-number-parser.txt): `d02330cc5123b8be9d99bc042a5262fad59a71c60138568edd59c1d6a85514d5`

## Legacy refusal retry workers and reset boundary (2026-11-04)

**Confirmed static**, pinned2.26.43.72, same APK SHA-256 as this bundle.
Checked DEX exports are not REA Evidence envelopes. Only selected branches
of the full Fea.run are audited; unrelated selectors remain outside scope.

- Retained **14o.BLn selector78** reads code/text, clears manager arrays
  A08/A07/A02, setsA0X=12, clearsA0a/A0Z under monitor, calls13z.A07.
  New **A07 ->A0C(0)** only clears **A09** under monitor. It is not itself
  a database/key reset or the whole backoff reset.
- In the **EMZ.A02=false** legacy error path,406 storesA0Y=406 and
  submits **Fea9** to0YM.A02("prekeys",worker). **Fea.run11ba–11d2**
  invokes **13z.A05**, then **A0S(reason7)**. This differs from the
  modern **Fea13** path1174–119c, which invokesA06 thenA0R(reason7).
  Do not substitute modern PQ-only SET for the legacy error continuation.
- **13z.A05** acquires0Wk.A04/FjI, calls **0WJ.A0m(9)**, then
  **0WJ.A0k** (legacy ensure previously traced), closesFjI. Failure
  cleanup closes it and propagates exceptions; no successful A0S follows
  if A05 throws. Exact FjI locking/transaction semantics are not rederived.
- **0WJ.A0m** records shared preference **identity_reset_reason=9** then
  callsA0B. **A0B** calls **0Xp.A0E ->A0B()Z**, then conditionally
  (`0x5245`) invokes native jvidispatchIO(0,wajContext), and callsEoe.A02
  on normal/exception cleanup paths. The SQLite helperA0B, native operation
  andEoe effects are **untraced**: do not infer their complete deletion scope
  or implement automatic identity/session reset from method/log names.
- Legacy5xx scheduling at14o034a onward chooses Fea10 or11 by flag0x47af
  after the already retained delay/allowance calculation. Both run119c–11ba
  and1242–1258, submitting **Fea8** to named prekeys work. Fea8 checks
  managerA06 under monitor; when true it calls **A0E(null,reason8,true)**.
  If admitted, it calls **A03(nullCallback,manager,null,null,false,false)**;
  if not admitted/backofffalse it clearsA0Y and returns. Thus scheduled
  work does not unconditionally upload or regenerate. A0E's override is
  explicit; complete admission/lifecycle conditions remain required.

**Rust impact:** the saved-refusal hold remains an explicit incomplete-policy
state. Porting native retry requires correct strategy, worker admission and
reset semantics; this evidence does not authorize deletion or resetting real
account material. No production change or live operation in this chunk.

Additional export hashes:

- [retry-legacy-retry-workers.txt](retry-legacy-retry-workers.txt): `fb2b39ffc5c0f6be036a15ed005c1f2c73f0dcbc70ae087c3d3c29e8a7490909`
- [retry-legacy-regenerate.txt](retry-legacy-regenerate.txt): `6c038bf74ce7782a090d2f573b92077a2e5a2a4fb52e77cc0f5c40c1a8da5ed7`
- [retry-legacy-state-clear.txt](retry-legacy-state-clear.txt): `64db0cc6046673119c8aa5bf785c2efdb1d00f6ed8c38be11bc0f5309e4b5004`
- [retry-legacy-regeneration-storage.txt](retry-legacy-regeneration-storage.txt): `4154b37a7ba2b5e079e3050fd7ba7694aac4bd3196edbe0d09056c4b813d9d37`
- [retry-legacy-clear-timestamp.txt](retry-legacy-clear-timestamp.txt): `a22c18346f91bde5083d37e9b9a13b5795a392f1d809ae1ae410ebd8f515f442`
- [retry-legacy-reset-boundary.txt](retry-legacy-reset-boundary.txt): `311333af004a1fdd9625ec4d0ba49c5cd94001a0d7fc0b863657d816de1e6bef`
- [retry-legacy-store-cleanup.txt](retry-legacy-store-cleanup.txt): `752a3b4007437a9d72b14da5b9c34145fda350a8c54b91022efa1b96eeade069`

## SQLite reset base implementation and helper selection (2026-11-04)

**Confirmed static**, pinned2.26.43.72, same APK SHA-256 as this bundle.
Checked DEX exports are not REA Evidence envelopes. Base implementation
and selected provider branches are traced, not every configured subclass.

- **0Xp.A0B** obtains the write-lock object, checks current-thread read hold
  count and locks only when no read hold exists. It closesSQLiteOpenHelper,
  calls virtualA0C and unlocks on normal/exception exit when it acquired
  the write lock. A read-held branch logs deletion without write lock;
  do not claim universal exclusive-lock protection.
- **0Xp.A0C** optionally clearsA00 cache underA0B monitor (flag0x64c7),
  getsDatabaseName/path, attempts File.delete for an existing database, logs
  failure, invokes0KT.A03(file,label), then checks absence of the database
  and `-wal`, `-journal`, `-shm` paths before returningtrue. It returnsfalse
  if those checks fail. 0KT's own sidecar-deletion mechanics are untraced;
  absence checks are not proof each sidecar is explicitly deleted here.
  PriorA0E ignores this boolean, so it does not itself guarantee deletion
  succeeded or stop downstream cleanup onfalse.
- **Eoe.A02** clearsA00 map, A02 set andA01 map under monitor. Their
  semantic payloads and native jvidispatchIO effects remain untraced.
- Retained0WJ ctor005c–0070 obtainsA0Q helper through **0Xd.A00**,
  which returns lazyA01. New0Xd ctor uses **1X1(selector38)**. Itsinvoke
  branch0260–0296 reads **flag0x646a**: true binding0xc2d=3117, false
  binding0xc2c=3116. Existinggroup0/A01 dispatch selects1Uw.A0T.
  Case3117 constructs0Xv; case3116 calls0Xa.A00 returning28o, whose
  construction and overridden cleanup behavior are not traced here.
- **0Xv.<init>** passes literal **axolotl.db** to0Xt constructor. This
  identifies that helper's configured database name, not all runtime variants
  or exact tables recreated afterward.0Xt constructor chain/virtual cleanup
  overrides still require proof before assuming base methods apply unchanged.
- The attempted `apk-virtual-overrides` with0Xv as declaring class rejects
  because no exact declaration exists there. Its emptystdout export is a
  bounded tool result; it does not prove inherited method behavior or absence
  of intermediate overrides. Base0Xp invocation is separately retained.

**Impact:** legacy406 cleanup cannot be implemented as merely refilling
prekey inventory. Full helper selection, reconstruction and native reset
semantics remain open. Rust's temporary saved-refusal hold is retained.
No account store was deleted/reset; no production change or live traffic.

Additional export hashes:

- [reset-sqlite.txt](reset-sqlite.txt): `44402ac1bff1f6fda519f8e442f8343815eaf132f7c8a948f7540855c9e3c00e`
- [reset-cache.txt](reset-cache.txt): `93dca850eb014c21dcf29084a124f3ba2237c49c2289b6e3084bcb77abda3a08`
- [reset-file-delete.txt](reset-file-delete.txt): `04bbb80f9e2ca1718f7a7177caddf3255d9d0b165d147f1746c13fd23903a253`
- [reset-delete-overrides.txt](reset-delete-overrides.txt): `ff086e05c1e161a0f8d3194bec92443e267f9ebfa9839d73860e77b90f2a90bc`
- [reset-concrete-helper.txt](reset-concrete-helper.txt): `88d10ec08e9b5255982f7f605c0ff5b31d6974cd5b6b975dc46ef2c369cb1fa3`
- [reset-concrete-overrides.txt](reset-concrete-overrides.txt): `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- [reset-helper-provider.txt](reset-helper-provider.txt): `c1026df211b676889ca890e6b5b523c5490f3b109cb9a9b00716d39fd3d0603e`
- [reset-helper-lazy.txt](reset-helper-lazy.txt): `0f23ca95487f4cafa3349c9151abac71f1bbbb30510fa7331a7495820964b232`
- [reset-helper-factory.txt](reset-helper-factory.txt): `4b8e4195403c5ca7072089492e18dc2fdb962cf8b5155f84538913bc8b6014bd`
- [reset-helper-bindings.txt](reset-helper-bindings.txt): `9b5098a540e5115973a1b6f1345fe62848bbc7da66f478d1f93ba146983e8001`

## Last-resort producer and pre-upload timestamp (2026-11-04)

**Confirmed static**, pinned2.26.43.72, same APK SHA-256 as this bundle.
Checked DEX exports are not REA Evidence envelopes. No runtime generation,
rotation interval or migration-completion observation is claimed.

- **0WJ.A0e** obtainsFjI, computes08q.A00/1000 seconds and checks native
  loaderBbM. If unavailable it logs and returnsnull after closing.
  It queries **0Yd.A00(false)**. Existing result is reconstructed through
  0Yj.A00 and returned; no age comparison occurs on this selected path.
- **0Yd.A00** queries `last_resort_key = 1 AND sent_to_server = ?`
  ordered **_id DESC LIMIT1**; false binds`0`, true binds`1`. Thus newest
  is by database row order, not highestprekeyID, and A0e reuses newest
  **unsent** LR. **0WJ.A0b** uses the same false query without generation.
- With no unsentLR, **A0e00f4–0130** getsidentityAnw, reads raw
  **next_kyber_prekey_id=N**, generates **EqE.A00(identity,ML_KEM_1024,N)**,
  computes nextcursor **((N+1)%0xfffffe)+1**. This matches the current
  positive native cursor convention, not Rust's historical monotonic/local
  cursor. Do not reinterpret persisted Rust cursors from this trace.
- **A0e0130–01b8** obtainsFjI/database transaction, stores the serialized
  Dfd as singletonERb via **0Yd.A01(nowSeconds,rows,1)**, saves nextcursor
  and marks/ closes transaction before reconstruction/return. Exact SQLite
  conflict policy0KU.A06 and FjI semantics remain untraced.
- **0Yd.A01** writes sent_to_server=false, direct_distribution=false,
  last_resort_key=the suppliedinteger(1 here), and optional **upload_timestamp**
  from its Long argument. **Therefore this producer writes upload_timestamp
  before server acknowledgement**. The name is not evidence of sent=true.
  The existing migration predicate specifically requires uploaded LR; do
  not replace it with timestamp presence or row existence.
- A0e catches UnsatisfiedLinkError around selected generation work, logs
  and returnsnull; other exceptions close wrappers and propagate. Native
  primitive/signing itself is the previously retainedEqE trace.

**Rust impact / remaining port:** current SET helper creates LR only if noLR
exists and omitsPQ afterLR sent; nativeA0e can create a new unsentLR when only
sentLR exists. This does **not** imply it should be called at every ADD or
startup; entry routing still decides. Current ADD strategy, configured
inventory/batch values, cursor migration and actor transitions remain
incomplete. No production request or cursor semantics changed in this chunk.
No live account traffic.

Additional export hashes:

- [last-resort-producer.txt](last-resort-producer.txt): `165f855614afc9f3cbbf62355318d756453dfb964b65396a82af8342eddae8ee`
- [last-resort-unsent.txt](last-resort-unsent.txt): `4885300896395f84913fafe78f62923a00cd61ea596e9d084857d83c8cd4eebd`
- [last-resort-query.txt](last-resort-query.txt): `391c5d016d8cf3101b475d0b027174bcbd153784e3514062fb1ee01e34441a9c`
- [last-resort-storage.txt](last-resort-storage.txt): `03b76a20ee172dae0efc271825eb9bb99e0150f9baa042fff883b53787abb2c5`

## Legacy success to migrated ADD admission (2026-11-04)

**Confirmed static**, pinned2.26.43.72, same APK SHA-256 as this bundle.
Checked DEX exports are not REA Evidence envelopes. Only selectedFcj.run
default branch(selector3) is audited, not its unrelated branches.

- Retained **14o.BLn selector77** snapshots/clears pending arrays and LR
  under monitor, clears limited-mode state, cancels098 work, then builds
  **Fcj(manager,PQarrayWithLR,legacyArray,3,legacyNeed,PQNeed)** at02a8–02b4
  for named prekeys execution. Constructor stores its object arguments in
  reversed field order: A01=third(legacy), A02=second(PQ); booleans unchanged.
- **Fcj.run000e–007e** underFjI calls **0WJ.A0x(legacy)** then
  **A0w(PQ)** (the stored mark-sent helpers previously traced), conditionally
  invokes09S.A19(false)/A17(false) afterA1J, calls13z.A04 and closesFjI.
  Failure propagates with wrapper close; no successful followup is claimed
  after storage error or inferred cross-family SQLite atomicity.
- **13z.A04** takes/removes pending shared preference identity_reset_reason
  (default-1 means absent) and emits a2lU event with that reason plus current
  seconds. Event sink semantics remain untraced; this is not itself another
  key reset or upload. **Fcj0084–0098** clearsA09 throughA0C(0), callsA09.
  **13z.A09** clearsA06 and resets0Vf only when backoffA06 wastrue.
- **Fcj0098–00cc** schedules nothing when both needsfalse. Either needtrue
  schedules **Fc8(manager,7,legacyNeed,PQNeed)** on098.execute. New constructor
  binds fields without swapping flags. Retained **Fc8.run** has selectors0..6
  only; selector7 reachesdefault and calls **13z.A0V(legacyNeed,PQNeed)**.
- EarlierA0V evidence therefore applies to **legacy SET success**, not just
  notifications: first admissionA0E(null,reason2,false), thenEx8.A03 migration
  check, then generation/availability checks and **Fd5(useSet=false)** on
  migrated path. Non-migrated returns without this ADD upload. Already-retained
  Ex8 migration predicate combines gate21198 and uploaded LR existence.

**Confirmed Rust gap:** currentactor follows accepted legacy count_low with
unconditional currentlegacy SET helper, ignoresPQ-only followup and lacks
this entry-specific migrated ADD admission. Full fix requires strategy
integration (ADD shape/batching, family-specific demand, generation and
durable reservation), not merely calling the modern callback's helper.
This trace closes the legacy producer-to-A0V edge needed for that port; no
production request, migration state or retry behavior changed here.
No live traffic.

Additional export hashes:

- [legacy-success-worker.txt](legacy-success-worker.txt): `77499542ac96fae3386051d06a7dbe59cd11e1e39c8449f1014686846f30091e`
- [legacy-success-ctor.txt](legacy-success-ctor.txt): `9b9c5442325a44193d85c28d3e1fec35209376ed6d427cb6ded3682f970fa946`
- [legacy-success-backoff-clear.txt](legacy-success-backoff-clear.txt): `cfdb7f1287494c24651bada35ae003a53c6289dbd6c47095a7cc9e4f5cf6d13e`
- [legacy-success-complete.txt](legacy-success-complete.txt): `7ecdefbef8fb0cbd4f8c6b944418e2b057ecb2735b4ebd26c38930c1cf13fa26`
- [legacy-success-followup-ctor.txt](legacy-success-followup-ctor.txt): `b90893bb52154b9a8bad77bb421eabf289c996ce6eaaf66ac45e58a852337047`

## New-store startup and digest entry distinction (2026-11-04)

**Confirmed static**, pinned2.26.43.72, same APK SHA-256 as this bundle.
Checked DEX exports are not REA Evidence envelopes. Selected normal setup
branches only; unrelated caller bodies are not claimed audited.

- Retained **0ZJ.A0z050c–0514** captures09S.A1J into v45. At0ce6 it
  branches on that captured flag; true invokes **13z.A0N**. A0N rechecks
  A1J and submits **Fea(selector6)** to0YM.A00("prekeys") and waits on
  FutureTask.get. A0z catches selected execution/interruption exceptions.
- **09S.A1J** reads **signal_protocol_store_is_new**, defaultfalse, under
  preference monitor. A19 writes that same flag (the previously traced
  success worker clears it). This is distinct from key count or PQ migration.
- Retained **Fea6,0b30–0b9c** obtainsFjI and calls **0WJ.A10 ->0Yj.A01**.
  The latter counts legacy prekeys where sent_to_server=0 AND
  direct_distribution=0, returning nonzero; missing count result throws.
  With unsent keys, worker closesFjI then calls **A0S(reason1)**. Without
  them it callsA0k ensure, closes and returns without sending in that
  invocation (0b90 branch to005a return-void). Generation is not the same as send.
- In A0z's selected **non-new-store** branch0d1e–0d4c, **09S.A1I** reads
  **need_to_get_pre_key_digest**, defaulttrue. When true, **13z.A0M**
  separately checks0AI.A06 readiness and last attempt uptimeA01 under monitor.
  A01=0 permits immediately; otherwise **elapsed>60000ms** permits and
  updatesA01 to currentuptime. Equality60000 still skips. If admitted it sends
  **Messagewhat0,arg1=88,arg2=0** through0Sf.A0V. Digest writer/reply
  lifecycle remains untraced here; this is not an upload admission.
- Exact A0S caller search also finds **13z.A0Q**. Its retained body calls
  A0l, then with gateenabled marks previouslysent KEM rows unsent and clears
  upload_timestamp beforeA0S(inputReason). Its upstream callers are FcT,
  Fdf andFeK variants; contexts remain untraced. **Do not classify A0Q as
  generic startup** or port its mark-unsent behavior to reconnect merely
  because it eventually callsSET.

**Rust gap:** startup currently checks inventory and may generate/send in one
epoch, without these persisted new-store/digest flags and digest lifecycle.
This is explicit local behavior, not native startup parity. Port requires
registration flag producers, digest request/callback and applicable lifecycle
ownership. No production change or account flag/reset operation here.
No live traffic.

Additional export hashes:

- [startup-set-callers.txt](startup-set-callers.txt): `e5ff93345c5429afbeae4e606add9e7980b5cb04e91de0e43b9943e714c94b49`
- [startup-unsent-predicate.txt](startup-unsent-predicate.txt): `85d5c3b61ae6e6493a0228bc13d28a45687005866eb16224beb8fb6ada98fc77`
- [startup-manager-entry.txt](startup-manager-entry.txt): `06d9f847e78d64c77d327ecd0e94a034cefc8bf3119238c637083b589cef56d5`
- [startup-manager-callers.txt](startup-manager-callers.txt): `425939b4dc2cecec2eea188add8cb1dbd3c087d86a49eee45ee69295aa1145a5`
- [startup-unsent-query.txt](startup-unsent-query.txt): `07918d7a6e491559702a184fc9a4afb0e0bcae146536d0a5de9f6f7ecccc2da0`
- [startup-worker-callers.txt](startup-worker-callers.txt): `cf91556425dd52364617b3a2b068635af7d2d31141c96f2b8021e55910cd76f4`
- [startup-dispatch.txt](startup-dispatch.txt): `c78187b325d39aecedd26c44ab31d7203c329b37b0929a714de1903d7ad4c5d5`
- [startup-dispatch-callers.txt](startup-dispatch-callers.txt): `d701156af38d4264bc0c93f43a0cc343a972522f0da236151974e64dc9f2c744`
- [startup-registration-flag.txt](startup-registration-flag.txt): `917e619b900144aad7a39b1f0b275582a94c61fb596c17c9a787363d6702ddc3`
- [startup-registration-clear.txt](startup-registration-clear.txt): `5f527e3bb0ccf3ec0daf932ef38a3846d1f96fc03098a1eeaab0a1837cd4451f`
- [startup-upload-flag.txt](startup-upload-flag.txt): `9cc48cb08e54924b4194438d7a7824687e5fcae2b7bf31ef8af4de38ff29010f`
- [startup-migration-entry.txt](startup-migration-entry.txt): `c2ebb2b52b52ad6139f73419c9206cb7af5b2461d97fa4e3b078d9e38246ccc3`

## Digest request and wire callback grammar (2026-11-04)

**Confirmed static**, pinned2.26.43.72, same APK SHA-256 as this bundle.
Checked DEX exports are not REA Evidence envelopes. Existing fullwriter
reachout-iq-write-admission.txt is reused (SHA256 retained in legacy writer
section); no duplicate fullwriter export. Only selected digest branch audited.

- **0v0.A00 arg1=88 ->230c–23c6** increments1rj.A00, hex-encodesID,
  inserts **291(owner,selector1)** into sharedA03 reply map **beforeCyt**.
  ParentIQ attrs: id,xmlns=encrypt,type=get,to=0mL.A00. Single emptydigest
  child: v1=null at00ea, reused at23a8. A failedwrite is not proven safe replay.
-291 extends1ru. Retained1rA.A03 exactresult/error removes map entry before
  callback. **1ru.A02 selector1,00de–021c** requires viaA0G: digest,
  registration,type,skey/id,list,hash. Each list child must be an `id` node
  with3 bytes, not akey wrapper. Empty/no list childarray becomesemptyIDs.
  Optionalpq_list has null IDs ifabsent and emptyIDs ifpresent with nochildren;
  its direct children similarly must beid/3bytes. Optionalpq_last_resort_key
  requires nestedid/3bytes. These presence distinctions survive forwarding.
- **0Tp.A03** requires nonnull bytecontent of exactly suppliedlength, else
  throwsLIL. Callback requires registration4bytes,type1byte,skeyID3bytes;
  forwards type[0] without validating itsvalue here. **Hashlength32** when
  pq_last_resort_key ispresent; otherwise branch021c sets **20**. Presence
  ofpq_list alone doesnot select32. No hash algorithm or local equality
  validation is inferred solely from byte size.
- **1lZ.A1R ->A0d** forwards registration,type,signedKeyId,keyIds,hash and
  optionalpqKeyIds/pqLastResortKeyId inBundle throughCPm. Message **what0,
  arg1=82,arg2=0**. Retained14o.BLn has selector82, but its verifier and
  digest matching/reset consequences remain untraced.
- Error path differs from upload290: retained1rA.A03 non290 branch iterates
  codederrorchildren, Integer.parseInt without0HF fallback, invokes1ru.A01
  for eachpresent code. **291 selector1** routes **404 ->1lZ.A0i ->arg1=83**,
  **503 ->A0j ->arg1=84**; other numeric codes return. No-code children skip.
  Missing/malformed error behavior and exception recovery are not invented
  from the upload parser, nor are repeated codedchildren assumed unique.

**Remaining:** manager digest verification/algorithm, response-origin checks,
reset/catchup rules, native timeout/cleanup, persistedstartup flags andRust
request admission. No digest request capability or automatic reset added.
No production change or live traffic.

Additional export hashes:

- [digest-callback.txt](digest-callback.txt): `a03740bfbc3c63db14fcc0704a8665760b987e7260ed829bd3ce4324e1103c15`
- [digest-result-dispatch.txt](digest-result-dispatch.txt): `b5f8f8f5f62bb9bc5cefdaabe45cbb926572de3f5bb376d1668cb296ba682fa6`
- [digest-success-event.txt](digest-success-event.txt): `411d95a02e8a36e6b50982496147edc1fe26ee9a1afc98760f014bed91144ef7`
- [digest-bytes-helper.txt](digest-bytes-helper.txt): `dd77b241fbb948d54e1990dce1039a04e68bd733bbcd9c0c83087510e47c77d7`
- [digest-event-message.txt](digest-event-message.txt): `9805ed5cedfba8c3cd4e38d5075eaf380ef38430227a573cb5839929a5f9c10d`
- [digest-not-found.txt](digest-not-found.txt): `595410bcb925f5f3ee3e13009e4dd63c711b73f534f253bbc76eda1e1fddee4c`
- [digest-server-error.txt](digest-server-error.txt): `a4ef2f579f831eafc254eb8ea3f46fb9cf45b3b47e97236e84fe17a256801c3c`
- [digest-not-found-message.txt](digest-not-found-message.txt): `9355e1f3a328f5431ab93c872ed9435ccf25e360dd147712a177b571a0fdf2c4`
- [digest-server-error-message.txt](digest-server-error-message.txt): `8cf13d0f09d619b87af7b4b38d00f8f53e1ea7f8912c3a9eecc96b8e0f79252f`

## Digest local material verification and resynchronization (2026-11-04)

**Confirmed static**, pinned2.26.43.72, same APK SHA-256 as this bundle.
Checked DEX exports are not REA Evidence envelopes. Fdf.run nonzero
selector1 branch only; its unrelated zero branch is not audited.

- Retained **14o.BLn82** readsBundle, preserves null/empty PQ ID distinction,
  calls **13z.A0P** before schedulingFdf(selector1). NewA0P clears **A01=0**
  under monitor: the earlier60s uptime check is an attempt/inflight guard
  reset on this result path, not evidence of a persisted cooldown.
- Fdf constructor maps listener,registration,signedID,legacyIDs,hash,PQIDs,
  LRID andtype to its fields. **run006c–00c8** requires typebyte5; converts
  registration with1nr.A01 and compares local0YT.A08; then compares signed
  ID to current0WJ.A0c().A01. Localidentity comes from0WJ.A19.
- Legacy records are read and parsed into SparseArray, then selected in
  **server-provided ID order**. A missing/failed row means no complete array;
  array count must match server ID count. SQL scanorder is not hashorder.
  No ID uniqueness check occurs on selected loop: do not silently sort or
  deduplicate IDs when reconstructing this digest.
- Digest algorithm at0260–0296 is **SHA-256** only whenEx8.A04 gate is
  enabled **and LRID ispresent**, otherwise **SHA1**. DBb.A0x independently
  resolvesliteralSHA-256. PQ-list presence alone doesnot selectSHA-256.
- Updates inexact order: **identity bytes || signedprekey.A00(public) ||
  signedprekey.A02(signature) || each legacy.A00(public)**. WhenPQ gate is
  enabled andLRID present, load newest **uploaded** LR through0Yd.A00(true),
  require itsID equal serverLRID, then append **LR.A00(public) ||
  LR.A02(signature)**. Independently, withPQgate enabled and nonemptyPQ IDs,
  load rowsbyIDs, reconstruct
  **server order**, require allpresent, append each **ordinaryPQ.A00(public)**.
  IDs themselves,ordinaryPQ signatures,registration andtype are not fed to
  MessageDigest on this selected path. ETs public/signature mapping comes
  from prior legacy/KEM serialization traces; no private key hash is inferred.
- At055e–058e comparesdigest bytes with serverhash viaArrays.equals. Success
  sets localworkflagfalse, avoidingA0Q. Registrationmismatch uses **A0Q(5)**;
  selectedtype/signedID/missinglegacy/LR/PQ mismatches reach **A0Q(3)**;
  hashmismatch setsreason3 thenA0Q. Defaultreason12/exception handling is
  retained but full catch-path outcome is not independently enumerated here.
  PriorA0Q trace marks sent KEM keys unsent and re-entersSET, so this is state
  resynchronization, not mere diagnostic logging.
- On selected normal completion **09S.A17(false)** clears persisted
  **need_to_get_pre_key_digest**. It is separate from clearingA01 attempttime
  or successful uploadedLR migration. Full preference producers and pending
  digest/request ownership remain open.

**Important boundary:** wire parser chooses hashlength32 based solelyonLR
presence, while verifier also checksPQ gate foralgorithm. Do not collapse
those independent checks. Missing local keys or a parsed server digest are
not proof of successful reconciliation. No Rust digest request, mark-unsent
recovery, account reset or live traffic added by this research.

Additional export hashes:

- [digest-verifier-worker.txt](digest-verifier-worker.txt): `32df299819b5bdc0ad940d62e213f15eeaf8ce226cc6cf0f5dde9c3dc88f9e69`
- [digest-verifier-ctor.txt](digest-verifier-ctor.txt): `57bdfa4d59c08785fe8b7fce5a06ea3886fd4b7867bc29b99a27674e24692d2e`
- [digest-attempt-clear.txt](digest-attempt-clear.txt): `60c829af81fff2e1fd6ebdf2b0ae32bd4b0b774578049786864f2aa773316b00`
- [digest-sha-helper.txt](digest-sha-helper.txt): `a2efcb6b06279aa8cab9377bc8e7d50fdc0d032c777f119365792dff4a575492`
- [digest-preference-clear.txt](digest-preference-clear.txt): `4982d067679a8b1da01af3ad4bcf75a58175eb039c00dc8a37b34ac035db8827`

## Digest not-found/server-error and metadata resync scope (2026-11-04)

**Confirmed static**, pinned2.26.43.72, same APK SHA-256 as this bundle.
Checked DEX exports are not REA Evidence envelopes. Only FeK selector21
and selected14o error branches are audited; unrelated worker branches remain
outside scope. No Rust implementation of these state changes yet.

- Prior291selector1 maps404 ->83,503 ->84. **14o.BLn83,0998–09c4**
  clears attempttimeA0P then schedules **FeK(listener,21)**. **FeK08b4–08d6**
  calls **13z.A0Q(reason4)** then **09S.A17(false)**, clearingneed-digest
  on normalcompletion. IfA0Q throws the followingclear is not shown guaranteed.
- **14o.BLn84,07d6–07ec** logsservererror and calls **A0P** only. Thus
  this selected503 path clears attempttime but doesnot clear persisted
  need_to_get_pre_key_digest, markkeysunsent or directlyqueue anotherupload.
  A later startup may still enterdigest when its separateflag allows. No
  synthetic exponential retry is inferred from503 alone.
- **13z.A0Q** firstcalls **0WJ.A0l**. NewA0l updates legacy **prekeys**
  where **sent_to_server !=0** to false and upload_timestamp=0; deletes
  **prekey_uploads WHERE key_type=0**. It wrapsdatabase transaction/FjI,
  preservingprivate record fields and IDs in these explicit statements.
- PreviouslyretainedA0Q additionally performs analogous KEM sentflag/time
  updates whenPQgateenabled. Its **0Yd.A03** nowtraced deletes only
  **prekey_uploads WHERE key_type=1**. Neither these statements norA0l
  deleteprivate prekeys, rotateidentity or invoke the406 deleteStoreA0m path.
  Full nativewrapper atomicity and exceptional outcomes remain separate.
- Gateenabledmark-unsent includes uploadedLR rows, so the same migration
  predicate can becomefalse untilSETack. This is explicit digestresync,
  not authority to resetmigration on ordinary reconnect. A0Q closes its
  scopes and callsA0S(inputreason) usingexisting material.

**Port constraints:** correlate a fresh digest/404 to its original request,
account and local generation before any statechange. RustOutcomeUnknown
reservations are a distinct local state and must not be converted toUnsent
merely because native updates formerlysent rows. Historical digest imports
oroffline inspection are not reset authority. Current saved-refusal hold
remains; no new retry/reset operation or live accounttraffic here.

Additional export hashes:

- [digest-resync-legacy.txt](digest-resync-legacy.txt): `8823a93bebf5301b703e756666d1e98d7955031743d69a59b9c7452a296c3125`
- [digest-none-worker.txt](digest-none-worker.txt): `76e4d710c781be5935fb76b7841afdc5ae4aa46a6b214407309ee967d74ba536`
- [digest-resync-kem-history.txt](digest-resync-kem-history.txt): `c0773dc2f020747600e1da17c4790a63d3bb2655fbacff062ee7a117adbfb990`

## Count notification ACK token and queue stages (2026-11-04)

**Confirmed static**, same pinned APK SHA-256 reverified this turn. Checked DEX
CLI exports with signatures/exception tables; no runtime, Java source, or new
REA Evidence envelopes are claimed.

- **15v.A09**, 0060–0066 and00a8–00ae, calls inherited0v6.A07(true) only after
  the legacy/PQ count parsing paths. Legacy schedules FcQ selector4 through
  named0YM.A02("prekeys",Runnable); PQ schedules FeQ selector0x1c through the
  same named queue. Both count branches request deferred metadata, rather than
  constructing an immediate socket ACK in this method.
- **0v6.A07(true)** consumes its AtomicReference2DT with getAndSet(null), puts
  Eoh->1k2 in A0A under a monitor, adjusts a scheduled Runnable with6000ms
  (0x1770), and constructs7p2(handler,Eoh) plus89Y metadata. The selected method
  does not establish the scheduled Runnable's timeout action. false mode does
  not construct7p2. Missing deferred context throws on the true path.
- **89Y.A01** calls A00(A04,null,self,null). **89Y.A00**,0000–000e, atomically
  takes and clears its7p2 reference. Only a nonnull token enters dispatch: it
  removes Eoh from the handler A0A map, dispatches, and then removes Eoh.A08
  from A0B. A repeated call on this same metadata object does no further ACK
  dispatch. This is not evidence of permanent deduplication across notification
  redelivery, new objects, process restarts, or different session generations.
- **0v6.A00** branches on1k2.BX2(). true casts to1k3 and submits a singleton2Dx
  through1ic.A0G. **1k3.BX2** returns constanttrue. false calls0Sf.A0O with
  the original Eoh; the concrete1k2 implementation for a particular live count
  notification is still unobserved. The abstract1k2 contract is retained.
- **0Sf.A0O -> A07 -> 1i8.A03** creates Message(what0,arg1=0x4c/76,arg2=0,obj=Eoh)
  and callsA0V. A07 also logs an optionalccq flag; its selected method does not
  directly write a stanza. **0Sf.A0V** passesfalse,false intoA09.
- **0Sf.A09** checks readiness, callsA05 on its ready branch andA00 on the
  not-ready/default final-false branch, returning a readiness Boolean. **A05**
  can convert viaA00 and dispatch through1A0.CdY rather than synchronously
  completing a socket write. Other branches include an explicit drop or throw
  when sending-channel state is inconsistent; this is not unconditional delivery.
- **0Sf.A00** asks1i8.A07 to recognize a pre-ACK candidate; absent candidate
  returnsnull. For a recognized candidate it performs loggable-stage handling,
  adds1vZ to1iV.A0I, uses200 as a scheduling threshold and conditionally schedules
  TB4 selector8, then returns Message(what0,arg1=0x2b/43,arg2=0,obj=1vZ).
  This selected method shows an in-memory collection and executor path, not a
  proven durable ACK outbox. Exact conversion eligibility, bulk flush,43 pre-ACK
  writer body and crash/reconnect draining remain to be traced (76 continuation
  below closes its direct writer/builder link).

**Ordering consequence:** the previously retained15v.A02 decision-before-A01
call order remains confirmed, but A01 is metadata consumption and queue
admission, not socket completion. A0V uploads also scheduleFd5 after synchronous
inventory work. This evidence does not order the two asynchronous writer paths.
Rust's capture->ACK write->upload decision behavior remains a known difference;
no socket reorder or pre-ACK key reservation is implemented from this trace.
Freezing a Rust outcome_unknown upload before an ACK write could itself strand
never-written keys on ACK failure; a faithful port needs the decision/queue/
reservation/write boundaries modeled separately. That risk is an inference,
not observed APK behavior. No automatic replay or duplicate suppression added.

Exports (same apk-smali command as bundle header; caller inventory uses apk-callers):

- [prekey-ack-bulk-predicate.txt](prekey-ack-bulk-predicate.txt): `4e02520fa32e9b80d8bbec0524df8ff86cdc8d15e77cca7f7ac9953e762e5cae`
- [prekey-ack-contract.txt](prekey-ack-contract.txt): `2e82756c809b4ef2c7e2e0b42a496cbb37fbf45d64bfcbdf9b4b8897d6351aef`
- [prekey-ack-count-dispatch.txt](prekey-ack-count-dispatch.txt): `c0fd293b063f5afd47e4b68c4f8d2db48c18e8cd0a508cf617a7546d365a2a8a`
- [prekey-ack-dispatch.txt](prekey-ack-dispatch.txt): `42e84943e757ca535f95aa5b5cc6d5957a2550d12099df726335b99f3bdeb224`
- [prekey-ack-enqueue.txt](prekey-ack-enqueue.txt): `2fd03f7e1badafae636dcb81508843c130e4f0bf5236ba70b127e04dc2242903`
- [prekey-ack-handler-metadata.txt](prekey-ack-handler-metadata.txt): `4e1031bef6abe7c1d3dfc8d273f94a2e248d1139adebd5ee37e68844ba714750`
- [prekey-ack-message.txt](prekey-ack-message.txt): `8ccfc367b2b56d5f10ae18f600e9c3af3548864f65fe35c74b3b723e5daccb37`
- [prekey-ack-metadata-callers.txt](prekey-ack-metadata-callers.txt): `70bf329ea1a36762c15c067c005ee62919910040948ee6821df77336f096f680`
- [prekey-ack-metadata-constructor.txt](prekey-ack-metadata-constructor.txt): `37078d424e8a6f16d738c721c29660f5a57a82236feca43f77ef4f2dc32b1e50`
- [prekey-ack-metadata.txt](prekey-ack-metadata.txt): `5c33d76b2c14d5f7f04251bfa3abe865bcbd9b14fb45564d65664f6f46f25cbd`
- [prekey-ack-not-ready-queue.txt](prekey-ack-not-ready-queue.txt): `6bf3bbd4a8cda9691971e0a5003066c0c2e41086e8451c72acad294f06571bb0`
- [prekey-ack-producer.txt](prekey-ack-producer.txt): `2f929170ac3ebeadd015dff89c02d526e2fc274ca90f18e1452b4f8b17a6f4eb`
- [prekey-ack-queue-admission.txt](prekey-ack-queue-admission.txt): `0ca962de6520bb563aaa26e201a139412a3a8001491c214374f5f94e86d7d3c8`
- [prekey-ack-ready-dispatch.txt](prekey-ack-ready-dispatch.txt): `e195fd133b5cd248d2b1c933a4e80979026779b34e4672b2bb70dd2e8bc5087c`
- [prekey-ack-send-admission.txt](prekey-ack-send-admission.txt): `b6cf93a6091f2f3cbb5366d05fcb61e26ca7b5f20cc52c4fd6aaac222041104b`
- [prekey-ack-template.txt](prekey-ack-template.txt): `501a406d4fe003738f86ed40035ba3106fe5b285a06b214330894f1354876504`

### ACK writer dispatch and shape continuation

**Confirmed static:** retained [writer admission](../../rea/2.26.43.72-reachout-state/reachout-iq-write-admission.txt)
switch at00f0 maps76->11b8 and43->0cf8. Case76 calls0v2.A02(Eoh), which
builds F3e.A05(Eoh.A04,Eoh) and invokes1rj.A04(node,Eoh.A00). Case43 extracts
1vZ.A00 andA01 and calls0v2.A01(node,43,id). The latter writer body and candidate
conversion/flush remain outside this selected continuation.

F3e.A05 emits ack attributes in order id,to,class,type, then optional participant,
recipient,edit andextra attributes. For class=receipt/type=delivery it omits type;
otherwise it takes Eoh.A09. edit is included only if nonempty and notliteral0.
An optional supplied0Tp is wrapped as one child. This confirms the ordinary
notification/encrypt ACK class/type shape, but count-specific eligibility for
participant/recipient/edit/extra attributes needs Eoh producer evidence. Rust
count routing currently excludes participants and itsACK omits those optional
fields; no unsupported propagation is added from this generic builder alone.

- [prekey-ack-wire-producer.txt](prekey-ack-wire-producer.txt): `fe18318954e371d556d39749bf721e1b55e6ccb22434841700af43bead6618b4`
- [prekey-ack-wire-builder.txt](prekey-ack-wire-builder.txt): `9b174be1661356e1de1b0aefc721272b152193f8ebc010c3f4dab168007757e0`

## Pre-ACK conversion, SQLite persistence and retirement (2026-11-04)

**Confirmed static**, same pinned43.72 APK digest reverified. This continuation
closes the earlier unknown about whether a durable pre-ACK path exists. It does
not establish that every live count notification selects that path, or port it.
All files below are checked DEX CLI exports, not REA Evidence envelopes.

### Conversion and writer

1i8.A07 checks Message.arg1=76 at0014–001c and targets0202. It requires Eoh,
builds F3e.A05(Eoh.A04,Eoh), retains Eoh.A00, and constructs1vZ(node,null,id)
at02a0–02ac. Thus the notification ACK's exact generic builder is shared by
direct76 and converted43 paths; conversion is not an unrelated new ACK grammar.
Other supported message cases include9,11,129,157,419,431,475 and71–73; unsupported
cases returnnull. This inventory does not establish authorization for all those
message types in Rust.

The existing0v0.A00 writer switch43->0cf8 extracts1vZ.A01/A00 and calls0v2.A01.
The newly retained0v2.A01 calls1rj.A04(node,loggableId) before logging. Therefore
43 writes an already-built ACK/receipt node; it does not await an IQ reply or
retire durable rows in that selected writer method. Loggable ID1vZ.A00 and SQLite
row ID1vZ.A02 are separate identities and must not be conflated.

### Persistence and selection

1iV.A0F snapshots A0I under a monitor unless A08 reports its configured capacity
condition; that branch clears pending memory and records dropped count. A08 is
a count/capacity predicate (A02>0 and A01>=max(A02,A0J.size)), not a traced
Boolean enable/disable gate. A01 reads1hy.A04 and caps values above2000; A02
returns0 when A01<=0, otherwise reads1hy.A05. Effective descriptor values remain
unknown; these are not new Rust defaults.

A0F's nonempty snapshot serializes each1vZ.A01 via27E.A01 into ContentValues
ptn and inserts into SQLite preacks through0KU.A09 inside a1hk transaction.
Nonnegative returned IDs produce new1vZ(node,Long(rowId),loggableId) values.
After nonempty insert results, A07 receives the new wrappers and a bounded
snapshot-sized prefix is removed from A0I. Exceptions are caught/logged as
failed_to_persist_pre_acks and reset the scheduling latch; an empty result does
not take that successful transfer branch. Exact partial/ambiguous commit failure
semantics are not inferred from helper names or logs.

A0E starts loading once using AtomicBoolean/getAndSet, schedules2JI selector41,
and starts A05's periodic writer (1hy.A06 delay clamped to at least100ms).
A03 waits on the load latch for at most60000ms and reports timeout; a timeout is
not proof of an empty durable queue. The load Runnable and effective timing
values remain untraced here. 1iY.A03 queries SELECT _id,ptn FROM preacks WHERE
_id>? ORDER BY _id LIMIT?, and reports a nonpositive page limit before substituting1 if that helper returns. Its parser
and further paging logic remain retained without claiming complete reload parity.

A0D selects replay material from persisted A0J then pending A0I without clearing
them. With A01<=0 it returns the combined list or its last2000 entries. Positive
mode uses A00's bounded selection algorithm (untraced here). A0C selects a
confirmation marker, with deferred-batch state checks, rather than deleting it.
Neither selection constitutes confirmed delivery.

### Confirmation and cumulative retirement

1t2.A00 gets a1vZ marker through A0C, then calls1A7.A0A with8RG(callback holding
that marker) and the literal timing argument32000. Its log calls this sendPingAndClearPreacks. 8RG.CK4's
default selector calls1iV.A0J(marker) and optionally reschedules1t2.A01. The
latter is a scheduler: it removes/reposts a Runnable, using60000ms in the selected
passive branch or1hy.A03 clamped5000..60000ms otherwise. The exported filename
prekey-ack-replay-request.txt contains this scheduler, not a request constructor.
1A7.A0A obtains an ID via0Sf.A0G, constructsDzb selector16 and forwards the
node, callback, ID, selector22 and that timing argument to0Sf.A0Q. Dzb request
construction, A0Q timing units/correlation and error callback remain to trace.

A0J can hold removal while replay is deferred or the connection is passive;
otherwise it schedules Fef selector13. Fef.run's switch13->0364 removes the
pending prefix through an exact matching1vZ when found, or searches persisted
markers by row ID / node equality according to the selected mode. It updates
counts and calls1iY.A02 only when it obtains a row watermark. A02 deletes
preacks WHERE _id<=? with a nonnegative input check and catches/logs exceptions.
This is cumulative row retirement, not deletion by stanza ID or an inferred
ban/rate-limit signal.

The exact A0J caller inventory also includes0ZJ.A1S,2Gc.BsT,Fef.run andFeq.run.
0ZJ.A1S at006c compares its captured integer with A1L; a mismatch logs
superseded by a reconnect, not retiring and returns before retirement. Matching
state chooses A0H(epoch) or A0J(A08), clearing the held marker afterward.
2Gc.BsT invokes its wrapped success callback and can callA0J for a held marker;
its construction/origin is not yet traced, so socket-write success is not assumed
to be that marker's confirmation authority. Fef/Feq alternative caller branches
are not exhaustively audited by this selected13 branch.

**Port consequence:** Rust needs distinct capture, business-decision, ACK-node
queue, optional durable flush, write, epoch/confirmation watermark and retirement
states for native recovery parity. Existing raw inbound capture and an ACK socket
write alone do not implement this native queue. No automatic replay, duplicate
suppression, new ping, wait or deletion policy was added from this trace. Count
worker wire-order parity, bulkCCQ handling, complete reload/drain gates and
crash/ambiguous-commit behavior remain open. No live traffic was performed.

Exports:

- [prekey-ack-connection-drain.txt](prekey-ack-connection-drain.txt): `fa6d80803738f5f1e9d5262176a548d00b716c721bc79e614e29e08d04fcbbf3`
- [prekey-ack-conversion.txt](prekey-ack-conversion.txt): `eb41c1a8e70420f09881e13784678553df2d77898588ecc7f2c2bc90f6a11fac`
- [prekey-ack-delete-watermark.txt](prekey-ack-delete-watermark.txt): `a50b2a9865bc157581ea8788c6337820d3286ed9a6b20371bdf36a6d55a34b42`
- [prekey-ack-delivery-callback.txt](prekey-ack-delivery-callback.txt): `3236c6386ccc2b03bda3973a0a0052b3199b22f541584b6655a0e129b6f2f488`
- [prekey-ack-delivery-callers.txt](prekey-ack-delivery-callers.txt): `6e26f12354cb8f25327db36cea506573ea4e78b447684ffcdebd72f1c650c7a0`
- [prekey-ack-load-entry.txt](prekey-ack-load-entry.txt): `6c59893caabc6526b56768b976b4f35a999711866f88cadd31d94db848d144f5`
- [prekey-ack-periodic-writer.txt](prekey-ack-periodic-writer.txt): `868ef749294c68d310d40e5c6b4178e2e13c44295b6672ef249df795dbc18c13`
- [prekey-ack-preack-writer.txt](prekey-ack-preack-writer.txt): `ce5e593015b01265120626826bbe3d673c3629317f967ba0ffbf05921c96c54f`
- [prekey-ack-queue-A01.txt](prekey-ack-queue-A01.txt): `23658696947eea1c17c8e6701765ae7b699713d07c0daf93bd6d9fd5a6db47ac`
- [prekey-ack-queue-A02.txt](prekey-ack-queue-A02.txt): `40a045c726871f33b72b83252211d433fb0c98f7c7288dd09122183c6f714039`
- [prekey-ack-queue-A03.txt](prekey-ack-queue-A03.txt): `bc63d2beb4ab2986eafb8341bc6a6666c6adf61e05110ae82456b16c7d0c08d8`
- [prekey-ack-queue-A08.txt](prekey-ack-queue-A08.txt): `36022f21708b1b8a293ce59452885500da997b2b2b67b37e4bc5660cdb5ec1e2`
- [prekey-ack-queue-A0B.txt](prekey-ack-queue-A0B.txt): `b73ef3d336a5d65b1f0f7c80aa29a20620a5da25ae1f56bfdd6ad74ad0d3f9a0`
- [prekey-ack-queue-A0C.txt](prekey-ack-queue-A0C.txt): `ccf740b782e250f035dd700a9161fe21f82121ecc8c3b051a267f06be8213da9`
- [prekey-ack-queue-A0D.txt](prekey-ack-queue-A0D.txt): `8420f2fa7b6fac16d0d2821d98ee464c2fafeb91901f98e55d77bafcfa2653ef`
- [prekey-ack-queue-A0F.txt](prekey-ack-queue-A0F.txt): `ea2e082385f1bdc961553c77388a3e5577be2e96165ad3ac812947e44ce2418d`
- [prekey-ack-queue-A0J.txt](prekey-ack-queue-A0J.txt): `8077405fb8ecc8ad6dc68a565f1d3f5ebeb28de5a275672c12f802bac9db4a6c`
- [prekey-ack-queue-refs.txt](prekey-ack-queue-refs.txt): `971dd56e1ef64792206eef56ac46ced9043909f37aedd7c92ab5331a92d0eb9b`
- [prekey-ack-removal-worker.txt](prekey-ack-removal-worker.txt): `a4b7dda79fb51ca0b2c321c478ff1d10386e1016908adac5de07d1995b01cf65`
- [prekey-ack-replay-request.txt](prekey-ack-replay-request.txt): `fea24a37d67ec6a894f4e6e105c584ce1da87e5c93e4e1962abc9998b68ecfd2`
- [prekey-ack-replay-result.txt](prekey-ack-replay-result.txt): `94b158e03ae6f81108abbbaf335e2812f6af9cea7e9a91b3ecdbc786bd014c4d`
- [prekey-ack-replay-send.txt](prekey-ack-replay-send.txt): `2231a7523884c7e23fe69d4cbca75abb785270b22f5ea625a71b9585bb6b03bc`
- [prekey-ack-store.txt](prekey-ack-store.txt): `d2c50ce3377d739be960efc9ae11b743d484ed9d12dc9841de73e3b760c2c237`

- [prekey-ack-confirmation-ping.txt](prekey-ack-confirmation-ping.txt): `7f10dab1e03cbda36b03ceb97720a858ded99a81613afbf236b53a4a85d60ad2`

## Confirmation ping request and independent timeout (2026-11-04)

**Confirmed static**, pinned43.72 APK digest reverified; checked CLI exports,
not live traffic or REA Evidence envelopes.

1A7.A0A's previously retained Dzb(String,I) constructor specializes selector16.
It calls B1x.A0c (iq builder), B21.A1B (type=get), appends xmlns=w:p and to=0mL.A00,
then B23.A0e appends the destination and conditionally the supplied ID through
0Tk.A06. No child is added in this constructor. 0mL.<clinit> constructs its
singleton, its constructor supplies empty Jid user, and getServer returns
s.whatsapp.net. These are the semantic fields of Rust's existing keepalive IQ;
attribute byte order, ID-generation/validation and queue policy equivalence are
not inferred. The full Dzb constructor export includes other overloads too.

0Sf.A0Q forwards callback,node,id,selector22 and timing argument toA06. On ready
admission A06 first calls0ln.A08 to register the callback, then constructs and
dispatches the outgoing message throughA01/A05. Its disconnected branch calls
A0E(id) and callback.BsS without taking that send branch. Native0ln.A08 stores
the callback by ID under a monitor; for positive delay it schedules Handler
messagewhat4 keyed by that ID using sendMessageDelayed. Thus the32000 supplied
by the pre-ACK ping manager is a32-second callback delay, registered before
writer dispatch. It is independent of a recurring heartbeat phase. Timeout
message handling/cancellation and all error callbacks remain separate trace
boundaries; this does not establish total request latency under blocked workers.

**Confirmed local defect, fixed:** Rust's actor previously expired any pending
keepalive at the next heartbeat tick. Maintenance writes occur inside that
heartbeat before creating the ping, so a delayed maintenance write could consume
nearly its entire response window. The new paused-clock regression delays a
usync write19seconds within a20-second heartbeat: old code closes at the next
tick after the ping is only1second old. Corrected code stays responsive then
returns the existing recoverable timeout exactly20seconds after ping admission.

Rust now records (id,deadline) before sending each ping and selects its timeout
independently. Heartbeats still perform maintenance but do not create a second
ping while one is pending. Exact result/error correlation and the configured
local response duration remain unchanged. This is a local scheduling correction,
not a port of the native pre-ACK confirmation queue: the native32-second delay
is not substituted for Rust's existing configurable keepalive response window.
An actor blocked in an awaited write may still observe deadlines late; this
change does not claim fully independent reader/writer scheduling or interrupt I/O.

Regression: cargo test --locked -p whatsapp-api --all-features
 delayed_heartbeat_work_does_not_shorten_the_ping_response_window.
It also checks ordinary inbound server-ping processing while the outbound ping
remains unanswered; the existing native-derived pong omits the inbound ID.
Original regression failed with link closure; the corrected regression and
existing result/error/non-reply/silence keepalive tests pass offline.

Exports:

- [prekey-ack-ping-admission.txt](prekey-ack-ping-admission.txt): `a9950bb7cfad49f8d6d9d63fc20232c97c3018474add06d31ceed965b9c04e28`
- [prekey-ack-ping-constructor.txt](prekey-ack-ping-constructor.txt): `1ed579a36122786ffbab593b6442d12c018efaff836c23154dab0c30660ec935`
- [prekey-ack-ping-destination-class.txt](prekey-ack-ping-destination-class.txt): `9b7609bba92f00a97aeba1053898348d6f4c39cd17cb1c8a0001067ae16856d3`
- [prekey-ack-ping-destination-constructor.txt](prekey-ack-ping-destination-constructor.txt): `89b8f6102b053f434f2e2ff2111f82788ae0cd649ac1d5657bbbea01cce38851`
- [prekey-ack-ping-destination.txt](prekey-ack-ping-destination.txt): `980fe80b7482094daa3b7da1b6664efd2789018c46a6792ce436d99c8beaf33f`
- [prekey-ack-ping-identity.txt](prekey-ack-ping-identity.txt): `e336aa46c6992ae815e754c8d26ee7c50548f3e4af951f9673d4ce6f4ace3a81`
- [prekey-ack-ping-request-owner.txt](prekey-ack-ping-request-owner.txt): `1e1cfa0680809b0ffe191a5f394920317bea49600ecd12affab395cd38ac7456`
- [prekey-ack-ping-root.txt](prekey-ack-ping-root.txt): `e7bcf562b6d84960044e5cab5394998726164c1210dda625c0ef3094c1e7f251`
- [prekey-ack-ping-timeout-registration.txt](prekey-ack-ping-timeout-registration.txt): `99acfbec072ba7b3e3cd1818627a04fa3664293655cc0a429564b416f57dfee8`
- [prekey-ack-ping-type.txt](prekey-ack-ping-type.txt): `203fcd9e64b071140ef6579afb8cceada18c23762247371686f9889150e55be8`

## Pre-ACK reload, capacity and connection-generation ownership (2026-11-04)

**Confirmed static**, pinned43.72 SHA256 reverified. Checked DEX exports only;
no live branch/configuration values or REA Evidence envelopes are claimed.

### Configuration and initial load

1hy.<clinit> declares A04(property0x882e) andA05(0x882f) integer descriptor
values0/0; these are the replay window and cap read by1iV.A01/A02. A03(0x8709)
declares60000/60000 for ping scheduling, A06(0x8294)10000/10000 for periodic
flush (clamped at least100ms byA05), andA07(0x989a)0/0. A00 Boolean0x86ff is
constructed withfalse,false. A01(0x98e7) andA02(0x7c82) usefalse,true; which
Boolean member is default/alternate is not inferred here. Effective MobileConfig
and runtime descriptor overrides remain unknown; these are not new Rust defaults.

The retained2JI.run selector41 targets1fac. It reads the current window and:

- Window<=0: calls1iY.A01 dictionary check, then selects all _id,ptn rows ORDER BY
  _id. Normal results append toA0J and its size becomesA01 (persisted count).
  Caught load exceptions log failed_to_load_pre_acks and supply an empty result.
- Positive window: counts total rows (clamped0..i32::MAX), loads the oldest
  configured page ORDER BY _id LIMIT?, then can advance by WHERE _id>lastRowId
  when the decoded page is empty but a last row ID exists. It gives up after
  eight skipped pages. Count and page operations share an Fbg resource scope;
  its precise SQLite isolation/transaction semantics are not established here.
  The resulting list appends toA0J, total count becomesA01, andA06 records
  whether a nonempty loaded page is present. Normal branches release A0C latch;
  unexpected exceptions outside the selected catches are not claimed to do so.

Previously retained1iY.A00 preserves the last visited SQLite ID even when the
node decoder returnsnull and skips adding that row. Loaded1vZ has its persisted
row ID and loggable ID=-1; it does not invent an original live loggable ID.
1iY.A01 compares wa_dictionary_version with3, updates it when different, and
for an old nonzero version other than3 deletes the preacks table and returns
true. Old0 or3 does not take that delete branch. This is pre-ACK codec migration,
not authority to reset identities, prekeys, business records or raw captures.
The preference apply and SQL deletion are not shown as one atomic migration.
Value3 is a storage preference, not an APK release or proof of wire-dictionary
compatibility between builds.

### Selection, periodic capacity pruning and transaction insertion

The newly retained1iV.A00 computes min(pending+loaded, min(2000,window)). It can
restrict that count to loaded rows when not all persisted rows are loaded, the
loaded window is full, or the configured cap condition is reached. Together with
A0D (persisted first, then pending), it does not authorize leapfrogging an unseen
persisted prefix with newly produced ACKs. Nonpositive-mode last2000 selection
remains as previously documented; no unconditional cap2000 on the database is
inferred.

2JI selector42 targets1d76: it flushesA0F, callsA04, then with positive cap
prunes persisted rows when total exceeds max(cap,loadedWindow.size). Its SQL
selects IDs above the oldest retained boundary but deletes ORDER BY _id DESC,
LIMIT500 each pass. Thus it prunes the newest excess rows, not the oldest
unconfirmed prefix. It updates counts and reschedulesA05 even on its selected
exception path. Corrupt-page handling and cap pruning must not be merged into
confirmation-based retirement.

A separate1iV.A0I(0aq,List) path inserts serialized ptn values using the supplied
SQLite wrapper, creates row-ID-bearing1vZ wrappers for nonnegative insert IDs,
and registers TB5 selector11 through0aq.A04; the capacity path registersFec24.
0aq.A04 asserts inTransaction and registers an MZH adapter through0HC.A00.
Its exact commit/rollback callback semantics remain a next boundary; registering
this callback is not yet proof that it runs only after a successful commit.
The captured filename prekey-ack-transaction-persist.txt describes this method;
it is not an epoch snapshot API.

### Connection generation and sent cursor

1iV.A0A under a monitor clears sent row watermarkA04, clearsA05 and increments
A03, returning that connection generation. This is an in-memory replay epoch,
not AccountGeneration or a durable registration/identity epoch.

A0K(marker,generation), logged onBatchSent, compares that generation withA03.
A stale callback leaves the cursor alone. A matching persisted marker stores
its row ID inA04; a marker without a row ID instead setsA05=true. This method
marks sending progress; it does not itself delete SQL rows.

A0G(generation) clears the replay-deferred hold (A02=-1) only when the generation
matches; stale callbacks leave the current hold unchanged. A0H schedules Fec25.
Fec.run selector25->0154 rechecks generation at execution, reads A04, and if
present removes loaded entries at/below that row ID then calls1iY.A02's
cumulative SQL delete. Its updated persisted count is never below loaded size.
Thus the generation check remains inside delayed retirement, not merely at
scheduling time. Serialization against a simultaneous A0A epoch change is not
proved by that entry check alone. Sent cursor and clear-hold initialization callers, FcN's other
watermark write and complete batch delivery authority remain open.

**Port implications:** future Rust ACK recovery must distinguish durable row ID,
loggable ID, account generation, connection generation, loaded/pending windows,
sent cursor and confirmed retirement. Neither an empty loaded page nor a stale
connection callback authorizes clearing the queue. Declared defaults and these
selected methods do not establish full current-build replay/lifecycle parity.
No Rust recovery, send, reset, replay, wait or deletion policy changed here.

Independent SQL check: [preack_sql_reference.py](../../oracles/preack_sql_reference.py)
executes the retained predicates against in-memory synthetic rows with gaps in
row IDs. [Fixture](../../../../crates/whatsapp-api/fixtures/prekeys/2.26.43.72-preack-sql-reference.json)
checks strict-after paging, cumulative<= retirement, and pruning1205 rows to200
in newest-first delete batches500,500,5, preserving the oldest200. This verifies
SQLite predicate semantics only, not native callback, codec, load or crash parity.
Command: python3 docs/protocol/oracles/preack_sql_reference.py --check.

Exports:

- [prekey-ack-config-defaults.txt](prekey-ack-config-defaults.txt): `fdba4733593018a8fb594ee2b49a767e4e0aa3c5c8dee10e2ae2ca0be86eca09`
- [prekey-ack-epoch-begin.txt](prekey-ack-epoch-begin.txt): `0debb61a95c3ba5fee275f4d0e4462a2b609a33465599c85dbc270fc7a7a4c4c`
- [prekey-ack-epoch-retirement.txt](prekey-ack-epoch-retirement.txt): `edabfc9f78bb58af1f7d10dda3c2f9d597e886421b147296e93bf9b68dfd6551`
- [prekey-ack-epoch-worker.txt](prekey-ack-epoch-worker.txt): `3c62379a77a87336b3e9ab0acabb9e9ee57ee833a20a910557376ad12f9b5770`
- [prekey-ack-loader-worker.txt](prekey-ack-loader-worker.txt): `1f2028887956226083d0595ed98d5c118d82ed2b419d4a7f39353a3d8e4d5c0a`
- [prekey-ack-replay-clear-hold.txt](prekey-ack-replay-clear-hold.txt): `1261b91c08208fa33c1fe474fe38712b2f4fb6f062fed33928d928ed2ff39543`
- [prekey-ack-replay-selection.txt](prekey-ack-replay-selection.txt): `fbcda8d018559fac515357ddec02a7673b8955f282d3dc0cf47d22d81cda4d0a`
- [prekey-ack-sent-snapshot.txt](prekey-ack-sent-snapshot.txt): `48b423d8cb97aff3267834c8d01daf98b1282b841323adcbd3984e823a326d9e`
- [prekey-ack-sent-watermark-writes.txt](prekey-ack-sent-watermark-writes.txt): `a0aa1aa1bb968db4c1a9cbff69ab833c09e84a89984cda159a8e29e3f8e48572`
- [prekey-ack-transaction-callback.txt](prekey-ack-transaction-callback.txt): `e2d49eb8be1f175c28e9be6e9fab47443f8dd34fa7ba92a9d02c6eabc5c31088`
- [prekey-ack-transaction-persist.txt](prekey-ack-transaction-persist.txt): `844a43d0a263b4032a91a35b6bb4ee40eafdde0e1a3bb9af3de98180a882d935`

## ACK transaction completion and rollback bridge (2026-11-04)

**Confirmed static**, pinned43.72 APK SHA256 reverified. This closes the prior
normal-path transaction callback boundary. Checked DEX exports, not source,
runtime observations or REA Evidence envelopes. Ambiguous storage failures and
all count-specific bulk/config selection remain outside this claim.

### Registration is not execution

Previously retained0aq.A04 asserts SQLite.inTransaction and registers
MZH(Runnable,selector1) with0HC.A00. A00 gets a ThreadLocal callback map, puts
that adapter under a fresh object key and invokes its onBegin immediately.
Selector1's onBegin/onCommit/onRollback are no-ops; it runs the Runnable only
in NdI.BjU. The newly retained0aq.A03 uses selector2 for the counterpart channel.
The NdI contract exposes BjQ, BjU and all three SQLite lifecycle methods.

| MZH selector | Action relevant to registered object |
| --- | --- |
| 0 | Forwards onBegin/onCommit/onRollback to SQLiteTransactionListener |
| 1 | Runs Runnable in BjU; direct SQLite callbacks are no-ops |
| 2 | Runs Runnable in BjQ; direct SQLite callbacks are no-ops |

These selector mappings are explicit branches, not guesses from obfuscated names.
The map's fresh keys do not establish durable ACK deduplication across calls.

### Outer transaction completion

1hk constructor starts SQLite.beginTransactionWithListener(0HC) for the outer
transaction, sets 0HC.A01 ThreadLocal=true and rollback markerA02=false. Its
nested branch uses0KU.A0E -> SQLite.beginTransaction without resetting the outer
callback state. An optional supplied SQLite listener is wrapped as selector0.

1hk.A00 marks its local success flag then calls0KU.A0G ->
SQLite.setTransactionSuccessful. 1hk.close first calls0KU.A0F ->
SQLite.endTransaction and captures any thrown failure. Only when that returned
without a captured failure, the local success flag is set, and the rollback
marker is nottrue does it call0HC.A02. Other paths call0HC.A01, then a captured
end failure is rethrown. The normal closed flag is set after callback handling;
exceptions thrown by callbacks themselves are not claimed to be swallowed.

0HC.onCommit forwards onCommit to a snapshot of listeners, then clears the outer
active flag (also in its selected exception cleanup). onRollback forwards that
callback, then clears outer active and sets rollback markertrue, including its
selected exception cleanup. Neither callback directly executes MZH selector1's
Runnable.

0HC.A02 releases callbacks only when outer active==false. If rollback marker is
true it delegates toA01; otherwise it snapshots and clears the callback map,
then invokes BjU on each adapter. A01 similarly waits for outer active==false,
snapshots/clears and invokes BjQ. Therefore the selected ACK projection Runnable
runs after successful outer completion, not while an inner wrapper closes or
inside SQLite.onCommit, and not on the normal rollback channel. A captured
endTransaction failure selects the non-success release path; its precise engine/
disk outcome is not established by this static control flow.

Exact caller inventory for0HC.A02 reports1hk.close. Exact1iV.A0I insertion
callers are1ic.A0A andFlr.invokeSuspend. The latter coroutine remains untraced;
these inventories do not rule out inlining/other interface behavior.

### From persisted ACKs to the in-memory window

1iV.A0I, retained earlier, inserts ACK blobs using the supplied0aq transaction
and registers TB5 selector11 via0aq.A04. TB5.run switch11->0430 reads the window
mode flag and load latch. Its selected non-window/load-complete path calls
1iV.A07 directly; other paths schedule TB8 selector8 on the queue executor.
That TB8 branch remains a trace boundary. A07 itself returns if initial load
is incomplete; it is not authority to erase SQL rows because memory was skipped.

A07 updates the persisted-memory window only after its latch check. It compares
incoming and loaded row-ID boundaries; when out-of-order entries need merging
in positive-window mode, it combines and sorts using Fey14 and truncates via
the configured window helper. Normal append/window-capacity branches and the
persisted count update are retained. Exact Fey14 comparator and helper semantics
are not reimplemented or claimed fully audited here. Callback order must not be
mistaken for monotonic database insertion order.

### Bulk route bridge and limits

The earlier0v6.A00 BX2=true route calls1ic.A0G. That method validates homogeneous
stanza runtime classes, then constructs8b6 with selector1 and can execute it
inline or via an executor. The retained constructor binds that selector and its
objects; run selector1 calls1ic.A0A with the same manager, callbacks, lists and
long argument. Error handling calls a separate cleanup Runnable before rethrow.
Many admission/cache/claim details insideA0G are retained but not exhaustively
interpreted by this bridge trace.

1ic.A0A opens a0aq/1hk transaction, invokes queue-processing helpers and, when
the supplied pre-ACK list is nonempty, calls1iV.A0I with that same0aq. It then
registers its success Runnable throughA04 and optional rollback Runnable through
A03, marks successful, closes1hk and closes0aq. Empty-work paths can invoke a
Runnable without opening this transaction; this is not proof every bulk count
notification inserts an ACK row. Complete source of the pre-ACK list and actual
count-specific selection remain open. Queue helpers' internal business semantics
are not inferred merely because they run inside this transaction scope.

**Port consequence:** a faithful Rust bridge needs durable ACK insertion in the
business transaction and memory/scheduler promotion after confirmed completion,
with a separate rollback path and recovery from durable rows if promotion never
runs. This is an implementation requirement inferred from the normal native
path, not a completed Rust capability or proof against power loss/fsync failure.
No Rust ACK replay, send, ordering, timeout, reset or deletion behavior changed.

Exports:

- [prekey-ack-txn-adapter.txt](prekey-ack-txn-adapter.txt): `fb9a8c21dc0fb49e1ee90f435fee653c8069b619b7a5a987b9cac2df568ba3a7`
- [prekey-ack-txn-begin.txt](prekey-ack-txn-begin.txt): `a48d10a15ba84cb89483411719f2bb6a2af7b3f3e6d15c703e718a134d8ba53f`
- [prekey-ack-txn-bulk-admission.txt](prekey-ack-txn-bulk-admission.txt): `98c02644887a6216b37841f4baed8fb76c32988a0a7c4f0e6cac0c394b2d0426`
- [prekey-ack-txn-bulk-transaction.txt](prekey-ack-txn-bulk-transaction.txt): `153ce16bcae85dee04f8e980a0fb41a853be54fccbf4ac9475cf3c258cb0988d`
- [prekey-ack-txn-bulk-worker-constructor.txt](prekey-ack-txn-bulk-worker-constructor.txt): `92d94abf02a7f188cb96d49b099b261baa1bb3ef6bbabcd6876ca21d7342faec`
- [prekey-ack-txn-bulk-worker.txt](prekey-ack-txn-bulk-worker.txt): `826baf3e1355b6e93e5c024f9c328dbcdef0324ec2feb8fe405e7bcf263db164`
- [prekey-ack-txn-close.txt](prekey-ack-txn-close.txt): `7e639790c9f027e259583516944941eb69ab0e1586f7bcaf071e7b5424370402`
- [prekey-ack-txn-completion-callers.txt](prekey-ack-txn-completion-callers.txt): `93873c448ece56b0364f1ffc65c3c3ec7ae56c5d08e884b267daed0ca3c08628`
- [prekey-ack-txn-contract.txt](prekey-ack-txn-contract.txt): `452a91f9d16c030b19e0c2f7c160859614cd49b0e1b23706530cc1ac66268c55`
- [prekey-ack-txn-end.txt](prekey-ack-txn-end.txt): `fec4e6e0d607c0876e3fca30f52bf30375027a1997385c9cb3cf1794dc16e23e`
- [prekey-ack-txn-insertion-callers.txt](prekey-ack-txn-insertion-callers.txt): `5e3fd39dcba9b22596532b9efacb3347c1a9cd78842965b18cc1d25749bbc9e6`
- [prekey-ack-txn-lifecycle.txt](prekey-ack-txn-lifecycle.txt): `8a41e423ef4af69a711ee25c20e9c10fdae0bcc589d87a390041d3be5d80c579`
- [prekey-ack-txn-mark-successful.txt](prekey-ack-txn-mark-successful.txt): `7b4f0ef50a4fefa6315b283fbbf3eeba24cd0763fd7bc2afffd2ccc52bc6c5d4`
- [prekey-ack-txn-memory-projection.txt](prekey-ack-txn-memory-projection.txt): `f37a35c89a135d3880e342ce2ebd1da855be5ed316b4fdd8fc0bb4e3d9cd6b0a`
- [prekey-ack-txn-nested-begin.txt](prekey-ack-txn-nested-begin.txt): `dd5a6d1203ddfcde15e04418ffa39d5bf9a81553061fec4cfe23889c58baa813`
- [prekey-ack-txn-projection.txt](prekey-ack-txn-projection.txt): `31af988f595c811213c3154b71d2c45a51c2fbdded64694dd194567bb335e926`
- [prekey-ack-txn-register.txt](prekey-ack-txn-register.txt): `7dcc7322dcc8cc1413b30723dc2936593c02e80d3e04a42e6e1a9c176a2d4a2b`
- [prekey-ack-txn-rollback-registration.txt](prekey-ack-txn-rollback-registration.txt): `e3609a3ab7864c3c02582891cf17de28b48ccccd959f66f3bd6318920e5b5729`
- [prekey-ack-txn-success-marker.txt](prekey-ack-txn-success-marker.txt): `00ac7bb171f8f0137d3fba8bf80e98894c90031fda43f95190d6f27a00eb49f6`

## ACK projection serialization, merge order and refill (2026-11-04)

**Confirmed static**, pinned43.72 digest reverified. Checked DEX CLI exports,
not runtime execution or REA Evidence envelopes. This continuation closes the
TB8 projection and comparator/helper boundaries left open in the preceding trace.

### Scheduled projection is not a second load barrier

TB5 selector11 constructs TB8(list,manager,8) when projection must be queued.
The Object/Object/I constructor binds A00=manager,A01=list and$t=8. TB8.run
switch8->10a6 simply calls1iV.A07(manager,list); it does not independently await
the initial load latch. A07's previously retained incomplete-load branch logs
and returns, without deleting persisted rows. Do not infer a second load or an
unconditional memory promotion merely from scheduling TB8.

1iV constructor creates A0B=new098(provider08Z,false), two queue lists, the
initial-load latch1, and the start-loading atomicfalse. This instance owns the
executor used by the load, periodic flush and scheduled projection paths already
retained. It initializes replay-deferred A02=-1. A static field-write inventory
for generation A03:I finds only the explicit DEX write in1iV.A0A; Java's default
zero initialization and reflective/native possibilities are not additional DEX
writes or disproved by that inventory.

### Serial queue execution

098.execute synchronizes, wraps the Runnable in1VR(selector0), offers it to an
ArrayDeque and starts A00 only when no active Runnable exists. A00 polls one
head, records it as active, then dispatches through08Z.CbD for this false-mode
instance (CbN is the alternative mode). Constructor bindings for1VR are retained.

1VR selector0 invokes the wrapped Runnable under a separate monitor, updates
its counters/thread marker, and calls098.A00 before returning. The selected
exception path also attempts A00 before rethrowing. Selector1 is the delayed
adapter: it removes its delayed-map entry then calls098.execute, rejoining the
same serial queue rather than directly running the payload in parallel.

**Bounded conclusion:** jobs submitted to the same instance do not overtake a
previously queued load task. This is not a global database lock, proof that all
projection submissions follow load startup, or serialization against every
synchronous A0A/A0K call on other threads. Underlying08Z dispatch rejection,
process death and unscheduled callback loss remain outside the progress claim.
The earlier epoch-worker entry check still is not an atomic SQL-generation CAS.

### Exact row-order merge helpers

1iV.A07 constructs Fey(14); its constructor binds$t directly. Fey.compare
switch14->01dc reads1vZ.A02 (SQLite row ID), substitutes Long.MAX_VALUE for a
missing ID, then2LC.A02 ->1lN.A00 ->Comparable.compareTo performs natural Long
comparison. Loggable ID1vZ.A00, stanza ID and account generation are not used
as comparator tie-breaks. A null ID compares equal to a real Long.MAX_VALUE ID;
this fallback is not evidence such a null marker was persisted or confirmed.

For the out-of-order positive-window branch,07Z.A15 creates an ArrayList with
loaded collection entries first and incoming List entries appended.07Z.A1O's
Collection path copies to an array and02C.A07 invokes Arrays.sort(comparator).
07Z.A1L returns the first requested number (zero gives empty; negative throws).
These are concatenate/sort/take operations, with no distinct-by-row-ID step.
For example the static sequence for loaded[40,80],incoming[20,60],window3 gives
[20,40,60]; duplicate[40,40] is not eliminated by these helpers. These are derived
synthetic examples, not observed APK runs or complete merge-state guarantees.
Memory projection must not be mistaken for SQL insertion or ACK retirement.

### Refill preserves read failure versus exhaustion

1iV.A04 returns for window<=0. Under its monitor it computes requested=
min(persistedTotal,window)-loadedSize, and only queries for a positive deficit.
The strict-after cursor is the loaded tail's row ID when present, else0. It
calls1iY.A03(requested,cursor), whose SQL and corrupt-page handling were retained
in earlier exports.

A null read result logs refillWindow read failed/retrying next cycle and returns
without applying the successful-result updates. A nonnull list appends to the
loaded window and sets A06 according to whether this fetched page is nonempty.
If fewer entries arrived than requested, total count decreases by that deficit
but never below loaded size. This is count reconciliation following a completed
read, not cumulative SQL deletion or confirmation. Full DB/cursor snapshot
isolation and exact concurrent mutation behavior remain separate questions.

**Port consequence:** keep load scheduling, persisted ACK rows, ephemeral window
projection, strict-after refill, sent marker and confirmed retirement separate.
Do not add silent deduplication, erase durable rows on a skipped projection, or
convert a backend error into an empty successful read as claimed native parity.
Rust ACK recovery is still unported; this evidence adds no sends, replay,
ordering/timing, reset or deletion behavior. No live account operations.

Exports:

- [prekey-ack-generation-writes.txt](prekey-ack-generation-writes.txt): `bcad7319dd43783d6215eb8ff610638a46e47f9e35b7112536cba2aed769a8b6`
- [prekey-ack-projection-constructor.txt](prekey-ack-projection-constructor.txt): `b9489d2e40a44fa5fc8115a331850cb23294253404657cfb259894865564e600`
- [prekey-ack-projection-worker.txt](prekey-ack-projection-worker.txt): `d3c417eb0c436d31d290d139511f912988de4d8b03e2d484e99c1382728f91fe`
- [prekey-ack-queue-constructor.txt](prekey-ack-queue-constructor.txt): `0884612f1b230f3149b5d5b69be1417bc0f075dbf00625122a739263ba65ffc5`
- [prekey-ack-serial-executor.txt](prekey-ack-serial-executor.txt): `436856f64de0703eba8d031176caf894184dfdfc9330e7b62aba9e2d077e6202`
- [prekey-ack-serial-worker-constructor.txt](prekey-ack-serial-worker-constructor.txt): `b466b7b103f6bbdbb9fb61050cf813742a8e2c4d1507de894cd7d209b340174a`
- [prekey-ack-serial-worker.txt](prekey-ack-serial-worker.txt): `05bc6e8b4ca8bc05d365884b7a49de0293bfed403d3d8888420fb4dd2d8ddf4b`
- [prekey-ack-window-array-sort.txt](prekey-ack-window-array-sort.txt): `911354be07389bb8800d45820961e6339b35c159888e96d0da1c52dcd59cba7a`
- [prekey-ack-window-combine.txt](prekey-ack-window-combine.txt): `a6523510de1e98faf3f93476bfe3beade42a03f542b8637fbdaf8c286b6dfd91`
- [prekey-ack-window-comparator-constructor.txt](prekey-ack-window-comparator-constructor.txt): `9d5e79311d3d10b412dec71bfe699d5cb9db59013f370c91abbc03a6221bc45f`
- [prekey-ack-window-comparator.txt](prekey-ack-window-comparator.txt): `d4c076c957846867550ad8d2e1351443d6a5edfcc3c12514d7a06e144901f172`
- [prekey-ack-window-compare-helper.txt](prekey-ack-window-compare-helper.txt): `841bcbdea488030bbc3c047c0cb93daef6eb45708b09857d8331979857a5a8a1`
- [prekey-ack-window-natural-order.txt](prekey-ack-window-natural-order.txt): `66a5fb25a5cebf2f14797186f095282e4cca0df55062027b44de8ea9fdb6389a`
- [prekey-ack-window-null-row.txt](prekey-ack-window-null-row.txt): `af8e5b12c36c4e0ce5a37914dd10c515bd67397dc79d293764b04e39bccf4e59`
- [prekey-ack-window-refill.txt](prekey-ack-window-refill.txt): `1ecfb73888fa2ad6be6f83ae520e0af41d5f84d36244f8c381c37994f4c86030`
- [prekey-ack-window-sort.txt](prekey-ack-window-sort.txt): `40af51f80d36d33e0fcdf6866a0e63531d0791e61293f602a29f872069f5cd50`
- [prekey-ack-window-take.txt](prekey-ack-window-take.txt): `af7242b8182dd1a899fc56b226048d1627ba68b0216b484eb127bbb2e3519b1b`

## Message ACK wrapper origin follow-up (2026-11-04)

**Confirmed static:** the previously untraced2Gc selector1 constructor originates
in0Th.A02 registration; it snapshots the pre-ACK marker before send dispatch.
Matched ACK callback success invokes the wrapped result before A0J retirement.
[Exact origin, ordering and limits](../2.26.43.72-sender-key-ack-confirmation/README.md).
This supersedes the earlier constructor-origin unknown for this selected branch,
not the limits on other callers or socket-write confirmation authority.
