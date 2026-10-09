# Receipt read-policy classifier boundaries

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Twelve-topic recipe](apk/recipes/2.26.37.71-receipt-read-classifiers.json),
[complete selected methods](apk/2.26.37.71-receipt-read-classifiers/targets.json),
[two supplemental helper dumps](receipt-read-classifier-helpers.txt).
This extends [read policy](receipt-read-policy.md); effective classification,
preferences and server synchronization remain **unknown**. Offsets are
method-relative bytes; field names/enum members are build-specific.

## Contact-policy inputs

**Confirmed:** 1qW.A00 accepts only 0DE.A0h (JID type 7), then returns
Boolean property 1844 (0x734). This branch is narrower than general UserJid.
1dt.A01 returns false for null JID. It lazily reads string property 1607
(0x647), comma-splits a nonempty string and caches that List statically.
It compares Jid.user via List.contains, not the full raw JID or a device ID.
No trimming/case normalization is visible in this caller; the split helper's
complete semantics, cache reset and effective property values are untraced.
An empty/missing configuration returns false without installing that list.

0yy.A07 returns true exactly for classifier codes -1 or -3. Its A00 loads
a nullable chat model through 0Fs.A00(...,false). When property 25917
(0x653d) is false, it also consults 0yy.A01's optional override list; a list
containing the chat forces code 1. A missing model also returns 1. Otherwise
it returns 0Ze.A07(), a synchronized read of the model's integer A07.
The override helper reads call_not_spam_jids through 0V4.A02, comma-splits
nonempty input and delegates JID conversion to 0DE.A0C; absent/empty input
returns null. Parsing authority, invalid entries, field producers and persistence
remain **unknown**. These are local classifier codes, not message statuses,
server refusal codes or permission to replace saved classification.

0DV.A0L tests only A02 (CUg) nonnull. Thus the previously retained 0yy.A0D
branch combines chat classification with contact-addon presence. Neither the
getter nor the diagnostic establishes an authenticated contact category.
0rB.A00 separately tests contact.A01 (another 0DV) nonnull; the parent/linked
contact relationship's full provenance is untraced.

## Dependent-account branches

**Confirmed:** 0Sa.A03 delegates the result of 0Sb.A04. That method first
requires 0SW.A04()==0Sd.A02. It then uses WaAgeExperienceRepository strings
A06/A07 converted by 4nR.A00: either equals 4MZ.A03, or A06 equals 4MZ.A07,
returns true. Other branches return false. Exact enum names, age-state provenance,
conversion defaults and synchronization remain **unknown**. No age or account
state is manufactured or inferred for the current account.

8mp.A06 requires 1xj.A1W true, nonnull chat and 0DE.A0a true, casts to 0ZY,
then accepts when A5B.A0A(chat,0Sd.A04) or A0A(chat,0Sd.A05) is true.
A5B's stored-state lookup and mode authority remain untraced. It is not
proof that an ordinary LID alone satisfies either state.

8mp.A01 requires 0Sa.A07 and 8mp.A02(contact); complete subordinate policy
remains untraced. These methods must not be replaced by a single generic
contact-known Boolean.

8mp.A09 first requires 0DE.A0l, then rejects own-user equality (1xj.A1X),
0DE.A0P, 0hd.A02, 1xf.A1Y, 8kT.A1U, 8kT.A1V or its A06 result.
Complete helper classifiers remain untraced; own-user equality and the numeric
bot classifier reuse existing [own identity](own-identity.md) and
[route evidence](receipt-route.md).

**Confirmed side effect:** after those gates it obtains A5B/9LJ. When
9LJ.A02 differs from its AtomicInteger and 0Sa.A00 is true, it attempts
AtomicBoolean false->true and, on success, schedules AhT discriminator 47.
An exception during scheduling resets that Boolean and rethrows. Regardless
of whether that attempt schedules work, the admitted normal path returns true.
The refresh worker, concurrency/flag reset and state-loading authority remain
**unknown**. This predicate is therefore not established as a pure read of
one Boolean, nor does a true result prove refresh completion.

## Premium-query mode

**Confirmed:** 0jZ.A04 returns true for A02()==AzO.A04 or AzO.A03.
The underlying mode producer and enum meanings remain **unknown**. This narrows
the premium-message query switch retained in read policy, without proving which
SQL scope applies to any saved account.

Full contact hydration/trust, classifier writers, effective preferences,
settings synchronization, receipt_user admission and primary identity cleanup
remain unfinished. No Rust runtime, account state, keys, request IDs,
reservations or live requests changed.

```sh
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-receipt-read-classifiers.json \
  --output docs/protocol/apk/2.26.37.71-receipt-read-classifiers --check
```
