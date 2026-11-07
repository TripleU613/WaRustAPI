# Ban appeal

When the server sends `failure reason=403`, an `appeal_token` is carried in the
response. This token can be used to submit an appeal for restoration of the
account.

## What is saved

The `appeal_token` is automatically extracted and persisted as the `noise_appeal_token`
credential the moment a `failure` node arrives during login. Future restarts can still
send it without making another login attempt.

When no credential exists, `ban-appeal` recovers the token from the preserved `failure`
capture (encoded in the Checkpoints table as `inbound:sha256:<hash>`) and persists it
with `source_step: failure-capture`. The `--appeal-token` flag remains available as a
manual override for any token value.

## The request

```
POST https://acs.whatsapp.com/graphql
Authorization: Bearer <saved `login` credential>
```

Sends JSON (one of two forms):

**Named operation** (default when `--doc-id` is not given; server acceptance unverified):
```json
{
  "operationName": "SubmitAnAppealForIP",
  "variables": {
    "input": {
      "appeal_token": "<saved_appeal_token>",
      "unban_reason": "<optional_reason_text>"
    }
  }
}
```

**Persistent-query doc_id** (alternative, if named operation is rejected):
```json
{
  "doc_id": "<15_digit_candidate_id>",
  "variables": {
    "input": {
      "appeal_token": "<saved_appeal_token>",
      "unban_reason": "<optional_reason_text>"
    }
  }
}
```

## Unknown details

- The persistent-query `doc_id` for `SubmitAnAppealForIP` was not recovered from
  static APK analysis
- Seven 15-digit literals exist in `classes9.dex`; their mapping to this operation
  is unverified, so none is established here as a valid submission doc_id
- Appeal outcome handling and follow-up flows are not documented

## How to run

The saved `noise_appeal_token` credential is read automatically:

```sh
whatsapp-cli --profile <name> ban-appeal --allow-live-request
```

Optional flags:
- `--message "<reason_text>"`: include an appeal reason
- `--doc-id <15_digits>`: use persistent-query ID instead of named operation
- `--appeal-token <token>`: override the saved token (not normally needed)

## How to read the summary

The response is summarized redacted:
- HTTP status code
- Response body size in bytes
- Fixed recognized structural field names and total field count
- GraphQL error-array entry count (when present)

Free-form values, server-selected key names and error code/type/message values
are omitted. A structural `errors` field/count does not prove the appeal outcome.

No plaintext details are logged. The appeal has not been sent live yet.

## Response-summary redaction correction

**Confirmed Rust source defect, corrected offline**, baseline `bd787981`.
The summary was documented as redacted, but copied top-level error_message/error/
status values and error-array code/type strings, and exposed arbitrary object key
names. A response could echo a credential/identifier into these outputs, including
BanAppealResult Debug. Local synthetic sentinel responses reproduced that path.

The summary now retains only HTTP status, body size, fixed recognized structural
field names, total field count and error-array entry count. Response-selected
text and field values are never copied into it. Invalid JSON/non-object roots
retain only local status/size. API schema and CLI documentation describe the
changed diagnostic shape; former free-form errors-array details are absent.

The offline regression places sentinels in top-level values, nested errors,
arbitrary key names, numeric error codes and invalid/non-object bodies; neither
summary JSON nor result Debug contains them. No request-construction, appeal-token
selection, retry or identity behavior changes. No appeal or account operation is
performed. Operation/doc_id validity and server outcome handling remain unknown.

## Authorization header byte preservation

**Confirmed Rust construction defect, corrected offline**, baseline `a93bb93f`.
The saved login credential was decoded with `String::from_utf8_lossy` before
forming Authorization. Invalid UTF-8 bytes therefore became replacement-character
bytes instead of retaining the stored credential. The header was also not marked
sensitive for HTTP Debug formatting.

Construction now prefixes the exact saved bytes with `Bearer `, validates them
with the HTTP HeaderValue parser, and marks the resulting header sensitive.
Invalid header bytes refuse before send with a fixed redacted error. Offline
controls verify ASCII and non-UTF-8 byte preservation, sensitive Debug formatting,
and refusal of CRLF/NUL inputs. Non-UTF-8 HTTP-header acceptance is only a local
parser fact, not proof that ACS accepts such credentials. No new token syntax or
server policy is inferred. Optional-auth selection, operation/doc_id, retries and
account binding are unchanged and remain subject to the existing unknowns. No
appeal or live account operation was performed.

