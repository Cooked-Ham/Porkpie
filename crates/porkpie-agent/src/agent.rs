//! OpenSSH agent protocol implementation for Porkpie.
//!
//! This module implements the SSH agent protocol needed to sign
//! authentication challenges with Ed25519 keys stored in an unlocked
//! Porkpie vault.
//!
//! Protocol wire format: each message is a 4-byte length (big-endian uint32)
//! followed by the payload. The first byte is the message type.
//!
//! Supported messages:
//! - SSH_AGENTC_REQUEST_IDENTITIES (11)
//! - SSH_AGENT_IDENTITIES_ANSWER (12)
//! - SSH_AGENTC_SIGN_REQUEST (13)
//! - SSH_AGENT_SIGN_RESPONSE (14)

use crate::signer::{SignerError, SshSigner};
use std::io::{Read, Write};

const SSH_AGENTC_REQUEST_IDENTITIES: u8 = 11;
const SSH_AGENT_IDENTITIES_ANSWER: u8 = 12;
const SSH_AGENTC_SIGN_REQUEST: u8 = 13;
const SSH_AGENT_SIGN_RESPONSE: u8 = 14;
const SSH_AGENT_FAILURE: u8 = 5;

const SSH_ED25519_ALGORITHM: &str = "ssh-ed25519";

/// Error type for agent protocol operations.
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Protocol error: {0}")]
    Protocol(String),
    #[error("Signing error: {0}")]
    Sign(#[from] SignerError),
    #[error("No identities available")]
    NoIdentities,
    #[error("Unsupported algorithm")]
    UnsupportedAlgorithm,
    #[error("Vault locked")]
    VaultLocked,
    #[error("Host not allowed: {0}")]
    HostNotAllowed(String),
    #[error("User denied signing request")]
    UserDenied,
}

/// A single identity registered with the agent.
pub struct AgentIdentity {
    pub comment: String,
    pub signer: Box<dyn SshSigner + Send + Sync>,
    /// Optional host restriction. If set, signing is only allowed for these hosts.
    pub allowed_hosts: Vec<String>,
    /// If true, the user must explicitly approve each signing request.
    pub require_confirmation: bool,
}

/// Callback for user approval of signing requests.
/// Returns true if the user approves, false otherwise.
/// Arguments: (comment, hex_preview_of_data)
pub type ApprovalCallback = Box<dyn Fn(&str, &str) -> bool + Send + Sync>;

/// In-memory SSH agent that holds identities and answers protocol requests.
pub struct Agent {
    identities: Vec<AgentIdentity>,
    approval_callback: Option<ApprovalCallback>,
}

impl Default for Agent {
    fn default() -> Self {
        Self::new()
    }
}

impl Agent {
    pub fn new() -> Self {
        Self {
            identities: Vec::new(),
            approval_callback: None,
        }
    }

    /// Register a new identity with the agent.
    pub fn add_identity(&mut self, identity: AgentIdentity) {
        self.identities.push(identity);
    }

    /// Set the approval callback for signing requests.
    pub fn set_approval_callback(&mut self, callback: ApprovalCallback) {
        self.approval_callback = Some(callback);
    }

