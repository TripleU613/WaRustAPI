# Current MEX request envelope comparison

Confirmed static APK2.26.43.72 code264307238 SHA256 `887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`. Supported xtask DEX fallback; previously diagnosed Android provider remains unavailable. No execution/live traffic or REA Evidence envelope claim.

Reproduce from repo root:

```sh
APK=../android-lab/apks/WhatsApp.1.apk
target/debug/xtask apk-smali "$APK" --class 'LX/0mI;' --method '<init>' --signatures --exceptions
target/debug/xtask apk-smali "$APK" --class 'LX/0Tk;' --method A06 --signatures --exceptions
```

Reuse complete0mF/0mK constructors in [server-JID evidence](../2.26.43.72-mex-server-jid-equality/README.md).0mF serializes queryId and variables, then supplies its server query identifier to0mI and0mK.0mI emits trace/flow_id.0mK emits iq to server singleton, xmlns=w:mex,type=get,id, then trace/query; query_id receives the server identifier. These selected fields match Rust group_info_request's existing37.71-derived envelope. No current group-specific server query ID, variables schema or live request parity is established by matching shared wrapper construction.

0mI supplies length bounds1..36 to0Tk.A06;0mK supplies query_id1..20 and body1..0x80e800. A06 uses Java String.length, and calls00L.A0C(false,diagnostic) for violated bounds. Its subsequent return for a non-null string remains true; null/nonoptional has a distinct false return after the assertion call. Effective assertion/exception behavior is not reconstructed. Do not describe all violations as normal false-return omission or copy it as a safe input validator. Rust decimal/length/control validation remains intentionally stricter local policy. Local JSON response size limits are a separate inbound surface.

No confirmed construction correction was justified in Rust by this comparison. Current query identifier resolution, group-specific schema/configuration, exact UTF-16 boundary/assert behavior, transport scheduling and response adoption remain incomplete. checksums.json binds every other file. No runtime request/retry or account operation.
