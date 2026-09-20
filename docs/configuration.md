# Configuration reference

The private registration configuration file (`request.json`) holds phone, identity and request parameters. The authoritative schema is `LabRegistrationConfig` in `/docs/api/openapi.json`; see also the [persisted registration lab](protocol/registration-lab.md) and [onboarding commands](accounts.md).

## Fields

| Field | Type | Default | Meaning | Working value |
| --- | --- | --- | --- | --- |
| `phone.country_code` | int (1–999) | required | E.164 country calling code (not ISO 3166) | `1` |
| `phone.national_number` | string, digits | required | national significant number, no trunk zero | example `5551234567` |
| `language` | string | required | request language | `"en"` |
| `locale` | string | required | request locale, e.g. `US` or `CA` (not a GPS claim) | `"US"` |
| `token` | string (base64) | — | Phone-bound registration token, derived from 2.26.37.70 APK material | from `whatsapp-cli token derive --material <zip>` |
| `registration_id` | int (1–16380) | required | Signal registration id, fresh per account | random, set by `accounts new` |
| `signed_pre_key_id` | int (0–16777215) | required | signed-prekey id (24-bit wire encoding) | `1` (set by `accounts new`) |
| `method` | enum | `"sms"` | Delivery method: `"sms"` or `"voice"` | `"sms"` |
| `client.profile_id` | string | `"android-2.26.37.70-registration-lab"` | Build identifier | `"android-2.26.37.70-registration-lab"` |
| `client.user_agent` | string or null | null | HTTP User-Agent | `"WhatsApp/2.26.37.70 Android/unknown Device/unknown-unknown"` |
| `device.facebook_device_id` | UUID string or null | null | null = the library generates the value once and persists it in the store; set only to override | `null` |
| `device.experiment_id` | 16-byte array or null | null | null = the library generates the value once and persists it in the store; set only to override | `null` |
| `device.access_session_id` | 16-byte array or null | null | null = the library generates the value once and persists it in the store; set only to override | `null` |
| `device.recovery_token` | opaque byte array or null | null | null = the library generates the value once and persists it in the store; set only to override | `null` |
| `device.backup_token` | opaque byte array or null | null | null = the library generates the value once and persists it in the store; set only to override | `null` |
| `fields.exist` | object (string→string) | `{}` | Extra form fields for `/v2/exist` preflight (UTF-8 strings) | `{}` |
| `fields.exist_bytes` | object (string→string or int[]) | `{}` | `/v2/exist` byte fields | `{}` |
| `fields.code` | object string→string | `{}` | extra `/v2/code` text fields | `{}` |
| `fields.code_bytes` | object string→(string or int[]) | `{}` | `/v2/code` byte fields; the observed APK fields | the eight fields: mcc `"000"`, mnc `"000"`, sim_mcc `"000"`, sim_mnc `"000"`, education_screen_displayed `"false"`, prefer_sms_over_flash `"false"`, feo2_query_status `"did_not_query"`, client_metrics `"{\"attempts\":1}"` |
| `fields.register` | object (string→string) | `{}` | Extra form fields for `/v2/register` | `{}` |
| `fields.register_bytes` | object string→(string or int[]) | `{}` | `/v2/register` byte fields | `{}` |
| `fields.security` | object (string→string) | `{}` | Extra form fields for `/v2/security` | `{}` |
| `fields.security_bytes` | object string→(string or int[]) | `{}` | `/v2/security` byte fields | `{}` |
| `transport.body_mode` | enum | `clear_form` when the key is absent; the template ships `enc_empty_h` | Request encoding: `"clear_form"`, `"enc"`, or `"enc_empty_h"` | `"enc_empty_h"` |
| `transport.timeout_ms` | int (1–300000) | `45000` | whole-request deadline incl. response streaming | `45000` |
| `transport.connect_timeout_ms` | int (1–timeout_ms) | `30000` | connection deadline | `30000` |
| `transport.max_response_bytes` | int (1–1048576) | `1048576` | response byte budget (library ceiling 1 MiB) | `1048576` |
| `transport.reconnect.max_attempts` | int (0–100) | `5` | consecutive automatic reconnects after classified recoverable loss; 0 disables | `5` |
| `transport.reconnect.base_delay_ms` | int (1–max_delay_ms) | `2000` | first delay, doubles per attempt with jitter | `2000` |
| `transport.reconnect.max_delay_ms` | int (base–300000) | `60000` | delay ceiling | `60000` |
| `transport.action_delay_ms` | int (0–300000) | `0` | pause between consecutive queued-action sends in one session | `0` by default; e.g. `5000`–`15000` for human-scale pacing |
| `diagnostics` | bool | `true` | print sanitized request/response metadata, never bodies | `true` |
| `preflight` | bool | `false` | run a `/v2/exist` probe before `/v2/code` | `false` |

