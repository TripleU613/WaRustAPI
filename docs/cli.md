# whatsapp-cli reference

Generated from the command definitions by the `cli_reference_is_current` test; regenerate with `UPDATE_CLI_DOCS=1 cargo test -p whatsapp-cli --features experimental-transport cli_reference`. Every command here also has an operation in docs/api/openapi.json (`x-cli`).

## Global options

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --store | STORE | no | `protocol.redb` | `WHATSAPP_API_STORE` |  | Path of the account protocol database |
| --account | ACCOUNT | no | `default` | `WHATSAPP_API_ACCOUNT` |  | Local account identifier |
| --key-env | KEY_ENV | no | `WHATSAPP_API_MASTER_KEY` |  |  | Name of the environment variable holding the hex master key |
| --profile | PROFILE | no |  | `WHATSAPP_API_PROFILE` |  | Named account profile (`<profiles-dir>/<NAME>.json`) supplying the store, account, key variable and config defaults; explicit flags win |
| --profiles-dir | PROFILES_DIR | no | `lab/profiles` | `WHATSAPP_API_PROFILES_DIR` |  | Directory holding account profiles |
| --json | JSON | no |  |  | `true`, `false` | Machine-readable JSON output where available |


## whatsapp-cli fingerprint-display

Compute native fingerprint displays from explicit private inputs (offline; no verification)

