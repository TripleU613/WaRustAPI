# Receipt factory read policy and execution guard

2026-10-09. **Confirmed static**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
[Eleven-topic recipe](apk/recipes/2.26.37.71-receipt-read-policy.json),
[full signatures, operands and catches](apk/2.26.37.71-receipt-read-policy/targets.json).
Offsets are method-relative bytes. This closes immediate predicates of the
[previously retained factory](receipt-factory.md), not the effective saved
account policy or a complete receipt_user implementation.

## Global preference and chat exceptions

**Confirmed:** 08l.A1H reads SharedPreferences through A0Q/A02 and obtains
read_receipts_enabled with default true (0010..001e). A baked default is
not evidence that the preference currently equals true. Full setter/server
synchronization and preference-store selection remain **unknown**.

Previously retained 0zd.A07 accepts any of 0DE.A0m/A0b/A0h before reading
this preference or either private predicate. A0b was already traced as type
21 in [state-only evidence](receipt-state-only.md); new A0h is nonnull JID
type 7. A0m's types 1/2 with exclusions reuse
[receipt admission](receipt-admission.md). No human chat label is invented
for type 7 or 21 from the integer alone. Other chats require the preference
and both 0zd.A00 and A01 to be true.

The factory changes incoming status 8/13 to 5 only for a true fromMe key
when A07 is false. It preserves message, device, original user and timestamp.
This is a local factory policy, not a new wire receipt, account permission
or instruction to erase original authenticated receipt observations.

## Contact predicate A00

**Confirmed:** 0zd.A00 derives a nullable UserJid via 0D8.A00. Null chat
or no derived user returns true; no contact record is fabricated. For a
nonnull chat/user:

- 0yy.A0D(user) true returns true immediately.
- Otherwise 0YA.A09(chat) supplies a nullable contact. A present contact
  must have nonnull A0D.A0J (0lr), and that object's A03 or A02 must be true.
  Missing A0J or both false returns false. 0lr.A03 tests A07 startsWith
  smb:, A02 tests startsWith ent:; null string returns false. These are
  literal prefix predicates, not authenticated business identity claims.
- An absent contact instead calls 25v.A00(user) and returns its Boolean.
  False logs Disabling read receipts for possible spam. The diagnostic
  does not establish how the account/user was classified.

0yy.A0D short-circuits true for 1qW.A00 or 1dt.A01, or BMv true. Otherwise,
A07 false returns true; A07 true requires loaded 0DV.A0L true. Their full
classifier/contact semantics remain **unknown**; BMv's saved positive
registration-device predicate reuses [history mode evidence](history-mode.md).
No unknown predicate is silently set to true in Rust.

25v.A00 chooses SELECT EXISTS from premium_message_info. Both 0jZ.A04 and
property 13196 (0x338c) true select account_jid_row_id; otherwise chat_row_id
is selected, with optional 0Yo.A02 mapping when property 12879 (0x324f)
is true. Null selected JID returns false. 1xj.A03 supplies the SQL parameter
row, a positive smb_mm_exists result returns true, and missing/nonpositive
results return false. Full row resolution, schema/population, classification,
effective properties and mapping trust are **unknown** here.

**Confirmed negative:** its outer exception table catches RuntimeException,
logs a message naming CursorWindowAllocationException and returns false.
That diagnostic is narrower than the actual catch type; it does not prove
only cursor-window allocation failed. Nested close exceptions retain
suppression handling. This is not authority for automatic network retries.

## Account/contact predicate A01

**Confirmed:** 0zd.A01 returns true for null chat or 0Sa.A06 false.
Otherwise it obtains the nullable contact using 0YA.A09 and calls
8mp.A03(contact,chat); true from that helper makes A01 false, false makes
A01 true. Both nested policy layers remain distinct from the global preference.

0Sa.A06 first requires its A03 true, then reads integer
privacy_dependent_account_messages with default 1 through 0SW/08n.
Only value 1 returns true. This integer is not read_receipts_enabled.
Full A03 account eligibility, preference writer and synchronization are
**unknown**.

8mp.A03 requires 0Sa.A06 and its A09(chat), and requires A06(chat) false.
Then null contact returns true; a present contact returns true when A01(contact)
is true, or when contact.A02 (CUg) is null and 0rB.A00(contact) is false.
Other branches return false. Its nested classifiers, contact producer/trust
and account mode remain **unknown**. These exact Boolean branches do not
establish a complete public account restriction model.

## Entry guard is thread affinity

**Confirmed negative:** 15A.A02's first 1f0.A00 call is not a read-setting
refresh, database commit or network request. It compares the current Looper
to its two Handler loopers; the first match skips the second comparison and
sets true, otherwise it computes the second equality. It passes that Boolean
to 00K.A0C. Full Handler construction and
assertion failure behavior remain **unknown**. Rust must preserve its actual
actor/store ordering, not invent a privacy refresh at this call.

Full self/foreign admission, settings synchronization, message state,
receipt_user seeding and primary identity-change cleanup remain unfinished.
No runtime, saved account, keys, request IDs, reservation or live request changed.

```sh
cargo run -p xtask -- apk-research-bundle ~/Downloads/WhatsApp.apk \
  --recipe docs/protocol/apk/recipes/2.26.37.71-receipt-read-policy.json \
  --output docs/protocol/apk/2.26.37.71-receipt-read-policy --check
cargo run -p xtask -- apk-research-diff \
  docs/protocol/apk/2.26.37.71-receipt-read-policy \
  docs/protocol/apk/2.26.37.71-receipt-read-policy
```

Follow-up: [read-policy classifiers](receipt-read-classifiers.md) narrows the
configured user list, chat codes, contact-addon presence and dependent-account
gates. A predicate can schedule a refresh; true does not prove completion.
Effective state and classifier writer authority remain unknown.
