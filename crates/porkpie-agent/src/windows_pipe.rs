//! Windows named-pipe SSH agent transport for Porkpie.
//!
//! Implements the OpenSSH agent protocol over a Windows named pipe.
//! Microsoft OpenSSH (the one shipping with Windows) uses the named pipe
//! `\\.\pipe\openssh-ssh-agent` by default.  Porkpie binds this pipe so
//! that `ssh-add`, `ssh`, Git for Windows, etc. talk to us directly.
//!
//! If the built-in Windows OpenSSH Authentication Agent service is running
//! it will already own the pipe.  We detect that and refuse to start with a
//! clear error message telling the user how to disable the service.

use crate::agent::{Agent, AgentError};
use std::sync::Arc;

/// Default OpenSSH agent pipe name used by Microsoft OpenSSH.
pub const DEFAULT_PIPE_NAME: &str = r"\\.\pipe\openssh-ssh-agent";

/// Detect whether the Windows OpenSSH Authentication Agent service is
/// running.  Returns `Ok(true)` if it appears to be running, `Ok(false)`
/// if not, and `Err` if we cannot determine.
#[cfg(windows)]
pub fn is_windows_ssh_agent_service_running() -> Result<bool, AgentError> {
    use std::process::Command;
    // sc query ssh-agent | findstr RUNNING
    let output = Command::new("sc")
        .args(["query", "ssh-agent"])
        .output()
        .map_err(|e| {
            AgentError::Io(std::io::Error::other(format!(
                "failed to query ssh-agent service: {e}"
            )))
        })?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.contains("RUNNING"))
}

#[cfg(not(windows))]
pub fn is_windows_ssh_agent_service_running() -> Result<bool, AgentError> {
    Ok(false)
}

/// Run the agent on a Windows named pipe.
///
/// Creates a named pipe server at `pipe_name`, listens for connections, and
/// handles each client sequentially (one at a time, which is sufficient for
/// the OpenSSH agent protocol as the client reconnects per operation).
#[cfg(windows)]
pub async fn run_windows_named_pipe(
    pipe_name: &str,
    agent: Arc<std::sync::Mutex<Agent>>,
) -> Result<(), AgentError> {
    use tokio::net::windows::named_pipe::{NamedPipeServer, ServerOptions};

    // Check for the built-in Windows ssh-agent service
    if is_windows_ssh_agent_service_running()? {
        return Err(AgentError::Io(std::io::Error::other(
            "The Windows OpenSSH Authentication Agent service is running.\n\
             It owns the default agent pipe.  Disable it first:\n\
             Stop-Service ssh-agent; Set-Service ssh-agent -StartupType Disabled",
        )));
    }

    let mut server: Option<NamedPipeServer> = None;

    loop {
        let new_server = ServerOptions::new()
            .first_pipe_instance(server.is_none())
            .access_inbound(true)
            .access_outbound(true)
            .pipe_mode(tokio::net::windows::named_pipe::PipeMode::Message)
            .max_instances(1)
            .create(pipe_name)
            .map_err(|e| {
                AgentError::Io(std::io::Error::other(format!(
                    "failed to create named pipe {pipe_name}: {e}"
                )))
            })?;

        server = Some(new_server);
        let pipe = server.as_mut().unwrap();

        // Wait for a client to connect
        pipe.connect().await.map_err(|e| {
            AgentError::Io(std::io::Error::other(format!(
                "named pipe connect failed: {e}"
            )))
        })?;

        // Clone the Arc so the handler can own it
        let agent_clone = Arc::clone(&agent);
        let pipe_name = pipe_name.to_string();

        // Handle this client.  When done, we create a new pipe server for
        // the next client.
        if let Err(e) = handle_client(pipe, agent_clone).await {
            eprintln!("[porkpie-agent] connection error on {pipe_name}: {e}");
        }

        // We need a fresh pipe server for the next client.  Drop the old one.
        server = None;
    }
}

#[cfg(windows)]
async fn handle_client(
    pipe: &mut tokio::net::windows::named_pipe::NamedPipeServer,
    agent: Arc<std::sync::Mutex<Agent>>,
) -> Result<(), AgentError> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    loop {
        let mut len_buf = [0u8; 4];
        match pipe.read_exact(&mut len_buf).await {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(()),
            Err(e) => return Err(AgentError::Io(e)),
        }
        let len = u32::from_be_bytes(len_buf) as usize;
        if len == 0 || len > 256 * 1024 {
            return Err(AgentError::Protocol(format!(
                "message length {len} out of bounds"
            )));
        }
        let mut payload = vec![0u8; len];
        pipe.read_exact(&mut payload)
            .await
            .map_err(AgentError::Io)?;

        let response = {
            let agent = agent.lock().unwrap();
            agent.process_request(&payload)?
        };

        let response_len = response.len() as u32;
        pipe.write_all(&response_len.to_be_bytes())
            .await
            .map_err(AgentError::Io)?;
        pipe.write_all(&response).await.map_err(AgentError::Io)?;
        pipe.flush().await.map_err(AgentError::Io)?;
    }
}

/// Non-Windows stub.
#[cfg(not(windows))]
pub async fn run_windows_named_pipe(
    _pipe_name: &str,
    _agent: Arc<std::sync::Mutex<Agent>>,
) -> Result<(), AgentError> {
    Err(AgentError::Io(std::io::Error::other(
        "Windows named pipe transport is only available on Windows",
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(windows)]
    use crate::agent::Agent;
    #[cfg(windows)]
    use crate::in_memory_signer::Ed25519Signer;
    #[cfg(windows)]
    use crate::AgentIdentity;

    #[test]
    fn service_query_on_non_windows_returns_false() {
        // On non-Windows we should never think the service is running
        let result = is_windows_ssh_agent_service_running();
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn named_pipe_server_starts_and_accepts_client() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::windows::named_pipe::ClientOptions;

        let pipe_name = r"\\.\pipe\porkpie-test-pipe";
        let mut agent = Agent::new();
        let signer = Ed25519Signer::generate();
        agent.add_identity(AgentIdentity {
            comment: "test-key".to_string(),
            signer: Box::new(signer),
            allowed_hosts: vec![],
            require_confirmation: false,
        });
        let agent = Arc::new(std::sync::Mutex::new(agent));

        // Spawn the server
        let server_handle =
            tokio::spawn(async move { run_windows_named_pipe(pipe_name, agent).await });

        // Give the server a moment to create the pipe
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Connect a client
        let mut client = ClientOptions::new().open(pipe_name).unwrap();

        // Send SSH_AGENTC_REQUEST_IDENTITIES
        let request = vec![11u8]; // SSH_AGENTC_REQUEST_IDENTITIES
        let len = request.len() as u32;
        client.write_all(&len.to_be_bytes()).await.unwrap();
        client.write_all(&request).await.unwrap();
        client.flush().await.unwrap();

        // Read response
        let mut len_buf = [0u8; 4];
        client.read_exact(&mut len_buf).await.unwrap();
        let resp_len = u32::from_be_bytes(len_buf) as usize;
        let mut response = vec![0u8; resp_len];
        client.read_exact(&mut response).await.unwrap();

        assert_eq!(response[0], 12); // SSH_AGENT_IDENTITIES_ANSWER

        // Shut down the server by dropping the client (server will see EOF)
        drop(client);
        server_handle.abort();
    }
}
