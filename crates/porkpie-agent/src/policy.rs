//! Host/key policy structs for SSH agent support.

/// Policy governing which hosts a key may be used for and whether
/// user confirmation is required before signing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostKeyPolicy {
    /// Host names or patterns the key is allowed to authenticate to.
    /// Empty means no restrictions (all hosts allowed).
    pub allowed_hosts: Vec<String>,
    /// If true, the agent must prompt the user before every sign operation.
    pub require_confirmation: bool,
}

impl HostKeyPolicy {
    /// Create a policy that allows all hosts without confirmation.
    pub fn unrestricted() -> Self {
        Self {
            allowed_hosts: vec![],
            require_confirmation: false,
        }
    }

    /// Create a policy that requires confirmation for every sign operation.
    pub fn require_confirmation() -> Self {
        Self {
            allowed_hosts: vec![],
            require_confirmation: true,
        }
    }

    /// Check whether `host` is allowed by this policy.
    pub fn is_host_allowed(&self, host: &str) -> bool {
        if self.allowed_hosts.is_empty() {
            return true;
        }
        self.allowed_hosts.iter().any(|h| h == host)
    }
}

/// Identity exposed to an SSH agent for a single key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SshKeyIdentity {
    /// Human-readable comment (often the trailing field of an OpenSSH public key line).
    pub comment: String,
    /// Full OpenSSH public key line, e.g. `ssh-ed25519 AAAAC3Nza... comment`.
    pub public_key: String,
    /// Algorithm name extracted from the public key line.
    pub algorithm: String,
}
