# Terminal stream errors and ACK diagnostics

## Saved-capture attribution (2026-10-08)

**Confirmed implementation:** `transport::inbox::preserve` saves encoded nodes
under `inbound:sha256:<content digest>` without a capture time or connection
epoch. Repeated identical nodes share a key. `lab-inbox --inspect` scans all
saved captures and deduplicates printed shapes; it is not a latest-session log.
The 2026-10-08 receive stdout/stderr proves authenticated activation and early
terminal closure, but contains no terminal tag or numeric reason. The stored
`stream:error`/ack shape cannot independently be attributed to that session.
There was no before-session capture census. Earlier wording identifying it as
*the* live terminal ACK is too strong; its origin remains **unknown**.

**Observed offline:** a saved stream:error has no outer attributes; one ack
child has class notification, an ID attribute and no to attribute. The fixed-label inspection identifies type w:gp2; no ID or address is exposed.
This does not establish what we transmitted, that an ACK was rejected during
the recent window, a ban, a server wait or causation by the AB-property query.
The query remains sent/outcome_unknown with its original reservation.

**Implemented:** `lab-inbox --inspect` includes terminal_ack_class,
terminal_ack_type, terminal_ack_id_present and terminal_ack_to_present when
one ack child occurs under stream:error. Only fixed class/type labels are
printed; unsupported values become unknown, absent values become absent.
Two or more ack children report terminal_ack=ambiguous. It never echoes an ID,
address or unknown field value. This is offline inspection; it sends nothing,
changes no capture key and applies no retry/recovery policy.

Tests cover known labels, echoed-ID/address/unknown-value redaction, unrelated
stanzas, missing children and duplicate-ack ambiguity. Presence flags say only
that an attribute exists, not that it is valid or that the APK requires it.

## APK anchors and remaining scope

**Confirmed static 2.26.37.71:** exact stream:error anchors locate 1R7.A02/A03
and 1Rl.<clinit>; WriterThread/write/send-message-ack locates 0y0.A00.
1R7.A02 is the companion interceptor path: it recognizes stream:error and,
under a separate 0GD.A02 predicate, parses an integer code with default zero
before routing to 1Dz.A1D. Its separate error branch reads class/id/code/text.
This method is not proof of the primary client's required closure handling or
an interpretation of this saved ACK. Primary routing, the active predicate,
0y0's complete ACK construction and attribution/correlation of this echoed group-notification ACK remain untraced.

[Scoped raw evidence](terminal-ack-evidence.txt) retains the exact anchor scan
and complete A02 disassembly with exception tables. No live capture, raw APK,
private number, key or echoed identifier is included.


**Validation:** 47 experimental CLI checks passed (46 unit and one API-symbol),
including two new diagnostic regressions. Strict CLI/all-target Clippy,
regenerated CLI reference, experimental build and formatting passed on system
Rust 1.99.0. Rebuilt offline inspection identifies the saved ACK type as w:gp2;
no network session was opened. Complete workspace tests were not repeated for
this CLI-only diagnostic chunk. Historical capture origin remains unknown.

Repository hygiene passed: 915 local links, tracked-file budgets, package
licenses and 3,840,464 bytes across five checksum-verified APK bundles.


## Network terminal provenance (2026-10-08)

**Implemented local diagnostic policy:** the comparison login reader and active
receive loop call `inbox::preserve_in_epoch`. The encoded capture, terminal
observation metadata and latest-terminal pointer commit in one transaction.
Only failure and stream:error get metadata. Existing content-addressed capture
keys and their wire data remain unchanged; no identity/session is migrated.

`lab-inbox --inspect` includes terminal_observation for those stanzas:
first_observed_at and last_observed_at are local Unix milliseconds; last_epoch
is actor-local and can repeat after process restart; observations counts newly
instrumented network observations, saturating at u64::MAX; latest_terminal
identifies the most recent instrumented terminal commit by commit order. The
wall clock can move backwards. These are local observations, not APK/server
fields, globally unique session identifiers or a complete per-session journal.
They establish latest-observed provenance when compared with the current run's
start/end, not the origin of every prior occurrence of the same content.

Old and offline-only captures report unknown. `preserve` and offline replay do
not add, increment or overwrite metadata. A later real network re-observation
of an old content hash records its current observation, preserving the wire;
first_observed_at then means first **instrumented** observation, not the first
time that content ever arrived. Inspection applies no recovery policy and does
not replay the uncertain AB-property request.

**Tested offline:** first/repeated observations, epoch reset/backward wall clock,
latest marker supersession, nonterminal exclusion, private-content redaction,
reopen persistence, unchanged content keys/wire and offline non-backfill. Invalid
stored observation metadata aborts the transaction: even the preceding capture
write and latest-marker update roll back. Failure diagnosis is preserved rather
than silently replacing corrupt metadata. Older capture origin remains unknown.


### Validation and bounded saved-account check

**Verification:** all 702 workspace/all-feature tests passed (583 API unit,
four codec integration, nine crash-matrix, 46 CLI unit, one API-symbol and
59 xtask). The existing real session-loop terminal fixture additionally now
asserts one recorded observation, matching epoch/time and latest marker; its
targeted rerun passed. Strict workspace/all-target Clippy passed again after
that test change. Warning-free Rustdoc, 24 default CLI checks, regenerated CLI
reference, experimental CLI build and formatting passed on system Rust 1.99.0.
Pinned Rust 1.98.1 proof is not claimed.

