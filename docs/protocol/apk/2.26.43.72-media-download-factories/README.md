# Download factory selection — APK 2.26.43.72

**Confirmed selected static / research only**, build264307238, APK SHA256
`887a6f94f6614791cfa845fe0b9990c064326d8b89cd71d81e817fc9fd530c2e`.
Complete selected DEX exports via xtask fallback; no runtime observation, new REA
Evidence IDs, Rust runtime change or account traffic. Byte PCs are method-relative;
branch displacements are16-bit code units. The factory findings refine the
[range/result consumer boundary](../2.26.43.72-media-download-range-consumers/README.md).

## Exact constructor census and enum selection

Direct DEX constructor censuses find FOUR MVv sites ([census](mvv-constructor-callers.txt)):
LHb.A02, MtW.Cb6, MtX.Cb6, MtY.Cb6. Both [MuY](muy-constructor-callers.txt) and
[MuZ](muz-constructor-callers.txt) have a single direct constructor site in MF7.A00.
Counts are invocation sites, not all runtime instantiations or reflection/JNI proof.

[MF7.A00](stream-factory.txt) compares its LPG argument by object identity with
LPG.A02 at0048. Equality selects MuZ at004c..0056; every other value goes to MuY
at0058..0062. The [enum initializer](stream-mode-enum.txt) explicitly maps A01 to
PULL(ordinal0) and A02 to PUSH(ordinal1). Thus PUSH-to-MuZ is confirmed, rather
than guessed from log text. A null argument would also take the MuY branch at
this leaf; the factory does not itself prove nonnull input on every caller path.

[MF7.A01](stream-mode-selector.txt) is a separate mode selector: MX3.A03==2 selects
category `video`; ==3 selects a photo category (`photo_hd`, potentially `photo_sd`
based on positive dimensions, max(width,height), a config threshold and A0n).
Other A03 values return PULL. PUSH requires that the selected category belongs to
a configured comma-separated trimmed/normalized set AND another feature flag is
true. Otherwise it returns PULL. Runtime configuration values and Fws.A12's exact
normalization are not established here. Neither media kind alone nor a hardcoded
mode string selects PUSH unconditionally; no Rust mode header is invented.

## Stream eligibility and sidecar acquisition are distinct

[MX3.A07](stream-eligibility.txt) returns true exactly when A03 is2 or3. It does
not check sidecar presence, URL, account state, length, key or network eligibility.
The category labels above tie these values to this selector's video/photo branches;
full protobuf-to-MX3 construction remains untraced.

[MX3.A05](sidecar-projection.txt) calls an optional A0t Callable to obtain LvR.
Absent Callable returns null; an Exception from the selected call/cast is logged
and returns null. This is not a blanket exception-to-success rule. Raw byte-field
presence alone cannot establish that the callable would yield usable range state.

The selected LHc path in [MtX.Cb6](transfer-mtx.txt) checks A07 at0efe and MWP.A0b
at0f0a. False A07 or true A0b reaches the normal MVv construction path (the latter
passes through additional metrics first). Otherwise it obtains the sidecar via
A05 at0f16. Missing LvR/A01 bytes goes to a failure result30 at0f50..0f74, NOT
an automatic full-download fallback. For supported message predicates there are
additional type/index checks at0f30..0f50. File prerequisites, reuse of an existing
Nbk through AIj, and cached/selected mode are separate at0fc8..1016. MF7.A00 is
called at108c, while the normal branch constructs MVv at12e0.

This is a selected branch of a synthetic discriminator class: MtX.Cb6 starts with
`$t` switch and also contains other families. Do not treat every string or path in
that class as an unconditional chat download rule. MWP.A0b's meaning, sidecar
producer construction and message predicate internals remain unknown here.

## Context-specific normal versus stream producers

[MtW.Cb6](transfer-mtw.txt), marked FileMediaDownload in its own logs, constructs
MVv at044e and calls A04 at0470/0494. It does not call MF7.A00 in this selected
method. This bounds a normal file-transfer path; it is not proof that every file
or image ignores streaming metadata.

[LHb.A02](normal-factory.txt) contains status and channel-thumbnail handling as
well as encrypted-media paths. It can reuse Nbk.AIj or create a stream handler
through MF7.A00 at054c; it separately constructs MVv at08da and invokes A04.
[MtY.Cb6](transfer-mty.txt) similarly contains newsletter and chat paths, with
MF7.A00 at0dc6 and MVv construction at107a. These complete exports preserve
branch/context boundaries; their entire scheduling/reupload/output flows have
NOT been classified as equivalent to Rust's explicit encrypted full-file GET.

## Practical conclusion and remaining gap

**Confirmed:** stream/native full-file selection is driven by input category,
sidecar Callable result, operation predicates, existing handler reuse, runtime
mode/config and context. MuY/MuZ and MVv share0lO HTTP response acceptance but
have different upstream state. Absence of sidecar is not universally evidence
that native falls back to a normal GET. No arbitrary mandatory sidecar field,
PUSH mode, telemetry, automatic range request or retry is added to Rust.

Rust currently selects its own explicit full-ciphertext receiver, requires hashes
and decrypts/authenticates the complete body. Native factory findings do not yet
prove identical native image/view-once operation selection or allow206/416 to be
adopted merely by status. The native shared leaf remains200/206-capable; Rust's
200-only policy remains a known implementation difference pending supported-input
binding and complete response/range/recovery validation.

**Still unknown:** descriptor/protobuf-to-MX3 and A0t producer, actual MWP.A0b
predicate, runtime flags/config, native initial file state/start/end for the exact
Rust profile, operation-specific Content-Range interpretation, identity/account
cancellation, final consumer ownership and live server interoperability. This
research neither confirms server acceptance nor identifies a ban trigger.

Reproduce constructor censuses with `apk-callers <apk> --class '<descriptor>'
--method '<init>'`. Export `apk-smali <apk> --class '<descriptor>' --method <name>
--signatures --exceptions` for MF7.A00/A01, LPG.<clinit>, LHb.A02, MtW/MtX/MtY.Cb6,
MX3.A07/A05. [checksums.json](checksums.json) pins every retained file including
this report. No unpinned APK or unrelated upstream source was substituted.
