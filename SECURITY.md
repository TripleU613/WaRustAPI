# Security

Security fixes currently target the main development branch.

## Reporting a vulnerability

Use the repository's private
[Report a vulnerability](https://github.com/TripleU613/WaRustAPI/security/advisories/new)
form if private reporting is enabled. If the form is unavailable, open a
minimal issue asking a maintainer for a private contact method.

Include the affected commit, feature set, platform, impact, and a sanitized
reproduction.

## Dependency advisories

CI runs `cargo audit` against `Cargo.lock` on every dependency change and on a
schedule, and Dependabot proposes updates.
