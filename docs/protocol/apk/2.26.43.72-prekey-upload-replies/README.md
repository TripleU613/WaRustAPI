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
