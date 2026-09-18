# Continuous integration

[Contributor guide](../CONTRIBUTING.md) · [Workflow](../.github/workflows/ci.yml)

CI selects work from changed paths instead of building the whole workspace on
every push. It runs for main-branch pushes, pull requests, merge queues, and
manual requests. Feature-branch pushes without a PR do not run the workflow.
Draft PRs receive repository hygiene checks; marking a PR ready runs validation.
Superseded runs for the same event/ref are cancelled.

## Change routing

| Change | Selected work |
| --- | --- |
| Root/docs Markdown, research snapshots, issue templates, editor/Git settings | Repository hygiene only; no Cargo build or dependency fetch |
| CLI Rust | CLI lint/tests |
| xtask Rust | xtask lint/tests; its audit/protobuf modules also trigger their corresponding checks |
| Library Rust or embedded fixtures | Workspace lint, default/all-feature tests, package and strict rustdoc |
| Library tests only | Workspace lint and tests; no package rebuild |
| Protobuf inputs or generated protobuf | Workspace checks, regeneration check, package |
| Cargo manifests, lockfile, toolchain, workflows, or CI tools | Full normal validation |
| Unknown paths or unavailable Git baseline | Full normal validation, never a silent skip |

The planner reads a local, NUL-delimited Git diff with rename detection off, so
deletions and both sides of renames affect selection. Full history is fetched
only for the planner. It compares the PR base with the checked-out merge commit,
or the push's previous commit with HEAD. The planner is covered by unit tests.

The fast jobs compile tiny dependency-free Rust utilities with the runner's
preinstalled stable compiler. They do not compile workspace crates, install the
pinned product toolchain, download Cargo dependencies, or need the APK.

## Checks

- Repository hygiene: tracked-file size cap (2 MiB), excluded raw/private file
  types, inline local Markdown link targets, matching package license copies,
  and exact APK research checksums/file list/2 MiB aggregate budget.
- Workflow edits: actionlint validates syntax, expressions, and shell snippets.
- Rust: formatting, Clippy with warnings denied, unit/integration/doc tests,
  existing codec property tests and crash-recovery tests. API changes test both
  default and combined features. CLI/xtask-only changes use scoped Cargo args.
- Build contract: dependency report verification only when graph/auditor
  inputs change; protobuf regeneration only when schema/generator inputs change.
- Packaging: standalone package verification and rustdoc with warnings denied.

Heavy test and package jobs wait for lint to succeed. Every job has a timeout.
Caches are scoped by job/feature and lockfile/toolchain; regular PR jobs restore
caches but only main writes them. Actions are pinned to full commits and the
token is read-only. Checkouts do not retain credentials. No workflow runs APK
downloads, live WhatsApp registration, OTP requests, or account mutations.

`CI required` always reports a final result. A selected job that fails, is
cancelled, or unexpectedly skips fails that result. Intentionally unselected
jobs may skip. This is the single status to select in branch protection;
the workflow does not change repository protection settings itself.

## Extended checks

Monday's scheduled run, or a manual run with `full: true`, additionally runs:

- each optional feature independently;
- release-mode integration/property/crash tests with 4,096 property cases;
- workspace coverage using cargo-llvm-cov;
- the full test suite on current stable Rust, in addition to pinned 1.94.1.

LCOV reports are retained as workflow artifacts for 14 days. Coverage is
reported without an invented percentage threshold. The extended job is part
of the final status when requested. These expensive checks do not run for
ordinary pushes or PR updates.

## Dependency advisories

[Dependency security](../.github/workflows/security.yml) uses cargo-audit
against RustSec on dependency changes, weekly, or manually. It does not build
the application or create issues. Its status is separate from `CI required`;
do not make a path-filtered status universally required for docs-only PRs.
The project's native-dependency audit and RustSec check answer different questions.

The first [live RustSec run](https://github.com/TripleU613/TeleWhatsApp/actions/runs/35302844049)
reports existing advisories for `rsa` and the experimental HTTPS provider's
older `rustls-webpki` line, plus an unmaintained `paste` warning. No advisories
are suppressed by this CI change. A red advisory scan requires dependency
remediation; offline tests do not establish that the experimental provider is
suitable for production.

## Local checks

The initial [normal-validation run](https://github.com/TripleU613/TeleWhatsApp/actions/runs/35302927716)
passed routing tests, hygiene, lint, both test configurations, generation and
dependency-contract checks, packaging, strict rustdoc, and the aggregate gate.
Extended scheduled/manual checks were intentionally not selected in that run.

```sh
rustc --edition=2024 --test .github/ci/plan.rs -o /tmp/telewhatsapp-ci-tests
/tmp/telewhatsapp-ci-tests
rustc --edition=2024 .github/ci/hygiene.rs -o /tmp/telewhatsapp-ci-hygiene
/tmp/telewhatsapp-ci-hygiene
rustfmt --edition 2024 --check .github/ci/plan.rs .github/ci/hygiene.rs
actionlint
```

Hygiene uses Git, jq, and sha256sum supplied by the Ubuntu runner. Its Markdown
check covers inline local file targets, not external URL availability or anchors.
The private/raw file check is a filename policy, not a general secret scanner.
Stage new files before using the local hygiene command, since it checks Git's
tracked file list. See the contributor guide for Cargo checks.
