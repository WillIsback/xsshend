#[cfg(test)]
mod config_tests {
    use std::fs;
    use tempfile::TempDir;
    use xsshend::config::{HostEntry, HostsConfig};

    /// Crée une configuration de test
    fn create_test_config() -> HostsConfig {
        let json_content = r#"
        {
            "Production": {
                "Region-A": {
                    "Public": {
                        "WEB_SERVER_01": {
                            "alias": "web01@prod-web-01.example.com",
                            "env": "PROD"
                        },
                        "API_SERVER_01": {
                            "alias": "api01@prod-api-01.example.com",
                            "env": "PROD"
                        }
                    },
                    "Private": {
                        "DATABASE_01": {
                            "alias": "db01@prod-db-01.example.com",
                            "env": "PROD"
                        }
                    }
                },
                "Region-B": {
                    "Public": {
                        "WEB_SERVER_02": {
                            "alias": "web02@prod-web-02.example.com",
                            "env": "PROD"
                        }
                    }
                }
            },
            "Staging": {
                "Region-A": {
                    "Public": {
                        "STAGE_WEB_01": {
                            "alias": "web01@stage-web-01.example.com",
                            "env": "STAGE"
                        }
                    }
                }
            }
        }"#;

        serde_json::from_str(json_content).unwrap()
    }

    #[tokio::test]
    async fn test_config_parsing() {
        let config = create_test_config();

        // Vérifier la structure de base
        assert!(config.environments.contains_key("Production"));
        assert!(config.environments.contains_key("Staging"));

        // Vérifier un serveur spécifique
        let prod = &config.environments["Production"];
        let region_a = &prod["Region-A"];
        let public = &region_a["Public"];
        let web_server = &public["WEB_SERVER_01"];

        assert_eq!(web_server.alias, "web01@prod-web-01.example.com");
        assert_eq!(web_server.env, "PROD");
    }

    #[tokio::test]
    async fn test_host_entry_serialization() {
        let host_entry = HostEntry {
            alias: "test@example.com".to_string(),
            env: "TEST".to_string(),
        };

        let json = serde_json::to_string(&host_entry).unwrap();
        let deserialized: HostEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(host_entry.alias, deserialized.alias);
        assert_eq!(host_entry.env, deserialized.env);
    }

    #[tokio::test]
    async fn test_filter_hosts_by_environment() {
        let config = create_test_config();

        let prod_hosts = config.filter_hosts(Some(&"Production".to_string()), None, None);
        assert_eq!(prod_hosts.len(), 4); // 4 serveurs en production

        let stage_hosts = config.filter_hosts(Some(&"Staging".to_string()), None, None);
        assert_eq!(stage_hosts.len(), 1); // 1 serveur en staging

        // Vérifier qu'on obtient bien des serveurs de production
        for (_, host_entry) in &prod_hosts {
            assert_eq!(host_entry.env, "PROD");
        }
    }

    #[tokio::test]
    async fn test_filter_hosts_by_region() {
        let config = create_test_config();

        let region_a_hosts = config.filter_hosts(None, Some(&"Region-A".to_string()), None);
        assert_eq!(region_a_hosts.len(), 4); // 3 prod + 1 stage dans Region-A

        let region_b_hosts = config.filter_hosts(None, Some(&"Region-B".to_string()), None);
        assert_eq!(region_b_hosts.len(), 1); // 1 serveur dans Region-B
    }

    #[tokio::test]
    async fn test_filter_hosts_by_type() {
        let config = create_test_config();

        let public_hosts = config.filter_hosts(None, None, Some(&"Public".to_string()));
        assert_eq!(public_hosts.len(), 4); // 4 serveurs publics

        let private_hosts = config.filter_hosts(None, None, Some(&"Private".to_string()));
        assert_eq!(private_hosts.len(), 1); // 1 serveur privé
    }

