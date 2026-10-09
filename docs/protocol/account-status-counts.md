# Account pending-work count correction

2026-10-09. **Confirmed local**, not an APK protocol requirement.
The existing offline `accounts status` command reported `outbox_pending` using
ReadTx::count(Table::Outbox), which includes retained history. Cancelled,
failed and in-flight entries could therefore inflate a field described as
pending messages. Retained history must not be deleted to make the count smaller.

Outbox::pending_count now scans one consistent read snapshot in 256-row pages,
counts entries with progress **Queued**, **Prepared** or **RetryScheduled**, and
retains only one page rather than allocating the full pending list. It shares
the exact progress predicate with Outbox::pending. Cancelled, Failed and InFlight
are excluded. This counts logical pending work across all namespaces; transport
reservations, message-type support, account settings and server waits are not
evaluated here. A positive count is not permission or proof that a send is
currently eligible. No queue, progress, history, reservation or identity changes.

accounts status uses this count. Successful field names and types remain the
same; malformed saved rows produce the existing account error result rather
than a partial count. No new CLI flag or HTTP route is introduced. The public
library method accepts ReadTx and returns Result<usize>; its human inspection
path is the existing CLI command.

**Confirmed test:** a synthetic 300-entry history has all three pending states
beyond the first 256 entries, mixed with Cancelled/InFlight/Failed. The old row
count is 300; actual pending count and pending-list length are both 3. Empty
state returns zero. A malformed later row refuses counting and remains in the
store (301 rows), demonstrating no silent skip or repair. The fixture is local
outbox state, not a fabricated server response or native receipt_user row.

**Observed offline:** after rebuilding the CLI, `accounts status --name l`
with the existing profile's original private key reported registered=true in
saved local state and **outbox_pending=0**. This is not fresh server registration
or availability verification. The earlier attempted positional-name invocation
was rejected by CLI parsing; the actual command uses --name. No old-binary
account count was captured, so the synthetic 300-versus-3 fixture is the
before/after evidence; zero is the actual corrected-account observation.
The [saved receipt inspection](receipt-ack-correlation.md#original-saved-account-inspection-restored-2026-10-09)
separately confirms nine stable acknowledged reservations. No socket/live request,
new key, reset, retransmission or identity replacement occurred.

```sh
whatsapp-cli --json accounts status --name l
```

Supply the existing profile's configured master-key environment input locally;
never paste it into chat or commit it. The inspection command itself accepts
no key-value flag and does not create another key if that input is absent.

Validation: all **824 workspace/all-feature tests** passed (688 API unit,
4 codec, 9 crash, 52 CLI, 1 OpenAPI symbol, 70 xtask). Strict workspace/all-target/
all-feature Clippy, warning-free rustdoc, default CLI tests, CLI-reference
regeneration, formatting, whitespace and hygiene passed (1,934 local links;
unchanged 85 checksum bundles / 18,104,328 bytes). The rebuilt CLI's saved-account
check above completed offline; no fresh live compatibility is claimed.
