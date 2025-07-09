use anyhow::Result;
use clap::{Arg, Command};
use std::path::PathBuf;

mod config;
mod core;
mod ssh;
mod ui;
mod utils;

use config::HostsConfig;
use core::uploader::Uploader;
use ui::MultiScreenTuiApp;

fn main() -> Result<()> {
    // Ne pas initialiser de logger ici - sera fait selon le mode

    let app = Command::new("xsshend")
        .version("0.1.0")
        .about("Outil Rust de téléversement multi-SSH avec interface TUI")
        // Arguments globaux pour mode interactif direct
        .arg(
            Arg::new("interactive")
                .long("interactive")
                .short('i')
                .help("Lance le mode interactif pour sélectionner fichiers et serveurs")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("list")
                .long("list")
                .short('l')
                .help("Affiche la liste de toutes les cibles disponibles")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("online-only")
                .long("online-only")
                .short('o')
                .help("En mode TUI, affiche seulement les serveurs en ligne (avec timeout de connectivité)")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("files")
                .help("Fichiers à téléverser (optionnel en mode interactif)")
                .num_args(0..)
                .value_name("FILE"),
        )
        .subcommand(
            Command::new("upload")
                .about("Téléverse des fichiers vers plusieurs serveurs SSH")
                .arg(
                    Arg::new("files")
                        .help("Fichiers à téléverser")
                        .required(true)
                        .num_args(1..)
                        .value_name("FILE"),
                )
                .arg(
                    Arg::new("env")
                        .long("env")
                        .help("Environnement spécifique (Production, Staging, etc.)")
                        .value_name("ENV"),
                )
                .arg(
                    Arg::new("region")
                        .long("region")
                        .help("Région spécifique")
                        .value_name("REGION"),
                )
                .arg(
                    Arg::new("type")
                        .long("type")
                        .help("Type de serveurs (Public, Private)")
                        .value_name("TYPE"),
                )
                .arg(
                    Arg::new("dest")
                        .long("dest")
                        .help("Répertoire de destination")
                        .value_name("PATH")
                        .default_value("/tmp/"),
                )
                .arg(
                    Arg::new("interactive")
                        .long("interactive")
                        .short('i')
                        .help("Mode interactif pour sélectionner les serveurs")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("dry-run")
                        .long("dry-run")
                        .help("Simulation sans transfert réel")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("verbose")
                        .long("verbose")
                        .short('v')
                        .help("Mode verbeux")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("ssh-key")
                        .long("ssh-key")
                        .short('k')
                        .help("Clé SSH à utiliser (nom du fichier sans extension, ex: id_ed25519)")
                        .value_name("KEY_NAME"),
                )
                .arg(
                    Arg::new("ssh-key-interactive")
                        .long("ssh-key-interactive")
                        .help("Sélection interactive de la clé SSH à utiliser")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(Command::new("list").about("Liste les serveurs disponibles"));

    let matches = app.get_matches();

    // Gérer le flag --list/-l en priorité
    if matches.get_flag("list") {
        println!("🔍 Liste des cibles SSH disponibles:\n");

        // Charger la configuration avec vérification
        let config = match HostsConfig::load() {
            Ok(config) => config,
            Err(e) => {
                eprintln!("❌ Erreur lors du chargement de la configuration: {}", e);
                std::process::exit(1);
            }
        };

        config.display_all_targets();
        return Ok(());
    }

    // Si aucune sous-commande n'est fournie, lancer le TUI par défaut
    if matches.subcommand().is_none() && !matches.get_flag("interactive") {
        // Le logger TUI sera initialisé dans MultiScreenTuiApp

        // Charger la configuration avec vérification
        let config = match HostsConfig::load() {
            Ok(config) => config,
            Err(e) => {
                eprintln!("❌ Erreur lors du chargement de la configuration: {}", e);
                std::process::exit(1);
            }
        };

        // Lancer le TUI multi-écrans
        let mut tui_app = if matches.get_flag("online-only") {
            MultiScreenTuiApp::new_with_connectivity_check(&config, 5)? // timeout 5s
        } else {
            MultiScreenTuiApp::new(&config)?
        };

        tui_app.run()?;
        return Ok(());
    }

    // Initialiser env_logger pour le mode CLI
    env_logger::init();

    // Vérifier si le mode interactif global est activé
    if matches.get_flag("interactive") {
        log::info!("🚀 xsshend - Mode Interactif");

        // Charger la configuration avec vérification
        let config = match HostsConfig::load() {
            Ok(config) => config,
            Err(e) => {
                eprintln!("❌ Erreur lors du chargement de la configuration: {}", e);
                std::process::exit(1);
            }
        };

        // Sélectionner les fichiers (de la ligne de commande ou interactivement)
        let files: Vec<PathBuf> = if let Some(file_args) = matches.get_many::<String>("files") {
            file_args.map(PathBuf::from).collect()
        } else {
            Vec::new() // On laissera l'interface TUI gérer la sélection des fichiers
        };

        // Lancer l'interface TUI hiérarchique complète
        let mut tui_app = if matches.get_flag("online-only") {
            log::info!("🔍 Mode connectivité: vérification des serveurs en ligne...");
            MultiScreenTuiApp::new_with_connectivity_check(&config, 5)? // timeout 5s
        } else {
            MultiScreenTuiApp::new(&config)?
        };

        // Si des fichiers sont fournis en ligne de commande, les pré-sélectionner
        if !files.is_empty() {
            tui_app.set_selected_files(files)?;
        }

        tui_app.run()?;

        return Ok(());
    }

    match matches.subcommand() {
        Some(("upload", sub_matches)) => {
            let files: Vec<PathBuf> = sub_matches
                .get_many::<String>("files")
                .unwrap()
                .map(PathBuf::from)
                .collect();

            println!("🚀 xsshend - Téléversement Multi-SSH");

            // Charger la configuration avec vérification
            let config = match HostsConfig::load() {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("❌ Erreur lors du chargement de la configuration: {}", e);
                    std::process::exit(1);
                }
            };

            // Déterminer les serveurs cibles
            let target_hosts = if sub_matches.get_flag("interactive") {
                // Mode interactif - lancer l'interface TUI hiérarchique
                log::info!("🚀 Mode interactif - Interface TUI hiérarchique");

                let mut tui_app = MultiScreenTuiApp::new(&config)?;
                tui_app.set_selected_files(files.clone())?;
                tui_app.run()?;

                return Ok(());
            } else {
                // Mode filtré par arguments
                let env = sub_matches.get_one::<String>("env");
                let region = sub_matches.get_one::<String>("region");
                let server_type = sub_matches.get_one::<String>("type");

                config.filter_hosts(env, region, server_type)
            };

            if target_hosts.is_empty() {
                log::error!("❌ Aucun serveur trouvé avec les critères spécifiés");
                return Ok(());
            }

            // Gestion de la sélection de clé SSH
            let selected_ssh_key = if sub_matches.get_flag("ssh-key-interactive") {
                // Sélection interactive de la clé SSH
                println!("🔑 Sélection de la clé SSH...");
                use crate::ssh::keys::SshKeyManager;

                let key_manager = match SshKeyManager::new() {
                    Ok(manager) => manager,
                    Err(e) => {
                        log::error!(
                            "❌ Impossible d'initialiser le gestionnaire de clés SSH: {}",
                            e
                        );
                        std::process::exit(1);
                    }
                };

                match key_manager.select_key_interactive() {
                    Ok(Some(key)) => Some(key.clone()),
                    Ok(None) => {
                        log::warn!("⚠️ Aucune clé SSH sélectionnée");
                        None
                    }
                    Err(e) => {
                        log::error!("❌ Erreur lors de la sélection de clé SSH: {}", e);
                        std::process::exit(1);
                    }
                }
            } else if let Some(key_name) = sub_matches.get_one::<String>("ssh-key") {
                // Clé spécifiée par nom
                println!("🔑 Recherche de la clé SSH: {}", key_name);
                use crate::ssh::keys::SshKeyManager;

                let key_manager = match SshKeyManager::new() {
                    Ok(manager) => manager,
                    Err(e) => {
                        log::error!(
                            "❌ Impossible d'initialiser le gestionnaire de clés SSH: {}",
                            e
                        );
                        std::process::exit(1);
                    }
                };

                match key_manager.get_key_by_name(key_name) {
                    Some(key) => {
                        println!("✅ Clé SSH trouvée: {}", key.description());
                        Some(key.clone())
                    }
                    None => {
                        log::error!("❌ Clé SSH '{}' non trouvée", key_name);
                        std::process::exit(1);
                    }
                }
            } else {
                // Pas de clé spécifiée, utiliser le comportement par défaut (ssh-agent)
                None
            };

            // Destination et fichiers
            let destination = sub_matches.get_one::<String>("dest").unwrap();
            let file_refs: Vec<&std::path::Path> = files.iter().map(|p| p.as_path()).collect();

            // Créer l'uploader
            let mut uploader = Uploader::new();

            // Si une clé SSH spécifique a été sélectionnée, l'indiquer
            if let Some(ref key) = selected_ssh_key {
                log::info!("🔑 Utilisation de la clé SSH: {}", key.description());
                // TODO: Passer la clé à l'uploader une fois que le SshConnectionPool supporte les clés spécifiques
            }

            if sub_matches.get_flag("dry-run") {
                // Mode dry-run - simulation
                uploader.dry_run(&file_refs, &target_hosts, destination)?;
            } else {
                // Mode direct avec pool SSH optimisé
                uploader.upload_files(&file_refs, &target_hosts, destination)?;
            }
        }
        Some(("list", _sub_matches)) => {
            println!("🔍 Liste des cibles SSH disponibles:\n");

            let config = match HostsConfig::load() {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("❌ Erreur lors du chargement de la configuration: {}", e);
                    std::process::exit(1);
                }
            };

            config.display_all_targets();
        }
        _ => {
            log::info!("Utilisez 'xsshend --help' pour voir les commandes disponibles");
        }
    }

    Ok(())
}
