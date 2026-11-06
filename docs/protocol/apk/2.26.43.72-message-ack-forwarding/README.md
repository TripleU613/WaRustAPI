# Ordinary message ACK forwarding — APK 2.26.43.72

**Confirmed static**, checked DEX text; no runtime observation or REA Evidence
ID. APK SHA-256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`
reverified 2026-11-05. Exports use `target/debug/xtask apk-smali APK --class
CLASS --method METHOD --signatures --exceptions`. Offsets are method-relative
bytes; branch displacements are signed DEX code units (two bytes).

## Selected ordinary message paths reach callback lookup

The fresh 1rH.A03 export is byte-identical to the retained
[complete parser](../2.26.43.72-distribution-ack-admission/native.txt).
The message class branch starts at 0282. After its eligibility and required
address/ID checks, the success-event call 1lZ.A10 at 046a..0470 is followed by
`0470..0474 goto/16 -0x1ca`: 0x470 - 2*0x1ca = 0xdc.
The refusal-event call 1lZ.A0z at 04a0..04a6 is followed by
`04a6..04aa goto/16 -0x1e5`: 0x4a6 - 2*0x1e5 = 0xdc.
At 00dc..00e8, A03 calls 1lZ.A1D with original node v1 and Eoh key v0,
then returns. The event calls receive converted DeviceJid values in other
registers; they do not replace v0/v1 before these jumps.

Together with the previously retained
[concrete generic bridge and callback equality](../2.26.43.72-ack-key-normalization/README.md),
this connects these selected ordinary message event paths to 1lY.Big,
0ln.A09 and handler6's Eoh-key lookup. It does not establish that every ACK
reaches that lookup: numeric error459 returns through A1F before the class
switch; malformed and other early paths have their own conditions. Event
emission before forwarding is static call ordering, not proof asynchronous
consumers finish before the matched callback.

## Destination helper is migration-specific

0x9.A01 validates A06 or A09 and explicitly asserts that the input needs FBID
migration. Its A09 branch returns a constructed 0xC with a fixed service ID;
the alternate branch converts through 0Dn.A00 and 0Dn.A08. A09 itself requires
0Dn.A0Y and equality with one fixed public service user string. Those literals
are APK service constants, not account data. Complete exports are retained.
This is not evidence of a general PN/LID alias resolver. A06, A0Y, A08 and all
concrete type/server mappings are not established by this bounded inspection.

The earlier SendE2EMessageJob.A06 export contains calls to this helper and
assignments from job.A0K, but its large branch graph has not been fully proven
to carry either value unchanged to every Efa construction. Do not infer a
universal outgoing/incoming address equality rule from this partial provenance.

## Constructor and port boundary

Efa.<init> initializes only A0A to a new map. Reference fields, including type,
therefore retain Java's null defaults until explicitly assigned. This proves a
builder default, not that every relevant producer leaves type null. The exact
native out-of-band direct sender-key producer remains unbound. No Rust type,
participant, recipient or sender admission rule is added on that assumption.
Ewq's retained constructor separately accepts a DeviceJid and a Jid; callback
identity cannot safely be guessed solely from the serialized destination.

**Unknown:** exact out-of-band producer key inputs, alias selection, complete
parser/type mapping and durable distribution confirmation. Existing Rust
class/ID/auth-challenge guards and uncertain reservations remain unchanged.
No replay, reset or live account operation. The adjacent IQ-handler census
found outer result/error admission already present in prekey upload, device
query, media routing, action replies and active retry refetch paths; that is a
bounded source review, not proof every network family is fully audited.

`checksums.json` records the size and SHA-256 of every other bundle file.
