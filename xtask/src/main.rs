//! Repository automation in Rust only (docs/design/README.md sections 2.2, 2.4, 6.1).
//!
//! Commands:
//! * `proto-gen`   — regenerate committed protobuf Rust code with protox + prost-build.
//! * `proto-check` — regenerate into a temporary directory and fail on any diff.
//! * `dependency-audit` — inspect the resolved graph and write a report.
//! * `reference-manifest <apk>` — extract pinned-reference facts from an APK.
//! * `apk-strings <apk>` — dump all dex string constants (with optional filter).
//! * `apk-tokens <apk>` — recover the binary-node token dictionary from bytecode.
//! * `apk-xref <apk> --anchor <s>…` — find classes/methods referencing string constants.
//! * `apk-callers <apk> --class <c> --method <m>` — find callers of a method.
//! * `apk-field-refs <apk> --class <c> --field <f> --type <t>` — exact field access sites.
//! * `apk-smali <apk> --class <filter>` — dump Dalvik instructions for matching classes.
//! * `apk-registration-token <apk> …` — derive the comparison build's code-request token.
//! * `apk-token-material --from-dir <dir> --logo-from <apk> --out <zip> [--verify-config <json>]` — assemble token material archive.
//! * `apk-research-bundle <apk> --output <dir> [--check]` — compact media-free code snapshot.

// A command-line tool talks to its operator through stdout/stderr.
#![allow(clippy::print_stdout, clippy::print_stderr)]

