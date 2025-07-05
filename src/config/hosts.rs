use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use dirs::home_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostEntry {
    pub alias: String,
    pub env: String,
}

pub type HostGroup = HashMap<String, HostEntry>;
pub type ServerType = HashMap<String, HostGroup>;
pub type Region = HashMap<String, ServerType>;
pub type Environment = HashMap<String, Region>;

#[derive(Debug, Serialize, Deserialize)]
pub struct HostsConfig {
    #[serde(flatten)]
    pub environments: Environment,
}

impl HostsConfig {
    /// Charge la configuration depuis ~/.ssh/hosts.json
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        
        if !config_path.exists() {
            anyhow::bail!(
                "Fichier de configuration non trouvé: {}\n\
                Créez ce fichier avec la structure des serveurs SSH.\n\
                Voir examples/hosts.json pour un exemple.",
                config_path.display()
            );
        }

        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Impossible de lire {}", config_path.display()))?;

        let config: HostsConfig = serde_json::from_str(&content)
            .with_context(|| format!("Erreur de parsing JSON dans {}", config_path.display()))?;

        Ok(config)
    }

    /// Retourne le chemin vers le fichier de configuration
    pub fn get_config_path() -> Result<PathBuf> {
        let home = home_dir().context("Impossible de déterminer le répertoire home")?;
        Ok(home.join(".ssh").join("hosts.json"))
    }

    /// Filtre les hôtes selon les critères
    pub fn filter_hosts(
        &self,
        env_filter: Option<&String>,
        region_filter: Option<&String>,
        type_filter: Option<&String>,
    ) -> Vec<(String, &HostEntry)> {
        let mut results = Vec::new();

        for (env_name, regions) in &self.environments {
            // Filtrer par environnement
            if let Some(env) = env_filter {
                if env_name != env {
                    continue;
                }
            }

            for (region_name, server_types) in regions {
                // Filtrer par région
                if let Some(region) = region_filter {
                    if region_name != region {
                        continue;
                    }
                }

                for (type_name, hosts) in server_types {
                    // Filtrer par type
                    if let Some(server_type) = type_filter {
                        if type_name != server_type {
                            continue;
                        }
                    }

                    for (host_name, host_entry) in hosts {
                        let full_name = format!("{}:{}:{}:{}", env_name, region_name, type_name, host_name);
                        results.push((full_name, host_entry));
                    }
                }
            }
        }

        results
    }

    /// Affiche la liste des hôtes
    pub fn display_hosts(&self, env_filter: Option<&String>) {
        println!("📋 Liste des serveurs disponibles:\n");

        for (env_name, regions) in &self.environments {
            if let Some(env) = env_filter {
                if env_name != env {
                    continue;
                }
            }

            println!("🌍 {}", env_name);
            
            for (region_name, server_types) in regions {
                println!("  📍 {}", region_name);
                
                for (type_name, hosts) in server_types {
                    println!("    📂 {}", type_name);
                    
                    for (host_name, host_entry) in hosts {
                        println!("      🖥️  {} → {}", host_name, host_entry.alias);
                    }
                }
                println!();
            }
        }
    }

    /// Retourne tous les hôtes sous forme de liste plate
    pub fn get_all_hosts(&self) -> Vec<(String, &HostEntry)> {
        self.filter_hosts(None, None, None)
    }

    /// Compte le nombre total d'hôtes
    pub fn count_hosts(&self) -> usize {
        self.get_all_hosts().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_config_parsing() {
        let json_content = r#"
        {
            "Production": {
                "Region-A": {
                    "Public": {
                        "WEB_01": {
                            "alias": "web01@prod.example.com",
                            "env": "PROD"
                        }
                    }
                }
            }
        }
        "#;

        let config: HostsConfig = serde_json::from_str(json_content).unwrap();
        assert_eq!(config.count_hosts(), 1);
    }

    #[test]
    fn test_host_filtering() {
        let json_content = r#"
        {
            "Production": {
                "Region-A": {
                    "Public": {
                        "WEB_01": {
                            "alias": "web01@prod.example.com",
                            "env": "PROD"
                        }
                    },
                    "Private": {
                        "DB_01": {
                            "alias": "db01@prod.example.com",
                            "env": "PROD"
                        }
                    }
                }
            },
            "Staging": {
                "Region-A": {
                    "Public": {
                        "WEB_01": {
                            "alias": "web01@stage.example.com",
                            "env": "STAGE"
                        }
                    }
                }
            }
        }
        "#;

        let config: HostsConfig = serde_json::from_str(json_content).unwrap();
        
        // Test filtrage par environnement
        let prod_hosts = config.filter_hosts(Some(&"Production".to_string()), None, None);
        assert_eq!(prod_hosts.len(), 2);

        // Test filtrage par type
        let public_hosts = config.filter_hosts(None, None, Some(&"Public".to_string()));
        assert_eq!(public_hosts.len(), 2);

        // Test filtrage combiné
        let prod_public = config.filter_hosts(
            Some(&"Production".to_string()), 
            None, 
            Some(&"Public".to_string())
        );
        assert_eq!(prod_public.len(), 1);
    }
}
