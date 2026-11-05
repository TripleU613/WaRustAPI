# Conditional startup availability and unified-session search boundary

**Confirmed static, APK2.26.43.72**, checked DEX TXT, not observed traffic or
REA Evidence envelopes. Pinned SHA256:
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Reproduce methods with `xtask apk-smali APK --class CLASS --method METHOD
--signatures --exceptions`; literal inventory uses `xtask apk-xref APK
--anchor unified_session`. Selected methods are complete. Startup excerpts are
explicitly labelled and match the [full retained method](../../rea/2.26.43.72-authkey-outcomes/startup-trigger-checked.txt).

## No inbound unified-session literal found in this inventory

The exact unified_session literal inventory returns two code references, both
0Sb outbound buildersA01/A02. No inbound handler appears under that literal.
This is not proof the server sends no reply or that the APK has no acknowledgement:
tokenized dispatch, generic response routing, dynamically supplied names and
other protocol representations are outside this literal query. The prior writer
trace remains a local no-IQ-ID path, not proof of global server ACK policy.
No fabricated success/error response or timeout requirement is added.

## Following startup write is conditional bare availability

After config GET and nullable unified-session writing,0ZJ.A0z at0b9c skips the
following block when currentv9 is nonzero. Ifv9 is zero it calls1rj.A01 only
whenv64 is nonzero. In the method signaturev64 is the first Boolean argument;
the selected entry log labels its copied value available. Full caller policy,
all reaching definitions/branches and force-passive interactions are not closed
by this excerpt. It is not evidence every connection announces availability.

1rj.A01 constructs presence with type=available, using the literal allowed list
available/unavailable, then calls1r2.Cyu(node,1). There is no name, to, id,
namespace, IQ wrapper or child in this selected builder. It is distinct from
the display-name writer branch which can add name. No explicit callback-map
insertion, result wait or timer is present inA01 itself; fullCyu effects and
server response policy remain outside this selected method.

The entry obtainsv9 from0GS.A02. That predicate reads integer
companion_registration_state (default0) and returns true for2..6 or10..17,
false otherwise. Numeric enum meanings and all producer transitions remain
untraced here. Moreoverv9 is reused/reassigned later in the large startup
method, so this entry predicate must not be blindly substituted for the later
0b9c value without a reaching-definition/control-flow proof. The retained
method's direct reassignments are visible; no global primary/companion send
policy is claimed from these two local observations.

## Current Rust comparison and remaining requirements

Rust's messaging::available_presence builds the same bare presence/type=available
shape; the current actor emits it after successful passive/active activation as
recorded in the [startup audit](../../network-startup-audit-2026-11-04.md).
Shape agreement is not proof of native conditional startup ordering, mode parity,
server response policy or native availability selection. Existing display-name
queuing is a separate operation. This chunk establishes additional current-build
writer evidence and leaves policy differences explicit rather than introducing
an unproven gate or always-on new startup request.

Full availability caller provenance, v9 control flow, companion/passive state
semantics, Cyu outcomes, complete bootstrap ordering, ACK policy and generation
recovery remain open. The Rust builder comment now removes its unproven universal-availability claim.
No Rust behavior, sender, retry, HTTP route or live
account/network operation changed.
