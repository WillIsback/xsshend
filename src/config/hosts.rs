use anyhow::{Context, Result};
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

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
            // Tenter de créer le fichier d'exemple automatiquement
            if let Err(e) = Self::create_default_config() {
                anyhow::bail!(
                    "Fichier de configuration non trouvé: {}\n\
                    Erreur lors de la création automatique: {}\n\
                    Créez ce fichier manuellement avec la structure des serveurs SSH.\n\
                    Voir examples/hosts.json pour un exemple.",
                    config_path.display(),
                    e
                );
            } else {
                println!(
                    "✅ Fichier de configuration créé automatiquement: {}",
                    config_path.display()
                );
                println!("📝 Éditez ce fichier pour ajouter vos serveurs SSH.");
            }
        }

        // Vérifier et proposer la génération de clés SSH si nécessaire
        if let Err(e) = Self::ensure_ssh_keys() {
            eprintln!("⚠️  Avertissement concernant les clés SSH: {}", e);
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
            if let Some(env) = env_filter
                && env_name != env
            {
                continue;
            }

            for (region_name, server_types) in regions {
                // Filtrer par région
                if let Some(region) = region_filter
                    && region_name != region
                {
                    continue;
                }

                for (type_name, hosts) in server_types {
                    // Filtrer par type
                    if let Some(server_type) = type_filter
                        && type_name != server_type
                    {
                        continue;
                    }

                    for (host_name, host_entry) in hosts {
                        let full_name =
                            format!("{}:{}:{}:{}", env_name, region_name, type_name, host_name);
                        results.push((full_name, host_entry));
                    }
                }
            }
        }

        results
    }

    /// Affiche toutes les cibles SSH disponibles de manière hiérarchique
    pub fn display_all_targets(&self) {
        let mut total_targets = 0;

        for (env_name, env) in &self.environments {
            println!("📁 {} (--env {})", env_name, env_name);

            for (region_name, region) in env {
                println!("  📂 {} (--region {})", region_name, region_name);

                for (type_name, server_type) in region {
                    println!("    📂 {} (--type {})", type_name, type_name);

                    for (server_name, host_entry) in server_type {
                        println!(
                            "      🖥️  {} → {} ({})",
                            server_name, host_entry.alias, host_entry.env
                        );
                        total_targets += 1;
                    }
                }
            }
            println!(); // Ligne vide entre environnements
        }

        println!("📊 Total: {} cibles disponibles", total_targets);
        println!("\n💡 Exemples d'utilisation:");
        println!("   xsshend upload --env Production file.txt");
        println!("   xsshend upload --env Staging --region Region-A file.txt");
        println!("   xsshend upload --region Region-A --type Public file.txt");
    }

    /// Crée un fichier de configuration par défaut avec des exemples
    pub fn create_default_config() -> Result<()> {
        let config_path = Self::get_config_path()?;

        // Créer le répertoire ~/.ssh s'il n'existe pas
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Impossible de créer le répertoire {}", parent.display())
            })?;
        }

        // Contenu d'exemple basé sur examples/hosts.json
        let example_config = Self::get_example_config();

        // Écrire le fichier
        fs::write(&config_path, example_config)
            .with_context(|| format!("Impossible d'écrire {}", config_path.display()))?;

        Ok(())
    }

    /// Récupère le contenu d'exemple du fichier hosts.json
    pub fn get_example_config() -> &'static str {
        include_str!("../../examples/hosts.json")
    }

    /// Vérifie l'existence de clés SSH et propose leur génération si nécessaire
    pub fn ensure_ssh_keys() -> Result<()> {
        let home = home_dir().context("Impossible de déterminer le répertoire home")?;
        let ssh_dir = home.join(".ssh");

        // Vérifier si le répertoire .ssh existe
        if !ssh_dir.exists() {
            fs::create_dir_all(&ssh_dir).with_context(|| {
                format!("Impossible de créer le répertoire {}", ssh_dir.display())
            })?;
            println!("📁 Répertoire ~/.ssh créé");
        }

        // Clés SSH typiques à vérifier (par ordre de préférence)
        let key_paths = [
            ("ed25519", ssh_dir.join("id_ed25519")),
            ("rsa", ssh_dir.join("id_rsa")),
            ("ecdsa", ssh_dir.join("id_ecdsa")),
        ];

        // Vérifier si une clé existe déjà
        for (key_type, key_path) in &key_paths {
            if key_path.exists() {
                println!(
                    "🔐 Clé SSH {} trouvée: {}",
                    key_type.to_uppercase(),
                    key_path.display()
                );
                return Ok(());
            }
        }

        // Aucune clé trouvée, proposer de générer une clé
        println!("❌ Aucune clé SSH trouvée dans ~/.ssh/");
        println!("🔑 Pour utiliser xsshend, vous avez besoin d'une clé SSH privée.");

        // Demander à l'utilisateur s'il veut générer une clé
        print!("Voulez-vous générer une nouvelle clé SSH Ed25519 ? (o/N): ");
        io::stdout()
            .flush()
            .context("Impossible de vider le buffer stdout")?;

        let mut response = String::new();
        io::stdin()
            .read_line(&mut response)
            .context("Impossible de lire la réponse de l'utilisateur")?;

        let response = response.trim().to_lowercase();
        if response == "o" || response == "oui" || response == "y" || response == "yes" {
            Self::generate_ssh_key(&ssh_dir)?;
        } else {
            println!("⚠️  Génération de clé SSH ignorée.");
            println!("💡 Vous pouvez générer une clé manuellement avec :");
            println!("   ssh-keygen -t ed25519 -C \"votre_email@example.com\"");
        }

        Ok(())
    }

    /// Génère une nouvelle clé SSH Ed25519
    fn generate_ssh_key(ssh_dir: &Path) -> Result<()> {
        let key_path = ssh_dir.join("id_ed25519");

        // Demander une adresse email pour le commentaire
        print!("Entrez votre adresse email (optionnel, pour identifier la clé): ");
        io::stdout()
            .flush()
            .context("Impossible de vider le buffer stdout")?;

        let mut email = String::new();
        io::stdin()
            .read_line(&mut email)
            .context("Impossible de lire l'adresse email")?;
        let email = email.trim();

        // Préparer la commande ssh-keygen
        let mut cmd = Command::new("ssh-keygen");
        cmd.arg("-t")
            .arg("ed25519")
            .arg("-f")
            .arg(&key_path)
            .arg("-N")
            .arg(""); // Pas de passphrase par défaut pour simplifier

        if !email.is_empty() {
            cmd.arg("-C").arg(email);
        }

        println!("🔄 Génération de la clé SSH en cours...");

        // Exécuter la commande
        let output = cmd
            .output()
            .context("Impossible d'exécuter ssh-keygen. Assurez-vous qu'OpenSSH est installé.")?;

        if output.status.success() {
            println!(
                "✅ Clé SSH Ed25519 générée avec succès: {}",
                key_path.display()
            );
            println!("📋 Clé publique: {}.pub", key_path.display());

            // Afficher la clé publique pour faciliter l'ajout aux serveurs
            if let Ok(public_key) = fs::read_to_string(format!("{}.pub", key_path.display())) {
                println!("\n📄 Contenu de votre clé publique:");
                println!("{}", public_key.trim());
                println!("\n💡 Copiez cette clé publique sur vos serveurs avec:");
                println!("   ssh-copy-id -i ~/.ssh/id_ed25519.pub user@hostname");
            }
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Échec de la génération de clé SSH: {}", stderr);
        }

        Ok(())
    }

    // Unused method - commented out for optimization
    // pub fn count_hosts(&self) -> usize {
    //     self.get_all_hosts().len()
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(config.filter_hosts(None, None, None).len(), 1);
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
            Some(&"Public".to_string()),
        );
        assert_eq!(prod_public.len(), 1);
    }
}
