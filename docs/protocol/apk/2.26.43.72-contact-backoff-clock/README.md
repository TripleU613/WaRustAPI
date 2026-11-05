# Contact backoff clock and NTP scheduling, pinned build

**Confirmed static**, APK2.26.43.72 code264307238 SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
No target execution, NTP request or live account operation. Existing xtask DEX
exports are not REA Evidence envelopes. This reanchors the earlier
[37.71 clock/NTP analysis](../../token-clock-producers.md) and extends the
[43.72 contact gate trace](../2.26.43.72-contact-backoff-gates/README.md).
The newly traced scheduler closes an upstream unknown from that earlier audit;
Rust NTP acquisition and contact deadline enforcement remain unported.

## Distinct clock domains and restart behavior

08q.A00 samples elapsedRealtime and prefers nonzero A03(NTP), then A04(WhatsApp
server), adding the sampled elapsed value. With neither baseline it uses
wallTime-A02(persisted device-minus-WhatsApp-server difference). Contact
backoff observers and selected executor use this A00 domain. Token scheduling
uses08q.A04, which skips NTP. Do not substitute either getter for the other.

Constructor loads A02 from client_server_time_diff with default0. It rebuilds
NTP A03 only if client_ntp_time_diff exists, last_ntp_client_time>0 and
Math.abs(currentWall-lastClientTime)<86400000. This accepts either direction
of ordinary wall-clock skew within one day; equality is excluded. Arithmetic
is Java signed long, including subtraction/abs overflow, not arbitrary-precision
absolute age. A01 reconstructs A03=wall+ntpOffset-elapsed. Its sign is
NTP-minus-device, opposite A02's sign. Constructor does not reconstruct A04.

The selected A00/A02 getters have no age test; the one-day cache admission
rule does not expire an existing in-process NTP baseline. A02 getter returns
NTP-baseline+elapsed or0. A07 accepts positive supplied server milliseconds,
sets A04=server-elapsed and A02=device-server, persists only the latter, and
does not clear A03. Physical write census finds A03 only inA01, A04 only
inA07 and A02 in constructor/A07. Physical A01 callers are constructor and
EYi.A00. These are exact DEX inventories, not reflection/native closure.

Current Rust confirmed-clock state models the A04 domain with process-lifetime
baseline provenance and persisted WhatsApp offset. It must not be presented
as NTP-preferred A00 parity. The existing receipt-clock fallback also explicitly
marks NTP as unported. Historical message/receipt inference cannot fill A03.

## Newly traced scheduling: 12-hour period, independent six-hour guard

Eoc.A05's Djl branch reads effective Boolean property170(0xaa):

| Selection | Scheduling behavior |
| --- | --- |
|true |Cancels an existing alarm PendingIntent, creates periodic NtpSyncWorker work with interval43200000ms(12h), flex21600000ms(6h), tag tag.whatsapp.time.ntp and unique name name.whatsapp.time.ntp, passing02X.A01 policy discriminator toMDJ.A05.|
|false |Cancels work by both unique name and tag, prepares UPDATE_NTP PendingIntent requestcode16, and setsInexactRepeating alarmtype3, firstElapsed=now+43200000 and interval43200000. Missing manager/intent logs and skips that scheduling call.|

Km7.A01 writes the interval and clamps the flex with0HS.A05 between300000
and interval; the selected12h/6h inputs pass unchanged. MDJ.A05 preserves the
A01 policy discriminator intoMW2 and invokesA02. Its ultimate database
replacement/keep semantics remain untraced; no named policy is invented.
Android alarmtype3's platform delivery semantics are outside this selected
static proof; values do not promise exact delivery times.

Both setup branches reach Djl.A00(null,this) immediately after their selected
scheduling path. WorkManager setup also records /ntp/work_manager_init in
ntp-scheduler preferences; EbM.A00 increments /ntp event counters separately.
The selected AlarmService.A0B setup branch iterates its configured Eoc action
collection and invokesA05 on each. Complete DI collection membership and all
upstream setup triggers remain open; this is not a universal startup mandate.

Djl.A00 optionally takes a300000ms wake lock, invokes sharedEYi.A00 and releases
it through the normal/exception paths. Worker.A0D delegates to the sameEYi
routine. EYi's first gate skips acquisition when process-static successful-sync
elapsedA06>0 and elapsedNow-A06<21600000. Equality does not skip. That six-hour
recent-success gate is separate from the12-hour period/six-hour flex and the
one-day constructor cache check. A06's sole physical write is EYi's success
branch at053c..0540; it is not a persisted restart cooldown. Early recent-sync
return precedes that write.

## Acquisition/admission remains separate from cadence

The retained fullEYi body confirms configured android:string/config_ntpServer
with fallback2.android.pool.ntp.org, resolved-address iteration, port123 and
20000ms per-socket timeout. It sends a timestamped packet and requires the
response originate timestamp to match the sent value before computing offset.
Correlation is not cryptographic authentication. The selected receive path
alone does not establish packet length/source/stratum/leap/mode validation.

Adoption computes destination+offset and compares absolute differences to both
WhatsApp A04 and wall time. Only when both exceed86400000 and effective
property2225(0x8b1) is true does this selected guard reject the sample. At equality
or when either difference is within the limit this branch does not reject it.
Property values remain unknown. On adoptionEYi callsA01 and saves
client_ntp_time_diff plus currentWall last_ntp_client_time. It does not overwrite
A02/A04. Offset arithmetic, timestamp-era conversion and fallback branches
remain attributable to the full exports and prior analysis; this is not a new
Rust acquisition implementation or proof of live clock accuracy.

## Search boundaries and port requirements

No exact literal-name xrefs for the three tested worker-name strings and zero
direct constructor invokes were found. Those negative searches do not imply
no scheduled worker: Eoc.A05 usesconst-class and WorkManager. Exact work-name
and tag xrefs locate that method. Reuse symbolic code references rather than
inferring reachability from a literal-name search alone.

Before activating contact waits: select/bind the native factory and observer;
port the applicable A00 source/admission/lifecycle, effective configuration and
clock-change behavior; store deadlines with owned settlement atomically; recheck
before reservation; preserve pending/uncertain work and unrelated ACK/receive.
If porting NTP, keep cadence, flex, in-process success guard and constructor
cache admission separate. A saved offset, elapsed baseline or diagnostic wait
must not authorize replay, identity reset or fabricated telemetry. No timer,
request, retry/reset policy or live state changed in this research chunk.

## Reproduction and verification

Use existingxtask apk-smali APK --class CLASS --method METHOD --signatures
--exceptions for08q(fullclass),EYi.A00,NtpSyncWorker(fullclass),Djl(fullclass),
Eoc.A05,AlarmService.A0B,Km7.A01,0HS.A05,MDJ.A05 andEbM(fullclass).
Useapk-field-refs for08q.A02/A03/A04:J --writes-only andNtpSyncWorker.A06:J;
apk-callers for08q.A01,EYi.A00,Eoc.A05 andNtpSyncWorker.<init>; apk-xref for
name.whatsapp.time.ntp/tag.whatsapp.time.ntp and the explicitly listed negative
worker-name anchors. Quote descriptors/metacharacters in a shell.
Schedule excerpts identify full-export SHA256 and exact byte-PC ranges;
omitted branches are not inferred absent. Checksums cover all other files.
No raw APK, private state, NTP execution or bulk decompiled tree is committed.
