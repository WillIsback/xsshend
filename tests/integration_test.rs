#[cfg(test)]
mod integration_tests {
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;
    use tempfile::TempDir;

    /// Structure pour gérer un environnement de test isolé
    struct TestEnvironment {
        temp_dir: TempDir,
        home_dir: PathBuf,
        ssh_dir: PathBuf,
    }

    impl TestEnvironment {
        fn new() -> Self {
            let temp_dir = TempDir::new().unwrap();
            let home_dir = temp_dir.path().join("home");
            let ssh_dir = home_dir.join(".ssh");

            fs::create_dir_all(&ssh_dir).unwrap();

            TestEnvironment {
                temp_dir,
                home_dir,
                ssh_dir,
            }
        }

        fn create_test_config(&self) -> PathBuf {
            let config_content = r#"
            {
                "Test": {
                    "Local": {
                        "Mock": {
                            "TEST_SERVER": {
                                "alias": "testuser@localhost",
                                "env": "TEST"
                            }
                        }
                    }
                }
            }"#;

            let config_path = self.ssh_dir.join("hosts.json");
            fs::write(&config_path, config_content).unwrap();
            config_path
        }

        fn create_test_ssh_key(&self) -> PathBuf {
            let key_content = "-----BEGIN OPENSSH PRIVATE KEY-----\ntest_key_content\n-----END OPENSSH PRIVATE KEY-----";
            let pub_content = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAITEST test@example.com";

            let key_path = self.ssh_dir.join("id_ed25519");
            let pub_path = self.ssh_dir.join("id_ed25519.pub");

            fs::write(&key_path, key_content).unwrap();
            fs::write(&pub_path, pub_content).unwrap();

            // Permissions sécurisées pour la clé privée
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&key_path).unwrap().permissions();
                perms.set_mode(0o600);
                fs::set_permissions(&key_path, perms).unwrap();
            }