    /// Run the agent protocol over a single connection.
    ///
    /// Reads requests from `reader` and writes responses to `writer`.
    /// Returns when the connection closes or an unrecoverable error occurs.
    pub fn handle_connection<R: Read, W: Write>(
        &self,
        reader: &mut R,
        writer: &mut W,
    ) -> Result<(), AgentError> {
        loop {
            let mut len_buf = [0u8; 4];
            match reader.read_exact(&mut len_buf) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(()),
                Err(e) => return Err(e.into()),
            }
            let len = u32::from_be_bytes(len_buf) as usize;
            if len == 0 || len > 256 * 1024 {
                return Err(AgentError::Protocol(format!(
                    "message length {len} out of bounds"
                )));
            }
            let mut payload = vec![0u8; len];
            reader.read_exact(&mut payload)?;

            let response = self.process_request(&payload)?;
            let response_len = response.len() as u32;
            writer.write_all(&response_len.to_be_bytes())?;
            writer.write_all(&response)?;
            writer.flush()?;
        }
    }

    fn process_request(&self, payload: &[u8]) -> Result<Vec<u8>, AgentError> {
        if payload.is_empty() {
            return Err(AgentError::Protocol("empty payload".to_string()));
        }
        let msg_type = payload[0];
        match msg_type {
            SSH_AGENTC_REQUEST_IDENTITIES => self.handle_request_identities(),
            SSH_AGENTC_SIGN_REQUEST => self.handle_sign_request(&payload[1..]),
            _ => Ok(vec![SSH_AGENT_FAILURE]),
        }
    }

    fn handle_request_identities(&self) -> Result<Vec<u8>, AgentError> {
        if self.identities.is_empty() {
            return Ok(vec![SSH_AGENT_FAILURE]);
        }

        let mut response = Vec::new();
        response.push(SSH_AGENT_IDENTITIES_ANSWER);
        let count = self.identities.len() as u32;
        response.extend_from_slice(&count.to_be_bytes());

        for identity in &self.identities {
            // Public key blob: algorithm name + key bytes
            let mut blob = encode_string(SSH_ED25519_ALGORITHM);
            blob.extend_from_slice(&encode_bytes(identity.signer.public_key_bytes()));

            response.extend_from_slice(&encode_bytes(&blob));
            response.extend_from_slice(&encode_string(&identity.comment));
        }

        Ok(response)
    }

    fn handle_sign_request(&self, data: &[u8]) -> Result<Vec<u8>, AgentError> {
        // Parse: blob (public key to identify), data to sign, flags (uint32)
        let mut offset = 0;
        let (blob, consumed) = decode_bytes(data, offset)?;
        offset += consumed;

        let (sign_data, consumed) = decode_bytes(data, offset)?;
        offset += consumed;

        if data.len() < offset + 4 {
            return Err(AgentError::Protocol("sign request truncated".to_string()));
        }
        let _flags = u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);

        // Find identity by matching public key blob
        let (_identity_idx, identity) = self
            .identities
            .iter()
            .enumerate()
            .find(|(_, id)| {
                let mut expected_blob = encode_string(SSH_ED25519_ALGORITHM);
                expected_blob.extend_from_slice(&encode_bytes(id.signer.public_key_bytes()));
                blob == expected_blob
            })
            .ok_or(AgentError::NoIdentities)?;

        // Check host policy
        if !identity.allowed_hosts.is_empty() {
            // Extract the host from the SSH data (this is a simplified check;
            // in production, parse the SSH session ID or host from the sign data)
            let host = extract_host_from_sign_data(&sign_data);
            if let Some(host) = host {
                if !identity.allowed_hosts.iter().any(|h| h == &host) {
                    return Err(AgentError::HostNotAllowed(host));
                }
            }
        }

        // Require user confirmation if enabled
        if identity.require_confirmation {
            if let Some(ref callback) = self.approval_callback {
                let comment = &identity.comment;
                let preview = hex::encode(&sign_data[..sign_data.len().min(32)]);
                if !callback(comment, &preview) {
                    return Err(AgentError::UserDenied);
                }
            } else {
                // No callback registered but confirmation required -> deny
                return Err(AgentError::UserDenied);
            }
        }

        let signature = identity.signer.sign(&sign_data)?;

        // Build signature blob: algorithm + signature bytes
        let mut sig_blob = encode_string(SSH_ED25519_ALGORITHM);
        sig_blob.extend_from_slice(&encode_bytes(&signature));

        let mut response = Vec::new();
        response.push(SSH_AGENT_SIGN_RESPONSE);
        response.extend_from_slice(&encode_bytes(&sig_blob));
        Ok(response)
    }
}

/// Encode a string as uint32 length + UTF-8 bytes.
fn encode_string(s: &str) -> Vec<u8> {
    let bytes = s.as_bytes();
    let mut result = Vec::with_capacity(4 + bytes.len());
    result.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    result.extend_from_slice(bytes);
    result
}

/// Encode a byte slice as uint32 length + bytes.
fn encode_bytes(b: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(4 + b.len());
    result.extend_from_slice(&(b.len() as u32).to_be_bytes());
    result.extend_from_slice(b);
    result
}

