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
