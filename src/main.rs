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
            println!("Fichiers à téléverser: {:?}", files);

            // Charger la configuration
            let config = HostsConfig::load()?;
            
            if sub_matches.get_flag("interactive") {
                // Mode interactif
                let selected_hosts = prompts::select_hosts(&config)?;
                println!("Serveurs sélectionnés: {:?}", selected_hosts);
            } else {
                // Mode filtré par arguments
                let env = sub_matches.get_one::<String>("env");
                let region = sub_matches.get_one::<String>("region");
                let server_type = sub_matches.get_one::<String>("type");
                
                let filtered_hosts = config.filter_hosts(env, region, server_type);
                println!("Serveurs trouvés: {:?}", filtered_hosts);
            }

            if sub_matches.get_flag("dry-run") {
                println!("🔍 Mode dry-run activé - aucun transfert réel");
                return Ok(());
            }

            // TODO: Implémenter la logique de téléversement
            println!("✅ Téléversement terminé (fonctionnalité à implémenter)");
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
