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

1. Create the account's private directory and configuration from the template
   (`lab-signup template`, then edit the phone, country and token as
   `docs/protocol/registration-lab.md` describes); generate its master key with
   `keygen` and export it under a new variable name.
2. Add the profile, then run the registration steps with `--profile b`:
   `lab-signup --allow-live-request begin --method sms`, enter the code
   locally when it arrives, and confirm with `lab-signup status`.
3. Check the account with `--profile b connect` or a short
   `listen --allow-live-request --seconds 20`.

The two numbers must never share a store, a config or a key variable.

## Two-account check

```
whatsapp-cli lab-pair --from a --to b --text "pair check" --seconds 45 --allow-live-request
```

queues one text from `a` to `b`'s own number, runs both sessions at the same
time (the sender with outbox transmission, the receiver without), then prints
the sender's delivery facts and whether the receiver's journal projected the
text. It refuses profiles that share a store or a key variable. Both master
keys must be exported before running it.
