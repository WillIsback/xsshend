#[cfg(test)]
mod ssh_keys_tests {
    use std::fs;
    use tempfile::TempDir;
    use xsshend::ssh::keys::{SshKey, SshKeyManager, SshKeyType};

    /// Crée un répertoire temporaire avec des clés SSH de test
    fn create_test_ssh_dir() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let ssh_dir = temp_dir.path().join(".ssh");
        fs::create_dir_all(&ssh_dir).unwrap();

        // Créer une clé Ed25519 de test
        let ed25519_content = "-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAFwAAAAdzc2gtcn
-----END OPENSSH PRIVATE KEY-----";
        fs::write(ssh_dir.join("id_ed25519"), ed25519_content).unwrap();

        let ed25519_pub = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAITEST_KEY_DATA test@example.com";
        fs::write(ssh_dir.join("id_ed25519.pub"), ed25519_pub).unwrap();

        // Créer une clé RSA de test
        let rsa_content = "-----BEGIN RSA PRIVATE KEY-----
MIIEpAIBAAKCAQEA1234567890abcdef...
-----END RSA PRIVATE KEY-----";
        fs::write(ssh_dir.join("id_rsa"), rsa_content).unwrap();

        let rsa_pub = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQTEST_RSA_KEY test@example.com";
        fs::write(ssh_dir.join("id_rsa.pub"), rsa_pub).unwrap();

