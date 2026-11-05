# FBNS token receipt and optimistic server marker — APK 2.26.43.72

**Confirmed static**, checked DEX TXT, not runtime observations or REA Evidence
envelopes. Pinned APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reproduce: `xtask apk-smali APK --class CLASS --method METHOD --signatures
--exceptions`; token literal references use `xtask apk-xref APK --anchor fbns_token`.
Small selected methods are complete. Fei.run is explicitly excerpted with its
complete-export hash, dispatcher, selector0 region, out-of-line continuations
and exception table; unrelated selector bodies are omitted. Only registered-token
handling and the authentication entry below are interpreted in detail.

## Android IPC authentication and worker admission

The outgoing1ur.A00 creates a fresh Intent restricted to Context.getPackageName,
obtains an activity PendingIntent with requestCode0 and raw flags0x0c000000,
and puts it under auth_pending_intent in an auth_bundle extra. This is Android
IPC metadata, not a WhatsApp wire attribute or fabricated token proof.

WAFbnsPreloadReceiver.onReceive ignores null Intent; otherwise it constructs
Fei through factoryA00 with selector0 and Context/Intent/receiver arguments,
then schedules the worker through its wakeful executor. The exact factory and
constructor connect those arguments to fields; the run switch maps0 to0806.
Executor completion/wakelock ownership beyond this method remains untraced.

The worker requires action com.facebook.rti.fbns.intent.RECEIVE, an auth_bundle,
a typed PendingIntent under auth_pending_intent, nonnull creator package and
successful1ur.A01(Context,creatorPackage). That is the previously retained
[provider validator](../2.26.43.72-push-config-fbns-reconciliation/provider-check.txt),
not trust in an arbitrary String extra. It then branches on receive_type.
For registered it reads data, rechecks1uo.A01 and rejects empty data before
entering the out-of-line token state region at1e40. Receiver manifest admission,
PendingIntent extraction helper and OS/provider runtime authenticity are not
fully reconstructed; these selected checks are not a global trust proof.
The adjacent message branch callsF2Y.A02 after JSON extraction, including pn;
its full mapping remains a separate audit item. Selected provider ACK handling
is traced below.

## Token state transition and order

DBe.A05 resolves09S.A0S().A03, the same preference accessor family previously
used for fbns_token and last_server_fbns_token. The monitor region reads both
keys. For a nonempty received token T:

| Comparison at receipt | Selected behavior |
| --- | --- |
| T equals saved last_server and local token | Logs already saved; exits without SET or version rewrite |
| T equals saved last_server but differs from local token | Writes local token and current app version; no SET |
| T differs from saved last_server | Calls SET sender, writes last_server marker on normal return, then compares/saves local token if different |

The server-different branch at1eca obtainsEbv and callsA00 with null Fuc,
Ee7, account Integer, settings Long and pkey; provider token=T and platform=fbns.
The previously [traced sender](../2.26.43.72-push-config-corrective-submit/README.md)
builds config SET, defaults account count and applies local readiness/channel
admission. The call's boolean result is discarded: there is no move-result or
conditional between invoke at1ee6 and the following preference edit.

After a normal return,2L8.A1U writes last_server_fbns_token=T and calls apply,
then control rejoins the local-token comparison at1e86. A changed local token
is saved with fbns_app_vers=264307238 through6og.A1M's putInt/apply. The
unchanged-local branch does not refresh its app-version preference here.
A thrown sender failure does not follow the normal marker-update continuation.
No synchronous disk success is checked in these apply helpers.

Consequently last_server_fbns_token is not exclusively an authenticated server
view: the config response helper writes/removes it from supplied server state,
while this callback also writes it optimistically after local submit returns,
including false admission. It is not proof of wire delivery or server acceptance.
The null callback and discarded boolean do not grant replay authority after a
restart. This is native behavior to model carefully, not a confirmed existing
Rust defect or a reason to weaken durable uncertain-outcome protections.

## Provider message ACK after normal handling or JSON failure

**Confirmed static,2026-11-05:** the literal ACK-action reference inventory finds
one direct String reference inFei.run. That is a bounded literal search, not an
exhaustive census of dynamically constructed actions or every provider version.
The existing selected worker excerpts retain all instructions described here.

After authenticated receive_type=message admission, the selected body either
skips empty data, parses its JSON and invokesF2Y.A02, or catches JSONException.
Empty data branches from08d4 to0a00. A normal downstream return branches from
09ec to0a00. The catch table maps code units046c..04fa (printed bytes08d8..09f4)
to04fa (printed09f4); that catch logs invalid payload then falls through to0a00.
There is no payload-success boolean gate on this common ACK continuation.
Other thrown failures are not converted to this JSON catch merely by the ACK
flag's name; full downstream exception and deferred-work behavior remain open.

At0a00 the worker reads extra_notification_sender and extra_notification_id.
Null or String.isEmpty for either returns without broadcast. There is no trim
in these selected checks. It constructs action
com.facebook.rti.intent.ACTION_NOTIFICATION_ACK, copies extra_notification_id,
and sets boolean extra_processor_completed=true. It then requires the sender
String to belong to1uq.A00, the same three-package list retained in the earlier
[provider bundle](../2.26.43.72-push-config-fbns-reconciliation/provider-list.txt).
An unknown package logs instead of reaching the selected broadcast call.

The out-of-line continuation at1e18 sets Intent.package to the sender String,
adds1ur.A00's auth_bundle/PendingIntent metadata, and calls Context.sendBroadcast.
The ACK destination is supplied by extra_notification_sender; the earlier
PendingIntent creator was separately checked through1ur.A01. This ACK block does
not compare those two package values or rerun1ur.A01 for the sender. That is a
bounded description of these checks, not proof of an exploitable trust failure
or permission to bypass provider admission. Manifest/OS authority remains open.

The broadcast path catches RuntimeException: a DeadObjectException cause returns;
other causes rethrow in its out-of-line handler. No selected response wait,
delivery confirmation, retry timer or durable ACK reservation appears around this
call. Normal sendBroadcast return is not observed receipt by the provider.

This ACK is Android provider IPC, not an ack/class=notification WhatsApp stanza.
extra_processor_completed=true is sent even after the selected JSON parse
failure; it cannot establish successfulF2Y processing, decrypted message delivery,
state persistence or a server-side WhatsApp acknowledgement. It must not be used
to justify adding generic WhatsApp ACKs for unknown notification types. A future
FBNS port needs the provider-specific authority and lifecycle; no such runtime
path is enabled by this evidence-only change.

## Remaining work

Full provider/manifest authority, callback freshness, monitor/account-generation
binding, concurrent preference visibility, malformed IPC handling, optional SET
attributes, later acknowledgements, retry ownership, disk recovery and complete downstream FBNS
message processing/durable ACK ownership remain incomplete. No Rust behavior, provider invocation,
sender, HTTP route or live account/network operation is added.
