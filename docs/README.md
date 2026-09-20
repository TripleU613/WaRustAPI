# Documentation

Start with [Getting started](getting-started.md) for commands that work today
and [STATUS.md](../STATUS.md) for verified behavior and blockers.

## API reference

- [API reference](api/README.md)
- [OpenAPI document](api/openapi.json)

## Reference

- [CLI reference](cli.md): generated command tree
- [Configuration](configuration.md): full request.json fields, profiles, environment variables
- [Accounts and onboarding](accounts.md): multiple accounts, checking state, onboarding
- [Getting started](getting-started.md): basic commands and first steps

## Development and design

- [Contributing](../CONTRIBUTING.md): toolchain, checks, and review expectations.
- [Continuous integration](ci.md): change routing, test tiers, coverage, and security scans.
- [Design plan](design/README.md): architecture, contracts, milestones, and numbered sections.
- [Messaging implementation checklist](messaging-completion.md): verification status and blockers.
- [Repository map](protocol/repository-map.md): implementation locations.
- [Security](../SECURITY.md) and [third-party notices](../THIRD_PARTY_NOTICES.md).

## Protocol research

- [Evidence ledger](protocol/evidence-ledger.md)
- [Unknowns and blockers](protocol/unknowns.md)
- [Persisted registration lab](protocol/registration-lab.md)
- [Ban appeal](protocol/ban-appeal.md)
- [APK analysis procedure](protocol/apk-analysis.md)
- [Compact media-free APK code snapshot](protocol/apk/README.md)
- [Decompilation log](protocol/decompilation-log.md)
- [After-OTP flows](protocol/after-otp.md)
- [Chat actions](protocol/chat-actions.md)
- [Cross-device sync](protocol/cross-device-sync.md)
- [Groups](protocol/groups.md)
- [HTTP POST construction](protocol/http-post-analysis.md)
- [Messaging transport](protocol/messaging-transport.md)
- [Reusable component evaluation](protocol/reusable-components.md)
- [Generated dependency report](protocol/dependency-report.md)

## Architecture decisions

- [0001: Pure-Rust build contract](adr/0001-pure-rust-build-contract.md)
- [0002: Storage and encryption](adr/0002-storage-redb-and-at-rest-encryption.md)
- [0003: TLS provider gate](adr/0003-tls-provider-gate.md)
- [0004: Noise backend](adr/0004-noise-backend-snow-without-ring.md)
- [0005: Signal backend gate](adr/0005-signal-backend-gate.md)
- [0006: Protobuf generation](adr/0006-protobuf-generation.md)