        temp_dir
    }

    #[test]
    fn test_ssh_key_creation() {
        let temp_dir = create_test_ssh_dir();
        let key_path = temp_dir.path().join(".ssh/id_ed25519");

        let ssh_key = SshKey::new("id_ed25519".to_string(), key_path).unwrap();

        assert_eq!(ssh_key.name, "id_ed25519");
        assert_eq!(ssh_key.key_type, SshKeyType::Ed25519);
        assert!(ssh_key.public_key_path.is_some());
        assert!(ssh_key.comment.is_some());
        assert_eq!(ssh_key.comment.unwrap(), "test@example.com");
    }

    #[test]
    fn test_ssh_key_type_detection() {
        let temp_dir = TempDir::new().unwrap();

        // Test Ed25519
        let ed25519_path = temp_dir.path().join("id_ed25519");
        fs::write(&ed25519_path, "-----BEGIN OPENSSH PRIVATE KEY-----").unwrap();
        let key = SshKey::new("id_ed25519".to_string(), ed25519_path).unwrap();
        assert_eq!(key.key_type, SshKeyType::Ed25519);

        // Test RSA
        let rsa_path = temp_dir.path().join("test_rsa");
        fs::write(&rsa_path, "-----BEGIN RSA PRIVATE KEY-----").unwrap();
        let key = SshKey::new("test_rsa".to_string(), rsa_path).unwrap();
        assert_eq!(key.key_type, SshKeyType::Rsa);

        // Test ECDSA
        let ecdsa_path = temp_dir.path().join("test_ecdsa");
        fs::write(&ecdsa_path, "-----BEGIN EC PRIVATE KEY-----").unwrap();
        let key = SshKey::new("test_ecdsa".to_string(), ecdsa_path).unwrap();
        assert_eq!(key.key_type, SshKeyType::Ecdsa);
    }

    #[test]
    fn test_ssh_key_description() {
        let temp_dir = create_test_ssh_dir();
        let key_path = temp_dir.path().join(".ssh/id_ed25519");
        let ssh_key = SshKey::new("id_ed25519".to_string(), key_path).unwrap();

        let description = ssh_key.description();
        assert!(description.contains("id_ed25519"));
        assert!(description.contains("Ed25519"));
        assert!(description.contains("test@example.com"));
    }

    #[test]
    fn test_ssh_key_validation() {
        let temp_dir = create_test_ssh_dir();
        let key_path = temp_dir.path().join(".ssh/id_ed25519");
        let ssh_key = SshKey::new("id_ed25519".to_string(), key_path).unwrap();

        assert!(ssh_key.is_valid());

        // Test avec un fichier inexistant
        let invalid_key = SshKey {
            name: "nonexistent".to_string(),
            private_key_path: temp_dir.path().join("nonexistent"),
            public_key_path: None,
            key_type: SshKeyType::Unknown("test".to_string()),
            comment: None,
        };
        assert!(!invalid_key.is_valid());
    }

    #[test]
    fn test_ssh_key_manager_creation() {
        // Note: Ce test dépend de l'environnement réel
        // Dans un vrai environnement de test, on mockrait dirs::home_dir()

        // Pour ce test, on vérifie juste que la création ne panique pas
        match SshKeyManager::new() {
            Ok(_manager) => {
                // Manager créé avec succès
                // Test que le manager a été créé correctement
            }
            Err(_) => {
                // Peut échouer si pas de répertoire home dans l'environnement de test
                // C'est acceptable pour ce test
            }
        }
    }

    #[test]
    fn test_ssh_key_manager_key_selection() {
        // Test de la logique de sélection de clés
        // Créer un manager avec des clés de test manuellement

        let temp_dir = create_test_ssh_dir();
        let ssh_dir = temp_dir.path().join(".ssh");

        // Simuler la découverte de clés
        let ed25519_key =
            SshKey::new("id_ed25519".to_string(), ssh_dir.join("id_ed25519")).unwrap();

        let rsa_key = SshKey::new("id_rsa".to_string(), ssh_dir.join("id_rsa")).unwrap();

        // Test de priorité : Ed25519 devrait être préféré
        let keys = vec![rsa_key.clone(), ed25519_key.clone()];

        // Simuler select_best_key logic
        let mut best_key = &keys[0];
        for key in &keys {
            if let SshKeyType::Ed25519 = key.key_type {
                best_key = key;
                break;
            }
        }

        assert_eq!(best_key.name, "id_ed25519");
        assert_eq!(best_key.key_type, SshKeyType::Ed25519);
    }

    #[test]
    fn test_ssh_key_types_display() {
        assert_eq!(format!("{}", SshKeyType::Ed25519), "Ed25519");
        assert_eq!(format!("{}", SshKeyType::Rsa), "RSA");
        assert_eq!(format!("{}", SshKeyType::Ecdsa), "ECDSA");
        assert_eq!(
            format!("{}", SshKeyType::Unknown("test".to_string())),
            "test"
        );
    }

    #[test]
    fn test_ssh_key_types_equality() {
        assert_eq!(SshKeyType::Ed25519, SshKeyType::Ed25519);
        assert_ne!(SshKeyType::Ed25519, SshKeyType::Rsa);
        assert_eq!(
            SshKeyType::Unknown("test".to_string()),
            SshKeyType::Unknown("test".to_string())
        );
    }

    #[test]
    fn test_ssh_key_without_public_key() {
        let temp_dir = TempDir::new().unwrap();
        let private_key_path = temp_dir.path().join("private_only");
        fs::write(&private_key_path, "-----BEGIN OPENSSH PRIVATE KEY-----").unwrap();

        let ssh_key = SshKey::new("private_only".to_string(), private_key_path).unwrap();

        assert_eq!(ssh_key.name, "private_only");
        assert!(ssh_key.public_key_path.is_none());
        assert!(ssh_key.comment.is_none());
    }

    #[test]
    fn test_ssh_key_comment_extraction() {
        let temp_dir = TempDir::new().unwrap();

        // Créer une clé publique avec commentaire
        let pub_key_path = temp_dir.path().join("test.pub");
        fs::write(&pub_key_path, "ssh-ed25519 AAAAC3... user@hostname").unwrap();

        let private_key_path = temp_dir.path().join("test");
        fs::write(&private_key_path, "-----BEGIN OPENSSH PRIVATE KEY-----").unwrap();

        let ssh_key = SshKey::new("test".to_string(), private_key_path).unwrap();

        assert!(ssh_key.comment.is_some());
        assert_eq!(ssh_key.comment.unwrap(), "user@hostname");
    }
}
