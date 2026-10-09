# Receipt state-only suppression boundary

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Two-topic recipe](apk/recipes/2.26.37.71-receipt-state-suppression.json),
[complete methods](apk/2.26.37.71-receipt-state-suppression/targets.json).
This closes the immediate C8L/0Fs comparison in
[receipt admission](receipt-admission.md), separately from
[A07 state-only admission](receipt-state-only.md).
Offsets are method-relative bytes.

## Predicate and caller branch

**Confirmed:** C8L.A00(key,status) returns true exactly when key.A02
(fromMe) is false and status is 13, 16 or 17. True fromMe returns false
without testing those statuses. This helper does not query a timestamp, SQL,
a receipt user or a device version.

In the retained [15A.A02](apk/2.26.37.71-receipt-admission/receipt-admission.txt)
state-only pool, original receipt statuses map 8->10, 13->16 and 17->17;
others use -1 and skip the call. C8L sees that mapped state argument, not
the original receipt status. Thus this caller's mapped 10 bypasses the
comparison, while incoming-key mapped 16/17 enter it. Helper support for
argument 13 does not prove this pool produces 13.

At 0826..0858 a false C8L result branches directly to the allow value 1.
A true result requires nonnull key chat, calls 0Fs.A0B(chat) and compares
its long result to message.A0j. When the result is greater than or equal
to A0j, it selects value 0 and skips A07; a smaller value permits A07.
A missing required chat throws IllegalStateException at 0882..0892.
This is not a silent default chat or an instruction to manufacture one.
After A07, only a true result adds the CYV to this pool's result collection.

## Chat value and retained provenance

**Confirmed:** 0Fs.A0B calls the existing 0Fs.A02 cache loader, gets the
chat's 0Ze model and returns A0T. A missing model returns literal long 1,
not zero. The full loader is already retained in
[receipt counterpart evidence](apk/2.26.37.71-receipt-counterparts/chat-cache-loader.txt):
at 02cc..02d8 it obtains the last_read_message_row_id column index in v71;
at 0742..0754 it reads that index with Cursor.getLong and stores A0T under
the monitor. The corresponding second loading branch writes A0T at
1334..1346. This is a stored last-read message row boundary, distinct from
last_read_message_sort_id and receipt_user timestamp columns.

The [physical writer inventory](receipt-state-suppression-writers.txt)
found eleven A0T writes in eight method declarations: two loader stores,
three constructor stores and six other stores. Apart from the retained
loader provenance above, these writes and the schema/update authority are
**unknown** here. In-place semantic changes and transitive caller authority
are outside this field-instruction scan. No full chat read-state lifecycle
or receipt_user membership is established.

**Negative results:** the adjacent 0Fs.A0C was checked during lookup; it
uses A0d/A0b and an optional 1j2 helper, not A0T, and is not the caller's
boundary getter. The existing helper's missing-model value 1 is observable
in static code; it is not a server-prescribed WhatsApp requirement for our
Rust storage.

Full self-receipt trust, aggregate message state, ordinary user-row seeding
and primary cleanup remain unported. No runtime, account state, keys,
request IDs, reservations or live requests changed.

```sh
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-receipt-state-suppression.json \
  --output docs/protocol/apk/2.26.37.71-receipt-state-suppression --check
cargo run -p xtask -- apk-field-refs ~/Downloads/WhatsApp.apk \
  --class 'LX/0Ze;' --field A0T --type J --writes-only
```
