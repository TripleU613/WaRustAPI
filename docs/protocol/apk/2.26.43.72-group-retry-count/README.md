# Group retry encryption-child count — APK 2.26.43.72

**Confirmed static / Rust correction tested offline.** APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete checked DEX exports retain exact signatures and exception tables.
No APK execution, account, network request or live retry was performed.
Local extractor/interface records are not REA Evidence envelopes.

## Bind the callback instead of assuming a zero-caller method is unused

`FTy` implements **Fv3**, confirmed from digest-pinned DEX class_def interface
lists. Direct call search for FTy.AK7 returns0; interface Fv3.AK7 has calls from
FTe.ATZ and FTr.ATZ. The direct result is not a dead-code proof.

Complete SendE2EMessageJob.A06 at0f3a..0f6a constructs FTy with that job, then
passes it to EoG's constructor as Fv3. Complete EoG constructor stores it atA03.
Complete FTe.ATZ reads that callback, adds AK7(main result) to its outgoing child
list, obtains AKj fan-out children and wraps them in `participants` before
Ewq.A04 adopts the child list. Its main-result type2 branch and F3f.A06 mapper
identify **skmsg**. Complete FTr.ATZ is also already retained in the
[retry-root bundle](../2.26.43.72-retry-root-gates/root-normalization-smali.txt);
its separate addressing gates are not inferred away by this count trace.

This is a selected concrete construction/composition path. Alternative callback
FTx and calling signaling writers are not declared equivalent or fully covered.

## The same retryCount reaches main and pairwise enc nodes

- FTy.AK7 loads job.retryCount at000a or002e and passes it unchanged to F3f.A04
  (nonnull encrypted result) or A05 (null result).
- FTy.AKj loads the same field at001e..0022, moves it into v19 at00d0..00d4 and
  passes it as the integer argument to F3f.A08.
- F3f.A08, registers49/static18 one-word parameters, receives that argument in
  **v43**. It copies v43→v25 at0046..004a, then passes v25 to A04 at00d2..00d8
  and via v36 at02a0..02a6 for the two encrypted map branches. The separate
  no-ciphertext/list branches are not normal pairwise distribution ciphertext.
- F3f.A04 forwards its integer parameter to A09; its type mapper includes msg,
  pkmsg and skmsg. A09 emits `v`, `type`, then **count=String.valueOf(count)**
  if nonzero at0028..003e. It does not increment. Zero omits the attribute.

The [earlier receipt→job producer trace](../../retry-enc-count.md) established
received count is unchanged into retryCount. Thus the correction applies that
count to the selected main skmsg and the pairwise distribution enc children,
not to a root message attribute, fresh message ID or independent local counter.
No mandatory telemetry or account restriction rule is inferred.

## Rust scope and reproduction

The explicit CLI group/status comparison `answer_one` now applies received
count1/2 to root skmsg and participants/to/enc msg/pkmsg **after** the existing
first-send builder and **before** peer/checkpoint reservation. The helper preserves
ciphertext, original ID, addressing, phash and unrelated children/attributes.
Count is inserted after v/type as in A09; stale count attributes are replaced.
Original first sends still use the unchanged zero-count builder. Unsupported
counts or unexpected retry topology refuse locally. This is not full group
retry/root/session policy parity or live acceptance.

Regression: two distribution targets (msg/pkmsg), a root media skmsg, counts1/2,
replacement of an existing count, exactly one count per enc and encode/decode
round trips. Removing only count from the decoded resend must reproduce every
original node field and exact encoded bytes. Invalid counts and missing/wrong
root enc refuse. No socket, crypto preparation or native method executes in it.

Each TXT is complete stdout of the corresponding checked extractor command:

```sh
target/debug/xtask apk-smali /path/to/WhatsApp.1.apk \
  --class 'LX/FTy;' --signatures --exceptions
target/debug/xtask apk-smali /path/to/WhatsApp.1.apk \
  --class 'Lcom/whatsapp/jobqueue/job/SendE2EMessageJob;' --method A06 --signatures --exceptions
target/debug/xtask apk-smali /path/to/WhatsApp.1.apk \
  --class 'LX/FTe;' --method ATZ --signatures --exceptions
```

Also reproduce EoG.<init> and F3f.A04/A05/A06/A08/A09 with the same flags.
Caller exports use `apk-callers --class CLASS --method METHOD`; their filenames
name the queried callback/fanout operation. `inspect-interfaces.py APK` reproduces
callback-interfaces.txt and verifies the APK digest. Checksums cover all exports.
Full job/callback exports are static evidence, not whole-method semantic parity.
