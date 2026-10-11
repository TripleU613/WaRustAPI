# Native reaction text limit

2026-10-11. **Implemented/offline**, APK **2.26.37.71**, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
Ordinary reaction receive/send validation now uses the APK's 30 Unicode code
point limit. It replaces the previous 64-byte and control-character checks.
The existing reaction API, direct/group builders, outbox validation, outbound
eligibility, ordinary projection and status-like decode share this boundary.

## Exact source

The [recipe](apk/recipes/2.26.37.71-reaction-text.json) and
[checksummed DEX bundle](apk/2.26.37.71-reaction-text/targets.json) retain Bal.A01,
Bal.A02, CB2.A00, 6bF constructors and DGD.CF7. Independent complete REA evidence
retains the [builder](rea/2.26.37.71-reaction-text/reaction-builder.json),
[ordinary validator](rea/2.26.37.71-reaction-text/reaction-validation.json),
[text policy](rea/2.26.37.71-reaction-text/text-policy.json),
[constructor inventory](rea/2.26.37.71-reaction-text/text-class.json) and exact
[String constructor](rea/2.26.37.71-reaction-text/text-constructor.json).

Bal.A02 calls CB2.A00 with BWI.text_. Failure reports code 68. CB2 requires
nonnull text and constructs 6bF, accepting only A00.length <= 30. The String
constructor computes String.codePointCount over the entire UTF-16 string,
allocates an int array of that length, and fills it with codePointAt while
advancing Character.charCount. This is a code point count, not UTF-8 bytes,
UTF-16 code units or visible grapheme clusters. Supplementary emoji count as
one; combining marks, variation selectors and ZWJ count separately.

DGD.CF7's status-reaction path also calls CB2.A00 after key validation
(0064..0074). Its invalid branch declines the reaction. The existing status
like importer uses reactions::decode, so the shared correction applies there
without a separate validator. Bal's Java builder rendering has duplicate-block
warnings; DEX branch targets remain authoritative. Its own-envelope null-actor
branch rejoins the common BoQ call, which corrects older prose suggesting that
call was limited to non-own envelopes. Own-device reaction projection remains
unported in this chunk; those target/author branches need the same care as
[own-device votes](own-device-vote.md).

## Runtime and API contract

For valid UTF-8 protobuf strings, Rust chars count the same Unicode scalar
values represented by those Java code points. The shared valid_text check
requires at most 30; MAX_TEXT_BYTES becomes 120, the maximum UTF-8 byte length
of thirty scalar values. The public byte constant is an upper bound, not the
complete validator. Text and key bytes are never normalized or rewritten.
Malformed UTF-8 remains a protobuf refusal; unpaired Java surrogates are not
representable as valid Rust/protobuf UTF-8 strings.

The native predicate allows control characters within the count bound and
adds no emoji-category rule. Rust now follows that predicate; JSON serializers
retain their normal escaping. Empty decoded text remains a removal event.
High-level send requests continue to use None/remove for removal and refuse
an explicit empty Some text, preserving that API contract. Direct/group
builders share the predicate; chat ownership, parent ID, group participant and
other structural guards remain separate and unchanged.

Consequences include rejection of 31 ASCII characters that previously fit the
64-byte guard, and acceptance of thirty supplementary emoji (120 bytes) or
ten woman-technologist ZWJ sequences (30 code points, 110 bytes) previously
refused. This is the same limit enforced before durable enqueue and outbound
eligibility. Saved intents, attempts, facts, IDs, keys, idempotency records and
server waits are not rewritten. A historical intent that exceeds the native
limit no longer passes fresh validation/eligibility; no cancellation or send
outcome is inferred. Historical status-like inspection retains stored records
unchanged, rather than imposing the new receive predicate retroactively.

API descriptions and input text constraints now name the native count. No
route, option, dependency, automatic transmission or account migration is added.
Own-device ordinary reactions, encrypted-reaction effects, full edit/status
policy, native Unicode/UI selection equivalence and live acceptance remain
unfinished or unverified.

## Verification

The regression failed before implementation: the old decoder accepted 31 ASCII
characters. Independent fixtures exercise direct decode, status-like decode,
direct validation/encoding and group encoding at 30/31 ASCII or supplementary
characters, 10/11 ZWJ sequences, 15/16 decomposed pairs, control text and empty
removal. These boundaries distinguish bytes, UTF-16 units and graphemes.
An encrypted outbox test accepts the 120-byte text, rejects a new 31-character
intent with a fixed diagnosis and preserves all tables/journal/idempotent ticket
on rejection and identical repeat. Existing malformed key, parent attribution,
removal, namespace and transport tests remain active.

No live account was opened or request sent. Full APK parity and ban immunity
remain unproven; source-backed local validation is not fresh server acceptance.

Validation: 950 workspace/all-feature tests, strict workspace Clippy,
warnings-denied Rustdoc, 11 feature-minimal reaction tests, 26 default CLI tests
and experimental CLI build passed. The five-topic pinned recipe reproduced
all 10 files / 62,659 bytes. API-symbol, formatting and repository link/evidence
checks passed. No account or server request was changed.
