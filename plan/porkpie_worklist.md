Porkpie Agentic Work List
Project Status

Porkpie is currently a foundational Rust prototype, not a complete password manager.

The next work cycle should focus on making the existing implementation honest, build-clean, security-safe, and actually functional before adding new features.

Do not add polish features until the current foundation passes CI, stops overclaiming, and proves that real vault flows work.

Global Agent Instructions

You are working on Porkpie, a Rust-first, local-first, zero-knowledge, self-hostable password manager for developers and homelab users.

Repository: https://github.com/Cooked-Ham/Porkpie

Core product identity:

Name: Porkpie
Domain: porkpie.love
CLI: porkpie
Secret URI scheme: pie://
Positioning: local-first password manager for passwords, SSH keys, API secrets, server records, database credentials, recovery codes, and custom typed credentials

Hard rules:

Do not call the project complete until every completion gate passes.
Do not store plaintext secrets.
Do not log secrets.
Do not store the master password.
Do not use fake crypto.
Do not use base64 as encryption.
Do not use hardcoded keys.
Do not use static or reused nonces.
Do not allow server-side vault decryption.
Do not ship public default API keys.
Do not raw-derive Debug for secret-bearing structs.
Do not print secrets from the CLI unless the user explicitly requests a specific secret field.
Do not present static UI mockups as real features.
Do not silently overwrite sync conflicts.
Do not suppress Clippy to fake a green build.
Do not weaken tests just to pass CI.
If a feature is not implemented, mark it honestly as unavailable.

Required validation command before reporting completion:

cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo build --workspace

Every agent report must include:

Summary
Files changed
Commands run
Test results
Security notes
Remaining limitations
Next recommended task
Work Phase 1: Make CI and docs honest
Goal

Make the repo pass its own quality gate and stop claiming completion before the foundation supports it.

Tasks
1. Fix Clippy

Run:

cargo clippy --workspace --all-targets -- -D warnings

Fix all failures without using broad #[allow(...)].

Do not silence warnings unless there is a narrow, documented reason.

2. Update README and docs

Rewrite status language to say:

Porkpie is a foundational Rust prototype. It is not safe for real credentials yet.

Remove or rewrite claims like:

complete
production-ready
final
secure password manager
safe for real secrets

Add a visible warning near the top of the README:

Do not use Porkpie with real credentials yet. This is a prototype pending security hardening and review.
3. Add docs/STATUS.md

Create a status matrix:

Implemented
Partially implemented
Mock/static
Planned
Unsafe/incomplete

Include at minimum:

crypto
local vault
local storage
CLI
UI
sync
SSH agent
browser extension
mobile
passkeys
sharing
production security review
Acceptance
CI command passes.
README no longer overclaims.
docs/STATUS.md clearly describes what is real and what is not.


---



Work Phase 2: Remove immediate security footguns
Goal

Fix the most dangerous issues before feature work continues.

Tasks
1. Remove public default API key behavior

The server must not default to a known API key.

Change behavior so missing API key fails startup.

Add:

.env.example

Do not commit real .env.

Docker Compose must reference environment variables instead of shipping a real default secret.

2. Hash stored API keys

Do not store raw API keys.

Implement API key hashing at rest.

Use constant-time comparison where practical.

Add tests proving the raw API key string is not stored.

3. Remove raw Debug from secret-bearing structs

Audit all secret-bearing types.

Remove raw Debug derives from anything containing or wrapping:

passwords
API keys
SSH private keys
TOTP seeds
recovery codes
private notes
database credentials
server credentials
identity secrets

If debug output is needed, implement redacted Debug.

Example:

LoginSecret {
    username: "[redacted]",
    password: "[redacted]",
}

Add tests proving fixture secrets do not appear in debug output.

Acceptance
No public default API key remains.
API keys are not stored raw.
Secret-bearing structs do not expose raw debug output.
CI command passes.




---





Work Phase 3: Fix CLI secret behavior
Goal

Make the CLI safe by default and useful intentionally.

Tasks
1. Redact default output

Change porkpie item get and item listing behavior so secrets are redacted by default.

Default output must not reveal:

passwords
API keys
SSH private keys
TOTP seeds
recovery codes
secure note bodies
database passwords
2. Add explicit secret commands

Add commands:

porkpie secret read <item-id-or-name> <field>
porkpie secret write <item-id-or-name> <field>

The user must explicitly request a field to reveal a secret.

3. Add env injection

Add:

porkpie run -- <command>

This should inject selected secrets into the child process environment.

Do not dump secrets to terminal by default.

4. Clean up backup commands

Prefer explicit names:

porkpie backup export
porkpie backup import

Plaintext export must require:

--dangerous

and should be clearly marked unsafe.

Acceptance
CLI list/get are redacted by default.
Secret reveal requires explicit field selection.
CLI tests verify fixture secrets do not appear in normal stdout.
CI command passes.




---





Work Phase 4: Make local vault and store prove no plaintext leaks
Goal

Prove end-to-end that local storage does not contain plaintext secrets.

Tasks
1. Add end-to-end storage leak tests

Create a vault with fixture secrets like:

DO_NOT_LEAK_PASSWORD_123
DO_NOT_LEAK_API_KEY_123
DO_NOT_LEAK_PRIVATE_KEY_123

Persist the vault.

Read the raw SQLite database bytes.

Assert fixture strings are absent.

2. Test all sensitive item types

Include fixture values for:

login password
API key
SSH private key
secure note body
database password
recovery code
3. Keep low-level store tests separate

It is fine for the store layer to test raw byte persistence, but do not use those tests as proof of encrypted storage.

The proof must happen through the real core-to-store path.

Acceptance
Raw SQLite file does not contain fixture secrets.
Test fails if plaintext is accidentally stored.
CI command passes.




---





Work Phase 5: Add account secret key and recovery kit
Goal

Implement the stronger unlock model originally planned.

Target model
master password + local secret key + salt -> unlock key
unlock key unwraps vault key
vault key decrypts items
Tasks
1. Generate local secret key

During vault creation, generate a local secret key.

The secret key must not be sent to the server.

2. Add recovery kit

Create a recovery kit containing:

vault/account identifier
local secret key
recovery instructions
warning that losing master password and recovery material can mean losing the vault
3. Require secret key for unlock

Unlock should require both:

master password
local secret key
4. Add tests

Tests must prove:

correct password + correct secret key unlocks
wrong password fails
wrong secret key fails
password alone cannot unlock
server never receives secret key
Acceptance
Vault creation produces recovery kit.
Unlock requires password and secret key.
Recovery documentation exists.
CI command passes.




---





Work Phase 6: Add associated data to encryption
Goal

Bind ciphertext to stable metadata so encrypted blobs cannot be moved or reclassified silently.

Tasks
1. Update crypto API

Modify encryption/decryption APIs to accept associated data.

2. Bind stable metadata

Use associated data for fields such as:

vault ID
item ID
item type
schema version
revision ID where appropriate
3. Add tests

Tests must prove decryption fails if associated data is changed.

Required test cases:

wrong vault ID
wrong item ID
wrong item type
wrong schema version
4. Document format

Update:

docs/CRYPTO_FORMAT.md

Explain what is encrypted, what is metadata, and what is bound as associated data.

Acceptance
Tampered associated data fails decrypt.
Crypto docs match implementation.
CI command passes.




---





Work Phase 7: Make the UI real or clearly mark it as preview
Goal

Stop shipping static UI as if it is functional.

Tasks
1. Remove normal-flow preview data

Hardcoded preview items must not appear in the normal app path.

Demo data may exist only in an explicitly named demo mode.

2. Wire onboarding

Onboarding must create a real vault.

3. Wire unlock

Unlock screen must call real vault unlock logic.

Errors must be conditional.

Do not show permanent fake errors.

4. Wire item management

The UI must support real:

item list
item detail
item creation
item editing
item deletion
5. Wire lock behavior

Lock must clear decrypted UI state.

After lock, secrets must disappear from the UI.

6. Wire import/export

Encrypted backup export/import should call real logic.

Plaintext export must require explicit dangerous confirmation.

7. Mark unavailable features honestly

If something is not implemented, label it:

Not implemented yet

Do not render fake controls that imply functionality.

Acceptance

Manual QA flow must work:

Create vault.
Add login item.
Add SSH key item.
Lock vault.
Confirm secrets disappear.
Unlock vault.
Confirm items return.
Export encrypted backup.
Import backup into clean profile.
Confirm restored items decrypt.

CI command passes.




---





Work Phase 8: Make desktop and web apps launchable
Goal

Turn the app wrappers into actual runnable apps.

Tasks
1. Desktop

Add a real Dioxus desktop app entrypoint.

Document command:

cargo run -p porkpie-desktop
2. Web

Add a real Dioxus web app entrypoint.