    #[tokio::test]
    async fn test_filter_hosts_combined() {
        let config = create_test_config();

        // Production + Region-A + Public
        let filtered = config.filter_hosts(
            Some(&"Production".to_string()),
            Some(&"Region-A".to_string()),
            Some(&"Public".to_string()),
        );
        assert_eq!(filtered.len(), 2); // WEB_SERVER_01 et API_SERVER_01

        // Vérifier les serveurs obtenus
        let server_names: Vec<String> = filtered
            .iter()
            .map(|(name, _)| name.split(':').next_back().unwrap().to_string())
            .collect();

        assert!(server_names.contains(&"WEB_SERVER_01".to_string()));
        assert!(server_names.contains(&"API_SERVER_01".to_string()));
    }

    #[tokio::test]
    async fn test_filter_hosts_no_match() {
        let config = create_test_config();

        let no_match = config.filter_hosts(Some(&"NonExistent".to_string()), None, None);
        assert_eq!(no_match.len(), 0);
    }

    #[tokio::test]
    async fn test_config_creation_default() {
        let temp_dir = TempDir::new().unwrap();

        // Simuler le processus de création de config par défaut
        let example_config = r#"
        {
            "Production": {
                "Region-A": {
                    "Public": {
                        "EXAMPLE_SERVER": {
                            "alias": "user@example.com",
                            "env": "PROD"
                        }
                    }
                }
            }
        }"#;

        let config_path = temp_dir.path().join("hosts.json");
        fs::write(&config_path, example_config).unwrap();

        // Lire et parser la configuration
        let content = fs::read_to_string(&config_path).unwrap();
        let config: HostsConfig = serde_json::from_str(&content).unwrap();

        assert!(config.environments.contains_key("Production"));
    }

    #[tokio::test]
    async fn test_invalid_config_handling() {
        let invalid_json = r#"{ "invalid": json structure }"#;

        let result: Result<HostsConfig, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_empty_config() {
        let empty_config = HostsConfig {
            environments: std::collections::HashMap::new(),
        };

        let all_hosts = empty_config.filter_hosts(None, None, None);
        assert_eq!(all_hosts.len(), 0);
    }

    #[tokio::test]
    async fn test_config_with_special_characters() {
        let config_with_special = r#"
        {
            "Test-Env": {
                "Region_With_Underscores": {
                    "Type-With-Dashes": {
                        "SERVER_WITH_NUMBERS_123": {
                            "alias": "user@server-with-dashes.example.com",
                            "env": "TEST"
                        }
                    }
                }
            }
        }"#;

        let config: Result<HostsConfig, _> = serde_json::from_str(config_with_special);
        assert!(config.is_ok());

        let config = config.unwrap();
        let hosts = config.filter_hosts(Some(&"Test-Env".to_string()), None, None);
        assert_eq!(hosts.len(), 1);
    }

    #[tokio::test]
    async fn test_host_entry_fields() {
        let host_entry = HostEntry {
            alias: "testuser@testhost.com".to_string(),
            env: "TESTING".to_string(),
        };

        assert_eq!(host_entry.alias, "testuser@testhost.com");
        assert_eq!(host_entry.env, "TESTING");
    }

    #[tokio::test]
    async fn test_config_deep_nesting() {
        let config = create_test_config();

        // Tester l'accès aux niveaux profonds
        let web_server = &config.environments["Production"]["Region-A"]["Public"]["WEB_SERVER_01"];
        assert_eq!(web_server.alias, "web01@prod-web-01.example.com");

        let db_server = &config.environments["Production"]["Region-A"]["Private"]["DATABASE_01"];
        assert_eq!(db_server.alias, "db01@prod-db-01.example.com");
    }

    #[tokio::test]
    async fn test_case_sensitivity() {
        let config = create_test_config();

        // Les filtres sont sensibles à la casse
        let prod_hosts = config.filter_hosts(Some(&"Production".to_string()), None, None);
        let prod_lower = config.filter_hosts(Some(&"production".to_string()), None, None);

        assert_ne!(prod_hosts.len(), prod_lower.len());
        assert_eq!(prod_lower.len(), 0); // Pas de match avec la casse différente
    }
}
