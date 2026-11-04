# Current session source, primary Me and callback lifecycle

2026-11-04. **Confirmed static** unless labelled unknown. WhatsApp Android
**2.26.43.72**, SHA-256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
[Evidence](rea/2.26.43.72-session-source-me/README.md).
This continues [pairing input/correlation](primary-pairing-auth-current.md).
No source code, account state, pairing attempt or network traffic changed.
The selected transport authentication and primary Me lifecycle are now traced;
a complete account-switch/logout fence for own-device eligibility is still unknown.

## Frames are authenticated before IQ parsing

1s2.run's normal reader obtains nodes through123.A08. It then uses the stanza
tag to find the1rA parser and wraps the node in1uU; the checked stream at
104a..1072 confirms parser dispatch. 1rC.AL1 constructs the IQ parser1rD.
The other direct1rA.A03 caller,2Ek.A01, routes typed1k3 stanzas through a map
and reports parsing failures. It is not an independent cryptographic verifier.

123.A08 reads the three-byte frame length and exact framed ciphertext from
1qz.A00. It invokes1qw.A03.A00 with empty associated data and the receive
AtomicLong.getAndIncrement before interpreting the plaintext flags, decompressing
or decoding the binary stanza. Empty plaintext frames are skipped; header-only
frames are rejected. 1oU.A00 builds the12-byte nonce with1nr.A02 at offset4,
uses1pT over1pQ, decrypts with the saved key and finalizes through AOL.
1pQ.AVy names AES;1pT.AVy names /GCM. AOL rejects short data and a mismatched
tag with TRb;1oU wraps TRb in2AD and123 wraps it in29k. A failed tag therefore
does not become a parsed IQ response on this path. Full session termination and
counter cleanup on every exception are separate from this local rejection.

1qw's constructor installs its second byte-array argument as the inbound1oU
and its first as the outbound1oU. Its sole constructor caller is1oK.A01:
it derives64 bytes from the handshake chaining state and splits32/32.
The selected1o3.A03 server-hello path performs certificate validation before
writing ClientFinish and calling that split. Its complete smali shows:

- Legacy certificate: parse Dai and DfK, look up the named issuer in27C.A00,
  verify the signature over details, require certificate key bytes to equal
  the decrypted proposed server static key, and reject a present expiry strictly
  earlier than current time in seconds.
- WA6 branch (mode6): parse DC9 intermediate/leaf certificates and DC7 details,
  require intermediate serial to equal leaf issuer serial and intermediate
  issuer serial zero, require leaf key to equal the server static, then verify
  leaf and intermediate signatures through the intermediate and fixed root.
  The leaf verification uses the parsed leaf details' serialized bytes;
  the intermediate verification uses its retained details bytes.
- Validation failure reaches29h; successful selected branches continue to
  encrypted client static/login payload, ClientFinish write and transport split.

27C.<clinit> installs only WhatsAppLongTerm1 in an unmodifiable issuer map.
Its32-byte root matches the existing Rust COMPARISON_CHAIN_POLICY in
transport/handshake.rs. This is a current-build reanchor of existing server
certificate authority, not a new trust root. 1nk.A00 forwards signature
verification to1oA's provider; its implementation was not re-audited here.

The large1o3 constructor also contains cached-key/resumption and optional hybrid
branches. They are retained, but their selection, cache pin provenance and full
failure behavior remain **unknown** in this chunk. The certificate conclusions
above describe the selected A03 path, not every constructor branch.

## Login precedes the long-running reader; it is not an own-device assertion

Reuse current0ZJ.A0z in the authkey-outcomes bundle: it constructs123 from
1o3.A05's1qz, passes that123 into1rh, and runs the login sequence throughA12
before constructing1s2. A12 schedules1rl.A00 ->0ZJ.A1k ->A11 ->A1l ->A13.
The retained startup-callers A13 reads from that123, handles success/failure
and rejects unexpected nodes or unexpected stream end. This ties the selected
IQ reader to the encrypted connection's login sequence; it is not a separate
plaintext input source.

A13's success LID handling compares supplied and saved identity for diagnostics,
can fill absent LID through ChR under its flag, and does not create an active
own companion. The selected1rD/1lZ/1lY/0ln path from the preceding note has no
explicit IQ from comparison. Its lower-level session protection is now traced.
Authenticated transport plus callback ID still does not supply QR identity,
operator approval, own-device membership or an account-generation fence.

Rust already separates verified server certificate, encrypted login finish and
login-success in transport/handshake.rs and connection.rs. Its channel uses
ConnectionEpoch. These are real existing protections. They are not the missing
verified own-device pairing lifecycle; no new authority record was synthesized.

## Successful registration preferences precede persistent primary Me

The current exact-string scan for registration_jid and registration_lid finds
09S getters,09S.A11 setter and13N.A0H clear. A11 stores both strings, invokes
the registration identity bookkeeping and clears the wipe timestamp.
13N.A0I calls A11 then09S.A12 for country/phone and its own A03 bookkeeping.
A11 has other origins including companion107.BLn and WaConsentRepository;
the preference name alone does not identify the registration path.

