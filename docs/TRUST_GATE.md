# Porkpie Trust Gate

Porkpie defines four maturity stages for credential use. Each stage requires specific conditions before real secrets are stored.

## Developer Alpha

- **Fake/test credentials only.**
- Purpose: feature development, UI testing, sync protocol validation.
- No real passwords, API keys, or private keys.
- No external audit required.

## Personal Dogfood

- Developer may use **low-risk real credentials** with explicit personal risk acceptance.
- Purpose: real-world usage feedback, bug discovery, UX refinement.
- The developer acknowledges the risk of data loss or exposure.
- No external audit required.
- Not recommended for others.

## Public Recommendation

- Requires broader testing, community security review, release hardening.
- Purpose: general public use for low-to-medium risk credentials.
- Memory zeroization, keychain integration, and sync protocol must be validated.
- Bug bounty or public security review program recommended.
- No external audit required, but one is recommended.

## Commercial / Enterprise

- Requires **independent external security audit**.
- Purpose: business-critical credentials, team sharing, compliance requirements.
- SOC 2, ISO 27001, or equivalent compliance review.
- Penetration testing against sync API and keychain integration.
- Formal incident response plan.

## Current Status

Porkpie is currently at **Developer Alpha**. It is a foundational Rust prototype with real crypto and real architecture, but it is **not safe for real credentials yet**.

## External Audit

External audit is not a development blocker. It is a Commercial/Enterprise gate requirement. The codebase does not claim to be externally audited, and no docs should recommend broad real-credential use without review.

## Compliance with No-Excuses Protocol

The following checklist tracks the protocol requirements for the trust gate:

- [x] Developer Alpha defined
- [x] Personal Dogfood defined
- [x] Public Recommendation defined
- [x] Commercial/Enterprise defined
- [x] External audit requirement placed at Commercial/Enterprise gate only
- [x] No docs claim Porkpie is externally audited
- [x] No docs recommend broad real-credential use without review
- [x] Current status explicitly stated as Developer Alpha
