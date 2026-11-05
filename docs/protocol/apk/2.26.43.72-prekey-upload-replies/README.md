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
