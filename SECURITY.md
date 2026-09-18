# Security

This project is experimental and has no production-supported releases.
Security fixes currently target the main development branch.

## Reporting a vulnerability

Use the repository's private
[Report a vulnerability](https://github.com/TripleU613/TeleWhatsApp/security/advisories/new)
form if private reporting is enabled. If the form is unavailable, open a
minimal issue asking a maintainer for a private contact method, without
exploit details or sensitive attachments.

Include the affected commit, feature set, platform, impact, and a sanitized
reproduction. Never submit credentials, OTPs, PINs, private messages, raw
account databases, or account-bearing captures publicly. No response-time
commitment is currently published.

## Operational boundaries

Live messaging is not implemented. Experimental Noise and HTTPS features
are research interfaces, not production security guarantees. The local HTTP
control plane has no authentication and should remain on loopback.
Keep master keys separate from account databases. See the
[status](STATUS.md), [TLS decision](docs/adr/0003-tls-provider-gate.md), and
[Noise decision](docs/adr/0004-noise-backend-snow-without-ring.md).
