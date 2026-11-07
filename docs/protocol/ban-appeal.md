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