## Transport bounds at library entry

**Confirmed Rust settings omission, corrected offline**, baseline `803d40c3`.
Direct `submit_ban_appeal` callers previously bypassed TransportSettings validation
and response streaming used only a fixed 64 KiB ceiling, ignoring a smaller
configured max_response_bytes. CLI profile validation did not protect direct
library calls. Submission now validates transport settings before credential
lookup/client construction and applies min(configured response budget, 64 KiB)
to every streamed chunk before appending. Over-budget errors report that effective
limit; accumulated bytes are unchanged by the rejected chunk. Existing timeout
values are passed unchanged to reqwest after validation.

Offline regression covers zero timeout, connect timeout beyond whole-request
timeout, zero response budget, the default ceiling, a smaller three-byte budget,
exact-boundary acceptance across chunks and one-byte overflow refusal. No socket
or account operation is used. These are local configuration/resource invariants,
not APK/server timeout requirements. The fixed appeal hard ceiling, request
construction, retry policy and server acceptance unknowns otherwise remain.

## Active-session failure token account ownership

**Confirmed Rust credential ownership defect, corrected offline**, baseline
`e9672f9e`. The normal connection receive loop captured terminal failure nodes,
then copied appeal_token into whichever AccountRecord existed at that moment,
creating a default account when missing and discarding persistence errors.
The fake-link regression reproduced token attachment after a changed own JID.
This is local credential misattribution, not a proven server restriction cause.

The credential update now compares the initiating receive session's existing
account-authority fingerprint with the current saved account in the same write
transaction as token persistence. Changed authority refuses with a fixed redacted
error; a missing account is never created. Persistence faults propagate instead
of being discarded. The authenticated failure capture precedes this optional
credential update and remains preserved. All outcomes still end the connection
without recoverable reconnect, ACK or an appeal submission. Existing first-token/
64-credential policy and observation-timestamp tolerance are unchanged.

The actual receive-loop fake-link regression activates the session, changes/removes
the account, delivers a synthetic failure with an appeal token and awaits worker
termination. Same-account/observation-only controls save the credential; changed
JID/missing-account cases leave the Account table byte-identical. Diagnostics omit
the token, no outbound stanza is produced, and captured refusal plus account state
survive encrypted-store reopen. No real socket or server is used.

This fence covers the active receive-loop failure path only. Historical capture
recovery, old stored-token provenance, initial-login adapters and appeal submission
binding are separate audit boundaries; they are not retroactively authenticated
by this change. No server challenge/refusal is bypassed and no live account
operation or retry is added.

## Historical recovery account ownership

2026-11-07. **Confirmed local persistence defect/fixed offline**, baseline
`4c8624b6`: recovery decoded a preserved failure token, then persisted it into
whatever AccountRecord existed in a later write transaction, using a default
record if absent. The failing-before regression reproduced creation of an
account from a capture alone. This is local misattribution, not a confirmed
server restriction cause or a newly established native recovery policy.

Candidate selection now loads the existing account and computes the shared
account-authority fingerprint in the same read snapshot as capture decoding.
Persistence requires an existing account with that fingerprint inside its write
transaction; changed authority returns a fixed redacted InvalidInput(ban_appeal),
missing account returns NotRegistered, and no recovered token is returned on
those errors. A captured token alone cannot create an account. Observation-only
timestamp changes remain compatible with the existing shared authority policy.
Empty stores with no eligible capture still return None without creating state.
First-token/64-credential behavior, capture decoding/selection, return encoding,
scan bound and credential names remain unchanged. No request or appeal is sent.

Tests exercise actual public recovery with a capture and missing account,
checking unchanged capture/account/journal and encrypted reopen. A second test
uses the actual candidate-selection and persistence seams, stages an account JID
change, registration-id change or deletion between them, and verifies no mutation
or token disclosure. Same-account and observation-only controls persist the
credential; captures and resulting account state survive reopen. The existing
positive recovery fixture now explicitly creates its synthetic account.

