# Status processing gate and peer-category exclusion

**Confirmed static / peer exclusion implemented offline**, 2026-10-11.
WhatsApp 2.26.37.71, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
Status parser selection depends on cached configuration and migration state,
not just a broadcast address or a primary-account identity. Effective config
application remains unported; category=peer exclusion is now reflected in the
shared confirmation builder.

## Dispatch

[1RC.A06](apk/2.26.37.71-status-dispatch-gate/dispatch.txt), 0058..00c2,
selects the status parser only for a status candidate AND 0YZ.A0E() AND
category unequal to peer. Candidate address/metadata parsing is retained in
[broadcast dispatch](broadcast-dispatch.md). The exact peer exclusion is
independent of the configuration result: peer always takes CgC.A01, whose Bp2
handoff reaches the [ordinary receipt writer](broadcast-receipt.md).

## Cached configuration owner

The subsequent [effective Boolean provider trace](mobileconfig-boolean.md)
resolves readiness setup and the concrete managed Boolean context. It does
not yet establish the effective saved account configuration.

[0YZ.A0E](apk/2.26.37.71-status-dispatch-gate/receive-gate.txt) reads A05.get.
The [constructor DEX](apk/2.26.37.71-status-dispatch-gate/constructor.txt)
assigns A05 from 0WE.A00(new 1Xe(this,35)), at 00a4..00be.
The [exact field scan](rea/2.26.37.71-status-dispatch-gate/gate-field-refs.txt)
has one constructor write and reads in A0E/A08. The field type is 0WF, not 00s.
The [REA constructor observation](rea/2.26.37.71-status-dispatch-gate/constructor-observation.json)
reported no decompiled body; that is not evidence that this constructor is
native or abstract. Its DEX body establishes the provider assignment.

[0WE.A00](apk/2.26.37.71-status-dispatch-gate/cache-factory.txt) creates
0WF with an atomic reference to a lazy 00t. [00t.get](apk/2.26.37.71-status-dispatch-gate/lazy-get.txt)
synchronizes first evaluation, retains a non-null value and clears its provider
after success; errors propagate. [0WF.get](apk/2.26.37.71-status-dispatch-gate/cached-get.txt)
reads that provider/value holder. [A01](apk/2.26.37.71-status-dispatch-gate/cached-set.txt)
replaces it with an already-valued holder. Thus these getters are cached,
not a fresh computation from the latest props on every incoming message.

## Boolean inputs and exact time boundaries

[0YZ.A02](apk/2.26.37.71-status-dispatch-gate/receive-policy.txt) evaluates
cached A0I, cached A0H, property 19755, property 20675, in that short-circuit
order. [A0I](apk/2.26.37.71-status-dispatch-gate/cached-read.txt) reads A08,
initialized by 1Xe case 33; [A0H](apk/2.26.37.71-status-dispatch-gate/cached-write.txt)
reads A07, initialized by case 32.

Let P(n) mean the effective native 07t/00D.A10 Boolean property, not a raw
returned AB-property row. Define D by
[0YZ.A05](apk/2.26.37.71-status-dispatch-gate/rollback-policy.txt):

```text
D = P(28925) && disabled_timestamp > 0
    && currentTimeMillis - disabled_timestamp < 93_600_000
read initializer = P(16422) || D
write initializer =
    (P(13279) && (P(30957) ||
        (write_timestamp > 0 && currentTimeMillis - write_timestamp > 93_600_000)))
    || !D
receive initializer = cached_read && cached_write && P(19755) && P(20675)
```

The [1Xe DEX](apk/2.26.37.71-status-dispatch-gate/lambda.txt) at 02f6..0330
sets case 33 true when property 16422 is true. Its Java local appears
uninitialized on that branch, so the formula follows DEX. The
[write-policy DEX](apk/2.26.37.71-status-dispatch-gate/write-policy.txt)
likewise resolves the Java duplicate-block warning. The threshold is exactly
26 hours: D uses strict less-than, while the elapsed-write bypass uses strict
greater-than. No nonnegative elapsed-time guard is added; native signed-long
subtraction remains part of the semantics.

[Property source](apk/2.26.37.71-status-dispatch-gate/property-source.txt)
is the 07t instance injected through 0YZ's A03 field.
[Preference source](apk/2.26.37.71-status-dispatch-gate/preference-source.txt)
is the separately injected 11l. These identities and parsed raw AB-property
values do not establish MobileConfig precedence, defaults or effective values.

## Timestamp and refresh effects

Case 35 in 1Xe.get computes receive policy and inspects
recv_flow_enabled_timestamp. A true result without a positive saved enable
timestamp sets it to currentTimeMillis and clears recv_flow_disabled_timestamp.
A false result with a positive enable timestamp clears it and sets the disabled
timestamp to currentTimeMillis. [11l.A08](apk/2.26.37.71-status-dispatch-gate/enabled-timestamp.txt)
and [A07](apk/2.26.37.71-status-dispatch-gate/disabled-timestamp.txt) write those
exact preference names using apply(). The raw getters read default zero.

[0YZ.A08](apk/2.26.37.71-status-dispatch-gate/reset.txt) refreshes all four
cached holders. It also compares the old and new combined send/receive state,
updates migration timestamps on a transition and refreshes again.
[A03](apk/2.26.37.71-status-dispatch-gate/send-policy.txt) has the same Boolean
inputs with property order reversed; [A0G](apk/2.26.37.71-status-dispatch-gate/send-gate.txt)
reads its separate A06 cache. Separate lazy evaluations must not be collapsed
into an assumed atomic configuration snapshot. Reset catches and logs errors.
The [physical reset callers](rea/2.26.37.71-status-dispatch-gate/reset-callers.txt)
are retained; callback registration and effective update ordering remain open.

## Rust change and limits

The shared builder now suppresses class=status for exact category=peer,
including status addresses and status_notify metadata. The account-aware
helper requires an actual status-class receipt before consulting own-status
identity policy. It no longer reads unrelated own-LID authority for a
peer-category ordinary confirmation. Other routes, incoming nodes and durable
account state keep their existing handling. Actor and legacy CLI share it.

The regression failed on the previous extra class attribute. Offline checks
cover both broadcast address forms with status metadata, unchanged raw nodes,
the ordinary attribute shape, and a contextual peer receipt with conflicting
saved own LID that must not enter status ownership handling.
This is not proof of complete peer-message admission, ordinary own-message
sender type or read receipt policy.

The remaining code shortcut is the effective StatusInfraConfig gate for
non-peer status candidates. Saved AB-property replies are evidence-only;
their current inspection does not apply MobileConfig precedence or this gate.
Migration timestamps are not fabricated in house state. No outgoing requests,
identity replacement, saved wait change or fresh live sample was introduced.

## Reproduction

The [recipe](apk/recipes/2.26.37.71-status-dispatch-gate.json) retains twenty-one
DEX topics with full descriptors, exceptions and register flow. Twenty-two
full REA envelopes include class summaries and the missing-constructor-body
observation. Complete physical inventories are pinned separately.

```sh
target/debug/xtask apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-status-dispatch-gate.json \
  --output docs/protocol/apk/2.26.37.71-status-dispatch-gate --check
cargo test --locked -p whatsapp-api --all-features peer_category_uses_ordinary_receipts_even_with_status_metadata
```

Validation: 936 workspace/all-feature tests, strict workspace Clippy,
warnings-denied Rustdoc, default CLI/API symbol tests, experimental CLI build,
formatting and diff checks passed. The bundle contains 26 files / 105,140 bytes.
Hygiene checked 2,848 local links and 136 checksum bundles / 28,411,782 bytes.
No fresh live validation was performed.
