//! Dependency-free CI routing; compiled with rustc, never Cargo.
use std::{env, fs, io::Write, process::Command};

#[derive(Debug, Default, PartialEq, Eq)]
struct Plan {
    api: bool,
    cli: bool,
    xtask: bool,
    dependencies: bool,
    proto: bool,
    package: bool,
    workflow: bool,
}

impl Plan {
    fn full() -> Self {
        Self {
            api: true,
            cli: true,
            xtask: true,
            dependencies: true,
            proto: true,
            package: true,
            workflow: true,
        }
    }

    fn changed(&mut self, path: &str) {
        if path.starts_with(".github/workflows/") || path.starts_with(".github/ci/") {
            *self = Self::full();
        } else if path == "Cargo.lock"
            || path.ends_with("Cargo.toml")
            || path == "rust-toolchain.toml"
            || path.starts_with(".cargo/")
        {
            *self = Self::full();
        } else if path.starts_with("proto/") || path.starts_with("crates/whatsapp-api/src/proto/") {
            self.api = true;
            self.proto = true;
            self.package = true;
        } else if path.starts_with("crates/whatsapp-api/src/")
            || path.starts_with("crates/whatsapp-api/fixtures/")
            || path.starts_with("crates/whatsapp-api/tests/")
        {
            self.api = true;
            self.package = !path.starts_with("crates/whatsapp-api/tests/") || self.package;
        } else if path.starts_with("crates/whatsapp-cli/src/")
            || path.starts_with("crates/whatsapp-cli/tests/")
        {
            self.cli = true;
        } else if path.starts_with("xtask/") && path.ends_with(".rs") {
            self.xtask = true;
            self.dependencies |= path == "xtask/src/deps.rs";
            self.proto |= path == "xtask/src/proto.rs";
        } else if path == "docs/protocol/dependency-report.md" {
            self.dependencies = true;
        } else if path == "rustfmt.toml" || path == ".rustfmt.toml" {
            self.api = true;
        } else if path.ends_with(".md")
            || path.starts_with("docs/")
            || path.starts_with(".github/ISSUE_TEMPLATE/")
            || matches!(
                path,
                ".gitignore" | ".gitattributes" | ".editorconfig" | ".github/dependabot.yml"
            )
            || path
                .split('/')
                .next_back()
                .is_some_and(|name| name.starts_with("LICENSE"))
        {
            // Known non-build inputs: always checked by repository hygiene.
        } else {
            // Unknown inputs fail conservatively toward full validation.
            *self = Self::full();
        }
    }

    fn rust(&self) -> bool {
        self.api || self.cli || self.xtask
    }

    fn args(&self) -> &'static str {
        if self.api {
            "--workspace"
        } else if self.cli && self.xtask {
            "-p whatsapp-cli -p xtask"
        } else if self.cli {
            "-p whatsapp-cli"
        } else {
            "-p xtask"
        }
    }

    fn matrix(&self) -> String {
        if self.api {
            r#"{"include":[{"profile":"default","args":"--workspace"},{"profile":"all-features","args":"--workspace --all-features"}]}"#.into()
        } else {
            format!(
                r#"{{"include":[{{"profile":"affected-crates","args":"{}"}}]}}"#,
                self.args()
            )
        }
    }
}

fn git(args: &[&str]) -> Result<Vec<u8>, String> {
    let out = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).into());
    }
    Ok(out.stdout)
}

fn changes(base: &str, head: &str) -> Result<Vec<String>, String> {
    if ![base, head]
        .iter()
        .all(|sha| sha.len() == 40 && sha.bytes().all(|c| c.is_ascii_hexdigit()))
    {
        return Err("base and head must be full commit hashes".into());
    }
    let bytes = git(&[
        "diff",
        "--name-only",
        "--no-renames",
        "-z",
        base,
        head,
        "--",
    ])?;
    bytes
        .split(|b| *b == 0)
        .filter(|p| !p.is_empty())
        .map(|p| String::from_utf8(p.to_vec()).map_err(|e| e.to_string()))
        .collect()
}

