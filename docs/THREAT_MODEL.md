# Threat Model

This Threat Model outlines primary attack vectors and documents how Porkpie implements specific mitigations.

## Trust Assumptions
- **The Local Operating System is Trusted**: We assume the client OS has not been compromised by ring-0 rootkits or advanced malware capable of scraping dynamic memory regions or hooking keyboard input globally, as overcoming this is generally out-of-scope for user-space applications.
- **Dependencies are Verified**: Supply chain risks are mitigated by regular dependency scanning, though we implicitly assume standard library crates lack intentional backdoors.

## 1. Password Guessing and Brute-force

**The Threat:** An adversary obtains an encrypted local database file and attempts to rapidly decrypt it computationally by iterating through millions of guessable passwords.

**The Mitigation:** 
- The master password is not validated using simplistic hashing algorithms. Porkpie utilizes Argon2id as the Key Derivation Function (KDF) to stretch the master password into a strong encryption key.
- Argon2id requires intensive compute and memory parameters per guess, mathematically neutralizing the effectiveness of high-speed GPU or ASIC cracking configurations against reasonable master passwords.

## 2. Server Breach

**The Threat:** A malicious actor compromises the remote backend infrastructure and exfiltrates the synchronization node's entire primary data store.

**The Mitigation:**
- End-To-End zero-knowledge encryption ensures the server possesses absolutely no cryptographic material capable of decrypting user vaults. 
- The server stores arbitrary byte blobs and routing IDs; without the user's explicit vault credentials, the data is cryptographically unbreakable cipher noise.

## 3. Local Machine Physical Compromise

**The Threat:** An adversary steals a user's powered-off laptop containing the synced SQLite local store databases and configuration files.

**The Mitigation:**
- All local data persists entirely encrypted through XChaCha20Poly1305. The decryption key material relies solely on an Argon2id derivation executed exclusively against the master password and client-side decryption.
- No plaintext remnants, keys, or unencrypted artifacts are left on the local solid-state drives after the application process concludes or powers down. Without the master password, the stolen laptop yields no usable vault information.

## 4. Network Eavesdropping (Man-In-The-Middle)

**The Threat:** During data synchronization with remote devices, an attacker monitors transit networking equipment to dump packet streams containing user item synchronizations over public internet spaces.

**The Mitigation:**
- Vault chunks strictly transit as heavily formatted XChaCha20Poly1305 ciphertext containers carrying randomized, single-use 24-byte nonces.
- By design, payload traffic is heavily wrapped within standard HTTPS / TLS communication structures preventing interception.

## 5. Software Supply Chain Compromise

**The Threat:** A seemingly innocuous crate dependency introduces malicious macros capable of transmitting environment variables or memory payloads when compiled into the Porkpie binaries.

**The Mitigation:**
- Strict cargo locking patterns and automated integration linting limits transitive vulnerabilities.
- Cryptographic primitives depend entirely and exclusively upon audited structures housed within the RustCrypto ecosystem, limiting exposure to experimental and niche open-source packages.

## In-Scope vs Out-of-Scope 

**In-Scope:**
- Securing vault contents at-rest.
- Defending transit payloads against unauthorized decryption.
- Purging sensitive dynamic state materials while running.
- Ensuring strong derivation mechanisms to maximize master password potency.

**Out-of-Scope:**
- Defending against hardware keyloggers overriding user input streams.
- Preventing memory scraping vulnerabilities on rooted or deeply infected OS kernels.
- Enforcing password strength rules natively.
