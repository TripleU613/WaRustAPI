//! Repository automation in Rust only (README sections 2.2, 2.4, 6.1).
//!
//! Commands:
//! * `proto-gen`   — regenerate committed protobuf Rust code with protox + prost-build.
//! * `proto-check` — regenerate into a temporary directory and fail on any diff.
//! * `dependency-audit` — inspect the resolved graph and write a report.
//! * `reference-manifest <apk>` — extract pinned-reference facts from an APK.

mod axml;
mod deps;
mod manifest;
mod proto;

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
        Cmd::ProtoGen => proto::generate(&root),
        Cmd::ProtoCheck => proto::check(&root),
        Cmd::DependencyAudit { strict } => deps::audit(&root, strict),
        Cmd::ReferenceManifest { apk, check } => manifest::run(&apk, check.as_deref()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
