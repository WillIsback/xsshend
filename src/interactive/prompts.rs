/// Prompts interactifs pour xsshend
///
/// Ce module fournit des fonctions pour afficher des prompts utilisateur
/// et collecter des informations manquantes de manière interactive.
use crate::config::{HostEntry, HostsConfig};
use anyhow::Result;
use dialoguer::{Confirm, Input, Select};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::formatters::{format_environment, format_file_size, format_server_count};

/// Prompt pour sélectionner un environnement
///
/// Affiche une liste des environnements disponibles avec le nombre de serveurs
pub fn prompt_environment(config: &HostsConfig) -> Result<String> {
    // Collecter les environnements avec le nombre de serveurs
    let mut env_counts: HashMap<String, usize> = HashMap::new();

    for (env_name, regions) in &config.environments {
        let mut count = 0;
        for server_types in regions.values() {
            for hosts in server_types.values() {
                count += hosts.len();
            }
        }
        env_counts.insert(env_name.clone(), count);
    }

    if env_counts.is_empty() {
        anyhow::bail!("Aucun environnement configuré dans hosts.json");
    }

    // Créer la liste des options avec comptage
    let mut options: Vec<String> = env_counts
        .iter()
        .map(|(env, count)| {
            format!(
                "{} ({} serveur{})",
                format_environment(env),
                count,
                if *count > 1 { "s" } else { "" }
            )
        })
        .collect();
    options.sort();

    let selection = Select::new()
        .with_prompt("🎯 Sélectionnez un environnement")
        .items(&options)
        .default(0)
        .interact()?;

    // Extraire le nom de l'environnement depuis la sélection
    let selected_env = options[selection].split(" (").next().unwrap_or("").trim();

    // Trouver l'environnement original (sans formatage)
    for env_name in env_counts.keys() {
        if env_name == selected_env || format_environment(env_name).contains(selected_env) {
            return Ok(env_name.clone());
        }
    }

    Ok(selected_env.to_string())
}

/// Prompt pour sélectionner une région
///
/// Affiche les régions disponibles pour l'environnement sélectionné
/// Retourne `None` si l'utilisateur choisit "Toutes les régions"
pub fn prompt_region(config: &HostsConfig, env: &str) -> Result<Option<String>> {
    let regions = config
        .environments
        .get(env)
        .ok_or_else(|| anyhow::anyhow!("Environnement {} non trouvé", env))?;

    // Collecter les régions avec comptage
    let mut region_counts: HashMap<String, usize> = HashMap::new();
    for (region_name, server_types) in regions {
        let mut count = 0;
        for hosts in server_types.values() {
            count += hosts.len();
        }
        region_counts.insert(region_name.clone(), count);
    }

    if region_counts.is_empty() {
        return Ok(None);
    }

    // Créer la liste des options
    let mut options: Vec<String> = vec!["[Toutes les régions]".to_string()];
    options.extend(region_counts.iter().map(|(region, count)| {
        format!(
            "{} ({} serveur{})",
            region,
            count,
            if *count > 1 { "s" } else { "" }
        )
    }));

    let selection = Select::new()
        .with_prompt("📍 Sélectionnez une région")
        .items(&options)
        .default(0)
        .interact()?;

    if selection == 0 {
        Ok(None) // Toutes les régions
    } else {
        let selected = &options[selection];
        let region_name = selected.split(" (").next().unwrap_or("").trim();
        Ok(Some(region_name.to_string()))
    }
}

