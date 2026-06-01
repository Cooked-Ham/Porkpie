# Phase 09: Build SSH-Agent Foundation Honestly

## Binding

You are bound to Phase 09 only.

Your job is to create a real foundation for SSH-agent support without lying that OpenSSH integration is complete. Do not ship socket cosplay. Do not print private keys. Do not confuse a timer with an agent again, because apparently that happened.

## Goal

Add SSH key item behavior, public-key export, signer trait, and honest ssh-agent command status.

## Required Context

Read first:

- `crates/porkpie-agent/**`
- `crates/porkpie-types/**`
- `crates/porkpie-core/**`
- `crates/porkpie-cli/**`
- `crates/porkpie-store/**`
- `docs/STATUS.md`

## Allowed Areas

- `crates/porkpie-agent/**`
- `crates/porkpie-types/**`
- `crates/porkpie-core/**`
- `crates/porkpie-cli/**`
- tests/docs related to SSH agent status

## Forbidden

- Do not claim OpenSSH socket/named-pipe integration is done unless it actually works.
- Do not write private keys to disk plaintext.
- Do not print private keys by default.
- Do not log private keys.
- Do not add fake signer behavior in production path.
- Do not use `pie://` incorrectly: use it for field references where appropriate.

## Tasks

1. Ensure SSH key item supports:
   - private key
   - public key
   - comment
   - allowed hosts
2. Private key must be encrypted at rest.
3. Implement:

```bash
porkpie ssh public-key <pie-uri-or-item>
```

4. Add a signer trait.
5. Add in-memory signing test.
6. Add host/key policy structs.
7. Add:

```bash
porkpie ssh-agent
```

8. If real OpenSSH integration is not complete, the command must clearly say:

```text
OpenSSH agent socket/named-pipe integration is not implemented yet.
```

## Tests

- Public key can be displayed.
- Private key is not printed by default.
- Signer trait works with an unlocked in-memory key.
- Raw DB does not contain private key fixture.
- CLI/UX does not claim full SSH-agent support unless real.

## Acceptance Criteria

- SSH key item flow exists.
- Public key command works.
- Signer trait test passes.
- No private key leakage.
- SSH-agent status is honest.
- Global validation command passes.

## Required Final Report

Include:

- Summary
- Files changed
- Commands run
- Test results
- SSH key model
- Signer interface details
- OpenSSH integration status
- Remaining SSH-agent risks
- Next recommended phase
