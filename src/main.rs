use anyhow::Result;
use clap::{Arg, Command};
use std::path::PathBuf;

mod config;
mod ssh;
mod ui;
mod core;
mod utils;

use config::HostsConfig;
use ui::prompts;
use core::uploader::Uploader;

fn main() -> Result<()> {
    env_logger::init();

    let app = Command::new("xsshend")
        .version("0.1.0")
        .about("Outil Rust de téléversement multi-SSH avec interface TUI")
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
                    Arg::new("env")
                        .long("env")
                        .help("Environnement cible (Production, Staging, Development)")
                        .value_name("ENV")
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
                .arg(
                    Arg::new("env")
                        .long("env")
                        .help("Filtrer par environnement")
                        .value_name("ENV")
                )
        );

    let matches = app.get_matches();

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
                // Mode interactif
                prompts::select_hosts(&config)?
            } else {
                // Mode filtré par arguments
                let env = sub_matches.get_one::<String>("env");
                let region = sub_matches.get_one::<String>("region");
                let server_type = sub_matches.get_one::<String>("type");
                
                config.filter_hosts(env, region, server_type)
            };

            if target_hosts.is_empty() {
                println!("❌ Aucun serveur trouvé avec les critères spécifiés");
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
            } else if sub_matches.get_flag("interactive") {
                // Mode interactif avec confirmation
                uploader.upload_interactive(&file_refs, &target_hosts, destination)?;
            } else {
                // Mode direct
                uploader.upload_files(&file_refs, &target_hosts, destination)?;
            }
        }
        Some(("list", sub_matches)) => {
            let config = HostsConfig::load()?;
            let env_filter = sub_matches.get_one::<String>("env");
            
            config.display_hosts(env_filter);
        }
        _ => {
            println!("Utilisez 'xsshend --help' pour voir les commandes disponibles");
        }
    }

    Ok(())
}
