# Conversation counterpart mapping import entry points

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Four-topic recipe](apk/recipes/2.26.37.71-mapping-admission.json),
[complete methods and exception tables](apk/2.26.37.71-mapping-admission/targets.json),
[history processor's physical callers](mapping-admission-evidence.txt).
Offsets are method-relative bytes unless the inventory labels DEX byte offsets.
This closes the previously untraced physical callers of the
[conversation bulk mapping update](chat-counterparts.md#bulk-population-from-conversation-records)
only to their immediate input paths. It does not prove full authentication.

## Chat-transfer import

**Confirmed:** both CnI.A08 calls to 0pq.A0D belong to the importer whose
retained diagnostic identifies ChatTransferMessagesImporter. The method obtains
`migration/messages_export.zip` through its Dbt interface, reads header.json
as UTF-8 JSON, and selects its serialized-message description. It requires a
nonempty description path and case-insensitive `protobuf` type
(01c4..01f0); property 0x58ab chooses the two import branches.

The first branch opens a ZIP input stream at 029a..02a4 and parses delimited
BZA records at 02d2..02e6. On its first record it calls CnI.A09, then calls
0pq.A0B, A0C and A0D (A0D at 0314..031a) before iterating conversations.
The other branch also parses delimited BZA (05c0..05d4), calls A09 on the
first record, then A0B/A0C/A0D (A0D at 0602..0608). Both mapping updates
precede the later per-conversation A0E check; that check cannot be described as
this call's mapping-admission guard.

**Unknown:** Dbt's export provenance, full ZIP/header validation, A09's effects,
A0B/A0C prerequisites, effective property 0x58ab, import cancellation/transaction
boundaries and signature verification upstream. This is a stored transfer
source, not evidence that an arbitrary peer receipt can introduce an alias.
The retained whole method supports tracing these without another extraction.

## Initial history bootstrap

**Confirmed:** CoO.A02 accepts a byte array argument and immediately parses
BZA through GeneratedMessageLite.parseFrom (0008..001c). It calls A0C and
other processing, then switches on BZA.syncType_ (0124..0146). Case 0 enters
114e, labelled `apply_initial_bootstrap_sync_start`. That branch eventually
calls 0pq.A0D at 12ba..12c0, before iterating BZA.conversations_.
Other switch cases and their message/receipt imports are retained but are not
claimed completely traced here.

The bootstrap branch checks presence bit 0x80 for companionMetaNonce_ and
writes the supplied string to `companion_meta_nonce`; absent presence logs
`initial bootstrap did not contain companion meta nonce` and continues to the
next check. It is a presence/storage branch, not demonstrated nonce comparison
or proof of linked-device authority. The following branches handle
shareableChatIdentifierEncryptionKey_ and nctSalt_. Their bit conditions and
storage paths are retained; cryptographic interpretation is **unknown** here.
A missing-field diagnostic must not be converted into a universal mapping
rejection that this visible control flow does not establish.

## Immediate history input sources

The exact all-DEX caller scan finds two physical invokes of CoO.A02:

- **CoO.A06:** receives a File and retained chunk metadata, reads file bytes
  through AzT.A0Y with the literal bound 0x4000000 (64 MiB) at 0060..006e,
  then passes them to A02 at 0078..007e. A read failure has the retained
  `unable to read file data` diagnostic. The helper's precise limit/error
  implementation and the file's download/decryption provenance are **unknown**.
- **DNY.run:** the relevant runnable branch casts its state to
  HistorySyncCompanionWorker at 06b8, reads a saved chunk through 1h9.A02,
  and checks 1tI.A0A for nonnull inline bytes (0756..075e). It constructs
  Inflater(false), ByteArrayInputStream and InflaterInputStream, copies to a
  ByteArrayOutputStream through 0Wo.A00, and calls A02 at 0852..0858 with
  the inflated bytes and a null File. The retained inflation-failure path
  diagnoses the error and substitutes an empty byte array. This trace does
  not establish a decompression-size bound or a crypto verification step.

**Unknown:** how a chunk reaches 1h9/1tI, which account/device may stage it,
message-envelope authentication, downloaded history-media authentication,
all runnable discriminator/caller authority, and the actual peer sync receiver.
The full DNY.run is retained; its unrelated runnable cases are not protocol
claims. Inflating a payload is decoding, not authentication. No file, transfer,
chunk or mapping has been imported into the saved Rust account in this work.

## Consequence for the Rust port

**Inferred from these sources:** native receipt counterpart expansion consumes
chat migration state whose bulk producer is reachable from transfer, backup
restore and initial history-bootstrap processing. The producer's parsing checks
are insufficient by themselves to authorize a runtime alias from an incoming
receipt, raw PN/LID attribute or directory response. Complete upstream trust and
actual receipt_user row admission remain necessary for primary cleanup parity.
No Rust runtime, identity key, request ID, reservation or server wait changed.
No live request ran for this static trace.

```sh
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-mapping-admission.json \
  --output docs/protocol/apk/2.26.37.71-mapping-admission --check
cargo run -p xtask -- apk-research-diff \
  docs/protocol/apk/2.26.37.71-mapping-admission \
  docs/protocol/apk/2.26.37.71-mapping-admission
```

Agreement verifies static artifacts, not runtime authentication or sync parity.

## Queue producer follow-up (2026-10-09)

The immediate queue producer and SQL store are now retained in
[history companion queue staging](history-queue.md). Native peer processing
checks own-user and companion mode; complete linked-device/envelope trust and
history-media authentication remain untraced. Existing snapshots are unchanged.
