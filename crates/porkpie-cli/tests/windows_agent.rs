//! Windows SSH agent end-to-end verification test.
//!
//! This test runs only on Windows. It creates a real vault, adds an Ed25519
//! SSH key, starts the named-pipe agent, connects a client, and performs a
//! real SSH agent protocol handshake (REQUEST_IDENTITIES). This is as close
//! to a real `ssh-add -L` as we can get in an automated test.

#[cfg(windows)]
mod windows_agent {
    use porkpie_agent::{AgentIdentity, Ed25519Signer};
    use porkpie_cli::commands::ssh::run_agent_with_unlocked_vault;
    use porkpie_core::{LocalSecretKey, Vault};
    use porkpie_store::store_vault;
    use porkpie_types::{Item, ItemType, SSHKeySecret, VaultId};
    use std::path::PathBuf;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::windows::named_pipe::ClientOptions;

    async fn create_test_vault() -> (Vault, String, PathBuf) {
        let temp_dir = std::env::temp_dir().join(format!(
            "porkpie-windows-agent-test-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let db_path = temp_dir.join("test.db");
        let db_url = format!("sqlite://{}?mode=rwc", db_path.display());
        let pool = porkpie_store::connect_database(&db_url).await.unwrap();

        let password = "SuperSecretPassword123!";
        let secret_key = LocalSecretKey::generate();
        let (vault, _recovery_kit) = Vault::create("TestVault", password, &secret_key).unwrap();
        store_vault(&pool, &vault).await.unwrap();

        // Create an SSH key item from a known seed
        let seed: [u8; 32] = [42u8; 32];
        let signer = Ed25519Signer::from_seed(&seed);
        let public_key =
            base64::engine::general_purpose::STANDARD.encode(signer.public_key_bytes());
        let private_key = hex::encode(seed);
        let ssh_item = Item {
            id: porkpie_types::ItemId::new(),
            vault_id: vault.id,
            data: ItemType::SSHKey(SSHKeySecret {
                name: "test-ssh-key".to_string(),
                public_key,
                private_key,
                passphrase: None,
                comment: Some("test-key".to_string()),
                allowed_hosts: vec![],
                require_confirmation: false,
            }),
            created_at: porkpie_types::Timestamp::now(),
            updated_at: porkpie_types::Timestamp::now(),
            revision: 1,
        };

        // We need to modify the vault to add the item, but vault is immutable
        // after creation. Use a separate unlocked vault instance.
        let mut vault_with_item = vault;
        vault_with_item.items_mut().insert(ssh_item.id, ssh_item);
        store_vault(&pool, &vault_with_item).await.unwrap();

        (vault_with_item, password.to_string(), temp_dir)
    }

    #[tokio::test]
    async fn windows_agent_full_handshake() {
        let (vault, password, _temp_dir) = create_test_vault().await;
        let secret_key = LocalSecretKey::generate();
        let mut unlocked = vault;
        unlocked.unlock(&password, &secret_key).unwrap();

        // Start the agent in a background task on a custom pipe
        let pipe_name = r"\\.\pipe\porkpie-e2e-test";
        std::env::set_var("PORKPIE_SSH_AGENT_SOCK", pipe_name);

        let agent_handle =
            tokio::spawn(async move { run_agent_with_unlocked_vault(&unlocked).await });

        // Give the agent time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Connect a client and send REQUEST_IDENTITIES
        let mut client = ClientOptions::new()
            .open(pipe_name)
            .expect("failed to connect to test pipe");

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

        // Clean up
        drop(client);
        agent_handle.abort();
    }

    #[tokio::test]
    async fn windows_agent_fails_without_keys() {
        let temp_dir = std::env::temp_dir().join(format!(
            "porkpie-windows-agent-empty-test-{}",
            uuid::Uuid::new_v4()
        ));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let db_path = temp_dir.join("test.db");
        let db_url = format!("sqlite://{}?mode=rwc", db_path.display());
        let pool = porkpie_store::connect_database(&db_url).await.unwrap();

        let password = "SuperSecretPassword123!";
        let secret_key = LocalSecretKey::generate();
        let (vault, _recovery_kit) = Vault::create("TestVault", password, &secret_key).unwrap();
        store_vault(&pool, &vault).await.unwrap();

        let mut unlocked = vault;
        unlocked.unlock(password, &secret_key).unwrap();

        // Should fail with "No SSH key items found" (which returns Ok(()))
        let result = run_agent_with_unlocked_vault(&unlocked).await;
        assert!(result.is_ok(), "empty vault should return Ok with no keys");
    }
}