fn plan() -> Result<(), String> {
    let mut plan = Plan::default();
    let mut reason = "Changed paths";
    let event = env::var("CI_EVENT").unwrap_or_default();
    if event == "pull_request" && env::var("CI_DRAFT").as_deref() == Ok("true") {
        reason = "Draft pull request: hygiene only; ready_for_review runs validation";
    } else if env::var("CI_FULL").as_deref() == Ok("true") || event == "schedule" {
        plan = Plan::full();
        reason = "Explicit full validation";
    } else {
        let head = String::from_utf8(git(&["rev-parse", "HEAD"])?).map_err(|e| e.to_string())?;
        let base = env::var("CI_BASE").unwrap_or_default();
        match changes(&base, head.trim()) {
            Ok(paths) => {
                for path in paths {
                    plan.changed(&path);
                }
            }
            Err(error) => {
                eprintln!("Cannot determine changes: {error}; selecting full validation");
                plan = Plan::full();
                reason = "Unavailable baseline: full validation";
            }
        }
    }
    let outputs = format!(
        "rust={}\napi={}\ndependencies={}\nproto={}\npackage={}\nworkflow={}\nargs={}\nmatrix={}\n",
        plan.rust(),
        plan.api,
        plan.dependencies,
        plan.proto,
        plan.package,
        plan.workflow,
        plan.args(),
        plan.matrix()
    );
    print!("{outputs}");
    if let Ok(path) = env::var("GITHUB_OUTPUT") {
        fs::OpenOptions::new()
            .append(true)
            .open(path)
            .map_err(|e| e.to_string())?
            .write_all(outputs.as_bytes())
            .map_err(|e| e.to_string())?;
    }
    if let Ok(path) = env::var("GITHUB_STEP_SUMMARY") {
        let summary = format!(
            "## CI selection\n\n{reason}. Documentation and research-only changes skip Cargo compilation.\n\n```text\n{outputs}```\n"
        );
        fs::OpenOptions::new()
            .append(true)
            .open(path)
            .map_err(|e| e.to_string())?
            .write_all(summary.as_bytes())
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn acceptable(result: &str, selected: bool) -> bool {
    result == "success" || (!selected && result == "skipped")
}

fn gate() -> Result<(), String> {
    for job in [
        "PLAN", "HYGIENE", "LINT", "TEST", "AUDIT", "PACKAGE", "EXTENDED",
    ] {
        let selected = matches!(job, "PLAN" | "HYGIENE")
            || env::var(format!("NEED_{job}")).as_deref() == Ok("true");
        let result = env::var(format!("RESULT_{job}")).unwrap_or_default();
        if !acceptable(&result, selected) {
            return Err(format!(
                "{job}: expected selected={selected}, got {result:?}"
            ));
        }
    }
    println!("All selected checks passed; unselected jobs were deliberately skipped.");
    Ok(())
}

fn main() {
    let result = if env::args().nth(1).as_deref() == Some("gate") {
        gate()
    } else {
        plan()
    };
    if let Err(e) = result {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn route(paths: &[&str]) -> Plan {
        let mut p = Plan::default();
        for path in paths {
            p.changed(path);
        }
        p
    }
    #[test]
    fn docs_and_apk_snapshot_skip_builds() {
        assert_eq!(
            route(&[
                "README.md",
                "docs/protocol/apk/2.26.37.70/tokens.json",
                "docs/design/registration.md"
            ]),
            Plan::default()
        );
    }
    #[test]
    fn cli_is_scoped() {
        let p = route(&["crates/whatsapp-cli/src/http.rs"]);
        assert_eq!(p.args(), "-p whatsapp-cli");
        assert!(!p.package && !p.dependencies && !p.proto);
    }
    #[test]
    fn xtask_is_scoped() {
        let p = route(&["xtask/src/bundle.rs"]);
        assert_eq!(p.args(), "-p xtask");
        assert!(!p.api && !p.package);
    }
    #[test]
    fn api_runs_consumers_and_features() {
        let p = route(&["crates/whatsapp-api/src/client.rs"]);
        assert!(p.package);
        assert_eq!(p.args(), "--workspace");
        assert!(p.matrix().contains("all-features"));
    }
    #[test]
    fn fixtures_are_build_inputs() {
        assert!(route(&["crates/whatsapp-api/fixtures/reference-manifest.json"]).api);
    }
    #[test]
    fn schema_triggers_regeneration() {
        let p = route(&["proto/whatsmeow/waCert/WACert.proto"]);
        assert!(p.api && p.proto && p.package);
    }
    #[test]
    fn generated_schema_triggers_regeneration() {
        assert!(route(&["crates/whatsapp-api/src/proto/wa_cert.rs"]).proto);
    }
    #[test]
    fn dependency_inputs_get_full_checks() {
        for path in ["Cargo.lock", "xtask/Cargo.toml", "rust-toolchain.toml"] {
            assert_eq!(route(&[path]), Plan::full());
        }
    }
    #[test]
    fn workflow_changes_get_full_checks() {
        assert_eq!(route(&[".github/workflows/ci.yml"]), Plan::full());
    }
    #[test]
    fn unknown_inputs_are_not_silently_skipped() {
        assert_eq!(route(&["build.sh"]), Plan::full());
    }
    #[test]
    fn mixed_changes_keep_strongest_checks() {
        let p = route(&[
            "xtask/src/proto.rs",
            "README.md",
            "crates/whatsapp-cli/src/main.rs",
        ]);
        assert!(p.proto);
        assert_eq!(p.args(), "-p whatsapp-cli -p xtask");
    }
    #[test]
    fn test_only_changes_do_not_package() {
        let p = route(&["crates/whatsapp-api/tests/crash_matrix.rs"]);
        assert!(p.api && !p.package);
    }
    #[test]
    fn gate_rejects_failure_cancellation_and_unexpected_skip() {
        for state in ["failure", "cancelled", "", "timed_out"] {
            assert!(!acceptable(state, true));
            assert!(!acceptable(state, false));
        }
        assert!(!acceptable("skipped", true));
        assert!(acceptable("skipped", false));
        assert!(acceptable("success", true));
    }
    #[test]
    fn invalid_shas_cannot_be_git_options() {
        assert!(changes("--help", "HEAD").is_err());
    }
}