```
Usage: whatsapp-cli fingerprint-display [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --config | CONFIG | yes |  |  |  | Private JSON with normalized display identifiers and raw public key inventories |
| --qr-version | QR_VERSION | no |  |  |  | Export native V3/V4 comparison protobuf version 0 (keys) or 1 (digest) |
| --qr-output | QR_OUTPUT | no |  |  |  | Create a new private binary QR payload file; existing files are refused |
| --qr-scheme | QR_SCHEME | no |  |  | `native-v4`, `native-v3` | Explicit export algorithm (default native-v4); requires QR export |

## whatsapp-cli fingerprint-compare

Compare a private native V3/V4 binary payload against explicit private inputs (offline)

```
Usage: whatsapp-cli fingerprint-compare [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --config | CONFIG | yes |  |  |  | Private JSON with local/remote known inputs, same format as fingerprint-display |
| --scanned | SCANNED | yes |  |  |  | Private binary comparison protobuf; no image decoding or account lookup |
| --scheme | SCHEME | no | `native-v4` |  | `native-v4`, `native-v3` | Explicit native algorithm; current APK feature selection is not inferred |
| --v3-hosted | V3_HOSTED | no |  |  | `true`, `false` | Include V3 hosted-state checks (APK options 3 instead of 1) |

## whatsapp-cli outbox-send

Send one private-JSON text, image or voice note through the comparison outbox

```
Usage: whatsapp-cli outbox-send [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --config | CONFIG | no |  |  |  | Private request configuration; defaults to the profile's |
| --message | MESSAGE | yes |  |  |  |  |
| --allow-live-request | ALLOW_LIVE_REQUEST | no |  |  | `true`, `false` |  |

## whatsapp-cli media-list

List ordinary incoming attachments by opaque selector, with content redacted

```
Usage: whatsapp-cli media-list [OPTIONS]...
```

## whatsapp-cli media-open

Download/authenticate an ordinary image, voice note, WebP sticker or MP4 video/video note and open it locally

```
Usage: whatsapp-cli media-open [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --config | CONFIG | yes |  |  |  |  |
| --id | ID | yes |  |  |  |  |
| --allow-live-request | ALLOW_LIVE_REQUEST | no |  |  | `true`, `false` |  |

## whatsapp-cli media-save

Authenticate an ordinary attachment and save to a new private file

```
Usage: whatsapp-cli media-save [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --config | CONFIG | yes |  |  |  |  |
| --id | ID | yes |  |  |  |  |
| --output | OUTPUT | yes |  |  |  | Explicit destination; sender filename is never used as a path |
| --allow-live-request | ALLOW_LIVE_REQUEST | no |  |  | `true`, `false` |  |

## whatsapp-cli listen

Maintain an authenticated comparison connection and process incoming messages

```
Usage: whatsapp-cli listen [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --config | CONFIG | no |  |  |  | Private request configuration; defaults to the profile's |
| --allow-live-request | ALLOW_LIVE_REQUEST | no |  |  | `true`, `false` |  |
| --seconds | SECONDS | no | `0` |  |  | Stop after this many seconds; zero waits for Ctrl-C |
| --send-outbox | SEND_OUTBOX | no |  |  | `true`, `false` | Also transmit every eligible queued outbox intent, from any namespace, during this session. Inspect `outbox-pending` first |

## whatsapp-cli outbox-pending

List queued outbox intents that `listen --send-outbox` would transmit (offline)

```
Usage: whatsapp-cli outbox-pending [OPTIONS]...
```

## whatsapp-cli outbox-suspended

Inspect queued sends suspended by changed peer identities (offline, redacted)

```
Usage: whatsapp-cli outbox-suspended [OPTIONS]...
```

## whatsapp-cli outbox-group-preparations

Inspect saved normal group-send preparation without replaying queries (offline, redacted)

```
Usage: whatsapp-cli outbox-group-preparations [OPTIONS]...
```

## whatsapp-cli prekeys

Show one-time prekey counts and upload batches (offline, redacted)

```
Usage: whatsapp-cli prekeys [OPTIONS]...
```

## whatsapp-cli receipts

Show queued read, played and retry receipts and their outcomes (offline, redacted)

```
Usage: whatsapp-cli receipts [OPTIONS]...
```

## whatsapp-cli identity-conflicts

Inspect saved peer identity conflicts (offline; unauthenticated observations)

```
Usage: whatsapp-cli identity-conflicts [OPTIONS]...
```

## whatsapp-cli peer-identity

Inspect saved PN/LID pins, session agreement and retained-device key coverage (offline, redacted)

```
Usage: whatsapp-cli peer-identity [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --chat | CHAT | yes |  |  |  | Primary phone-number chat JID; never printed in the result |

## whatsapp-cli peer-verification

Record an explicit local human verification decision (offline; no scan or sync)

```
Usage: whatsapp-cli peer-verification [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --config | CONFIG | yes |  |  |  | Private JSON with peer, expected_fingerprint, expected_action_sequence, verified |

## whatsapp-cli peer-keys

Queue or inspect one peer directory key query (offline; a session transmits it)

```
Usage: whatsapp-cli peer-keys [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --chat | CHAT | yes |  |  |  | Primary phone-number chat JID, never printed in output |
| --show | SHOW | no |  |  | `true`, `false` | Inspect the latest saved response and pin matches without queueing |
| --refresh | REFRESH | no |  |  | `true`, `false` | Queue one fresh query only after a verified successful prior bundle |

## whatsapp-cli peer-identity-query

Queue or inspect one primary identity-only directory read (offline; no key adoption)

```
Usage: whatsapp-cli peer-identity-query [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --chat | CHAT | yes |  |  |  | Primary PN or LID chat JID, never printed in output |
| --show | SHOW | no |  |  | `true`, `false` | Inspect saved evidence without queueing or changing pins |

## whatsapp-cli abprops-query

Queue or inspect one initial AB-property read (offline; values are not applied)

```
Usage: whatsapp-cli abprops-query [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --show | SHOW | no |  |  | `true`, `false` | Inspect the saved response without queueing a read |

## whatsapp-cli actions

Show queued one-shot actions (block list, reports, lookups) and their outcomes (offline, redacted)

```
Usage: whatsapp-cli actions [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --show | SHOW | no |  |  |  | Print the request and reply stanzas of one action (numbers masked) |

## whatsapp-cli group

Group management over the durable action queue (offline; a session transmits)

```
Usage: whatsapp-cli group [OPTIONS]...
```

## whatsapp-cli group key-queries

Inspect saved frozen directory-key batch outcomes (offline, redacted)

```
Usage: whatsapp-cli group key-queries [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --request-id | REQUEST_ID | yes |  |  |  | Existing private message JSON request_id; never queries or resends |

## whatsapp-cli group receipts

Inspect per-device receipts for one saved comparison group send (offline, redacted)

```
Usage: whatsapp-cli group receipts [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --request-id | REQUEST_ID | yes |  |  |  | Existing private message JSON request_id; never creates or resends a message |

## whatsapp-cli group fanout-inspect

Inspect saved group fan-out counts offline; does not query or queue

```
Usage: whatsapp-cli group fanout-inspect [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --chat | CHAT | yes |  |  |  |  |
| --own | OWN | yes |  |  |  | Own primary identity in the group's PN/LID form |
| --hash | HASH | no |  |  | `true`, `false` | Explicitly reveal the private candidate participant hash |

## whatsapp-cli group create

Queue a group creation with the given subject and participants

```
Usage: whatsapp-cli group create [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --subject | SUBJECT | yes |  |  |  |  |
| --participant | PARTICIPANTS | yes |  |  |  | Participant JIDs (phone-number or LID form) |

## whatsapp-cli group info

Queue an interactive group info query

```
Usage: whatsapp-cli group info [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --chat | CHAT | yes |  |  |  |  |

## whatsapp-cli group refresh

Queue one classic batch refresh of the selected groups

```
Usage: whatsapp-cli group refresh [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --chat | CHATS | yes |  |  |  | Concrete group JIDs; repeat --chat for each group (local maximum 256) |
| --follow-up | FOLLOW_UP | no |  |  | `true`, `false` | Allow at most two further rounds after supported truncated results |

## whatsapp-cli group refresh-status

Inspect one saved batch projection outcome (offline, counts only)

```
Usage: whatsapp-cli group refresh-status [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --id | ID | yes |  |  |  | Existing batch action ID; does not queue another query |

## whatsapp-cli group subject

Queue a subject change

```
Usage: whatsapp-cli group subject [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --chat | CHAT | yes |  |  |  |  |
| --text | TEXT | yes |  |  |  |  |

## whatsapp-cli group description

Queue a description change (empty text deletes)

```
Usage: whatsapp-cli group description [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --chat | CHAT | yes |  |  |  |  |
| --text | TEXT | no | `` |  |  |  |
| --prev | PREV | no |  |  |  | The current description id when replacing one |

## whatsapp-cli group add

Queue adding participants

```
Usage: whatsapp-cli group add [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --chat | CHAT | yes |  |  |  |  |
| --participant | PARTICIPANTS | yes |  |  |  |  |

## whatsapp-cli group remove

Queue removing participants

```
Usage: whatsapp-cli group remove [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --chat | CHAT | yes |  |  |  |  |
| --participant | PARTICIPANTS | yes |  |  |  |  |

## whatsapp-cli group promote

Queue promoting participants to admin

```
Usage: whatsapp-cli group promote [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --chat | CHAT | yes |  |  |  |  |
| --participant | PARTICIPANTS | yes |  |  |  |  |

## whatsapp-cli group demote

Queue demoting admins

```
Usage: whatsapp-cli group demote [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --chat | CHAT | yes |  |  |  |  |
| --participant | PARTICIPANTS | yes |  |  |  |  |

## whatsapp-cli group leave

Queue leaving a group

```
Usage: whatsapp-cli group leave [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --chat | CHAT | yes |  |  |  |  |

## whatsapp-cli group invite

Queue fetching (or, with --revoke, resetting) the invite link

```
Usage: whatsapp-cli group invite [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --chat | CHAT | yes |  |  |  |  |
| --revoke | REVOKE | no |  |  | `true`, `false` |  |

## whatsapp-cli group list

Apply settled replies and list the stored group snapshots (numbers masked)

```
Usage: whatsapp-cli group list [OPTIONS]...
```

## whatsapp-cli accounts

Manage named account profiles (offline): list, add, show, new, status

```
Usage: whatsapp-cli accounts [OPTIONS]...
```

## whatsapp-cli accounts list

List the profiles in the profiles directory

```
Usage: whatsapp-cli accounts list [OPTIONS]...
```

## whatsapp-cli accounts add

Add a profile (refuses to overwrite)

```
Usage: whatsapp-cli accounts add [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --name | NAME | yes |  |  |  |  |
| --store | STORE | yes |  |  |  | Protocol database path for this account |
| --account | ACCOUNT | yes |  |  |  | Local account id inside that store |
| --config | CONFIG | no |  |  |  | Private request configuration path |
| --key-env | KEY_ENV | yes |  |  |  | Environment variable holding this account's master key |

## whatsapp-cli accounts show

Show one profile

```
Usage: whatsapp-cli accounts show [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --name | NAME | yes |  |  |  |  |

## whatsapp-cli accounts new

Create a new account with config, master key and profile

```
Usage: whatsapp-cli accounts new [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --name | NAME | yes |  |  |  | Account name |
| --country-code | COUNTRY_CODE | yes |  |  |  | Country calling code |
| --national-number | NATIONAL_NUMBER | yes |  |  |  | National phone number digits |
| --material | MATERIAL | yes |  | `WHATSAPP_API_TOKEN_MATERIAL` |  | Path to APK or material archive |
| --dir | DIR | no |  |  |  | Account directory (defaults to `lab/accounts/<NAME>`) |
| --method | METHOD | no | `sms` |  | `sms`, `voice` | Verification delivery method |

## whatsapp-cli accounts status

Show account status (registration, credentials, queues)

```
Usage: whatsapp-cli accounts status [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --name | NAME | no |  |  |  | Account name (all accounts if omitted) |

## whatsapp-cli token

Token derivation and verification

```
Usage: whatsapp-cli token [OPTIONS]...
```

## whatsapp-cli token derive

Derive the registration token from APK material

```
Usage: whatsapp-cli token derive [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --country-code | COUNTRY_CODE | yes |  |  |  | Country calling code |
| --national-number | NATIONAL_NUMBER | yes |  |  |  | National phone number digits |
| --material | MATERIAL | yes |  | `WHATSAPP_API_TOKEN_MATERIAL` |  | Path to APK or material archive |

## whatsapp-cli token verify

Verify a config token against APK material

```
Usage: whatsapp-cli token verify [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --config | CONFIG | yes |  |  |  | Path to the private JSON configuration |
| --material | MATERIAL | yes |  | `WHATSAPP_API_TOKEN_MATERIAL` |  | Path to APK or material archive |

## whatsapp-cli lab-pair

Two-account lab check: send a text from one profile and confirm the other profile receives it, running both sessions at once

```
Usage: whatsapp-cli lab-pair [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --from | FROM | yes |  |  |  | Sending profile name |
| --to | TO | yes |  |  |  | Receiving profile name |
| --text | TEXT | no | `pair check` |  |  | Text to send |
| --seconds | SECONDS | no | `45` |  |  | Session length for both accounts |
| --allow-live-request | ALLOW_LIVE_REQUEST | no |  |  | `true`, `false` |  |

## whatsapp-cli block

Durably queue a block of a contact (offline; `listen --send-outbox` transmits it)

```
Usage: whatsapp-cli block [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --chat | CHAT | yes |  |  |  | Contact JID, e.g. 15551234567@s.whatsapp.net |

## whatsapp-cli unblock

Durably queue an unblock of a contact (offline; `listen --send-outbox` transmits it)

```
Usage: whatsapp-cli unblock [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --chat | CHAT | yes |  |  |  | Contact JID, e.g. 15551234567@s.whatsapp.net |

## whatsapp-cli blocklist

Show the saved block list (offline, numbers masked), optionally queuing a fetch

```
Usage: whatsapp-cli blocklist [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --refresh | REFRESH | no |  |  | `true`, `false` | Also durably queue a blocklist fetch (`listen --send-outbox` transmits it) |
| --full | FULL | no |  |  | `true`, `false` | Forget the saved list hash first so the fetch returns the full list |

## whatsapp-cli report

Durably queue a spam report against a contact (offline; not live-verified)

```
Usage: whatsapp-cli report [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --chat | CHAT | yes |  |  |  | Contact JID, e.g. 15551234567@s.whatsapp.net |
| --flow | FLOW | no |  |  |  | Entry-point vocabulary string; defaults to "chat" |
| --known-chat | KNOWN_CHAT | no |  |  | `true`, `false` | Whether this was a known chat before reporting |

## whatsapp-cli react

Durably enqueue a reaction to a message; `listen --send-outbox` transmits it

```
Usage: whatsapp-cli react [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --chat | CHAT | yes |  |  |  | Chat JID, e.g. 15551234567@s.whatsapp.net |
| --id | ID | yes |  |  |  | Wire id of the message being reacted to (as shown by `events`) |
| --emoji | EMOJI | no |  |  |  | Reaction text (an emoji). Omit with --remove to clear a reaction |
| --remove | REMOVE | no |  |  | `true`, `false` | Remove this account's reaction instead of setting one |
| --ours | OURS | no |  |  | `true`, `false` | The target message was sent by this account (default: by the contact) |
| --request-id | REQUEST_ID | no |  |  |  | Caller idempotency key; defaults to `react-<id>-<emoji-hex or remove>` |
| --dry-run | DRY_RUN | no |  |  | `true`, `false` | Only report which identity the reaction key would use; enqueue nothing |

## whatsapp-cli poll

Durably enqueue a poll; `listen --send-outbox` transmits it

```
Usage: whatsapp-cli poll [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --chat | CHAT | yes |  |  |  | Chat JID |
| --name | NAME | yes |  |  |  | Poll question |
| --option | OPTIONS | yes |  |  |  | Options, two to twelve |
| --selectable | SELECTABLE | no | `0` |  |  | Options a voter may select; 0 (default) means any number |
| --request-id | REQUEST_ID | no |  |  |  | Caller idempotency key; defaults to a content digest |

## whatsapp-cli vote

Durably enqueue a vote on a poll; `listen --send-outbox` transmits it

```
Usage: whatsapp-cli vote [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --chat | CHAT | yes |  |  |  | Chat JID |
| --id | ID | yes |  |  |  | Wire id of the poll message |
| --option | OPTIONS | no |  |  |  | Selected option names; none retracts the vote |
| --ours | OURS | no |  |  | `true`, `false` | The poll was created by this account (default: by the contact) |
| --request-id | REQUEST_ID | no |  |  |  | Caller idempotency key; defaults to a content digest |

## whatsapp-cli disappearing

Durably enqueue a per-chat disappearing-message timer change (E2E protocolMessage); `listen --send-outbox` transmits it

```
Usage: whatsapp-cli disappearing [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --chat | CHAT | yes |  |  |  | Chat JID |
| --seconds | SECONDS | yes |  |  |  | New timer in seconds (0 = off; the UI offers 0, 86400, 604800, 7776000) |
| --request-id | REQUEST_ID | no |  |  |  | Caller idempotency key; defaults to a chat/timer digest (no phone number) |

## whatsapp-cli keep

Durably enqueue keeping, or un-keeping, a message despite the chat's disappearing timer; `listen --send-outbox` transmits it

```
Usage: whatsapp-cli keep [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --chat | CHAT | yes |  |  |  | Chat JID |
| --id | ID | yes |  |  |  | Wire id of the message to keep/un-keep |
| --ours | OURS | no |  |  | `true`, `false` | The target message was sent by this account (default: by the contact) |
| --undo | UNDO | no |  |  | `true`, `false` | Undo a previous keep instead of setting one |
| --request-id | REQUEST_ID | no |  |  |  | Caller idempotency key; defaults to a content digest |

## whatsapp-cli chat-privacy

Durably enqueue an advanced chat privacy ("limit sharing") change; `listen --send-outbox` transmits it

```
Usage: whatsapp-cli chat-privacy [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --chat | CHAT | yes |  |  |  | Chat JID |
| --on | ON | no |  |  | `true`, `false` | Turn limit sharing on |
| --off | OFF | no |  |  | `true`, `false` | Turn limit sharing off |
| --request-id | REQUEST_ID | no |  |  |  | Caller idempotency key; defaults to a chat/state digest (no phone number) |

## whatsapp-cli disappearing-default

Queue the account-wide default disappearing timer for new chats (`xmlns=disappearing_mode` IQ); a live session transmits it

```
Usage: whatsapp-cli disappearing-default [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --seconds | SECONDS | yes |  |  |  | New default timer in seconds |

## whatsapp-cli reply

Queue a quoted reply to a message in a direct chat

```
Usage: whatsapp-cli reply [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --chat | CHAT | yes |  |  |  | Chat JID |
| --id | ID | yes |  |  |  | Wire id of the message being replied to (as shown by `events`) |
| --text | TEXT | yes |  |  |  | Reply text |
| --ours | OURS | no |  |  | `true`, `false` | The quoted message was sent by this account (default: by the contact) |
| --request-id | REQUEST_ID | no |  |  |  | Caller idempotency key; defaults to a content digest |

## whatsapp-cli reproject

Re-project authenticated incoming records into events with current parsers (offline)

```
Usage: whatsapp-cli reproject [OPTIONS]...
```

## whatsapp-cli devices

Show a contact's saved device list, or queue a device sync for the next session

```
Usage: whatsapp-cli devices [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --chat | CHAT | yes |  |  |  | Contact JID, e.g. 15551234567@s.whatsapp.net |
| --request | REQUEST | no |  |  | `true`, `false` | Record the contact for the next `usync devices` query (offline) |

## whatsapp-cli device-query

Inspect one saved device query offline; counts only, never reserves or resets

```
Usage: whatsapp-cli device-query [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --id | ID | yes |  |  |  | Existing query ID from the send-preparation log |

## whatsapp-cli contacts

Manage the local address book and its WhatsApp-existence checks (offline)

```
Usage: whatsapp-cli contacts [OPTIONS]...
```

## whatsapp-cli contacts list

List saved contacts

```
Usage: whatsapp-cli contacts list [OPTIONS]...
```

## whatsapp-cli contacts add

Add a new contact

```
Usage: whatsapp-cli contacts add [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --phone | PHONE | yes |  |  |  | Phone number, `+E.164` or bare digits, 6-15 digits |
| --name | NAME | yes |  |  |  | Display name |
| --first-name | FIRST_NAME | no |  |  |  | Optional given name |

## whatsapp-cli contacts edit

Edit an existing contact's name

```
Usage: whatsapp-cli contacts edit [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --phone | PHONE | yes |  |  |  |  |
| --name | NAME | yes |  |  |  |  |
| --first-name | FIRST_NAME | no |  |  |  |  |

## whatsapp-cli contacts remove

Remove a saved contact

```
Usage: whatsapp-cli contacts remove [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --phone | PHONE | yes |  |  |  |  |

## whatsapp-cli contacts repair-lid

Repair a missing LID from one uniquely attributable saved contact reply (offline)

```
Usage: whatsapp-cli contacts repair-lid [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --id | ID | yes |  |  |  |  |

## whatsapp-cli contacts status

Inspect a saved contact-check status offline without queuing another query

```
Usage: whatsapp-cli contacts status [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --phone | PHONE | yes |  |  |  |  |

## whatsapp-cli contacts check

Queue a `usync` WhatsApp-existence check for a number and show any already-saved status; a session transmits the queued action

```
Usage: whatsapp-cli contacts check [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --phone | PHONE | yes |  |  |  |  |

## whatsapp-cli contacts sync

Queue a full `usync` sync of every saved contact, registering this account's address book with the server (offline; a session transmits it). Unlike `check`, which is a one-number existence query, this uses the APK's `full`/`interactive` mode pair

```
Usage: whatsapp-cli contacts sync [OPTIONS]...
```

## whatsapp-cli contacts fetch-app-state

Queue a read-only app-state fetch for every collection at version 0 (offline; a session transmits it). Carries no patch and mutates nothing; the reply gives the server's authoritative collection versions

```
Usage: whatsapp-cli contacts fetch-app-state [OPTIONS]...
```

## whatsapp-cli contacts publish-app-state

Queue an app-state `contactAction` patch publishing every saved contact, so companion devices receive this address book (offline; a session transmits it). Mints this device's app-state key if it has none

```
Usage: whatsapp-cli contacts publish-app-state [OPTIONS]...
```

## whatsapp-cli clear-chat

Clear a direct chat's locally decrypted message bodies (offline, local-only: see docs/protocol/chat-actions.md section 6 for the deferred companion sync)

```
Usage: whatsapp-cli clear-chat [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --chat | CHAT | yes |  |  |  | Chat JID, e.g. 15551234567@s.whatsapp.net |

## whatsapp-cli mark-read

Queue explicit read receipts for authenticated incoming messages (offline; the next `listen` session sends them once)

```
Usage: whatsapp-cli mark-read [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --config | CONFIG | no |  |  |  | Private request configuration; defaults to the profile's |
| --chat | CHAT | yes |  |  |  | The chat: the sender's JID for a direct message (e.g. 15551234567@s.whatsapp.net), or the group JID (`…@g.us`) for a group message, whose receipt names the sender itself |
| --id | IDS | yes |  |  |  | One or more incoming message ids as shown by `events` |

## whatsapp-cli lab-open-view-once

Display one downloaded view-once picture locally for 30 seconds, consuming it

```
Usage: whatsapp-cli lab-open-view-once [OPTIONS]...
```

## whatsapp-cli lab-download-view-once

Download one authenticated view-once image as private ciphertext; do not open it

```
Usage: whatsapp-cli lab-download-view-once [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --config | CONFIG | no |  |  |  | Private request configuration; defaults to the profile's |
| --allow-live-request | ALLOW_LIVE_REQUEST | no |  |  | `true`, `false` |  |

## whatsapp-cli lab-inbox

Decrypt captured direct replies locally, without network I/O

```
Usage: whatsapp-cli lab-inbox [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --inspect | INSPECT | no |  |  | `true`, `false` | Inspect saved plaintext protobuf structure without advancing sessions. Validates supported history-notice metadata and reports redacted counts; history remains unapplied |
| --replay-captures | REPLAY_CAPTURES | no |  |  | `true`, `false` | Re-run preserved incoming message captures through the normal receive path, offline. The server delivers a queued stanza once, so a stanza refused by a since-fixed guard is only recoverable from its capture. Already-committed messages dedupe; nothing is sent and no receipt goes out |
| --accept-authenticated-identity-changes | ACCEPT_AUTHENTICATED_IDENTITY_CHANGES | no |  |  | `true`, `false` | Permit authenticated remote prekey identity changes offline, preserving prior pins/sessions and emitting IdentityChanged. Not human verification |

## whatsapp-cli lab-transport

One comparison-server connection; optional bounded primary login

```
Usage: whatsapp-cli lab-transport [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --config | CONFIG | no |  |  |  | Existing private registration JSON, used to bind the saved account. Private request configuration; defaults to the profile's |
| --allow-live-request | ALLOW_LIVE_REQUEST | no |  |  | `true`, `false` | Authorize a single connection; there are no automatic retries |
| --login | LOGIN | no |  |  | `true`, `false` | Send one primary login after certificate verification |
| --recheck-refusal | RECHECK_REFUSAL | no |  |  | `true`, `false` | Authorize exactly one further login after a saved `failure`, to tell a temporary limit from a permanent refusal. Refuses an ambiguous saved outcome, refuses to carry a message, and preserves the earlier refusal rather than overwriting it |
| --message | MESSAGE | no |  |  |  | Private JSON: request_id plus exactly one target — `to` (direct), `group` (`…@g.us` sender-key fan-out) or `status` (a status post with its own audience, font and colours) |
| --receive | RECEIVE | no |  |  | `true`, `false` | Preserve incoming stanzas for up to 30 seconds without sending a chat |
| --media-route | MEDIA_ROUTE | no |  |  | `true`, `false` | Obtain and privately preserve one current media-upload route response |
| --push-name | PUSH_NAME | no |  |  |  | Set the display name others see beside this number: one `<presence name=… type="available"/>` after login. Fire-and-forget; the server does not acknowledge it |
| --typing | TYPING | no |  |  |  | Show a typing indicator in this chat (E.164 digits, a phone/LID JID or a `…@g.us` group), hold it, then send `paused` |
| --subscribe | SUBSCRIBE | no |  |  |  | During `--receive`, subscribe to this person's availability and typing (repeatable, at most 16), as opening their chat does. Each carries their trusted-contact token when one is saved |
| --typing-state | TYPING_STATE | no | `composing` |  |  | `composing` (typing), `recording` (voice note) or `paused` |
| --typing-seconds | TYPING_SECONDS | no | `5` |  |  | How long to hold `composing`/`recording` before `paused`, 1–25 s |

## whatsapp-cli ban-appeal

Submit a ban appeal for the active account via ACS GraphQL

```
Usage: whatsapp-cli ban-appeal [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --config | CONFIG | no |  |  |  | Private request configuration; defaults to the profile's |
| --allow-live-request | ALLOW_LIVE_REQUEST | no |  |  | `true`, `false` | Authorize this single invocation to contact acs.whatsapp.com |
| --doc-id | DOC_ID | no |  |  |  | Persistent query doc_id for SubmitAnAppealForIP (APK-derived candidate) |
| --message | MESSAGE | no |  |  |  | Optional appeal reason text included in the request |
| --appeal-token | APPEAL_TOKEN | no |  |  |  | Use this appeal token instead of the one saved from the failure node |
| --recover-token | RECOVER_TOKEN | no |  |  | `true`, `false` | Offline: recover the appeal token from preserved failure captures and save it, without contacting anything or submitting an appeal |

## whatsapp-cli lab-signup

Explicit comparison-build signup experiment; may affect an existing account

```
Usage: whatsapp-cli lab-signup [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --config | CONFIG | no |  |  |  | Private JSON LabRegistrationConfig file; keep under ignored lab/. Defaults to the profile's config, else lab/requests.json |
| --allow-live-request | ALLOW_LIVE_REQUEST | no |  |  | `true`, `false` | Authorize this single invocation to contact WhatsApp |

## whatsapp-cli lab-signup template

Print the editable profile template; no file, store or network access

```
Usage: whatsapp-cli lab-signup template [OPTIONS]...
```

## whatsapp-cli lab-signup preview

Validate and show a redacted request plan; no network or store access

```
Usage: whatsapp-cli lab-signup preview [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --show-phone | SHOW_PHONE | no |  |  | `true`, `false` | Explicitly show the configured phone in local output |

## whatsapp-cli lab-signup status

Local status only: sends no request

```
Usage: whatsapp-cli lab-signup status [OPTIONS]...
```

## whatsapp-cli lab-signup retry-corrected-token

One retry after bad_token; JSON may change only the token, never identity

```
Usage: whatsapp-cli lab-signup retry-corrected-token [OPTIONS]...
```

## whatsapp-cli lab-signup retry-corrected-request

Migrate an opaque legacy refusal using JSON corrections; one send only. Preserves identity; shares the legacy diagnostic's durable reservation

```
Usage: whatsapp-cli lab-signup retry-corrected-request [OPTIONS]...
```

## whatsapp-cli lab-signup resume-preflight

Recheck a saved preflight stopped by the old missing-recommendation guard

```
Usage: whatsapp-cli lab-signup resume-preflight [OPTIONS]...
```

## whatsapp-cli lab-signup inspect-consent

Inspect consent selectors once using saved identity; no OTP or SMS

```
Usage: whatsapp-cli lab-signup inspect-consent [OPTIONS]...
```

## whatsapp-cli lab-signup submit-consent

Submit one reviewed decision from private JSON matching saved selectors

```
Usage: whatsapp-cli lab-signup submit-consent [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --decision | DECISION | yes |  |  |  |  |

## whatsapp-cli lab-signup diagnose-legacy-refusal

One authorized resend of a legacy refusal whose diagnostics were lost. Known refusals/cooldowns are excluded; a durable latch prevents repeats

```
Usage: whatsapp-cli lab-signup diagnose-legacy-refusal [OPTIONS]...
```

## whatsapp-cli lab-signup diagnose-bad-parameter

One authorized repeat of HTTP 200 / bad_param to collect field hints. Same saved payload and keys; a durable latch prevents another repeat

```
Usage: whatsapp-cli lab-signup diagnose-bad-parameter [OPTIONS]...
```

## whatsapp-cli lab-signup diagnose-encrypted-parameter

One encrypted-envelope experiment after HTTP 200 / bad_param. Same inner fields and identity; no attestation, no automatic repeats

```
Usage: whatsapp-cli lab-signup diagnose-encrypted-parameter [OPTIONS]...
```

## whatsapp-cli lab-signup diagnose-unsigned-envelope

One ENC plus empty-H experiment; does not fabricate attestation

```
Usage: whatsapp-cli lab-signup diagnose-unsigned-envelope [OPTIONS]...
```

## whatsapp-cli lab-signup diagnose-additional-fields

One additive-code-field experiment from a private EndpointFields JSON file. Uses ENC plus empty H; rejects replacement of saved fields

```
Usage: whatsapp-cli lab-signup diagnose-additional-fields [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --fields | FIELDS | yes |  |  |  |  |

## whatsapp-cli lab-signup diagnose-flow-fields

One explicit post-failure reason/metrics experiment, using ENC plus empty H

```
Usage: whatsapp-cli lab-signup diagnose-flow-fields [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --fields | FIELDS | yes |  |  |  |  |

## whatsapp-cli lab-signup diagnose-unknown-agent

One APK unknown-build-properties User-Agent experiment with additive fields

```
Usage: whatsapp-cli lab-signup diagnose-unknown-agent [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --fields | FIELDS | yes |  |  |  |  |

## whatsapp-cli lab-signup begin

Request a code or resume an existing attempt; no automatic retries

```
Usage: whatsapp-cli lab-signup begin [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --method | METHOD | no |  |  | `sms`, `voice` |  |

## whatsapp-cli lab-signup resend-code

Ask for a *new* code on the attempt already in progress, because a delivered code expires long before the attempt does. Keeps the attempt, its identity, its keys and its history; honours the server's `retry_after`; refuses any state but a pending challenge

```
Usage: whatsapp-cli lab-signup resend-code [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --method | METHOD | no |  |  | `sms`, `voice` |  |

## whatsapp-cli lab-signup verify-code

Submit one OTP from redirected stdin (never a command-line argument)

```
Usage: whatsapp-cli lab-signup verify-code [OPTIONS]...
```

## whatsapp-cli lab-signup verify-pin

Submit one account PIN from redirected stdin; never resets the account PIN

```
Usage: whatsapp-cli lab-signup verify-pin [OPTIONS]...
```

## whatsapp-cli lab-signup resume-email-challenge

Local only: move a `challenge` refusal of /v2/register to the email second-factor step (2.26.37.71 launchEmailChallengeFlow). No request

```
Usage: whatsapp-cli lab-signup resume-email-challenge [OPTIONS]...
```

## whatsapp-cli lab-signup request-email-code

Ask the server to email the one-time code to the address on file (`/v2/code` with `method=email_otp`, 2.26.37.71 `LX/0GB.A01`)

```
Usage: whatsapp-cli lab-signup request-email-code [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --email | EMAIL | no |  |  |  | Not sent by the comparison client; rejected when given |

## whatsapp-cli lab-signup exist

One read-safe `/v2/exist` probe of method eligibility for the saved attempt (email one-time code, old-device confirmation, waits). Prints the server's answer and changes no state

```
Usage: whatsapp-cli lab-signup exist [OPTIONS]...
```

## whatsapp-cli lab-signup verify-email-code

Submit the emailed one-time code from redirected stdin (`/v2/register`)

```
Usage: whatsapp-cli lab-signup verify-email-code [OPTIONS]...
```

## whatsapp-cli keygen

Print a freshly generated master key (hex) to stdout. Store it in a secret manager and export it through the key environment variable

```
Usage: whatsapp-cli keygen [OPTIONS]...
```

## whatsapp-cli profile

Inspect the protocol profile and its evidence labels

```
Usage: whatsapp-cli profile [OPTIONS]...
```

## whatsapp-cli capabilities

Print the capability matrix

```
Usage: whatsapp-cli capabilities [OPTIONS]...
```

## whatsapp-cli serve

Serve the local HTTP control plane

```
Usage: whatsapp-cli serve [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --bind | BIND | no | `127.0.0.1:8787` | `WHATSAPP_API_BIND` |  | Address to listen on. Non-loopback addresses require an explicit opt-in because the control plane does not implement authentication |
| --allow-non-loopback | ALLOW_NON_LOOPBACK | no |  |  | `true`, `false` | Permit listening on a non-loopback address |

## whatsapp-cli store

Store operations

```
Usage: whatsapp-cli store [OPTIONS]...
```

## whatsapp-cli store init

Create the store (idempotent) and verify the master key opens it

```
Usage: whatsapp-cli store init [OPTIONS]...
```

## whatsapp-cli store inspect

Print table counts, journal head, generation and registration status

```
Usage: whatsapp-cli store inspect [OPTIONS]...
```

## whatsapp-cli store rotate-key

Rotate the master key: re-wrap the data keys under the key found in the given environment variable. The old key stops working immediately

```
Usage: whatsapp-cli store rotate-key [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --new-key-env | NEW_KEY_ENV | yes |  |  |  | Environment variable holding the new hex master key |

## whatsapp-cli registration

Primary registration steps

```
Usage: whatsapp-cli registration [OPTIONS]...
```

## whatsapp-cli registration begin

Start or resume a registration attempt

```
Usage: whatsapp-cli registration begin [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --country | COUNTRY | yes |  |  |  | Country calling code (explicit; never guessed) |
| --phone | PHONE | yes |  |  |  | Phone number in E.164 form (`+<cc><national>`) |
| --method | METHOD | no | `sms` |  | `sms`, `voice` | Challenge method |

## whatsapp-cli registration status

Show the current attempt

```
Usage: whatsapp-cli registration status [OPTIONS]...
```

## whatsapp-cli registration reconcile

Reconcile an attempt whose request outcome was lost

```
Usage: whatsapp-cli registration reconcile [OPTIONS]...
```

## whatsapp-cli registration reset

Discard a completed or blocked attempt (local only)

```
Usage: whatsapp-cli registration reset [OPTIONS]...
```

## whatsapp-cli connect

Attempt an authenticated connection (fails with a typed reason)

```
Usage: whatsapp-cli connect [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --timeout | TIMEOUT | no | `30` |  |  | Timeout in seconds |

## whatsapp-cli send

Durably enqueue a direct text message and print its ticket

```
Usage: whatsapp-cli send [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --to | TO | yes |  |  |  | Destination JID, e.g. 15551234567@s.whatsapp.net |
| --text | TEXT | yes |  |  |  | Message text |
| --request-id | REQUEST_ID | yes |  |  |  | Caller idempotency key |

## whatsapp-cli status

Show the status of a queued message

```
Usage: whatsapp-cli status [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --local-id | LOCAL_ID | yes |  |  |  | Local message id (32 hex chars) from the ticket |

## whatsapp-cli cancel

Cancel a queued message before transmission (offline; never a revocation)

```
Usage: whatsapp-cli cancel [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --local-id | LOCAL_ID | yes |  |  |  | Local message id (32 hex chars) from the ticket |

## whatsapp-cli two-factor

Set, change or turn off two-step verification (offline; a session transmits it). The PIN is read from redirected stdin, never an argument

```
Usage: whatsapp-cli two-factor [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --set-pin | SET_PIN | no |  |  | `true`, `false` | Read a six-digit PIN from stdin and set it |
| --disable | DISABLE | no |  |  | `true`, `false` | Turn two-step verification off. Sends an empty code |
| --email | EMAIL | no |  |  |  | Recovery email address to store alongside the PIN |

## whatsapp-cli push-name

Queue the display name as the `pushNameSetting` app-state record, so this account's other devices show it (offline; a session such as `listen` transmits it). Other people see the name through `lab-transport --push-name`, which is a separate stanza

```
Usage: whatsapp-cli push-name [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
|  | NAME | yes |  |  |  | The display name |

## whatsapp-cli about

Queue the profile "About" text (offline; a session such as `listen` transmits it). An empty text clears it, sent as a single space the way the official app does

```
Usage: whatsapp-cli about [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
|  | TEXT | yes |  |  |  | The About text; `""` clears it |

## whatsapp-cli status-viewers

List who has received, viewed, played or liked the statuses we posted (offline)

```
Usage: whatsapp-cli status-viewers [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --status | STATUS | no |  |  |  | Only this status, by its wire id |

## whatsapp-cli events

Read durable events for a consumer

```
Usage: whatsapp-cli events [OPTIONS]...
```

| Flag | Value | Required | Default | Env | Values | Help |
|------|-------|----------|---------|-----|--------|------|
| --summary | SUMMARY | no |  |  | `true`, `false` | Print event kinds only, without message content or identifiers |
| --consumer | CONSUMER | no | `cli` |  |  | Consumer name |
| --limit | LIMIT | no | `50` |  |  | Maximum events |
| --ack | ACK | no |  |  | `true`, `false` | Acknowledge the batch after printing it |