## Environment variables

| Variable | Meaning | Example |
| --- | --- | --- |
| `WHATSAPP_API_MASTER_KEY_*` | Master key for encrypted store (one per account) | `export WHATSAPP_API_MASTER_KEY_A=$(cat lab/accounts/a/master.key)` |
| `WHATSAPP_API_PROFILE` | Default profile name (overridable with `--profile`) | `export WHATSAPP_API_PROFILE=a` |
| `WHATSAPP_API_PROFILES_DIR` | Directory containing profile JSON files (default: `lab/profiles`) | `export WHATSAPP_API_PROFILES_DIR=lab/profiles` |
| `WHATSAPP_API_STORE` | Path to encrypted store (overridable with `--store`) | `export WHATSAPP_API_STORE=lab/accounts/a/signup.redb` |
| `WHATSAPP_API_ACCOUNT` | Account name in store (overridable with `--account`) | `export WHATSAPP_API_ACCOUNT=signup-a` |
| `WHATSAPP_API_TOKEN_MATERIAL` | Path to token-derivation material ZIP (for `whatsapp-cli token derive`) | `export WHATSAPP_API_TOKEN_MATERIAL=lab/whatsapp-2.26.37.70-token-material.zip` |
| `WHATSAPP_API_BIND` | HTTP control plane bind address (default: `127.0.0.1:8787`) | `export WHATSAPP_API_BIND=127.0.0.1:8787` |

`--key-env <VAR>` names the variable; profiles set it per account.

## Profile format

A profile is a small JSON file `lab/profiles/<name>.json` (directory ignored by Git):

```json
{
  "store": "lab/accounts/a/signup.redb",
  "account": "signup-a",
  "config": "lab/accounts/a/request.json",
  "key_env": "WHATSAPP_API_MASTER_KEY_A"
}
```

It holds no secrets; the master key is kept in the named environment variable. Commands fill store, account, and key_env from the profile when `--profile <name>` is given, but explicit flags still win.

## Token material (2.26.37.70)

The 2.26.37.70 APK is no longer on disk; `~/Downloads/WhatsApp.apk` is 2.26.37.71 and produces tokens that do not match the working account's saved token. Keep the 2.26.37.70 token-derivation material as:

```
lab/whatsapp-2.26.37.70-token-material.zip
```

The ZIP contains `classes.dex`, `META-INF/IMPORTED.DSA`, `AndroidManifest.xml`, `resources.arsc`, and `res/drawable-hdpi-v4/about_logo.png`. Build or rebuild the archive with `cargo run -p xtask -- apk-token-material --from-dir <unpacked 2.26.37.70 tree> --logo-from ~/Downloads/WhatsApp.apk --out lab/whatsapp-2.26.37.70-token-material.zip --verify-config <a known-good request.json>`.

Set `WHATSAPP_API_TOKEN_MATERIAL` to point at it, or pass `--material` explicitly to token commands:

```sh
whatsapp-cli token derive --country-code 1 --national-number 5551234567 --material lab/whatsapp-2.26.37.70-token-material.zip
whatsapp-cli token verify --config lab/accounts/a/request.json --material lab/whatsapp-2.26.37.70-token-material.zip
```
