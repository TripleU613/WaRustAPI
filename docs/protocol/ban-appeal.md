# Ban appeal

When the server sends `failure reason=403`, an `appeal_token` is carried in the
response. This token can be used to submit an appeal for restoration of the
account.

## What is saved

The `appeal_token` is automatically extracted and persisted as the `noise_appeal_token`
credential the moment a `failure` node arrives during login. Future restarts can still
send it without making another login attempt.

## The request

```
POST https://acs.whatsapp.com/graphql
Authorization: Bearer <noise_login credential>
```

Sends JSON (one of two forms):

**Named operation** (preferred):
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
- Seven 15-digit candidate IDs exist in `classes9.dex` and can be tried with
  `--doc-id` if the named-operation path is rejected
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
- Top-level JSON keys (no values)
- Recognized error codes (if any)

No plaintext details are logged. The appeal has not been sent live yet.
