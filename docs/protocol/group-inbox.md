# Participant-scoped incoming group records

2026-10-09. **Confirmed local implementation; inferred compatibility design.**
Incoming group records use `(group, original participant user namespace and
agent, original wire ID)`. Two members using the same wire ID no longer suppress
one another as duplicates. Sender-key ratchets remain scoped to the actual
participant device. Direct and broadcast/status record selectors are unchanged.

## APK evidence and limits

**Confirmed static, 2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`:
[reaction key evidence](apk/2.26.37.71-reaction-key/reaction-model-key.txt),
`LX/Bal;->A00`, offsets `002e..0046`, reads `BWI.key_`, then
`BZa.participant_`, and constructs a participant-bearing message model. Existing
BZa descriptor evidence identifies participant field 4. This confirms original
participant attribution matters; it does **not** trace the APK's database primary
key, replay cache, or complete hosted/agent normalization rules.

**Inferred / local contract:** dropping the device/kind representation from the
logical user selector permits references to the original participant user while
keeping its actual device in the saved authenticated route. PN and LID user
namespaces and agent components remain distinct. The selector is
`received:<group>:group-<sha256>`; SHA-256 covers a JSON tuple of participant user,
server, agent and original ID. The group binds the prefix. This hash is local
storage, never a replacement wire ID. New private records retain `wire_id` for
inspection and unqualified read selection. No phone mapping is inferred from
matching numeric PN/LID strings.

## Legacy preservation and atomicity

**Confirmed local:** historical `received:<group>:<id>` rows keep their original
bytes, keys and event markers. Lookup decodes the saved original route or its
original capture. A matching historical participant selects that original row;
a different participant can acquire its own new canonical row after decryption.
Unknown/malformed historical attribution refuses instead of guessing ownership.
Two rows claiming the same participant/ID refuse instead of choosing scan order.
Historical projection accepts both selectors and retains once-only markers.
No marker reset, identity rotation, account migration or server retry is needed.

Before a new crypto/event commit, the original legacy row is compared again in
the same write transaction. A concurrent legacy attribution change rolls back
the commit. Quote/reaction lookup uses this selector too: each participant gets
its own saved text preview; opaque media is still never copied or consumed.
Attachment tickets remain hashes of the original record key, so existing tickets
keep their selectors. New records get separate tickets by participant.

## Read receipt selection

**Confirmed local:** `mark-read --participant <PN/LID>` selects an original group
participant. The library uses `Receipt.from` for this explicit read selector.
Without it, a group/ID pair must identify exactly one retained message. An
ambiguous request rolls back the complete queue transaction. The receipt's wire
route still comes from the original authenticated message, including its device.
Direct reads refuse this selector. Automatic opt-in reads select the received
participant; a nonzero agent cannot be represented by `DeviceAddress`, so the
automatic path queues zero rather than reading another agent's message. CLI
agent selectors refuse before constructing `DeviceAddress`. These are local
representation limits, not server requirements. Read-on-receive remains opt-in.

## Validation and unknowns

**Confirmed offline:** independently generated Signal sender-key distributions
and encrypted messages exercise the same-ID PN/LID namespace collision. Each
body decrypts, persists separately and supplies its own quote preview. Ambiguous
reads leave storage unchanged; explicit automatic selectors queue once each.
Reopen/redelivery preserves ratchets and checkpoints. Selector fixtures cover
legacy ownership, missing routes, duplicate claims and concurrent-change rollback.
The trusted generated SKDM fixture isolates inbox decryption from directory
admission; it does not demonstrate live peer authentication.

**Unknown:** native SQLite composite-key parity, full hosted/agent reference and
read support, and live same-ID group collisions. No fresh group message or read
receipt is authorized by this offline port. Receive-only validation and final
check totals are recorded in STATUS and the messaging checklist.