            key_path
        }

        fn create_test_file(&self, name: &str, content: &str) -> PathBuf {
            let file_path = self.temp_dir.path().join(name);
            fs::write(&file_path, content).unwrap();
            file_path
        }

        fn run_xsshend(&self, args: &[&str]) -> std::process::Output {
            Command::new("./target/debug/xsshend")
                .args(args)
                .env("HOME", &self.home_dir)
                .output()
                .expect("Failed to execute xsshend")
        }
    }

    #[test]
    fn test_integration_init_command() {
        let test_env = TestEnvironment::new();

        // Exécuter la commande init
        let output = test_env.run_xsshend(&["init", "--force"]);

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("Initialisation de xsshend"));
        assert!(stdout.contains("Initialisation terminée"));

        // Vérifier que le fichier hosts.json a été créé
        let hosts_file = test_env.ssh_dir.join("hosts.json");
        assert!(hosts_file.exists());

        // Vérifier le contenu du fichier hosts.json
        let content = fs::read_to_string(&hosts_file).unwrap();
        assert!(content.contains("Production"));
        assert!(content.contains("Staging"));
    }

    #[test]
    fn test_integration_list_with_config() {
        let test_env = TestEnvironment::new();
        test_env.create_test_config();

        let output = test_env.run_xsshend(&["list"]);

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("Liste des cibles SSH"));
        assert!(stdout.contains("TEST_SERVER"));
    }

    #[test]
    fn test_integration_upload_dry_run_workflow() {
        let test_env = TestEnvironment::new();
        test_env.create_test_config();
        let test_file = test_env.create_test_file("test.txt", "integration test content");

        let output = test_env.run_xsshend(&[
            "upload",
            test_file.to_str().unwrap(),
            "--env",
            "Test",
            "--dry-run",
        ]);

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("Mode dry-run"));
        assert!(stdout.contains("TEST_SERVER"));
        assert!(stdout.contains("test.txt"));
        assert!(stdout.contains("Simulation terminée"));
    }

    #[test]
    fn test_integration_complete_workflow() {
        let test_env = TestEnvironment::new();

        // 1. Initialiser la configuration
        let init_output = test_env.run_xsshend(&["init", "--force"]);
        assert!(init_output.status.success());

        // 2. Lister les serveurs
        let list_output = test_env.run_xsshend(&["list"]);
        assert!(list_output.status.success());

        // 3. Créer un fichier de test
        let test_file = test_env.create_test_file("workflow_test.txt", "workflow content");

        // 4. Faire un dry-run upload
        let upload_output = test_env.run_xsshend(&[
            "upload",
            test_file.to_str().unwrap(),
            "--env",
            "Development",
            "--dry-run",
        ]);
        assert!(upload_output.status.success());
    }

    #[test]
    fn test_integration_ssh_key_detection() {
        let test_env = TestEnvironment::new();
        test_env.create_test_ssh_key();
        test_env.create_test_config();

        let output = test_env.run_xsshend(&["list"]);

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        // Le message d'erreur sur les clés SSH ne devrait pas apparaître
        assert!(!stdout.contains("Aucune clé SSH trouvée"));
    }

    #[test]
    fn test_integration_multiple_files_upload() {
        let test_env = TestEnvironment::new();
        test_env.create_test_config();

        let file1 = test_env.create_test_file("file1.txt", "content 1");
        let file2 = test_env.create_test_file("file2.txt", "content 2");
        let file3 = test_env.create_test_file("file3.txt", "content 3");

        let output = test_env.run_xsshend(&[
            "upload",
            file1.to_str().unwrap(),
            file2.to_str().unwrap(),
            file3.to_str().unwrap(),
            "--env",
            "Test",
            "--dry-run",
        ]);

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("file1.txt"));
        assert!(stdout.contains("file2.txt"));
        assert!(stdout.contains("file3.txt"));
    }

    #[test]
    fn test_integration_error_handling() {
        let test_env = TestEnvironment::new();
        test_env.create_test_config();

        // Test avec un fichier inexistant
        let output = test_env.run_xsshend(&[
            "upload",
            "/nonexistent/file.txt",
            "--env",
            "Test",
            "--dry-run",
        ]);

        assert!(!output.status.success());
        // Le programme devrait échouer gracieusement
    }

    #[test]
    fn test_integration_config_without_ssh_keys() {
        let test_env = TestEnvironment::new();
        test_env.create_test_config();
        // Pas de clés SSH créées

        let test_file = test_env.create_test_file("test.txt", "test content");

        let output = test_env.run_xsshend(&[
            "upload",
            test_file.to_str().unwrap(),
            "--env",
            "Test",
            "--dry-run",
        ]);

        // Le dry-run devrait fonctionner même sans clés SSH
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("Mode dry-run"));
    }

    #[test]
    fn test_integration_filter_combinations() {
        let test_env = TestEnvironment::new();

        // Créer une config plus complexe
        let complex_config = r#"
        {
            "Production": {
                "Region-A": {
                    "Public": {
                        "WEB_01": { "alias": "web@prod-a-pub.com", "env": "PROD" }
                    },
                    "Private": {
                        "DB_01": { "alias": "db@prod-a-priv.com", "env": "PROD" }
                    }
                },
                "Region-B": {
                    "Public": {
                        "WEB_02": { "alias": "web@prod-b-pub.com", "env": "PROD" }
                    }
                }
            },
            "Staging": {
                "Region-A": {
                    "Public": {
                        "WEB_STAGE": { "alias": "web@stage-a-pub.com", "env": "STAGE" }
                    }
                }
            }
        }"#;

        let config_path = test_env.ssh_dir.join("hosts.json");
        fs::write(&config_path, complex_config).unwrap();

        let test_file = test_env.create_test_file("test.txt", "test content");

        // Test avec différentes combinaisons de filtres
        let test_cases = vec![
            (
                vec!["--env", "Production"],
                "should contain WEB_01, DB_01, WEB_02",
            ),
            (
                vec!["--env", "Production", "--region", "Region-A"],
                "should contain WEB_01, DB_01",
            ),
            (
                vec!["--env", "Production", "--type", "Public"],
                "should contain WEB_01, WEB_02",
            ),
            (
                vec![
                    "--env",
                    "Production",
                    "--region",
                    "Region-A",
                    "--type",
                    "Public",
                ],
                "should contain WEB_01",
            ),
        ];

        for (filters, description) in test_cases {
            let mut args = vec!["upload", test_file.to_str().unwrap()];
            args.extend(filters);
            args.extend(vec!["--dry-run"]);

            let output = test_env.run_xsshend(&args);
            assert!(output.status.success(), "Failed for case: {}", description);

            let stdout = String::from_utf8(output.stdout).unwrap();
            assert!(
                stdout.contains("Mode dry-run"),
                "No dry-run message for: {}",
                description
            );
        }
    }

    #[test]
    fn test_integration_large_config_performance() {
        let test_env = TestEnvironment::new();

        // Générer une grande configuration avec de nombreux serveurs
        let mut large_config = String::from("{\n");
        for env_i in 0..5 {
            large_config.push_str(&format!("  \"Env_{}\": {{\n", env_i));
            for region_i in 0..3 {
                large_config.push_str(&format!("    \"Region_{}\": {{\n", region_i));
                for type_i in 0..2 {
                    large_config.push_str(&format!("      \"Type_{}\": {{\n", type_i));
                    for server_i in 0..10 {
                        large_config.push_str(&format!(
                            "        \"SERVER_{}_{}_{}_{}\": {{ \"alias\": \"user{}@server{}.com\", \"env\": \"ENV{}\" }}",
                            env_i, region_i, type_i, server_i, server_i, server_i, env_i
                        ));
                        if server_i < 9 {
                            large_config.push(',');
                        }
                        large_config.push('\n');
                    }
                    large_config.push_str("      }");
                    if type_i < 1 {
                        large_config.push(',');
                    }
                    large_config.push('\n');
                }
                large_config.push_str("    }");
                if region_i < 2 {
                    large_config.push(',');
                }
                large_config.push('\n');
            }
            large_config.push_str("  }");
            if env_i < 4 {
                large_config.push(',');
            }
            large_config.push('\n');
        }
        large_config.push_str("}\n");

        let config_path = test_env.ssh_dir.join("hosts.json");
        fs::write(&config_path, large_config).unwrap();

        let test_file = test_env.create_test_file("test.txt", "test content");

        // Test que même avec une grande config, le programme reste performant
        let start = std::time::Instant::now();
        let output = test_env.run_xsshend(&[
            "upload",
            test_file.to_str().unwrap(),
            "--env",
            "Env_0",
            "--dry-run",
        ]);
        let duration = start.elapsed();

        assert!(output.status.success());
        assert!(
            duration.as_secs() < 5,
            "Performance test failed - took too long: {:?}",
            duration
        );

        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(stdout.contains("Mode dry-run"));
    }

    #[test]
    fn test_integration_init_force_overwrite() {
        let test_env = TestEnvironment::new();

        // Créer un config initial
        test_env.create_test_config();
        let config_path = test_env.ssh_dir.join("hosts.json");
        let original_content = fs::read_to_string(&config_path).unwrap();

        // Exécuter init avec --force
        let output = test_env.run_xsshend(&["init", "--force"]);
        assert!(output.status.success());

        // Vérifier que le fichier a été remplacé
        let new_content = fs::read_to_string(&config_path).unwrap();
        assert_ne!(original_content, new_content);
        assert!(new_content.contains("Production")); // Contenu du template par défaut
    }

    #[test]
    fn test_integration_ssh_permissions() {
        let test_env = TestEnvironment::new();

        let output = test_env.run_xsshend(&["init", "--force"]);
        assert!(output.status.success());

        // Vérifier les permissions du répertoire .ssh
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let ssh_dir_perms = fs::metadata(&test_env.ssh_dir).unwrap().permissions();
            let permissions = ssh_dir_perms.mode() & 0o777;
            // Les permissions devraient être 0o700 ou au minimum permettre à l'utilisateur de lire/écrire/exécuter
            assert!(
                permissions & 0o700 == 0o700,
                "SSH directory permissions should include owner rwx, got: {:o}",
                permissions
            );
        }
    }
}
