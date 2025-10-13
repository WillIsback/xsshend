// Module commun pour les tests
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use xsshend::config::{HostEntry, HostsConfig};
use serde_json;

/// Gestionnaire d'environnement de test
pub struct TestEnvironment {
    pub temp_dir: TempDir,
    pub ssh_dir: PathBuf,
}

impl TestEnvironment {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let ssh_dir = temp_dir.path().join(".ssh");
        fs::create_dir_all(&ssh_dir).unwrap();

        TestEnvironment {
            temp_dir,
            ssh_dir,
        }
    }

    /// Créer un fichier de configuration de test
    pub fn create_hosts_config(&self) -> PathBuf {
        let config = self.get_test_hosts_config();
        let config_json = serde_json::to_string_pretty(&config).unwrap();
        let config_path = self.ssh_dir.join("hosts.json");
        fs::write(&config_path, config_json).unwrap();
        config_path
    }

    /// Créer des clés SSH de test
    pub fn create_test_ssh_keys(&self) -> Vec<PathBuf> {
        let mut key_paths = Vec::new();

        // Clé Ed25519
        let ed25519_private = self.ssh_dir.join("id_ed25519");
        let ed25519_public = self.ssh_dir.join("id_ed25519.pub");

        fs::write(&ed25519_private, Self::get_test_ed25519_private()).unwrap();
        fs::write(&ed25519_public, Self::get_test_ed25519_public()).unwrap();

        Self::set_secure_permissions(&ed25519_private);
        key_paths.push(ed25519_private);

        // Clé RSA
        let rsa_private = self.ssh_dir.join("id_rsa");
        let rsa_public = self.ssh_dir.join("id_rsa.pub");

        fs::write(&rsa_private, Self::get_test_rsa_private()).unwrap();
        fs::write(&rsa_public, Self::get_test_rsa_public()).unwrap();

        Self::set_secure_permissions(&rsa_private);
        key_paths.push(rsa_private);

        key_paths
    }

    /// Créer un fichier de test
    pub fn create_test_file(&self, name: &str, content: &str) -> PathBuf {
        let file_path = self.temp_dir.path().join(name);
        fs::write(&file_path, content).unwrap();
        file_path
    }

    /// Définir les permissions sécurisées pour les clés privées
    #[cfg(unix)]
    fn set_secure_permissions(file_path: &PathBuf) {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(file_path).unwrap().permissions();
        perms.set_mode(0o600);
        fs::set_permissions(file_path, perms).unwrap();
    }

    #[cfg(not(unix))]
    fn set_secure_permissions(_file_path: &PathBuf) {
        // Sur Windows, pas de gestion des permissions Unix
    }

    /// Configuration de test complète
    pub fn get_test_hosts_config(&self) -> HostsConfig {
        let mut environments = std::collections::HashMap::new();

        // Environment de production
        let mut production = std::collections::HashMap::new();
        let mut region_a = std::collections::HashMap::new();
        let mut region_b = std::collections::HashMap::new();

        // Region-A Public
        let mut public_a = std::collections::HashMap::new();
        public_a.insert("WEB_SERVER_01".to_string(), HostEntry {
            alias: "web01@prod-web-01.example.com".to_string(),
            env: "PROD".to_string(),
        });
        public_a.insert("API_SERVER_01".to_string(), HostEntry {
            alias: "api01@prod-api-01.example.com".to_string(),
            env: "PROD".to_string(),
        });

        // Region-A Private
        let mut private_a = std::collections::HashMap::new();
        private_a.insert("DATABASE_01".to_string(), HostEntry {
            alias: "db01@prod-db-01.example.com".to_string(),
            env: "PROD".to_string(),
        });

        region_a.insert("Public".to_string(), public_a);
        region_a.insert("Private".to_string(), private_a);

        // Region-B Public
        let mut public_b = std::collections::HashMap::new();
        public_b.insert("WEB_SERVER_02".to_string(), HostEntry {
            alias: "web02@prod-web-02.example.com".to_string(),
            env: "PROD".to_string(),
        });

        region_b.insert("Public".to_string(), public_b);

        production.insert("Region-A".to_string(), region_a);
        production.insert("Region-B".to_string(), region_b);
        environments.insert("Production".to_string(), production);

        // Environment de staging
        let mut staging = std::collections::HashMap::new();
        let mut stage_region_a = std::collections::HashMap::new();
        let mut stage_public = std::collections::HashMap::new();

        stage_public.insert("STAGE_WEB_01".to_string(), HostEntry {
            alias: "web01@stage-web-01.example.com".to_string(),
            env: "STAGE".to_string(),
        });

        stage_region_a.insert("Public".to_string(), stage_public);
        staging.insert("Region-A".to_string(), stage_region_a);
        environments.insert("Staging".to_string(), staging);

        // Environment de développement
        let mut development = std::collections::HashMap::new();
        let mut dev_local = std::collections::HashMap::new();
        let mut dev_services = std::collections::HashMap::new();

        dev_services.insert("DEV_DATABASE".to_string(), HostEntry {
            alias: "dev@dev-db.local.example.com".to_string(),
            env: "DEV".to_string(),
        });

        dev_local.insert("Services".to_string(), dev_services);
        development.insert("Local".to_string(), dev_local);
        environments.insert("Development".to_string(), development);

        HostsConfig { environments }
    }

    /// Contenu d'une clé Ed25519 de test
    fn get_test_ed25519_private() -> &'static str {
        "-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAFwAAAAdzc2gtcn