**Limits:** the fingerprint binds account continuity during this recovery
operation; it does not prove which historical account produced an unowned
capture. Historical token provenance, the fixed20,000-row discovery bound,
existing-token precedence, and appeal-submission binding remain separate.
Likewise initial-login integer parsing is not automatically a policy for
post-login terminal failures: the current normal receive path preserves and
terminates those without interpreting a reason code. No new malformed-terminal
reason gate, reconnect, replay, identity repair or native parity claim is added.

Validation: full workspace/all-feature suites passed, including1,416 library
tests,91 CLI tests and integration/xtask suites. Strict all-target/all-feature
Clippy, CLI-reference regeneration, OpenAPI library-symbol test, APK coverage,
formatting, whitespace and staged repository hygiene passed. No fresh APK trace
or live account operation was performed for this local ownership correction.

## Exhaustive paged token discovery

2026-11-07. **Confirmed local discovery defect/fixed offline**, baseline
`ac4ea629`: historical recovery read only the first20,000 physical checkpoint
rows, including unrelated records. A failure capture later in lexicographic
order therefore returned None even though its token remained preserved. The
failing-before regression reproduced this after encrypted-store reopen with
20,005 unrelated rows preceding eligible malformed-wire/JSON decoys and the
actual captured failure. This is a local lookup defect, not a native/server
requirement or evidence that a refusal should be retried.

Discovery now uses scan_after with256-row pages to exhaustion in the same read
snapshot. Its exclusive cursor advances by the final physical key of each page,
so unrelated or malformed records cannot stop traversal. Existing candidate
filters, first matching lexicographic selection, decoding, account authority
capture and checked persistence remain. Memory retained for each checkpoint
page is bounded; total scan work scales with stored history. No new capture
provenance or most-recent-token selection policy is inferred.

The regression invokes public recovery after reopen, checks the expected token
and byte-identical complete synthetic checkpoint history, then reopens again
and verifies resulting credential/account plus captures. Existing empty-store,
missing-account and changed-account controls remain. The previously documented
20,000-row discovery limitation is closed; historical account attribution, old
credential precedence and submission binding remain separate unknowns. No
account request, replay, identity reset or appeal submission is performed.

Validation: full workspace/all-feature tests passed, including 1,417 library
tests and the integration/CLI/xtask suites. Strict Clippy, CLI-reference
regeneration, OpenAPI library-symbol test, APK coverage, formatting, whitespace
and staged hygiene passed. No new APK claim or live operation.

## Stored appeal-token text is exact or refused

2026-11-07. **Confirmed local request-input defect/fixed offline**, baseline
`f40a7080`: read_stored_appeal_token decoded saved credential bytes through
String::from_utf8_lossy. Invalid UTF-8 could therefore become a different token
containing replacement characters before BanAppealInput JSON construction. The
failing-before public async regression returned a token for invalid byte input.
This is byte-to-text correctness, not newly traced native token syntax.

The read path now uses strict UTF-8 conversion through a shared local helper.
Invalid bytes return fixed redacted InvalidInput(ban_appeal) and remain stored
unchanged; they do not become None or trigger capture fallback. Valid ASCII and
Unicode text retain their exact UTF-8 bytes. Recovery uses the same helper,
although tokens selected through decoded node text are already UTF-8. No ASCII
restriction, token rotation, credential deletion or speculative format check is
added. Bearer login header byte handling is separate and remains unchanged.

The public read regression exercises ASCII, Unicode and invalid UTF-8 credentials,
checking unchanged account bytes, empty capture table, event head and encrypted
reopen. It invokes no HTTPS client or submission. Existing captured-token
recovery/account-ownership/pagination tests remain controls. Valid text alone
does not prove token authenticity, account provenance, service acceptance or the
complete native appeal request contract. No live account operation performed.

Validation: full workspace/all-feature tests passed (1,418 library tests plus
integration/CLI/xtask suites). Strict Clippy, CLI-reference regeneration,
OpenAPI library-symbol test, APK coverage, formatting, whitespace and staged
hygiene passed. No new native syntax or live acceptance evidence is claimed.
