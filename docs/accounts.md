# Running more than one account

Every account has its own encrypted store, its own private request
configuration and its own master key. Nothing is shared between accounts, so
two of them can run in separate processes, or together in `lab-pair`, without
touching each other's sessions.

## Profiles

A profile is a small JSON file `lab/profiles/<name>.json` (the directory is
ignored by Git; override it with `--profiles-dir` or `WHATSAPP_API_PROFILES_DIR`):

```json
{
  "store": "lab/<account>/signup.redb",
  "account": "signup-test",
  "config": "lab/<account>/request.json",
  "key_env": "WHATSAPP_API_MASTER_KEY_A"
}
```

It holds no secrets: the master key stays in the named environment variable,
one variable per account. Manage profiles with:

```
whatsapp-cli accounts add --name a --store … --account … --config … --key-env WHATSAPP_API_MASTER_KEY_A
whatsapp-cli accounts list
whatsapp-cli accounts show --name a
```

`--profile <name>` (or `WHATSAPP_API_PROFILE`) then fills `--store`,
`--account`, `--key-env` and every command's `--config` from the profile.
Explicit flags still win, so `--profile a --store other.redb` uses the other
store. Commands that need a config and get neither `--config` nor a profile
fail before touching anything.

## Onboarding a second number

Use `accounts new` to prepare a new account with automatic token derivation from
the 2.26.37.70 material:

```sh
export WHATSAPP_API_MASTER_KEY_B=$(cat lab/accounts/b/master.key)

whatsapp-cli accounts new --name b --country-code 44 --national-number 7700900123 \
  --material lab/whatsapp-2.26.37.70-token-material.zip [--method voice]
```

This creates `lab/accounts/b/master.key`, `lab/accounts/b/request.json` (with
the working recipe), and `lab/profiles/b.json`. Then run registration with
the profile:

```sh
whatsapp-cli --profile b lab-signup --allow-live-request begin --method sms

echo $code | whatsapp-cli --profile b lab-signup --allow-live-request verify-code
```

Check the account with `accounts status` or a short live listen:

```sh
whatsapp-cli --profile b accounts status

whatsapp-cli --profile b listen --allow-live-request --seconds 20
```

The two numbers must never share a store, a config or a key variable. See
[configuration reference](../configuration.md) for all fields.

## Checking account state

```sh
whatsapp-cli accounts status [--name <name>]
```

Without `--name`, prints status for all profiles. With `--name`, prints only that
profile. Output shows, per account:

- **Locked**: master key environment variable is not set (key_env name will show)
- **Registration state**: Preparing, Prepared, Awaiting, Registered, etc.
- **Last authentication**: timestamp of successful login (if any)
- **Login checkpoint**: `recovery`, `challenge`, etc. (if a checkpoint is active)
- **Appeal token saved**: present/absent (indicator for appeal eligibility)
- **Pending outbox**: number of queued message intents
- **Queued actions**: number of pending IQ operations

Example output:
```
Profile a:
  Status: Registered
  Last auth: 2026-09-20T02:00:00Z
  Pending: 0 messages, 3 actions
  Appeal: saved
```

## Two-account check

```
whatsapp-cli lab-pair --from a --to b --text "pair check" --seconds 45 --allow-live-request
```

queues one text from `a` to `b`'s own number, runs both sessions at the same
time (the sender with outbox transmission, the receiver without), then prints
the sender's delivery facts and whether the receiver's journal projected the
text. It refuses profiles that share a store or a key variable. Both master
keys must be exported before running it.
