# Broadcast receive dispatch and status hash exemption

**Confirmed static / runtime dispatch unported**, 2026-10-11. WhatsApp
2.26.37.71, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The shared broadcast namespace does not establish the native receive class.
This trace resolves parser selection and retains the next ordinary-message
consumer; it does not yet prove the complete decrypted-list-to-receipt path.

## Address and metadata selection

[CnN.A02](apk/2.26.37.71-broadcast-dispatch/node-flags.txt) parses the from
address, meta child and category. Its constructor arguments put the status
predicate into CaF.A0M and meta is_group_status==true into CaF.A0J, as confirmed
by the [CaF constructor](apk/2.26.37.71-broadcast-dispatch/flags-holder.txt).
[CkR.A00](apk/2.26.37.71-broadcast-dispatch/metadata-discriminator.txt) also
recognizes meta content_type=status_notify, or is_group_status=true.

[1RC.A06](apk/2.26.37.71-broadcast-dispatch/processor.txt), 0058..0086,
combines those conditions. The DEX branch at 007a explicitly sets v5 to true;
the [Java envelope](rea/2.26.37.71-broadcast-dispatch/processor.json) leaves
the corresponding local looking uninitialized, so DEX is authoritative.
The status path additionally requires 0YZ.A0E() and category different from
peer at 008a..00c2. The meaning and persisted authority of that account gate
remain outside this trace.

| Input | Selection before the account/category gates |
|---|---|
| status@broadcast | Status candidate |
| Ordinary broadcast list, without status metadata | Ordinary parser |
| meta is_group_status=true | Status candidate |
| meta content_type=status_notify | Status candidate |

The candidate path calls 1RE.A01 at 00f2; all other paths call CgC.A01 at
0146. Thus namespace alone is insufficient, and a blanket rule removing
status class from every non-status broadcast address is also insufficient.

## Ordinary holder and next consumer

[CgC.A01](apk/2.26.37.71-broadcast-dispatch/ordinary-parser.txt) finishes
with Cih.A01. The [holder builder](apk/2.26.37.71-broadcast-dispatch/ordinary-holder.txt)
constructs Bp2. In 1RC.A06, non-newsletter Bp2 becomes 1UM and Message.what=0
at 029a..02b6. Bp1 separately becomes 1UN and Message.what=281 at 02ba..02e8.
The newsletter branch uses the ordinary holder with Message.what=280.

The [physical 1UM.A00 field inventory](rea/2.26.37.71-broadcast-dispatch/ordinary-holder-refs.txt)
has one constructor write and two reads:

- [16p.BFC](apk/2.26.37.71-broadcast-dispatch/ordinary-dispatch.txt) accepts
  what=0 and runs Ahc(this, holder, holder.A01, holder.A00, 19).
- [16w.BFC](apk/2.26.37.71-broadcast-dispatch/newsletter-dispatch.txt)
  accepts what=280 and runs AgE(holder.A01, holder.A00, this, 2).

The next trace must follow Ahc.run case 19 through decryption/message creation
to the ordinary confirmation writer. Existing [primary-device evidence](message-primary-device.md)
establishes 0y2.A00 -> Cp2.A04 -> node write, but does not connect this Bp2
admission to that writer. The runtime broadcast-list class=status shortcut
therefore remains explicitly unverified. No receipt policy was changed by this
evidence chunk. Group-status metadata, peer-category handling and the account
gate also remain unported receive distinctions.

## Status phash exemption resolved

[8oR.getType](apk/2.26.37.71-broadcast-dispatch/status-type.txt) returns 5;
[9Ln.getType](apk/2.26.37.71-broadcast-dispatch/list-type.txt) returns 3.
The [generic factory](apk/2.26.37.71-message-primary-device/jid-factory.txt)
maps broadcast/status to the 8oR singleton and other broadcast users to 9Ln.
[0DE.A0i](apk/2.26.37.71-broadcast-dispatch/status-predicate.txt) tests type 5.
The older status document's unknown hash-switch mapping is now resolved.

[1Yx.A04](apk/2.26.37.71-broadcast-dispatch/phash-exemption.txt) requires
that predicate and [1Yx.A02](apk/2.26.37.71-broadcast-dispatch/phash-types.txt),
whose exact set is {56,67,68,79,86,93,121,125}. Status addresses satisfy the
address condition; ordinary lists do not. Other send prerequisites and the
hash input set when this exemption does not apply are not settled by this
predicate. No outgoing hash validation was weakened.

## Reproduction

The [recipe](apk/recipes/2.26.37.71-broadcast-dispatch.json) retains full DEX
signatures, exceptions and register flow for thirteen topics. Eleven complete
REA envelopes retain engine identity and evidence IDs. Physical caller/field
inventories are pinned separately, without claiming Java override coverage.

```sh
target/debug/xtask apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-broadcast-dispatch.json \
  --output docs/protocol/apk/2.26.37.71-broadcast-dispatch --check
```

No fresh live validation was performed. This is a static dispatch checkpoint,
not a claim of broadcast-list parity or account-ban immunity.