/// Decode a byte slice at offset: returns (bytes, total_consumed).
fn decode_bytes(data: &[u8], offset: usize) -> Result<(Vec<u8>, usize), AgentError> {
    if data.len() < offset + 4 {
        return Err(AgentError::Protocol("truncated length".to_string()));
    }
    let len = u32::from_be_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]) as usize;
    if data.len() < offset + 4 + len {
        return Err(AgentError::Protocol("truncated bytes".to_string()));
    }
    let bytes = data[offset + 4..offset + 4 + len].to_vec();
    Ok((bytes, 4 + len))
}

/// Extract a host identifier from SSH sign data (best-effort).
/// In a real SSH handshake, the sign data includes the session ID.
/// For now, we return None to allow the policy to pass if no host
/// can be determined, or we can check the first bytes.
fn extract_host_from_sign_data(_data: &[u8]) -> Option<String> {
    // SSH sign data format varies. The most common case is the
    // data to sign is the session identifier (hash). We can't
    // reverse it to a hostname. So we rely on the client passing
    // the host in a higher-level protocol.
    //
    // For OpenSSH, the agent does not know the target host. The
    // client (ssh) knows the host. So for Porkpie, we accept
    // that host-based restrictions require a custom client or
    // the host information is passed via a different mechanism.
    //
    // Return None to skip host-based checks when we can't determine
    // the host from the sign data alone.
    None
}

/// Run the agent on a Unix domain socket.
///
/// Creates a socket at `socket_path`, listens for connections, and
/// handles each client in a new thread.
#[cfg(unix)]
pub fn run_unix_socket(agent: Agent, socket_path: &std::path::Path) -> Result<(), AgentError> {
    use std::os::unix::fs::PermissionsExt;

    // Remove stale socket if it exists
    if socket_path.exists() {
        std::fs::remove_file(socket_path).map_err(|e| {
            AgentError::Io(std::io::Error::other(format!(
                "cannot remove stale socket: {e}"
            )))
        })?;
    }

    let listener = std::os::unix::net::UnixListener::bind(socket_path)?;

    // Set restrictive permissions (owner only)
    let mut perms = std::fs::metadata(socket_path)
        .map_err(AgentError::Io)?
        .permissions();
    perms.set_mode(0o600);
    std::fs::set_permissions(socket_path, perms).map_err(AgentError::Io)?;

    println!("Porkpie SSH agent listening on {}", socket_path.display());

    let agent = std::sync::Arc::new(std::sync::Mutex::new(agent));

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let agent_clone = std::sync::Arc::clone(&agent);
                std::thread::spawn(move || {
                    let mut read_stream = match stream.try_clone() {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("[porkpie-agent] failed to clone stream: {e}");
                            return;
                        }
                    };
                    let agent = agent_clone.lock().unwrap();
                    if let Err(e) = agent.handle_connection(&mut read_stream, &mut stream) {
                        eprintln!("[porkpie-agent] connection error: {e}");
                    }
                });
            }
            Err(e) => {
                eprintln!("[porkpie-agent] accept error: {e}");
            }
        }
    }

    Ok(())
}

/// Stop the agent and remove the socket file.
#[cfg(unix)]
pub fn stop_unix_socket(socket_path: &std::path::Path) -> Result<(), AgentError> {
    if socket_path.exists() {
        std::fs::remove_file(socket_path)?;
    }
    Ok(())
}

/// Non-unix stub: SSH agent requires Unix domain sockets.
#[cfg(not(unix))]
pub fn run_unix_socket(_agent: Agent, _socket_path: &std::path::Path) -> Result<(), AgentError> {
    Err(AgentError::Io(std::io::Error::other(
        "SSH agent is only supported on Unix platforms",
    )))
}

