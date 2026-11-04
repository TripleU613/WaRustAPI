# Current notification sync obligation evidence

Target: WhatsApp Android 2.26.43.72, local APK `android-lab/apks/WhatsApp.1.apk`
(outside checkout), SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Static only. Findings and unresolved prerequisites:
[notification-sync-obligations.md](../../notification-sync-obligations.md).

Canonical REA envelopes retain complete original records and limitations:

- `account-sync-handler.json`: `ev_11b5bb4eecdff25153029ea4e75ebc35d4f15fd010d1f5b2af80ac8ab84ee1d4`; `{'path': '/home/tripleu/Documents/Codex/2026-11-04-hey/android-lab/apks/WhatsApp.1.apk', 'class_name': 'X.C19N', 'method_name': 'A09', 'overload_index': 0}`.
- `authkey-action.json`: `ev_238174e84bdc63027704f2062f7c071953369a312e8c5e5e2e43feeb6aea409c`; `{'path': '/home/tripleu/Documents/Codex/2026-11-04-hey/android-lab/apks/WhatsApp.1.apk', 'class_name': 'X.C19i', 'method_name': 'A00', 'overload_index': 0}`.
- `server-sync-handler.json`: `ev_2db151d4a0eb936afafb6726d4e358ed6ed5181c9f6481c07d3a9245b6fd20ee`; `{'path': '/home/tripleu/Documents/Codex/2026-11-04-hey/android-lab/apks/WhatsApp.1.apk', 'class_name': 'X.C14l', 'method_name': 'A09', 'overload_index': 0}`.

Type-name-to-integer binding and base dispatch/ACK envelopes are already retained
under `../2.26.43.72-notification-ack-audit/`. No duplicate REA IDs copied.
The ownership sidecar maps the account handler to the bounded non-modify readreceipts port only. Other account children, refresh, server sync and authkey rotation remain unported obligations.

Checked DEX text (partial annotated instruction streams, not compilable smali):

- `*-dex.txt`: `target/debug/xtask apk-smali <apk> --class <exact LX/...;> --signatures --exceptions`; handler-only account export also used `--method A09`, counter used `--method A0G`.
- `*-constructor-dex.txt`: existing `apk-research-bundle` with exact APK hash, full signatures, register flow and exception tables, exact class and `<init>` method filters. Constructors bind 203,210,248 directly to the base registration array.
- `handler-overrides.txt`: `apk-virtual-overrides <apk> --class 'LX/0v6;' --method A09 --signature '(LX/0Tp;I)V'`; 114178 DEX declarations checked, 35 declared overrides. Hierarchy only; no execution or dynamic call completeness.
- `kind-xrefs.txt`: `apk-xref <apk> --anchor account_sync --anchor server_sync --anchor authkey_rotation`; string refs alone are not dispatcher proof.

REA lookup for raw 19h returned no matching class alias; its actual DEX declaration
and complete two-method structural body are retained instead. Earlier 17v account
string xref points at a dirty-bit processor, not the incoming 203 handler; the
constructor inventory resolved 19N. Neither negative lookup means absent code.

## Text digests

- `14l-constructor-dex.txt`: `9c5c0a70c06a48598db0b9ca5bf74e98a617a08cde6b7bdf9f5aefed25947d14`
- `19N-constructor-dex.txt`: `ca3c6361d43c22a59166bf00f8cde0937e07d9095df4e9c88e911ef21d1e55eb`
- `19h-constructor-dex.txt`: `94f6b1a146ec8fa134a84cc56ee29b4d0acafc9cdf9f81da3afea0a1ef9ecccb`
- `account-sync-dex.txt`: `b9667b9a61ebc98070c556829f568a3e7993acb63d950af6dc39276cefa33046`
- `authkey-action-dex.txt`: `046cfab39f552b8bac3430a3e316cf9c78e2e800ffa091e70700cdd33fc94d69`
- `authkey-callback-dex.txt`: `957244b0427461e8817f48ca4c77236b2377d7de6fe42c976bd96d71959485c8`
- `authkey-counter-dex.txt`: `f00ad35b35d817bbe519057b299abc3ff2d17ff96b8edbdb4eed973cc3a2fe3b`
- `authkey-handler-dex.txt`: `44e514ca1fb77c26c1ce4a1783ce868a53a96f073f9a5170ce3e5ce666187de4`
- `handler-overrides.txt`: `2f87eebba014ab37de4797e56d84f67fec29d9d24a16867fa251834366dcf7cf`
- `kind-xrefs.txt`: `0a928ced71c0634241efe864ff60fca7d2199dc1f1d69cbbcd29aaa0d7cc86f2`
- `server-sync-dex.txt`: `5bbf4e143c6c05b048f364a22e35f11e5f91c8e7adb931a9e3d04c4f2e10059f`
