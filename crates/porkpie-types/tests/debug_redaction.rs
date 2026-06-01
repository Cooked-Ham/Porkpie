use porkpie_types::*;
use std::collections::HashMap;

#[test]
fn login_secret_debug_redacts_password() {
    let secret = LoginSecret {
        username: "admin@corp.com".to_string(),
        password: "super_secret_p@ssw0rd".to_string(),
        url: Some("https://internal.corp.com".to_string()),
        notes: Some("Admin account".to_string()),
    };
    let debug = format!("{:?}", secret);
    assert!(!debug.contains("admin@corp.com"));
    assert!(!debug.contains("super_secret_p@ssw0rd"));
    assert!(!debug.contains("internal.corp.com"));
    assert!(!debug.contains("Admin account"));
    assert!(debug.contains("[redacted]"));
}

#[test]
fn api_key_secret_debug_redacts_key() {
    let secret = APIKeySecret {
        name: "Production Stripe".to_string(),
        key: "sk_live_abc123def456".to_string(),
        provider: "Stripe".to_string(),
    };
    let debug = format!("{:?}", secret);
    assert!(!debug.contains("sk_live_abc123def456"));
    assert!(!debug.contains("Production Stripe"));
    assert!(debug.contains("[redacted]"));
}

#[test]
fn ssh_key_secret_debug_redacts_private_key() {
    let secret = SSHKeySecret {
        name: "prod-server".to_string(),
        public_key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5".to_string(),
        private_key: "-----BEGIN OPENSSH PRIVATE KEY-----\nb3BlbnNzaC1rZXktdjEAAAAA\n-----END OPENSSH PRIVATE KEY-----".to_string(),
        passphrase: Some("my_ssh_passphrase".to_string()),
        comment: Some("prod key".to_string()),
        allowed_hosts: vec!["prod.example.com".to_string()],
    };
    let debug = format!("{:?}", secret);
    assert!(!debug.contains("BEGIN OPENSSH PRIVATE KEY"));
    assert!(!debug.contains("my_ssh_passphrase"));
    assert!(!debug.contains("AAAAC3NzaC1lZDI1NTE5"));
    assert!(!debug.contains("prod key"));
    assert!(!debug.contains("prod.example.com"));
    assert!(debug.contains("[redacted]"));
}

#[test]
fn secure_note_debug_redacts_content() {
    let secret = SecureNoteSecret {
        title: "Recovery Seed".to_string(),
        content: "word1 word2 word3 word4 word5 word6".to_string(),
    };
    let debug = format!("{:?}", secret);
    assert!(!debug.contains("Recovery Seed"));
    assert!(!debug.contains("word1 word2"));
    assert!(debug.contains("[redacted]"));
}

#[test]
fn server_secret_debug_redacts_credentials() {
    let secret = ServerSecret {
        hostname: "10.0.0.1".to_string(),
        port: 22,
        username: "root".to_string(),
        password: Some("r00t_p@ss".to_string()),
        notes: Some("Production box".to_string()),
    };
    let debug = format!("{:?}", secret);
    assert!(!debug.contains("10.0.0.1"));
    assert!(!debug.contains("root"));
    assert!(!debug.contains("r00t_p@ss"));
    assert!(debug.contains("[redacted]"));
}

#[test]
fn database_secret_debug_redacts_credentials() {
    let secret = DatabaseSecret {
        engine: "PostgreSQL".to_string(),
        host: "db.internal".to_string(),
        port: 5432,
        username: "pg_admin".to_string(),
        password: "db_p@ssw0rd".to_string(),
        database: "production".to_string(),
    };
    let debug = format!("{:?}", secret);
    assert!(!debug.contains("db.internal"));
    assert!(!debug.contains("pg_admin"));
    assert!(!debug.contains("db_p@ssw0rd"));
    assert!(!debug.contains("production"));
    assert!(debug.contains("[redacted]"));
}

#[test]
fn recovery_codes_debug_redacts_codes() {
    let secret = RecoveryCodesSecret {
        codes: vec!["A81729B1".to_string(), "J18A891K".to_string()],
    };
    let debug = format!("{:?}", secret);
    assert!(!debug.contains("A81729B1"));
    assert!(!debug.contains("J18A891K"));
    assert!(debug.contains("[redacted]"));
}

#[test]
fn custom_secret_debug_redacts_fields() {
    let mut fields = HashMap::new();
    fields.insert("secret_key".to_string(), "hidden_value_123".to_string());
    let secret = CustomSecret { fields };
    let debug = format!("{:?}", secret);
    assert!(!debug.contains("hidden_value_123"));
    assert!(!debug.contains("secret_key"));
    assert!(debug.contains("[redacted]"));
}

#[test]
fn item_type_debug_redacts_all_variants() {
    let variants: Vec<ItemType> = vec![
        ItemType::Login(LoginSecret {
            username: "u".to_string(),
            password: "p".to_string(),
            url: None,
            notes: None,
        }),
        ItemType::APIKey(APIKeySecret {
            name: "n".to_string(),
            key: "k".to_string(),
            provider: "p".to_string(),
        }),
        ItemType::SSHKey(SSHKeySecret {
            name: "n".to_string(),
            public_key: "pub".to_string(),
            private_key: "priv".to_string(),
            passphrase: None,
            comment: None,
            allowed_hosts: vec![],
        }),
        ItemType::SecureNote(SecureNoteSecret {
            title: "t".to_string(),
            content: "c".to_string(),
        }),
        ItemType::Server(ServerSecret {
            hostname: "h".to_string(),
            port: 22,
            username: "u".to_string(),
            password: None,
            notes: None,
        }),
        ItemType::Database(DatabaseSecret {
            engine: "e".to_string(),
            host: "h".to_string(),
            port: 5432,
            username: "u".to_string(),
            password: "p".to_string(),
            database: "d".to_string(),
        }),
        ItemType::Identity(IdentitySecret {
            name: "n".to_string(),
            email: "e".to_string(),
            phone: None,
            address: None,
        }),
        ItemType::SoftwareLicense(SoftwareLicenseSecret {
            product: "p".to_string(),
            key: "k".to_string(),
            version: None,
            expiry: None,
        }),
        ItemType::RecoveryCodes(RecoveryCodesSecret {
            codes: vec!["code1".to_string()],
        }),
        ItemType::Custom(CustomSecret {
            fields: HashMap::new(),
        }),
    ];

    for variant in &variants {
        let debug = format!("{:?}", variant);
        assert!(
            debug.contains("[redacted]"),
            "variant debug output missing redaction: {debug}"
        );
    }
}

#[test]
fn vault_debug_redacts_master_key() {
    let vault = Vault {
        id: VaultId::new(),
        name: "TestVault".to_string(),
        created_at: 1000,
        master_key_wrapped: vec![0xDE, 0xAD, 0xBE, 0xEF],
        sync_revision: 0,
    };
    let debug = format!("{:?}", vault);
    assert!(!debug.contains("DEADBEEF"));
    assert!(!debug.contains("222, 173, 190, 239"));
    assert!(debug.contains("[redacted]"));
}

#[test]
fn recovery_kit_debug_redacts_local_secret_key() {
    let kit = RecoveryKit::new("vault-1", &LocalSecretKey::generate(), 1000);
    let debug = format!("{:?}", kit);
    assert!(debug.contains("[redacted]"));
    assert!(!debug.contains(&kit.local_secret_key));
    // JSON serialization must still contain the key for recovery purposes
    let json = serde_json::to_string(&kit).unwrap();
    assert!(json.contains(&kit.local_secret_key));
}