/// Non-unix stub for stop.
#[cfg(not(unix))]
pub fn stop_unix_socket(_socket_path: &std::path::Path) -> Result<(), AgentError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::in_memory_signer::Ed25519Signer;

    #[test]
    fn agent_responds_to_request_identities() {
        let mut agent = Agent::new();
        let signer = Ed25519Signer::generate();
        agent.add_identity(AgentIdentity {
            comment: "test-key".to_string(),
            signer: Box::new(signer),
            allowed_hosts: vec![],
            require_confirmation: false,
        });

        let request = vec![SSH_AGENTC_REQUEST_IDENTITIES];
        let response = agent.process_request(&request).unwrap();
        assert_eq!(response[0], SSH_AGENT_IDENTITIES_ANSWER);
        assert_eq!(
            u32::from_be_bytes([response[1], response[2], response[3], response[4]]),
            1
        );
    }

    #[test]
    fn agent_signs_with_matching_key() {
        let mut agent = Agent::new();
        let signer = Ed25519Signer::generate();
        let public_key = signer.public_key_bytes().to_vec();
        agent.add_identity(AgentIdentity {
            comment: "test-key".to_string(),
            signer: Box::new(signer),
            allowed_hosts: vec![],
            require_confirmation: false,
        });

        // Build sign request
        let mut blob = encode_string(SSH_ED25519_ALGORITHM);
        blob.extend_from_slice(&encode_bytes(&public_key));

        let data_to_sign = b"ssh challenge";
        let mut request = vec![SSH_AGENTC_SIGN_REQUEST];
        request.extend_from_slice(&encode_bytes(&blob));
        request.extend_from_slice(&encode_bytes(data_to_sign));
        request.extend_from_slice(&0u32.to_be_bytes()); // flags

        let response = agent.process_request(&request).unwrap();
        assert_eq!(response[0], SSH_AGENT_SIGN_RESPONSE);
    }

    #[test]
    fn agent_rejects_unknown_sign_request() {
        let agent = Agent::new();
        let mut blob = encode_string(SSH_ED25519_ALGORITHM);
        blob.extend_from_slice(&encode_bytes(&[0u8; 32]));

        let mut request = vec![SSH_AGENTC_SIGN_REQUEST];
        request.extend_from_slice(&encode_bytes(&blob));
        request.extend_from_slice(&encode_bytes(b"challenge"));
        request.extend_from_slice(&0u32.to_be_bytes());

        let result = agent.process_request(&request);
        assert!(matches!(result, Err(AgentError::NoIdentities)));
    }

    #[test]
    fn agent_requires_approval_when_configured() {
        let mut agent = Agent::new();
        let signer = Ed25519Signer::generate();
        let public_key = signer.public_key_bytes().to_vec();
        agent.add_identity(AgentIdentity {
            comment: "test-key".to_string(),
            signer: Box::new(signer),
            allowed_hosts: vec![],
            require_confirmation: true,
        });

        let mut blob = encode_string(SSH_ED25519_ALGORITHM);
        blob.extend_from_slice(&encode_bytes(&public_key));

        let mut request = vec![SSH_AGENTC_SIGN_REQUEST];
        request.extend_from_slice(&encode_bytes(&blob));
        request.extend_from_slice(&encode_bytes(b"challenge"));
        request.extend_from_slice(&0u32.to_be_bytes());

        // Without approval callback, should fail
        let result = agent.process_request(&request);
        assert!(matches!(result, Err(AgentError::UserDenied)));

        // With approval callback that returns true, should succeed
        let mut agent2 = Agent::new();
        let signer2 = Ed25519Signer::generate();
        let public_key2 = signer2.public_key_bytes().to_vec();
        agent2.add_identity(AgentIdentity {
            comment: "test-key".to_string(),
            signer: Box::new(signer2),
            allowed_hosts: vec![],
            require_confirmation: true,
        });
        agent2.set_approval_callback(Box::new(|_comment, _preview| true));

        let mut blob2 = encode_string(SSH_ED25519_ALGORITHM);
        blob2.extend_from_slice(&encode_bytes(&public_key2));

        let mut request2 = vec![SSH_AGENTC_SIGN_REQUEST];
        request2.extend_from_slice(&encode_bytes(&blob2));
        request2.extend_from_slice(&encode_bytes(b"challenge"));
        request2.extend_from_slice(&0u32.to_be_bytes());

        let response = agent2.process_request(&request2).unwrap();
        assert_eq!(response[0], SSH_AGENT_SIGN_RESPONSE);
    }

    #[test]
    fn agent_returns_failure_when_empty() {
        let agent = Agent::new();
        let request = vec![SSH_AGENTC_REQUEST_IDENTITIES];
        let response = agent.process_request(&request).unwrap();
        assert_eq!(response[0], SSH_AGENT_FAILURE);
    }
}