**Observed live:** preflight found no eligible outbox, queued action, pending
receipt or selected-chat refresh. The existing initial AB-property read was
already outcome_unknown and was not requeued. One 32-second receive-only
listen authenticated, became active, completed its window and stopped cleanly
(exit 0), without --send-outbox. No instrumented terminal observation occurred;
offline inspection still has one historical terminal shape with unknown
provenance and zero known terminal observations. This clean session does not
explain the previous closure or live-test terminal persistence; the session-loop
fixture is the verification for the terminal code path. It also does not
establish fresh incoming-message decryption or long-term account reliability.

**Observed postflight:** actions, selected devices, selected pins, receipts and
outbox matched preflight; zero new actions. The initial read remains
outcome_unknown with the same action ID; no query was queued or replayed.
Login/captures can persist, so these comparisons do not claim all store tables
are unchanged. Private command output remains in ignored private local storage.

Repository hygiene passed: 917 local links, tracked-file budgets, package
licenses and 3,840,464 bytes across five checksum-verified APK bundles.

### Auth-challenge ACKs preserve uncertainty (2026-11-04)

**Confirmed static / fixed offline:** APK43.72 1rH.A03 diverts numeric459
to onAuthChallengeReceived before ordinary ACK class processing. The traced
manager transfers an existing request between maps; it is not an ordinary
terminal message failure or proof of a ban. Rust direct/group ACK settlement
now excludes459, preserving original reservations and uncertainty. Java decimal
spellings are covered; no challenge bypass, retry or account-wide policy added.
[Methods, before/after regression and unported boundaries](apk/2.26.43.72-ack-auth-challenge/README.md).
No live operations. Full challenge-controller parity remains incomplete.

### Undecodable comparison wait frames retained (2026-11-04)

**Confirmed local gap / fixed offline:** wait_for_captured persisted decoded
nodes but still used read_node, which discarded a decrypted frame on stanza
inflation or binary decode failure. It now reads FrameRead and preserves an
Undecodable frame through the existing encrypted inbox capture mechanism
before returning a fixed uncertainty/no-retry error. Already captured decoded
nodes remain. The loop never classifies the malformed frame as a response or
ACKs it, and never reads another node after that failure.

The stored record retains the exact decrypted, pre-inflation frame, fixed codec
error class, first observation time and epoch=None (comparison/lab window),
using the existing content-hash identity. The existing default `undecodable`
command lists these captures redacted and rechecks them with the current
decoder; it does not send traffic or automatically settle them. Capture-store
errors propagate and cannot become successful request settlement.

The shared wait-loop regression now includes malformed binary and invalid
compressed payload cases after two decoded unmatched nodes. It verifies the
error, all prior captures, exact saved frame bytes/class/size, absent actor
epoch and unchanged uncertain send intent after reopening the encrypted store.
Reader injection tests the shared control flow and capture storage; it is not
a Noise-handshake or live malformed-server test.

**Evidence/authority:** this is local diagnostic hardening, not a reconstructed
APK decoder-failure policy. Existing native result/error dispatch evidence
remains the admission reference; whether the APK continues or closes after each
decode failure remains untraced. The comparison waiter exits conservatively.
The ordinary actor's existing continue/capture policy is unchanged.

**Limits:** ciphertext that fails authentication, incomplete frames and data not
yet read do not yield a decrypted frame for this path. The legacy no-store
wait_for remains uncaptured. The earlier decoded-only comparison limitation is
closed for successfully decrypted records; no old missing evidence is restored.
This adds no replay, account reset, fabricated response, new route or live operation.

### Challenged-request selection and re-dispatch ownership (2026-11-04)

**Confirmed static, APK43.72:**0TV.A03 selects eligible held requests without
removing those source entries; over-budget entries are removed.0Sf.A08/A0L
re-dispatch through A04 with the original callback retained when absent from
the new invocation. A token-result callback and the selected handleConnected
block reach selection. The token callback calls key rotation before dispatch,
but does not prove rotation completion. Client-side selection counters are not
a server retry allowance or authority to replay Rust reservations.
[Exact methods, map inventories, count conditions and remaining gaps](apk/2.26.43.72-challenged-request-ownership/README.md).
The post-token helper is now connected; token construction/converters, request
family/callback ownership and durable restart recovery remain incomplete.
No production change, new retry authority or live operation.

### Client-auth-token writer grammar and validation (2026-11-04)

**Confirmed static, APK43.72:** exact Dzb(String,String,int) selector28 builds
a server IQ GET in urn:xmpp:whatsapp:push with a pn child containing UTF-8
manager input. That label is not proof of phone-number input; the branch does
not create an auth_token element. The ID helper checks String.length, not a
53-bit numeric value. Nonnull length violations log via00L.A0C and return true
in the selected validator;300 is not a proven server or hard constructor limit.
[Selected overload, helper chain, checksums and unknowns](apk/2.26.43.72-client-auth-token-writer/README.md).
Writer construction is now connected; input eligibility/authenticity, reply
converters and durable challenge recovery remain unported. No production
change, request sender, new retry authority or live traffic.

### Client-auth-token reply binding and byte grammar (2026-11-04)

**Confirmed static, APK43.72:** FUT's selected result pipeline first requires
iq, matching original request ID, parsed from equal to original destination
and type=result. It then reads opaque cat bytes with strict1..1024 bounds.
The value converter alone does not perform request correlation. Native child
lookup picks the first cat, without duplicate counting in this selected path.
Invalid header throws schema error; null token conversion after a valid header
returns without token persistence or held-request dispatch.
[Converters, helper chain and remaining eligibility/recovery limits](apk/2.26.43.72-client-auth-token-reply/README.md).
This closes the selected reply converter gap, not native JID parser, input
authenticity, failure/backoff or durable recovery parity. No production change
or live operation; Rust uncertain holds remain.
