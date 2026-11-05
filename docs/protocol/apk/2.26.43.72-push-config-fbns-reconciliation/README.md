# FBNS config reconciliation — APK 2.26.43.72

**Confirmed static**, checked DEX TXT, not runtime observations or REA Evidence
envelopes. Pinned APK SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reproduce: `xtask apk-smali APK --class CLASS --method METHOD --signatures
--exceptions`. Complete selected methods are retained and checksummed. Provider
check detail beyond the conditions named here is retained for later work.

## Gate and cached user predicate

The selected type6 success consumer and type27/404 error consumer call1uo.A00
with a projected FBNS String or null, respectively, after checking1uo.A01.
A00 rechecksA01 itself. A01 requires the A14(0x384c) predicate and a cached
Boolean computed under the instance monitor. The computation reads the current
user serial through UserManager and compares it to0; missing UserManager or a
caught Exception follows the true fallback. The cached value is therefore not
proof a fresh OS user check succeeded, nor evidence that every device enables
FBNS. Full feature-configuration provenance and cache invalidation remain open.

## Compare and retain the server view before refresh

When enabled,A00 reads local fbns_token (default null) and fbns_app_vers
(default0). It suppresses refresh only when the supplied server String is
nonempty, equals that local token, AND the saved version is264307238
(hex0xfc10226, this APK version code). Otherwise its selected boolean requests
refresh. No trim or token decoding occurs in this comparison.

Before branching on that boolean, it edits last_server_fbns_token: null input
removes that key; nonnull input, including an empty String, uses putString.
Both branches call apply. The selected body does not remove fbns_token or
rewrite fbns_app_vers. This distinction matters to404, whose upstream consumer
supplies null: deleting the saved server view is not deleting the local token.
The call to apply is not evidence of a completed durable write across a crash.
The method does not wait for a confirmed disk commit before provider work.

On the no-refresh path it logs no-need-to-refresh and finishes the selected
verification markers. Otherwise it attempts provider-token registration. Marker
start/end calls are conditional on1iC's AtomicBoolean; their presence does not
establish successful registration, persistence or server delivery.

## Refresh requests an Android provider, not a direct WhatsApp IQ

A00 iterates1uq.A00 in order and chooses the first package accepted by1ur.A01.
The retained static list is com.facebook.services, com.oculus.vrshell,
com.oculus.horizon.1uq also initializes a separate A01 list including a dev
package, but this selected refresh loop uses A00, not that four-member list.
No accepted package means no selected startService call.

1ur.A01 checks list membership, PackageManager/package/application availability,
enabled state, system/updated-system flag bits, and additional permission and
signature-digest conditions. Its package-specific branches and failure cases
are retained, not reduced to package presence. A similarly named installed app
is insufficient evidence that this check returns true. Runtime provider state
was not queried and no provider was installed or invoked.

For an accepted package,A00 builds action
com.facebook.rti.fbns.intent.REGISTER, includes pkg_name from Context and appid
from0cJ.A0G, checks list membership again, and sets a package-specific explicit
service ComponentName. It calls1ur.A00(Context,Intent), then startService.
The default component is PreloadedFbnsService; the two Oculus packages select
their named FBNS components.1ur.A00's augmentation and the actual appid source
are not reconstructed in this chunk. The server token String is not supplied
as an extra in this selected construction. Callback token acquisition and later
WhatsApp config SET are separate untraced steps, not successful outcomes of this
void call.

The service-start region has catches for IllegalStateException,
SecurityException and RuntimeException, with SDK/cause-dependent continuations
or rethrow. Do not treat its normal return/markers as a completed token request,
and do not infer an unconditional retry from these catches. The complete
exception paths remain in the checked export.

## Port boundary

This closes selected FBNS eligibility, server/local comparison, last-server-view
update ordering and provider IPC entry. Provider callback/token authenticity,
local token/version persistence,1ur.A00 effects, startup feature provenance,
service success, later config SET, generation fencing and durable recovery remain
open. There is no basis for fabricating an FBNS token or requiring this optional
provider flow for every Rust client. No Rust behavior, replay authority, HTTP
route, provider IPC or live account/network operation is added.