NhAAAAAwEAAQAAAQEAySTlOhKp8xZhWMkPQHVZ... (test key data)
-----END OPENSSH PRIVATE KEY-----"
    }

    fn get_test_ed25519_public() -> &'static str {
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAITEST_ED25519_PUBLIC_KEY_DATA test@example.com"
    }

    /// Contenu d'une clé RSA de test
    fn get_test_rsa_private() -> &'static str {
        "-----BEGIN RSA PRIVATE KEY-----
MIIEpAIBAAKCAQEAySTlOhKp8xZhWMkPQHVZ... (test RSA key data)
-----END RSA PRIVATE KEY-----"
    }

    fn get_test_rsa_public() -> &'static str {
        "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQTEST_RSA_PUBLIC_KEY_DATA test@example.com"
    }
}

/// Créer des hôtes de test simples
pub fn create_simple_test_hosts() -> Vec<(String, HostEntry)> {
    vec![
        (
            "Test:Local:Mock:SERVER_01".to_string(),
            HostEntry {
                alias: "user1@server1.example.com".to_string(),
                env: "TEST".to_string(),
            },
        ),
        (
            "Test:Local:Mock:SERVER_02".to_string(),
            HostEntry {
                alias: "user2@server2.example.com".to_string(),
                env: "TEST".to_string(),
            },
        ),
    ]
}

/// Données de test pour différents types de fichiers
pub struct TestFileData {
    pub name: String,
    pub content: String,
    pub expected_size: u64,
}

impl TestFileData {
    pub fn small_text() -> Self {
        TestFileData {
            name: "small.txt".to_string(),
            content: "Hello, World!".to_string(),
            expected_size: 13,
        }
    }

    pub fn empty_file() -> Self {
        TestFileData {
            name: "empty.txt".to_string(),
            content: "".to_string(),
            expected_size: 0,
        }
    }

    pub fn large_file() -> Self {
        TestFileData {
            name: "large.txt".to_string(),
            content: "x".repeat(1024 * 1024), // 1MB
            expected_size: 1024 * 1024,
        }
    }

    pub fn binary_file() -> Self {
        TestFileData {
            name: "binary.dat".to_string(),
            content: (0..256).map(|i| i as u8 as char).collect(),
            expected_size: 256,
        }
    }

    pub fn unicode_file() -> Self {
        TestFileData {
            name: "unicode.txt".to_string(),
            content: "Hello 🌍! Français, 中文, العربية".to_string(),
            expected_size: 41, // UTF-8 encoded length
        }
    }
}

/// Helper pour créer des arguments de commande de test
pub fn create_upload_args(file_path: &str, extra_args: &[&str]) -> Vec<String> {
    let mut args = vec!["upload".to_string(), file_path.to_string()];
    args.extend(extra_args.iter().map(|s| s.to_string()));
    args
}

/// Helper pour créer des combinaisons de filtres de test
pub fn get_filter_test_cases() -> Vec<(Vec<&'static str>, &'static str)> {
    vec![
        (vec!["--env", "Production"], "Production environment"),
        (vec!["--env", "Staging"], "Staging environment"),
        (vec!["--region", "Region-A"], "Region-A"),
        (vec!["--type", "Public"], "Public servers"),
        (vec!["--env", "Production", "--region", "Region-A"], "Production Region-A"),
        (vec!["--env", "Production", "--type", "Public"], "Production Public"),
        (vec!["--region", "Region-A", "--type", "Public"], "Region-A Public"),
        (vec!["--env", "Production", "--region", "Region-A", "--type", "Public"], "Production Region-A Public"),
    ]
}

/// Vérificateur d'assertions pour les sorties de commandes
pub struct CommandOutputAssertions<'a> {
    pub stdout: &'a str,
    pub stderr: &'a str,
    pub success: bool,
}

impl<'a> CommandOutputAssertions<'a> {
    pub fn new(output: &'a std::process::Output) -> Self {
        CommandOutputAssertions {
            stdout: std::str::from_utf8(&output.stdout).unwrap_or(""),
            stderr: std::str::from_utf8(&output.stderr).unwrap_or(""),
            success: output.status.success(),
        }
    }

    pub fn assert_success(&self) -> &Self {
        assert!(self.success, "Command failed. Stderr: {}", self.stderr);
        self
    }

    pub fn assert_failure(&self) -> &Self {
        assert!(!self.success, "Command unexpectedly succeeded. Stdout: {}", self.stdout);
        self
    }

    pub fn assert_contains(&self, text: &str) -> &Self {
        assert!(
            self.stdout.contains(text),
            "Output does not contain '{}'. Stdout: {}",
            text, self.stdout
        );
        self
    }

    pub fn assert_not_contains(&self, text: &str) -> &Self {
        assert!(
            !self.stdout.contains(text),
            "Output unexpectedly contains '{}'. Stdout: {}",
            text, self.stdout
        );
        self
    }

    pub fn assert_stderr_contains(&self, text: &str) -> &Self {
        assert!(
            self.stderr.contains(text),
            "Stderr does not contain '{}'. Stderr: {}",
            text, self.stderr
        );
        self
    }
}