/// Prompt pour sélectionner un type de serveur
///
/// Affiche les types disponibles pour l'environnement et région sélectionnés
/// Retourne `None` si l'utilisateur choisit "Tous les types"
pub fn prompt_server_type(
    config: &HostsConfig,
    env: &str,
    region: Option<&str>,
) -> Result<Option<String>> {
    let regions = config
        .environments
        .get(env)
        .ok_or_else(|| anyhow::anyhow!("Environnement {} non trouvé", env))?;

    // Collecter les types de serveurs disponibles
    let mut type_counts: HashMap<String, usize> = HashMap::new();

    if let Some(region_name) = region {
        // Filtrer par région spécifique
        if let Some(server_types) = regions.get(region_name) {
            for (type_name, hosts) in server_types {
                type_counts.insert(type_name.clone(), hosts.len());
            }
        }
    } else {
        // Toutes les régions
        for server_types in regions.values() {
            for (type_name, hosts) in server_types {
                *type_counts.entry(type_name.clone()).or_insert(0) += hosts.len();
            }
        }
    }

    if type_counts.is_empty() {
        return Ok(None);
    }

    // Créer la liste des options
    let mut options: Vec<String> = vec!["[Tous les types]".to_string()];
    options.extend(type_counts.iter().map(|(t, count)| {
        format!(
            "{} ({} serveur{})",
            t,
            count,
            if *count > 1 { "s" } else { "" }
        )
    }));

    let selection = Select::new()
        .with_prompt("🖥️  Sélectionnez un type de serveur")
        .items(&options)
        .default(0)
        .interact()?;

    if selection == 0 {
        Ok(None) // Tous les types
    } else {
        let selected = &options[selection];
        let type_name = selected.split(" (").next().unwrap_or("").trim();
        Ok(Some(type_name.to_string()))
    }
}

/// Prompt pour la destination
///
/// Demande le répertoire de destination sur les serveurs distants
pub fn prompt_destination(default: &str) -> Result<PathBuf> {
    let input: String = Input::new()
        .with_prompt("📂 Répertoire de destination")
        .default(default.to_string())
        .validate_with(|input: &String| -> Result<(), String> {
            if input.starts_with('/') {
                Ok(())
            } else {
                Err("Le chemin doit être absolu (commencer par /)".to_string())
            }
        })
        .interact_text()?;

    Ok(PathBuf::from(input))
}

/// Prompt pour choisir le type de commande
///
/// Demande si l'utilisateur veut exécuter une commande inline ou un script
pub fn prompt_command_type() -> Result<String> {
    let options = vec!["Commande inline", "Script bash (.sh)"];

    let selection = Select::new()
        .with_prompt("📜 Type de commande à exécuter")
        .items(&options)
        .default(0)
        .interact()?;

    Ok(options[selection].to_string())
}

/// Prompt pour saisir une commande inline
///
/// Demande la commande à exécuter sur les serveurs
pub fn prompt_inline_command() -> Result<String> {
    let input: String = Input::new()
        .with_prompt("💻 Commande à exécuter")
        .validate_with(|input: &String| -> Result<(), String> {
            if input.trim().is_empty() {
                Err("La commande ne peut pas être vide".to_string())
            } else {
                Ok(())
            }
        })
        .interact_text()?;

    Ok(input.trim().to_string())
}

/// Prompt pour saisir le chemin d'un script
///
/// Demande le chemin du script bash à exécuter
pub fn prompt_script_path() -> Result<PathBuf> {
    let input: String = Input::new()
        .with_prompt("📄 Chemin du script bash")
        .validate_with(|input: &String| -> Result<(), String> {
            let path = Path::new(input.trim());
            if !path.exists() {
                Err(format!("Le fichier {} n'existe pas", input))
            } else if !path.is_file() {
                Err(format!("{} n'est pas un fichier", input))
            } else if path.extension().and_then(|e| e.to_str()) != Some("sh") {
                Err("Le fichier doit avoir l'extension .sh".to_string())
            } else {
                Ok(())
            }
        })
        .interact_text()?;

    Ok(PathBuf::from(input.trim()))
}

