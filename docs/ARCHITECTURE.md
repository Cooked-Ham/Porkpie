# Architecture

## Overview
For security constraints, see [SECURITY_INVARIANTS.md](SECURITY_INVARIANTS.md). For schema outlines, see [DATA_MODEL.md](DATA_MODEL.md). For mitigations, see [THREAT_MODEL.md](THREAT_MODEL.md).
Porkpie is a fully offline-capable, cross-platform password manager constructed entirely natively with Rust. Built precisely from the ground up for strict type-safety and uncompromisable cryptographic behavior, it enforces zero-knowledge architecture. Every instance performs encryption strictly on the client layer using XChaCha20Poly1305 authenticated encryption, and a backend HTTP API functions strictly as a blind storage locker to route encrypted synchronization blobs between hardware devices.

## Component Diagram
`	ext
       [ apps/web ]     [ apps/desktop ]   [ apps/server ]
             |                 |                 |
             v                 v                 |
       +-----------+     +---------+             |
       |    UI     |     |   CLI   |             |
       +-----------+     +---------+             |
             |                 |                 |
             v                 v                 v
       +-------------------------------------+   |
       |                Core                 |   |
       +-------------------------------------+   |
             |                 |                 |
             v                 v                 |
       +-----------+     +-----------+     +-----------+
       |  Crypto   |     |   Store   |     |   Sync    |
       +-----------+     +-----------+     +-----------+
                               |                 |
                               v                 v
                       [ SQLite DB ]     [ Backend API ]
`

## Data Flow
`	ext
User creates vault → Master password input internally sourced
                           ↓
                   porkpie-crypto layer
           [ Argon2id transient key generation ]
                           ↓
                   Vault item creation
           [ XChaCha20Poly1305 item encryption ]
                           ↓
                 porkpie-store SQLite
           [ Saved locally to encrypted disk blocks ]
                           ↓
                     porkpie-sync
           [ Periodic encrypted blob pushes to generic sync agent ]
`

## Crate Responsibilities

### porkpie-types
Domain types (IDs, Items, Timestamps), error types mapping, and constants.

### porkpie-crypto
Exclusively houses key derivation functions (Argon2id), XChaCha20Poly1305 encryption/decryption execution traits, padding routines, MAC verifications, and strictly random nonce generations bounds. 

### porkpie-core
Bridges lower-level subsystems into cohesive session modules, governing the primary Vault structs, item iterations, and memory lifetimes.

### porkpie-store
Manages SQLite schema provisioning and SQLx execution bindings, routing the encrypted bytes into rows and querying synchronized timestamps efficiently.

### porkpie-sync
Encapsulates HTTP/REST client bindings required to fetch and update encrypted Vault blobs into remote server topologies, managing collision resolution.

### porkpie-api
Houses the Axum backend HTTP server logic to act as the primary endpoint receiving sync pulls and pushes from the porkpie-sync modules running on various user hardware.

### porkpie-cli
The foundational command-line interface developed via clap, allowing pure terminal environments to execute core unlock, export, and search routines interactively.

### porkpie-ui
The primary graphical subsystem integrating the Dioxus crate for responsive web, desktop, and hybrid interactions spanning search components and vault visualizations.

### porkpie-agent
An SSH signer foundation providing the `SshSigner` trait (algorithm, public key bytes, sign), `Ed25519Signer` in-memory implementation backed by `ed25519-dalek`, and `HostKeyPolicy` structs for restricting SSH key usage to allowed hosts. The `porkpie ssh-agent` CLI command starts a real OpenSSH-compatible Unix domain socket agent. Supports both raw Ed25519 seeds and standard OpenSSH private key PEM format. Encrypted OpenSSH keys are not supported. Windows named pipes are not supported.

### porkpie-import
A segmented parsing library focused entirely on safely consuming CSVs, JSON arrays, and external tool file structures, transforming them efficiently into Porkpie unencrypted objects prior to storage saves.

## Security Boundaries
- **Client Boundary:** Includes the CLI, UI, and Core instances. This boundary represents the absolute line dividing where encryption/decryption tasks occur. Memory within this boundary acts as the secure enclave natively processing sensitive strings.
- **Server Boundary:** Comprised of the Sync API endpoints and the Backend datastores. Operates 100% blind to user identities and relies directly on client boundary submissions.
- **Never Cross:** The Server Boundary must never receive processing mechanisms, key components, or plaintext JSON inputs originating from behind the Client Boundary.

## Encryption Boundaries
- **IN:** porkpie-crypto serves as the sole gateway responsible for executing cryptographically secure payload transformations (where crypto happens).
- **OUT:** All neighboring crates independently interact with porkpie-crypto, receiving opaque output arrays formatted perfectly for standard system operations.

