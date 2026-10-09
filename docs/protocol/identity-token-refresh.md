# Identity-change privacy-token refresh lifecycle

2026-10-09. **Confirmed static**, APK 2.26.37.71, SHA-256
`1755111afa2a1f24020411c1fd72ac13db1489b623a9ff7a30be3d98b7bc7d5c`.
The [26-topic recipe](apk/recipes/2.26.37.71-identity-token-refresh.json)
reproduces [bounded methods with registers and catches](apk/2.26.37.71-identity-token-refresh/targets.json).
This closes specific job/threshold/storage unknowns from the
[identity receipt trace](identity-receipts.md#privacy-token-refresh).
It does not implement a Rust refresh job or imply a new live capability.

## Which stored state authorizes the request

**Confirmed:** 0q2.A0O normalizes via A0L and consults a cache. A single-user
cache miss reads 0q4.A0J, whose SQL is `SELECT sent_tc_token_timestamp,
real_issue_timestamp FROM wa_trusted_contacts_send WHERE jid=?`.
0q4.A08 and HYS.<init> place sent_tc_token_timestamp in HYS.A00. The nullable
HYS.A01 is real_issue_timestamp when nonzero, the sent timestamp when the SQL
column is null, and null when that column is explicitly zero.

When 0q2.A02 supplies a counterpart, A0O fetches both users via 0q4.A0M,
then HOF.A00 takes the maximum sent timestamp and maximum nonnull issue value.
The returned HYS.A00 is the greater of those two maxima. 0C6.A0j/A0i compare
Comparable values and replace the candidate only when it is smaller, proving
maximum selection; A0i returns null for empty input. A1B filters null
collection entries before these maxima are computed. A0O drops an expired
combined value before caching it. The effective A0L/A02 counterpart gates are
not established by this job trace; existing canonicalization evidence remains
separate.

**Refuted shortcut:** Rust's incoming-trusted-contact rows store received bytes
and timestamps. This job's input is outbound issue/send state, not those bytes.
An existing outbound record also does not necessarily prove a server-acknowledged
send: counterpart aggregation can select real_issue_timestamp. The earlier
status note saying the job only refreshes a token it already holds was too
broad. No evidence here authorizes constructing an outbound record from an
incoming token or inventing prior issue history.

## Threshold and request construction

**Confirmed:** 0q3.A00 obtains 08B.A04 time, divides milliseconds by 1000,
and uses `now_seconds - 15,724,800` (182 days) as its fallback floor. For
positive bucket length L and valid-bucket count N, it returns
`max((now_seconds / L - (N - 1)) * L, fallback_floor)` with Java signed-long
arithmetic. Nonpositive L or N logs a bad configuration and returns the floor.
An unexpectedly low current bucket logs a strange configuration but still
computes that expression. This is a formula, not an assumed active expiry.
0q3.A01 takes the minimum of the bucket threshold from properties 996/997
(0x3e4/0x3e5) and A02, whose bucket properties are 865/909 (0x361/0x38d).
Effective property values and adjusted-clock parity remain **unknown/unported**.

**Confirmed:** GeneratePrivacyTokenJob.A0G reloads 0q2.A0O at execution time,
skips a missing value or one strictly below A01, and admits equality. It uses
the captured HYS.A00 for the token `t`, not current time. The writer constructs
`iq id=... to=s.whatsapp.net xmlns=privacy type=set` with one `tokens` child
and one `token jid=<UserJid> type=trusted_contact t=<stored timestamp>`.
No token bytes or fabricated proof are attached. GAZ.A1T supplies the server
JID singleton 1S7, whose getServer returns s.whatsapp.net; GAW.A0l names iq.
The job passes request code 299 and timeout 32,000 milliseconds to 0h8.A0T.
These native arguments do not establish this client's retry or timeout policy.

## Completion, errors and pending membership

**Confirmed:** constructor supplies generate-tc-token through Ayk.A1L into Azz job
parameters. A0E is empty.
CRp reparses persisted toJid, injects managers and calls HxT.A05 to restore
pending membership for a valid user, ignoring its insertion result. A0G refuses
an invalid parsed user before constructing any request.

The job constructs IX5 with selector 1 and captures the job, AzY future, user
and original timestamp. In C8f's nonzero-selector success branch, it calls
0q2.A0V with that captured user/time and resolves the future. It does not parse
received token bytes. BmU wraps the error node/id in HFe and rejects the future;
BjL rejects with HFb for delivery failure. The selector-zero backup branch is
unrelated and must not be interpreted as token behavior.

A0G waits on AzY.get. Normal completion unmarks HxT pending membership through
A03; missing/expired input also unmarks. The caught execution exception is
logged and rethrown, without an A03 finally in this method. A0F cancellation
unmarks when the user was parsed. A0I declines retry only when the exception's
cause is HFe with an error node whose extracted code is 400 through 499,
inclusive; other cases return true. These are IQ error codes, not HTTP statuses.
Base job-queue terminal/cancellation dispatch and retry timing remain **unknown**;
this does not prove every nonretry exception invokes A0F or releases membership.

## Successful outbound timestamp persistence

**Confirmed:** 0q2.A0V normalizes the user and opens a SQL transaction. It first
updates sent_tc_token_timestamp and nulls real_issue_timestamp where the stored
sent timestamp is at most the captured timestamp. If no such row matches, it
updates a nonnull real_issue_timestamp at most that value; otherwise it attempts
an insert with JID and sent timestamp using conflict mode 4. Row-count assertions
require at most one match. This protects a newer sent timestamp from overwrite.
After a successful database outcome it evicts the canonical cache entry and
may delete the counterpart's outbound row. That subsequent counterpart deletion
uses another database handle and is not part of the earlier transaction.
It neither clears Rust receipt-send reservations nor rewrites incoming token
bytes. SQL conflict/exception handling is retained in the bundle.

## Port boundary and reproduction

**Confirmed local audit:** authenticated Rust replacements already archive old
pins and ratchets, invalidate local verification, request device refresh and
mark indexed sender-key exposure for rotation atomically. Incoming privacy-token
storage is separate. Outbound issue/send storage, canonical counterpart gates,
active expiry properties, pending-job recovery and correlated refresh sending
still need a port. This finding changes that port's required input: an incoming
token alone cannot supply its outbound issue state. Existing account state,
keys, request IDs and waits are untouched by this static research.

Reproduce with `xtask apk-research-bundle` using the linked recipe and a new
output directory, or use `--check` on the committed bundle. `apk-research-diff`
compares snapshots and maps the affected identity_conflicts.rs/notifications.rs
files. Comparing this bundle to itself tests the diff plumbing, not cross-release
protocol equivalence. No native retry scheduling is enabled in Rust by this work.

**Confirmed verification:** export and checksum revalidation reproduce 26 topics
and 31 files / 104,087 bytes. Self-comparison reports all 26 topics and four
metadata artifacts unchanged. All 63 xtask tests, the CLI-reference regeneration
test, API/library-symbol test and formatting pass. Hygiene validates 1,272
local links and 22 APK bundles totaling 6,593,496 bytes. Runtime changes in this
chunk are limited to a source comment; no new behavior or account operation was
introduced, and the previous receive window is not counted as token refresh
validation. Outbound refresh and a live identity transition remain unverified.

**Subsequent confirmed producer trace:** [outbound issue producers](token-issue.md)
locates 0q2.A0W scheduling from a normal server-acceptance worker and namespace
transfer. A scheduled outbound sent_tc_token_timestamp can exist before token
request success, including a newly initialized row with real_issue_timestamp=0.
This narrows the old-state requirement; it does not authorize deriving it from
incoming token bytes or enabling untraced bucket/canonicalization defaults.
