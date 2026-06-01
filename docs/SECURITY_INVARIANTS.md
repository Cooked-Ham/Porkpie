# Security Invariants

These thirteen invariants are fundamental to Porkpie's operation. Violating any of them constitutes a critical security failure.

## 1. No Plaintext Secret Storage

**The Rule:** Secrets must NEVER be stored in plaintext anywhere.

**Scope:** Local database, server database, synchronization payloads, file backups, output logs, or test fixtures.

**Enforcement:**
- Use XChaCha20Poly1305 for all encryption.
- Apply zeroize to plaintext memory buffers immediately after encryption.
- Never log secrets.
- Export plaintext requires explicit --dangerous-export-plaintext flag.

**Good vs Bad:**
- *Good*: Secrecy<String> wrapping a plaintext credential.
- *Bad*: String typed field representing a password in a database struct.

## 2. No Master Password Storage

**The Rule:** The master password must not be stored in any application state or persistent layer.

**Why it matters:** The master password is the root of the cryptographic trust chain.

**Enforcement:**
- Derived transiently using Argon2id during vault unlock operations.
- The password itself is dropped instantly.

**Good vs Bad:**
- *Good*: Deriving key from std input and zeroizing input buffer.
- *Bad*: Holding user password string as application state.

## 3. Client-Side Unlock Only

**The Rule:** The central server never sees the master password or any decrypted vault data.

**Why it matters:** End-to-end encrypted design requires trust only on the client hardware.

**Enforcement:**
- Clients handle all decryption natively, locally.
- Sync protocols send blind ciphertext to backend API.

**Good vs Bad:**
- *Good*: Synchronizing a Vec<u8> blob to the server.
- *Bad*: Passing an item's title in a REST request to a remote search index.

## 4. Server Stores Encrypted Blobs Only

**The Rule:** The remote sync server cannot natively read any item or perform server-side decryption algorithms.

**Why it matters:** An attacker obtaining a full backup of the server's database will only retrieve unintelligible ciphertexts.

**Enforcement:**
- The Sync API expects heavily serialized JSON envelopes wrapped in XChaCha20Poly1305 blocks.

**Good vs Bad:**
- *Good*: Server receiving an EncryptedVaultPayload containing ciphertext and nonce.
- *Bad*: Server attempting to deserialize item structures with serde_json.

## 5. Authenticated Encryption Required

**The Rule:** Every individual item must be encrypted using Authenticated Encryption with Associated Data (AEAD), specifically XChaCha20Poly1305.

**Why it matters:** Data must be unreadable and tamper-proof. AEAD ensures modified data throws a decryption error.

**Enforcement:**
- All crypto traits utilize RustCrypto's xchacha20poly1305 crate.

**Good vs Bad:**
- *Good*: Utilizing XChaCha20Poly1305 and binding item metadata as associated data.
- *Bad*: Using standard AES-CBC without MAC authentication tags.

## 6. Wrong Password Fails

**The Rule:** Supplying an incorrect master password must strictly result in a decryption pipeline failure.

**Why it matters:** Prevents stealthy fallback operations, incorrect partial decryption states, or false-positive unlocks.

**Enforcement:**
- The XChaCha20Poly1305 decrypt trait method inherently generates an error when applying the wrong key.

**Good vs Bad:**
- *Good*: Returning a DecryptionError directly to the porkpie-ui layer.
- *Bad*: Catching the decryption failure and returning an empty vault instead.

## 7. Tampered Ciphertext Fails

**The Rule:** Any ciphertext that has been modified after encryption must fail to decrypt.

**Why it matters:** Mitigates chosen-ciphertext attacks and data corruption errors.

**Enforcement:**
- The built-in MAC (Message Authentication Code) inherent to XChaCha20Poly1305 inherently enforces this upon decryption.

**Good vs Bad:**
- *Good*: The AEAD tag check fails resulting in a hard error on vault initialization.
- *Bad*: Storing MACs separately and failing to verify them on load.

## 8. Locking Clears Memory

**The Rule:** All decrypted data structs and key strings must be thoroughly purged from memory when the vault is locked.

**Why it matters:** Prevents cold-boot attacks and memory dumping vectors.

**Enforcement:**
- The zeroize crate features must be active on all sensitive intermediate variables.

**Good vs Bad:**
- *Good*: Defining a ZeroizeOnDrop wrapper struct for the master cryptographic key.
- *Bad*: Leaving decrypted item arrays allocated in background memory loops.

## 9. Plaintext Export Requires Flag

**The Rule:** Exporting vault items in unencrypted plaintext requires passing an explicit and intentional --dangerous-export-plaintext flag.

**Why it matters:** Prevents accidental data spills transparently.

**Enforcement:**
- The CLI enforces this flag at the command parsing layer using clap.

**Good vs Bad:**
- *Good*: Command aborting with an error if the user invokes porkpie export --format=csv.
- *Bad*: Designing a generic export function that defaults to plaintext JSON.

## 10. No Crypto Shortcuts

**The Rule:** Strictly no mock cryptography, base64 "encryption", or static keys anywhere.

**Why it matters:** Real cryptography forces architectures to handle true entropy.

**Enforcement:**
- The porkpie-crypto module must exclusively implement legitimate external crate logic continuously. No placeholder implementation in security paths.

**Good vs Bad:**
- *Good*: Unit tests performing full Argon2id key derivations.
- *Bad*: Utilizing an if cfg!(test) to mock decrypt.

## 11. Logs Sanitized

**The Rule:** Error, trace, and info logs must never emit item titles, usernames, passwords, or decrypted secrets.

**Why it matters:** Logging software and terminal history commonly result in unexpected storage of sensitive data.

**Enforcement:**
- Structs containing sensitive parameters must not derive Debug.
- tracing macros must explicitly filter internal item schemas.

**Good vs Bad:**
- *Good*: log::info!("Successfully decrypted item {}", item.id)
- *Bad*: log::info!("Successfully decrypted credentials for target")

## 12. No Placeholder Implementations in Security Paths

**The Rule:** Cryptographically sensitive routes must be implemented with real error-returning code paths.

**Why it matters:** Allowing empty stubs bypasses verification sequences. Unimplemented crypto is an immediate QA failure.

**Enforcement:**
- Failing paths return specific typed errors and are exercised by tests.

**Good vs Bad:**
- *Good*: Emitting a compile-time or runtime panic on unresolved security paths.
- *Bad*: Returning Ok(true) for MAC validations to pass a test suite.

## 13. Unavailable Features Clearly Marked

**The Rule:** Incomplete user-facing functionality must strictly represent as "Not Implemented".

**Why it matters:** Prevents the user from assuming the manager is securely operating correctly when it defaults to mocked traits.

**Enforcement:**
- UI components explicitly expose shell-backed operations and display typed errors from unavailable platform integrations.

**Good vs Bad:**
- *Good*: An error alert informing the user that syncing is offline.
- *Bad*: The UI displaying that a fake backup succeeded using hardcoded stub methods.
