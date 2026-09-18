# Design plan

This is the implementation plan originally maintained in the root README.
Its numbered sections are preserved here. Requirements are distinct from
[implemented behavior and blockers](../../STATUS.md).

| Document | Original sections |
| --- | --- |
| [Scope and milestones](scope.md) | 0, 1, 4, 21, 22 |
| [Build and dependency policy](dependencies.md) | 2, 3, 18 |
| [Repository and runtime architecture](architecture.md) | 5, 8 |
| [Protocol evidence](evidence.md) | 6 |
| [Registration](registration.md) | 7 |
| [Domain model, encryption and public API](messaging.md) | 9, 10, 14, 15, 16, 17 |
| [Persistence and durable messaging](persistence.md) | 11, 12, 13 |
| [Verification and CI](verification.md) | 19 |
| [Operations and publication](operations.md) | 20 |

## Section index

- [0. The actual objective](scope.md#0-the-actual-objective)
- [1. Scope and release boundaries](scope.md#1-scope-and-release-boundaries)
- [2. Pure Rust means a verifiable build contract](dependencies.md#2-pure-rust-means-a-verifiable-build-contract)
- [3. Reuse protocol work without importing the wrong product](dependencies.md#3-reuse-protocol-work-without-importing-the-wrong-product)
- [4. Order of execution and acceptance gates](scope.md#4-order-of-execution-and-acceptance-gates)
- [5. Repository layout](architecture.md#5-repository-layout)
- [6. Evidence is part of the implementation](evidence.md#6-evidence-is-part-of-the-implementation)
- [7. Registration is its own durable state machine](registration.md#7-registration-is-its-own-durable-state-machine)
- [8. Runtime and protocol architecture](architecture.md#8-runtime-and-protocol-architecture)
- [9. Domain model, addresses and device identity](messaging.md#9-domain-model-addresses-and-device-identity)
- [10. Signal and sender-key integration](messaging.md#10-signal-and-sender-key-integration)
- [11. Persistence: one atomic protocol state, optional separate history](persistence.md#11-persistence-one-atomic-protocol-state-optional-separate-history)
- [12. Outbox: commit encryption state before transmission](persistence.md#12-outbox-commit-encryption-state-before-transmission)
- [13. Inbox, receipts and durable events](persistence.md#13-inbox-receipts-and-durable-events)
- [14. Public API contract](messaging.md#14-public-api-contract)
- [15. Groups and message operations](messaging.md#15-groups-and-message-operations)
- [16. Media without hidden native dependencies](messaging.md#16-media-without-hidden-native-dependencies)
- [17. Bridge compatibility without coupling the library to Telegram](messaging.md#17-bridge-compatibility-without-coupling-the-library-to-telegram)
- [18. Dependency policy](dependencies.md#18-dependency-policy)
- [19. Verification and CI](verification.md#19-verification-and-ci)
- [20. Operational behavior and publication](operations.md#20-operational-behavior-and-publication)
- [21. Exact starting instructions for the implementation agent](scope.md#21-exact-starting-instructions-for-the-implementation-agent)
- [22. Definition of done](scope.md#22-definition-of-done)
