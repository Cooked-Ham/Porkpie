# Phase 03: Strengthen Crypto Model

## Binding

You are bound to Phase 03 only.

Your job is to bring the crypto model closer to the intended Porkpie design. Touch crypto/core/store only where required. Do not modify UI except for compile fallout. Do not fake cryptographic behavior. Shocking that this needs saying, yet here we are.

## Goal

Add local secret key support, recovery kit generation, and AEAD associated-data binding.

## Required Context

Read first:

- `docs/CRYPTO_FORMAT.md`
- `docs/SECURITY_INVARIANTS.md`
- `docs/THREAT_MODEL.md`
- `crates/porkpie-crypto/**`
- `crates/porkpie-core/**`
- `crates/porkpie-types/**`
- relevant tests

## Allowed Areas

- `crates/porkpie-crypto/**`
- `crates/porkpie-core/**`
- `crates/porkpie-types/**`
- `crates/porkpie-store/**` only for metadata persistence changes
- `docs/CRYPTO_FORMAT.md`
- `docs/SECURITY_INVARIANTS.md`
- tests for these crates

## Forbidden

- No fake crypto.
- No base64-as-encryption.
- No hardcoded keys.
- No static nonces.
- No reused nonces.
- No server-side decrypt behavior.
- No weakening existing crypto tests.
- No printing/logging master password, local secret key, vault key, or item plaintext.

## Target Model

```text
master password + local secret key + salt -> unlock key
unlock key unwraps vault key
vault key decrypts items
```

## Tasks

### 1. Local Secret Key

- Generate a local secret key during vault creation.
- Secret key must not be sent to server.
- Secret key must be represented in a recovery-kit-friendly format.
- Store only the data needed to verify/decrypt with the correct password + secret key combination.

### 2. Recovery Kit

Generate recovery kit data containing:

- vault/account identifier
- local secret key
- recovery instructions
- warning about losing recovery material

Do not include plaintext vault contents.

### 3. Unlock Requirements

Unlock must require:

- master password
- local secret key

Tests must prove:

- correct password + correct secret key unlocks
- wrong password fails
- wrong secret key fails
- password alone cannot unlock

### 4. Associated Data Binding

Modify encryption/decryption APIs to accept associated data.

Bind ciphertext to stable metadata:

- vault ID
- item ID
- item type
- schema version
- revision ID where appropriate

Tests must prove decrypt fails if associated data changes.

Required cases:

- wrong vault ID
- wrong item ID
- wrong item type
- wrong schema version

### 5. Documentation

Update:

```text
docs/CRYPTO_FORMAT.md
```

Explain:

- what is encrypted
- what remains metadata
- what associated data is bound
- how the local secret key participates in unlocking
- what the recovery kit contains

## Acceptance Criteria

- Password alone cannot unlock.
- Wrong local secret key cannot unlock.
- Tampered associated data fails decrypt.
- Recovery kit exists.
- Crypto docs match implementation.
- Global validation command passes.

## Required Final Report

Include:

- Summary
- Files changed
- Commands run
- Test results
- New crypto model
- Recovery-kit behavior
- Associated-data binding fields
- Remaining crypto risks
- Next recommended phase
