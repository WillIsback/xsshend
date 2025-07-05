use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, MultiSelect, Select, Input, Confirm};
use std::path::Path;
use crate::config::{HostsConfig, HostEntry};

/// Demande à l'utilisateur de sélectionner des serveurs de manière interactive
pub fn select_hosts(config: &HostsConfig) -> Result<Vec<(String, &HostEntry)>> {
    println!("🎯 Sélection interactive des serveurs\n");

    // Obtenir tous les hôtes disponibles
    let all_hosts = config.get_all_hosts();
    
    if all_hosts.is_empty() {
        println!("❌ Aucun serveur trouvé dans la configuration");
        return Ok(vec![]);
    }

    // Créer les options pour le sélecteur
    let host_options: Vec<String> = all_hosts
        .iter()
        .map(|(name, entry)| format!("{} → {}", name, entry.alias))
        .collect();

    // Interface de sélection multiple
    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Sélectionnez les serveurs cibles")
        .items(&host_options)
        .interact()?;

    // Retourner les hôtes sélectionnés
    let selected_hosts: Vec<(String, &HostEntry)> = selections
        .into_iter()
        .map(|i| all_hosts[i].clone())
        .collect();

    if selected_hosts.is_empty() {
        println!("⚠️  Aucun serveur sélectionné");
    } else {
        println!("✅ {} serveur(s) sélectionné(s)", selected_hosts.len());
    }

    Ok(selected_hosts)
}

/// Demande la passphrase SSH de manière sécurisée
pub fn prompt_passphrase() -> Result<Option<String>> {
    let needs_passphrase = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Votre clé SSH nécessite-t-elle une passphrase ?")
        .default(false)
        .interact()?;

    if needs_passphrase {
        let passphrase = rpassword::prompt_password("🔑 Passphrase SSH: ")?;
        Ok(Some(passphrase))
    } else {
        Ok(None)
    }
}

/// Demande de confirmation avant le téléversement
pub fn confirm_upload(
    files: &[&Path], 
    hosts: &[(String, &HostEntry)],
    destination: &str
) -> Result<bool> {
    println!("\n📋 Récapitulatif du téléversement:");
    println!("   Fichiers: {:?}", files);
    println!("   Destination: {}", destination);
    println!("   Serveurs: {} cibles", hosts.len());
    
    for (name, entry) in hosts {
        println!("     • {} → {}", name, entry.alias);
    }

    let confirmed = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("\n🚀 Confirmer le téléversement ?")
        .default(true)
        .interact()?;

    Ok(confirmed)
}

/// Sélecteur d'environnement interactif
pub fn select_environment(config: &HostsConfig) -> Result<Option<String>> {
    let environments: Vec<String> = config.environments.keys().cloned().collect();
    
    if environments.is_empty() {
        return Ok(None);
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Sélectionnez un environnement")
        .items(&environments)
        .default(0)
        .interact()?;

    Ok(Some(environments[selection].clone()))
}

/// Demande le répertoire de destination
pub fn prompt_destination() -> Result<String> {
    let destination = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Répertoire de destination")
        .default("/tmp/".to_string())
        .interact_text()?;

    Ok(destination)
}
