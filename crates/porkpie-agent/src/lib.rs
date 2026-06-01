//! SSH agent foundation for Porkpie.
//!
//! This crate provides the signer trait, host/key policy structs, and an
//! in-memory Ed25519 signer.  It does **not** implement OpenSSH agent
//! socket/named-pipe integration; that is a future phase.

pub mod agent;
pub mod in_memory_signer;
pub mod policy;
pub mod signer;

pub use agent::{Agent, AgentError, AgentIdentity};
pub use in_memory_signer::Ed25519Signer;
pub use policy::{HostKeyPolicy, SshKeyIdentity};
pub use signer::{SignerError, SshSigner};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signer_trait_works_with_unlocked_in_memory_key() {
        let signer = Ed25519Signer::generate();
        let data = b"sign this";
        let signature = signer.sign(data).expect("signing must succeed");
        assert!(!signature.is_empty());
        assert_eq!(signer.algorithm(), "ssh-ed25519");
        assert_eq!(signer.public_key_bytes().len(), 32);
    }

    #[test]
    fn host_key_policy_allows_all_when_empty() {
        let policy = HostKeyPolicy::unrestricted();
        assert!(policy.is_host_allowed("any.host.com"));
    }

    #[test]
    fn host_key_policy_restricts_to_allowed_hosts() {
        let policy = HostKeyPolicy {
            allowed_hosts: vec!["github.com".to_string(), "gitlab.com".to_string()],
            require_confirmation: false,
        };
        assert!(policy.is_host_allowed("github.com"));
        assert!(policy.is_host_allowed("gitlab.com"));
        assert!(!policy.is_host_allowed("evil.com"));
    }

    #[test]
    fn host_key_policy_require_confirmation_flag() {
        let policy = HostKeyPolicy::require_confirmation();
        assert!(policy.require_confirmation);
        assert!(policy.is_host_allowed("any.host.com"));
    }

    #[test]
    fn ssh_key_identity_holds_comment_and_public_key() {
        let identity = SshKeyIdentity {
            comment: "my laptop".to_string(),
            public_key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5 my laptop".to_string(),
            algorithm: "ssh-ed25519".to_string(),
        };
        assert_eq!(identity.comment, "my laptop");
        assert!(identity.public_key.contains("ssh-ed25519"));
    }
}