A concrete primary successful-verification path is now connected using retained
register-phone-number and otp-pin-handoff evidence:

1. KotlinRegistrationBridge.A07 maps successful register response login into
   HS3.A0O and lid intoA0N. The wire field here is login, not a newly guessed jid
   field. Its status/reason mapping belongs to the retained registration parser.
2. VerifyPhoneNumber.A5n's selected success outcome invokesA2C. A2C passes those
   saved response fields and its held contact-point HFg intoA1j.
3. A1j calls HYT.A01 for normal modes. Modes4/5 explicitly skip this registration
   state write for re-verification. HYT.A01 passes the login/LID and held
   country/phone into13N.A0I, then calls13N.A0D.
4. A0D ->A0N starts passive connection if not already started. A0M reconstructs
   Me through13N.A06 from the saved registration preferences; absent parsed PN
   fails. It calls09O.A05 to install Me in memory and schedules INh discriminator1.
   That coroutine can fill LID under its flag, marks connection dirty and
   saves passive-start state. It does not itself call CrQ or prove file persistence.
5. RegisterNameManager.A05 later reconstructs Me through13N.A06 in setupMeObject.
   Missing parameters, CrQ failure and CrQ success have separate reported outcomes.
   Its concrete09O.CrQ call reaches the already retained Me file writer.
   Profile setup and passive in-memory installation are separate milestones.

The additional RequestCode success route is structurally retained:
VerifyPhoneNumber.A5o's selected response status A0C invokesA2M; A2M forwards
HiP.A0N/A0M throughA1j. Checked DEX confirms the selected gate.
GpO names status integer2 OK, while the ordinary sent result is a different
branch. This is not permission to treat SMS dispatch or any HTTP200 as completed
registration. HiP.A0N's exact field-reference scan finds both HTTP bridge and
RequestCodeRepository writers plus parcel construction; the upstream source and
transport authenticity of every alternate origin remain **unknown** here.

The successful-state sink is now traced, including durable registration
preferences and eventual persistent Me. The full HTTP response/request binding,
TLS owner, selected private attempt generation and every alternate registration
entrypoint were not re-audited here. Existing Rust registration state and
server-assigned login/LID must retain their actual completion provenance;
a file containing Me or user-supplied PN/LID is not equivalent.

## Disconnect clears active IQs, not a proven global account fence

0YB.A05 handleDisconnected invokes0ln.A0F. Checked A0F shows conditional timer
bookkeeping, then unconditionally posts handler message3 carrying the reason.
Checked0m2.handleMessage case3 at055a..05f8 snapshots and clears activeA0H under
the same lock before scheduling each failure callback. A0F does not itself
synchronously drain callbacks. The selected case does not clear waitingA0I.
0ln.A05 is incoming-handler initialization, not IQ reset; this negative branch
is retained so it is not confused with lifecycle cleanup.

This proves one disconnect invalidation mechanism. It does not prove clearing
waiting requests, canceling already queued/executing FUY callbacks, changing
account identity, or fencing workers that already captured EXq/Eo9.
The preceding note retains native FUY.A0N checks before preparation and Eo9
creation and old-request deactivation on replacement. Whether logout and every
account switch necessarily invoke that cancellation remains **unknown**.

F1G.A02's exact incoming-reference inventory is retained for the next audit:
EeE.A00/A01, F4O.onKey, Fbz.run, FdE.run, FeN.run, Fei.run, Fel.run,
TA6.run and TAw.run. Their presence alone does not classify them as logout.
G0j.A02's selected body is account-deletion reset, including13N.A0H and database/
preference cleanup; it is not proof of normal logout or account switching.
13N.A09 only clears phone-entry/verification activity preferences, not Me or
own devices. 13N.A0H clears registration JID/LID and country/phone; its callers
are retained, with full ordering and account-generation scope still unresolved.

## Exact next gates before automatic read-self ownership

- Close actual normal logout/account-switch callers and ordering against the
  F1G.A02 cancellation sites,0Vy.Eo9 clear, own-device cache/DB, Signal pin and Me
  selection. Prove stale callbacks/workers cannot adopt into a newly selected
  account, including restart and a response already posted before disconnect.
- Close cached-server-key provenance and generation on1o3's resume branches;
  preserve the existing checked certificate/AEAD/login sequence.
- Reuse the registration HTTP audit to bind the primary verified-response path
  to its saved attempt and TLS transport; close alternate primary preference
  origins only if they are needed for the supported route.
- Only then model authenticated own-account/device inventory and eligibility.
  Contact DeviceList, loaded Me, login-success, server device advertisement and
  completed native primary pairing remain distinct authorities. Existing
  negative automatic normal-read privacy guard is not read-self completion.

All findings are static. No account restriction cause or guarantee is inferred.
