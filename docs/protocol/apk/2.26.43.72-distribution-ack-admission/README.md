# Distribution ACK admission — APK2.26.43.72

**Confirmed static**, checked DEX text exports, not REA Evidence envelopes or
observed runtime. APK SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`
reverified2026-11-04. Offsets are method-relative bytes; branch deltas are DEX
code units. Commands: `target/debug/xtask apk-xref APK --anchor ack` and
`target/debug/xtask apk-smali APK --class CLASS --method METHOD --signatures --exceptions`.

## Confirmed class ownership

1rA.A02 maps1rH to the `ack` tag (0118..012e). 1rH.A03 reads `class` into
Efa.A06 at0064..0076. Efa.A00 passes that field as constructor argument5,
and Eoh.<init> stores it in Eoh.A06 (000a..000e). The ordinary message branch
compares that exact field with `message` at0282..0292; receipt, call and
status have separate switch branches. Notification and missing/unknown classes go to
1lZ.A1D, not the ordinary message branch.

**Important qualification:** A03 tests numeric error459 before the class switch
(00a2..00c6) and calls1lZ.A1F. This special refusal route is separate and is
not implemented or reinterpreted by this bounded preparation admission fix.

The fallback1lZ.A1D delegates through1lW.Big; its concrete implementation is
not connected here. Separately0ln.A09 packages node/key into handler message6.
0m2.handleMessage case6 removes a callback by Eoh key at0052..0058, not by
stanza ID alone. Eoh.equals compares class first (0026..0046), then from, ID,
participant, recipient, type, edit and other fields. The fallback error/edit
search after a failed exact lookup is itself gated on class `message`
(0062..0078); subsequent candidate comparisons use Eoh equality after clearing
edit. These observations refute treating native callback keys as an ID-only map,
but do not establish the full distribution callback registration path.

## Rust correction / offline proof

outgoing_group::Pending::matches previously accepted any ACK tag with the
distribution ID. It now also requires exact class `message`. The same predicate
guards success and error settlement, so another ACK family cannot release the
next distribution/group stanza or persist DistributionRefused for this waiter.
The existing normal_group_references_decrypt_with_exact_participants_and_never_replay
regression now injects missing, receipt, notification, call and case-mismatched
classes with/without error against actual prepared reaction distributions. It
fails before the guard and passes afterward; valid message ACKs still complete
the existing crypto/decrypt/reopen/no-replay flow.

**Unknown / next work:** exact native out-of-band distribution callback creation,
sender/participant/recipient normalization, durable confirmation coverage and
error459 handling for this local waiter. Eoh contains addresses but that is not
proof the waiter should require raw `from == outgoing to`; primary/companion and
PN/LID normalization need their own trace. No arbitrary sender rule was added.
The generic outgoing ACK ledger is unchanged and is not claimed to cover every
distribution. No live account or network operation occurred.

## Search boundary and negative result

Exact anchor `ack` produced24 physical string references; this is not an
exhaustive runtime call graph. The first1rL.A03 candidate handles stream/error
stanzas; 1rA.A02 identifies1rH as the ACK-tag handler. The1rL export is retained
to avoid repeating that detour. Existing native Android provider unavailability
was handled with the checked DEX CLI; these TXT exports are not fabricated REA
results. No new readiness or package installation was required.

## Checksums

- [router.txt](router.txt): `f1a9e0f0cfd210e978f8fc8f86167bf812a5196456de4c30939a2065a3954648`
- [native.txt](native.txt): `e809408589e3fa60971d9d4e1eaccb007e9d83525e74848e5f84ed187cd00b0d`
- [fields.txt](fields.txt): `349587fd5020bce518c36f486e5cfd40069338df46bbbbdcceb8f02195a1bc57`
- [value.txt](value.txt): `ef5655aede1d50217f2ab6dbfae846d2fa89b6dd7b530d82bba49f49e6bc9bd9`
- [fallback.txt](fallback.txt): `37ce6e21831661d8c3e8d35c56f4ad5af1fd61f99853ac70586cf5cbe362400d`
- [callback.txt](callback.txt): `df562e3811df46aa95c9fa1da42e06df89dd7064238e590602c7d75be6e877c7`
- [handler-callback.txt](handler-callback.txt): `4c8a65dfb8084a0e069c95bf190c90cf19c93a0dd6c6c362f198bacae16bc94f`
- [equality.txt](equality.txt): `06740cb4d370d788240a7d71ce0549d2f767811a20a73914575a03686070c174`
- [handler.txt](handler.txt): `d7b6358b54d457f4079315a840a52c31526d2f91f644829351ac598113ff482d`
