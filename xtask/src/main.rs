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
mod proto;
mod registration_token;
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
        Cmd::ApkResearchBundle { apk, output, check } => bundle::run(&apk, &output, check),
        Cmd::ProtoGen => proto::generate(&root),
        Cmd::ProtoCheck => proto::check(&root),
        Cmd::DependencyAudit { strict } => deps::audit(&root, strict),
        Cmd::ReferenceManifest { apk, check } => manifest::run(&apk, check.as_deref()),
        Cmd::ApkStrings { apk, contains } => dex::run(&apk, &contains),
        Cmd::ApkTokens { apk, check } => dex::run_tokens(&apk, check.as_deref()),
        Cmd::ApkXref { apk, anchors } => xref::run_xref(&apk, &anchors),
        Cmd::ApkCallers {
            apk,
            class_name,
            method_name,
        } => xref::run_callers(&apk, &class_name, &method_name),
        Cmd::ApkSmali {
            apk,
            class_filter,
            method_filter,
        } => xref::run_smali(&apk, &class_filter, method_filter.as_deref()),
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
