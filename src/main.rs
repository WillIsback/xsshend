use anyhow::Result;
use clap::{Arg, Command};
use std::path::PathBuf;

mod config;
mod ssh;
mod ui;
mod core;
mod utils;

use config::HostsConfig;
use ui::MultiScreenTuiApp;
use core::uploader::Uploader;
use utils::tui_logger;

fn main() -> Result<()> {
    // Initialiser notre logger TUI-aware au lieu d'env_logger::init()
    tui_logger::init_tui_aware_logger();

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
                        .value_name("FILE")
                )
                .arg(
                    Arg::new("region")
                        .long("region")
                        .help("Région spécifique")
                        .value_name("REGION")
                )
                .arg(
                    Arg::new("type")
                        .long("type")
                        .help("Type de serveurs (Public, Private)")
                        .value_name("TYPE")
                )
                .arg(
                    Arg::new("dest")
                        .long("dest")
                        .help("Répertoire de destination")
                        .value_name("PATH")
                        .default_value("/tmp/")
                )
                .arg(
                    Arg::new("interactive")
                        .long("interactive")
                        .short('i')
                        .help("Mode interactif pour sélectionner les serveurs")
                        .action(clap::ArgAction::SetTrue)
                )
                .arg(
                    Arg::new("dry-run")
                        .long("dry-run")
                        .help("Simulation sans transfert réel")
                        .action(clap::ArgAction::SetTrue)
                )
                .arg(
                    Arg::new("verbose")
                        .long("verbose")
                        .short('v')
                        .help("Mode verbeux")
                        .action(clap::ArgAction::SetTrue)
                )
        )
        .subcommand(
            Command::new("list")
                .about("Liste les serveurs disponibles")
        );

    let matches = app.get_matches();

    // Si aucune sous-commande n'est fournie, lancer le TUI par défaut
    if matches.subcommand().is_none() && !matches.get_flag("interactive") {
        log::info!("🚀 xsshend - Interface Interactive");
        
        // Charger la configuration
        let config = HostsConfig::load()?;
        
        // Lancer le TUI multi-écrans
        MultiScreenTuiApp::launch(&config)?;
        return Ok(());
    }

    // Vérifier si le mode interactif global est activé
    if matches.get_flag("interactive") {
        log::info!("🚀 xsshend - Mode Interactif");
        
        // Charger la configuration
        let config = HostsConfig::load()?;
        
        // Sélectionner les fichiers (de la ligne de commande ou interactivement)
        let files: Vec<PathBuf> = if let Some(file_args) = matches.get_many::<String>("files") {
            file_args.map(PathBuf::from).collect()
        } else {
            Vec::new() // On laissera l'interface TUI gérer la sélection des fichiers
        };

        // Lancer l'interface TUI hiérarchique complète
        let mut tui_app = MultiScreenTuiApp::new(&config)?;
        
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
            
            // Charger la configuration
            let config = HostsConfig::load()?;
            
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
                let region = sub_matches.get_one::<String>("region");
                let server_type = sub_matches.get_one::<String>("type");
                
                config.filter_hosts(None, region, server_type) // env = None
            };

            if target_hosts.is_empty() {
                log::error!("❌ Aucun serveur trouvé avec les critères spécifiés");
                return Ok(());
            }

            // Destination et fichiers
            let destination = sub_matches.get_one::<String>("dest").unwrap();
            let file_refs: Vec<&std::path::Path> = files.iter().map(|p| p.as_path()).collect();

            // Créer l'uploader
            let uploader = Uploader::new();

            if sub_matches.get_flag("dry-run") {
                // Mode dry-run - simulation
                uploader.dry_run(&file_refs, &target_hosts, destination)?;
            } else {
                // Mode direct
                uploader.upload_files(&file_refs, &target_hosts, destination)?;
            }
        }
        Some(("list", _sub_matches)) => {
            let config = HostsConfig::load()?;
            
            config.display_hosts(None); // No env filter
        }
        _ => {
            log::info!("Utilisez 'xsshend --help' pour voir les commandes disponibles");
        }
    }

    Ok(())
}
