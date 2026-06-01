//! OpenSSH agent protocol implementation for Porkpie.
//!
//! This module implements the minimal subset of the SSH agent protocol
//! needed to sign authentication challenges with Ed25519 keys stored
//! in an unlocked Porkpie vault.
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
}

/// A single identity registered with the agent.
pub struct AgentIdentity {
    pub comment: String,
    pub signer: Box<dyn SshSigner + Send + Sync>,
}

/// In-memory SSH agent that holds identities and answers protocol requests.
pub struct Agent {
    identities: Vec<AgentIdentity>,
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
        }
    }

    /// Register a new identity with the agent.
    pub fn add_identity(&mut self, identity: AgentIdentity) {
        self.identities.push(identity);
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
        let identity = self
            .identities
            .iter()
            .find(|id| {
                let mut expected_blob = encode_string(SSH_ED25519_ALGORITHM);
                expected_blob.extend_from_slice(&encode_bytes(id.signer.public_key_bytes()));
                blob == expected_blob
            })
            .ok_or(AgentError::NoIdentities)?;

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
}