mod axml;
mod bundle;
mod deps;
mod dex;
mod manifest;
mod mex;
mod proto;
mod registration_token;
mod token_arrays;
mod token_material_archive;
mod xref;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "xtask", about = "whatsapp-api repository tasks")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Export a bounded, media-free research snapshot of the pinned comparison APK.
    ApkResearchBundle {
        apk: PathBuf,
        /// New directory for the bundle, or existing directory with --check.
        #[arg(long)]
        output: PathBuf,
        /// Recompute and compare without changing any files.
        #[arg(long)]
        check: bool,
        /// Hash-pinned JSON topic/class/method/Rust-file mapping for another release.
        #[arg(long)]
        recipe: Option<PathBuf>,
    },
    /// Compare checksum-verified cached topic files without rescanning either APK.
    ApkResearchDiff { old: PathBuf, new: PathBuf },
    /// Locate renamed topic methods by checked exact-string anchor intersection.
    ApkResearchRemap {
        apk: PathBuf,
        #[arg(long)]
        recipe: PathBuf,
        /// Write a proposed hash-pinned recipe only when all topics are unique.
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Regenerate protobuf Rust code into crates/whatsapp-api/src/proto.
    ProtoGen,
    /// Verify committed protobuf Rust code is up to date.
    ProtoCheck,
    /// Audit the resolved dependency graph and write docs/protocol/dependency-report.md.
    DependencyAudit {
        /// Fail if a denied crate is present in the release graph.
        #[arg(long, default_value_t = true)]
        strict: bool,
    },
    /// Dump the dex string tables of an APK (static protocol reconnaissance).
    ApkStrings {
        /// Path to the APK (kept outside the repository).
        apk: PathBuf,
        /// Only print strings containing any of these substrings.
        #[arg(long = "contains")]
        contains: Vec<String>,
    },
    /// Recover the binary-node token dictionary from an APK's dex bytecode.
    ApkTokens {
        /// Path to the APK (kept outside the repository).
        apk: PathBuf,
        /// Compare against a committed token fixture and fail on mismatch.
        #[arg(long)]
        check: Option<PathBuf>,
    },
    /// Export selected persisted MEX query IDs as a bounded release-diffable JSON file.
    ApkMexQueries {
        apk: PathBuf,
        /// Exact operation names; no all-operation/bulk export.
        #[arg(long = "operation", required = true)]
        operations: Vec<String>,
        /// Build configuration namespace used by the APK request.
        #[arg(long, default_value = "whatsapp-android-mex")]
        namespace: String,
        /// Compare the complete export, including provenance, without writing.
        #[arg(long)]
        check: Option<PathBuf>,
    },
    /// Inspect the Rust group MEX node offline; does not open an account or queue an action.
    MexGroupPreview {
        #[arg(long)]
        queries: PathBuf,
        /// Synthetic group JID recommended: output contains the supplied value.
        #[arg(long)]
        chat: String,
        /// Explicit IQ ID; this command never creates a live reservation.
        #[arg(long)]
        id: String,
        /// Exact APK query context (for example INTERACTIVE or NEW_SUBJECT).
        #[arg(long, default_value = "INTERACTIVE")]
        context: String,
    },
    /// Inspect one MEX role string offline using the actual Rust converter.
    MexRoleInspect {
        /// Server enum string; omission exercises the missing-value fallback.
        #[arg(long)]
        value: Option<String>,
    },
    /// Validate a strict APK member-primary address offline, without echoing it.
    MexParticipantAddressInspect {
        /// Private JID; output contains only the recognized subtype.
        #[arg(long, allow_hyphen_values = true)]
        value: String,
    },
    /// Hash an explicit private address collection offline using APK-derived rules.
    GroupParticipantHashInspect {
        /// Private JSON array of address strings; local limit 64 KiB/1024 entries.
        #[arg(long)]
        input: PathBuf,
        /// Explicit hash version; this does not select an account feature.
        #[arg(long, value_parser = ["1", "2", "3"])]
        version: String,
        /// Prepare a set before hashing; otherwise preserve repeated entries.
        #[arg(long)]
        as_set: bool,
    },
    /// Resolve APK member-storage metadata offline from private JSON; prints explicit values.
    MexParticipantStorageInspect {
        /// Typed raw join/history, previous state and required clock_ms; local limit 16 KiB.
        #[arg(long)]
        input: PathBuf,
    },
    /// Inspect MEX JSON presence offline; group-numbers explicitly reveals numeric metadata.
    MexResponseInspect {
        /// Private JSON payload file (local budget 1 MiB).
        #[arg(long)]
        input: PathBuf,
        /// Payload child supplied to the actual Rust node parser.
        #[arg(long, default_value = "result", value_parser = ["result", "update"])]
        child: String,
        /// Include the APK QueryGroupInfo callback gate; no group conversion or state update.
        #[arg(long)]
        group_info: bool,
        /// Explicitly print private group numeric metadata as decimal strings; requires --group-info.
        #[arg(long, requires = "group_info")]
        group_numbers: bool,
        /// Print recognized roles of object edges, before member/JID validation; requires --group-info.
        #[arg(long, requires = "group_info")]
        edge_roles: bool,
        /// Explicitly print private phash/history/join scalars; requires --group-info.
        #[arg(long, requires = "group_info")]
        participant_scalars: bool,
        /// Explicitly print private converted members; requires --group-info.
        #[arg(long, requires = "group_info")]
        participant_members: bool,
    },
    /// Find which classes and methods reference each anchor string constant.
    ///
    /// Output: for each anchor, the class descriptor, method name, proto shorty,
    /// and byte offset of the const-string instruction in the dex blob.
    /// Use this to select targets for `apk-smali` without a Java decompiler.
    ApkXref {
        /// Path to the APK (kept outside the repository).
        apk: PathBuf,
        /// String(s) to search for (exact match against const-string literals).
        #[arg(long = "anchor", required = true)]
        anchors: Vec<String>,
    },
    /// Find methods that invoke a target class/method pair.
    ApkCallers {
        /// Path to the APK (kept outside the repository).
        apk: PathBuf,
        /// Exact target class descriptor (for example `LX/Foo;`).
        #[arg(long = "class")]
        class_name: String,
        /// Exact target method name.
        #[arg(long = "method")]
        method_name: String,
    },
    /// Find exact field read/write instructions across the APK DEX files.
    ApkFieldRefs {
        /// Path to the APK (kept outside the repository).
        apk: PathBuf,
        /// Exact declaring class descriptor.
        #[arg(long = "class")]
        class_name: String,
        /// Exact field name.
        #[arg(long = "field")]
        field_name: String,
        /// Exact field type descriptor (for example I or Ljava/lang/String;).
        #[arg(long = "type")]
        field_type: String,
        /// Include writes only (iput/sput); otherwise include reads too.
        #[arg(long)]
        writes_only: bool,
    },
    /// Dump the Dalvik instruction stream for classes whose name contains `--class`.
    ///
    /// Resolves string, type, method, and field references symbolically.
    /// Combine with `apk-xref` output: run xref first to find the class name,
    /// then run smali to read its methods.
    ApkSmali {
        /// Path to the APK (kept outside the repository).
        apk: PathBuf,
        /// Substring to match against class descriptors (e.g. `RegistrationManager`).
        #[arg(long = "class")]
        class_filter: String,
        /// Optional additional substring filter on method names.
        #[arg(long = "method")]
        method_filter: Option<String>,
        /// Include checked exception ranges and handler types (addresses in code units).
        #[arg(long)]
        exceptions: bool,
    },
    /// Decode a protobuf-lite message descriptor's field numbers.
    ///
    /// Read `newMessageInfo`'s field-name array and encoded descriptor rather
    /// than assuming static field-number values were recovered. This pairs each
    /// name with its number. Reading a number any other way — from position in
    /// the name array, or from the public schema, or from memory — has produced
    /// wrong answers here before, and a wrong field number is self-consistent
    /// locally and unreadable by every real client.
    ApkProtoFields {
        /// Path to the APK (kept outside the repository).
        apk: PathBuf,
        /// Substring to match against class descriptors (e.g. `LX/BZc`).
        #[arg(long = "class")]
        class_filter: String,
    },
    /// Derive the comparison APK's `/v2/code` token for a phone number.
    ApkRegistrationToken {
        /// Path to the exact APK whose resources and signature are inputs.
        apk: PathBuf,
        /// Explicit E.164 country calling code (the token itself excludes it).
        #[arg(long)]
        country_code: u16,
        /// National significant number, digits only and without a trunk zero.
        #[arg(long)]
        national_number: String,
    },
    /// Assemble a token material archive from unpacked APK tree and logo source.
    ApkTokenMaterial {
        /// Directory containing unpacked APK: AndroidManifest.xml, classes.dex, META-INF, optional resources.arsc.
        #[arg(long)]
        from_dir: PathBuf,
        /// APK or ZIP to extract logo from.
        #[arg(long)]
        logo_from: PathBuf,
        /// Output ZIP file (mode 0600, refuses to overwrite).
        #[arg(long)]
        out: PathBuf,
        /// Optional request config JSON to verify derived token against.
        #[arg(long)]
        verify_config: Option<PathBuf>,
    },
    /// Extract reference facts (hashes, manifest fields, signing certificate) from an APK.
    ReferenceManifest {
        /// Path to the APK (kept outside the repository).
        apk: PathBuf,
        /// Compare against a committed manifest and fail on mismatch.
        #[arg(long)]
        check: Option<PathBuf>,
    },
}

fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .expect("xtask lives one level below the workspace root")
        .to_path_buf()
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let root = workspace_root();
    let result = match cli.cmd {
        Cmd::ApkResearchBundle {
            apk,
            output,
            check,
            recipe,
        } => bundle::run(&apk, &output, check, recipe.as_deref()),
        Cmd::ApkResearchDiff { old, new } => bundle::compare(&old, &new),
        Cmd::ApkResearchRemap {
            apk,
            recipe,
            output,
        } => bundle::remap(&apk, &recipe, output.as_deref()),
        Cmd::ProtoGen => proto::generate(&root),
        Cmd::ProtoCheck => proto::check(&root),
        Cmd::DependencyAudit { strict } => deps::audit(&root, strict),
        Cmd::ReferenceManifest { apk, check } => manifest::run(&apk, check.as_deref()),
        Cmd::ApkStrings { apk, contains } => dex::run(&apk, &contains),
        Cmd::ApkTokens { apk, check } => dex::run_tokens(&apk, check.as_deref()),
        Cmd::ApkMexQueries {
            apk,
            operations,
            namespace,
            check,
        } => mex::extract(&apk, &namespace, &operations, check.as_deref()),
        Cmd::MexGroupPreview {
            queries,
            chat,
            id,
            context,
        } => mex::preview(&queries, &chat, &id, &context),
        Cmd::MexRoleInspect { value } => mex::inspect_role(value.as_deref()),
        Cmd::MexParticipantAddressInspect { value } => mex::inspect_participant_address(&value),
        Cmd::GroupParticipantHashInspect {
            input,
            version,
            as_set,
        } => mex::inspect_participant_hash(&input, &version, as_set),
        Cmd::MexParticipantStorageInspect { input } => mex::inspect_participant_storage(&input),
        Cmd::MexResponseInspect {
            input,
            child,
            group_info,
            group_numbers,
            edge_roles,
            participant_scalars,
            participant_members,
        } => mex::inspect_response(
            &input,
            &child,
            group_info,
            group_numbers,
            edge_roles,
            participant_scalars,
            participant_members,
        ),
        Cmd::ApkXref { apk, anchors } => xref::run_xref(&apk, &anchors),
        Cmd::ApkCallers {
            apk,
            class_name,
            method_name,
        } => xref::run_callers(&apk, &class_name, &method_name),
        Cmd::ApkFieldRefs {
            apk,
            class_name,
            field_name,
            field_type,
            writes_only,
        } => xref::run_field_refs(&apk, &class_name, &field_name, &field_type, writes_only),
        Cmd::ApkSmali {
            apk,
            class_filter,
            method_filter,
            exceptions,
        } => xref::run_smali(&apk, &class_filter, method_filter.as_deref(), exceptions),
        Cmd::ApkProtoFields { apk, class_filter } => xref::run_proto_fields(&apk, &class_filter),
        Cmd::ApkRegistrationToken {
            apk,
            country_code,
            national_number,
        } => registration_token::run(&apk, country_code, &national_number),
        Cmd::ApkTokenMaterial {
            from_dir,
            logo_from,
            out,
            verify_config,
        } => token_material_archive::run(&from_dir, &logo_from, &out, verify_config.as_deref()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod cli_tests {
    use super::*;

    #[test]
    fn participant_address_inspector_accepts_signed_private_values() {
        for value in ["-1-1@interop", "-0-1@interop", "--invalid-private-jid"] {
            let cli =
                Cli::try_parse_from(["xtask", "mex-participant-address-inspect", "--value", value])
                    .unwrap();
            assert!(
                matches!(cli.cmd, Cmd::MexParticipantAddressInspect { value: parsed } if parsed == value)
            );
        }
    }
}