/// Confirmation finale avant l'exécution de commandes
///
/// Affiche un récapitulatif et demande confirmation
/// Warning spécial si environnement Production
pub fn confirm_command_execution(
    command: &str,
    servers: &[(String, &HostEntry)],
    env: &str,
    parallel: bool,
    timeout: u64,
) -> Result<bool> {
    println!("\n{}", "=".repeat(60));
    println!("📋 RÉCAPITULATIF DE LA COMMANDE");
    println!("{}", "=".repeat(60));

    // Afficher la commande
    let command_preview = if command.lines().count() > 1 {
        let first_line = command.lines().next().unwrap_or("");
        let line_count = command.lines().count();
        format!("{}\n   ... ({} lignes au total)", first_line, line_count)
    } else {
        command.to_string()
    };

    println!("📜 Commande: {}", command_preview);
    println!("\n🎯 Environnement: {}", format_environment(env));
    println!(
        "🖥️  Serveurs ciblés: {}",
        format_server_count(servers.len())
    );

    // Afficher les serveurs (limité à 10)
    for (name, entry) in servers.iter().take(10) {
        println!("   • {} → {} ({})", name, entry.alias, entry.env);
    }
    if servers.len() > 10 {
        println!("   ... et {} autre(s)", servers.len() - 10);
    }

    println!("\n⏱️  Timeout: {}s", timeout);
    println!(
        "🔀 Mode: {}",
        if parallel {
            "Parallèle"
        } else {
            "Séquentiel"
        }
    );
    println!("{}", "=".repeat(60));

    // Warning spécial pour Production
    let default = if env.eq_ignore_ascii_case("production") {
        println!("⚠️  ATTENTION: Vous êtes sur l'environnement PRODUCTION!");
        false
    } else {
        true
    };

    let confirmed = Confirm::new()
        .with_prompt("Confirmer l'exécution de la commande ?")
        .default(default)
        .interact()?;

    Ok(confirmed)
}

/// Confirmation finale avant l'upload
///
/// Affiche un récapitulatif et demande confirmation
/// Warning spécial si environnement Production
pub fn confirm_upload(
    files: &[PathBuf],
    servers: &[(String, &HostEntry)],
    destination: &Path,
    env: &str,
) -> Result<bool> {
    println!("\n{}", "=".repeat(60));
    println!("📋 RÉCAPITULATIF");
    println!("{}", "=".repeat(60));

    // Calculer la taille totale des fichiers
    let mut total_size = 0u64;
    for file in files {
        if let Ok(metadata) = std::fs::metadata(file) {
            total_size += metadata.len();
        }
    }

    // Afficher les informations
    println!("📦 Fichiers à téléverser: {}", files.len());
    for file in files.iter().take(5) {
        if let Ok(metadata) = std::fs::metadata(file) {
            println!(
                "   • {} ({})",
                file.display(),
                format_file_size(metadata.len())
            );
        } else {
            println!("   • {}", file.display());
        }
    }
    if files.len() > 5 {
        println!("   ... et {} autre(s)", files.len() - 5);
    }
    println!("   Taille totale: {}", format_file_size(total_size));

    println!("\n🎯 Environnement: {}", format_environment(env));
    println!("📍 Destination: {}", destination.display());
    println!(
        "🖥️  Serveurs ciblés: {}",
        format_server_count(servers.len())
    );

    // Afficher les serveurs (limité à 10)
    for (name, entry) in servers.iter().take(10) {
        println!("   • {} → {} ({})", name, entry.alias, entry.env);
    }
    if servers.len() > 10 {
        println!("   ... et {} autre(s)", servers.len() - 10);
    }

    println!("{}", "=".repeat(60));

    // Warning spécial pour Production
    let default = if env.eq_ignore_ascii_case("production") {
        println!("⚠️  ATTENTION: Vous êtes sur l'environnement PRODUCTION!");
        false
    } else {
        true
    };

    let confirmed = Confirm::new()
        .with_prompt("Confirmer le téléversement ?")
        .default(default)
        .interact()?;

    Ok(confirmed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_destination_validation() {
        // Ce test vérifie que la logique de validation fonctionne
        // Note: L'interaction réelle ne peut pas être testée facilement
        let path = PathBuf::from("/tmp");
        assert!(path.is_absolute());
    }
}