Document the exact command to launch it.

3. Shared state

Make sure both use the same real UI flow or clearly document any storage differences.

Acceptance
Desktop app launches.
Web app launches.
README commands work exactly.
CI command passes.




---





Work Phase 9: Make sync deserve the word
Goal

Upgrade sync from partial encrypted push to real bidirectional prototype sync.

Tasks
1. Add encrypted vault registration

Server must support creating/registering encrypted vault metadata.

2. Push local revisions

CLI sync must push local encrypted revisions.

3. Pull remote revisions

CLI sync must pull remote encrypted revisions.

4. Apply remote revisions locally

Remote encrypted revisions must be stored locally and become visible after unlock.

5. Maintain sync cursor

Local sync state must track what has been pushed/pulled.

6. Handle conflicts

If two profiles edit the same item independently, preserve conflict versions.

Do not silently overwrite.

7. Add two-profile integration test

Test flow:

Profile A creates vault and item.
Profile A syncs.
Profile B syncs.
Profile B sees item after unlock.
Profile A and B both edit same item independently.
Sync both.
Conflict is preserved.
Server DB does not contain fixture plaintext.
Acceptance
Bidirectional sync works.
Conflict handling exists.
Server never sees plaintext item data.
CI command passes.




---





Work Phase 10: Build SSH-agent foundation honestly
Goal

Add the real foundation for future SSH-agent support without pretending OpenSSH integration is complete.

Tasks
1. Add SSH key item support

Ensure SSH key item type supports:

private key
public key
optional comment
optional allowed hosts

Private key must be encrypted at rest.

2. Add public-key command

Add:

porkpie ssh public-key <item-id-or-name>

This may print only the public key.

3. Add signer trait

Create a signer interface that can sign using an unlocked in-memory key.

4. Add tests

Tests must prove:

public key can be displayed
private key is not printed by default
signer interface works
private key fixture is absent from raw DB
5. Add honest ssh-agent command

Add:

porkpie ssh-agent

If OpenSSH integration is not done, it must clearly say so.

Do not pretend socket/named-pipe support exists if it does not.

Acceptance
SSH key item flow exists.
Public key command works.
Signer trait has tests.
No private key leakage.
CI command passes.




---





Work Phase 11: Harden API tests
Goal

Make the API reject unsafe or misleading payloads.

Tasks
1. Reject plaintext item-shaped payloads

Add tests that attempt to send payloads containing obvious fields like:

username
password
private_key
api_key
totp
notes

The API must reject these if they are sent as plaintext item bodies.

2. Auth tests

Add tests for:

missing auth
wrong auth
revoked/unknown key
missing required API key config
3. Logging safety

Ensure request bodies are not logged.

Do not log encrypted blobs either unless necessary, and never log plaintext item-shaped data.

Acceptance
Unsafe plaintext payloads are rejected.
Auth failures behave correctly.
Logs do not expose payload contents.
CI command passes.




---





Work Phase 12: Final QA pass
Goal

Run a hostile audit pass before claiming the next milestone.

Required command
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo build --workspace
Required searches

Search the repo for:

TODO
FIXME
dev-key-change-in-production
password
secret
private_key
api_key
master_password
plaintext
base64
Debug
println!
dbg!
tracing
unwrap()
expect()

These are not automatically bugs, but every hit in a critical path must be reviewed.

Required report

Produce:

docs/AUDIT_REPORT.md

Include:

what is implemented
what is still mocked
what is unsafe
what security issues were fixed
what security issues remain
whether real credentials are safe to use
exact validation commands and results
Acceptance
Audit report exists.
CI command passes.
Remaining risks are documented honestly.
Project status remains “prototype” unless every gate is complete.
Completion Gate

Porkpie may only be called a working MVP after all of this is true:

CI is fully green.
Clippy passes with -D warnings.
README commands work.
No public default API keys exist.
API keys are hashed at rest.
Secret-bearing structs do not expose raw debug output.
CLI redacts secrets by default.
Local DB plaintext leak tests pass.
Account secret key and recovery kit exist.
Associated data binding exists.
UI creates/unlocks/locks a real vault.
Desktop app launches.
Web app launches.
Sync is bidirectional.
Sync conflict handling exists.
SSH-agent status is honest.
API rejects plaintext item-shaped payloads.
Docs do not overclaim.
Audit report says what is still unsafe.

Until then, the correct label is:

Porkpie: foundational Rust prototype, not safe for real credentials yet.