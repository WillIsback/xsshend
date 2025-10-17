use anyhow::Result;
use clap::{Arg, Command};
use std::path::PathBuf;

mod config;
mod core;
mod ssh;
mod utils;

use config::HostsConfig;
use core::uploader::Uploader;

fn main() -> Result<()> {
    env_logger::init();

    let app = Command::new("xsshend")
        .version("0.3.2")
        .about("Outil Rust de téléversement multi-SSH (CLI uniquement)")
        .arg(
            Arg::new("list")
                .long("list")
                .short('l')
                .help("Affiche la liste de toutes les cibles disponibles")
                .action(clap::ArgAction::SetTrue),
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
                    Arg::new("dry-run")
                        .long("dry-run")
                        .help("Simulation sans transfert réel")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(Command::new("list").about("Liste les serveurs disponibles"))
        .subcommand(
            Command::new("init")
                .about("Initialise la configuration xsshend et aide à configurer SSH")
                .arg(
                    Arg::new("force")
                        .long("force")
                        .short('f')
                        .help("Remplace la configuration existante")
                        .action(clap::ArgAction::SetTrue),
                ),
        );

    let matches = app.get_matches();

    // Gérer le flag --list/-l en priorité
    if matches.get_flag("list") {
        println!("🔍 Liste des cibles SSH disponibles:\n");

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

    // Si aucune sous-commande n'est fournie, afficher l'aide
    if matches.subcommand().is_none() {
        println!("Utilisez 'xsshend --help' pour voir les commandes disponibles");
        println!("Exemples:");
        println!("  xsshend upload file.txt --env Production");
        println!("  xsshend --list");
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
            let config = match HostsConfig::load() {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("❌ Erreur lors du chargement de la configuration: {}", e);
                    std::process::exit(1);
                }
            };

            // Mode filtré par arguments
            let env = sub_matches.get_one::<String>("env");
            let region = sub_matches.get_one::<String>("region");
            let server_type = sub_matches.get_one::<String>("type");

            let target_hosts = config.filter_hosts(env, region, server_type);

            if target_hosts.is_empty() {
                println!("❌ Aucun serveur trouvé avec les critères spécifiés");
                return Ok(());
            }

            // SSH utilise automatiquement les clés disponibles et ssh-agent
            println!("🔑 Utilisation automatique des clés SSH disponibles");

            // Destination et fichiers
            let destination = sub_matches.get_one::<String>("dest").unwrap();
            let file_refs: Vec<&std::path::Path> = files.iter().map(|p| p.as_path()).collect();

            // Créer l'uploader simple
            let uploader = Uploader::new();

            if sub_matches.get_flag("dry-run") {
                // Mode dry-run - simulation
                uploader.dry_run(&file_refs, &target_hosts, destination)?;
            } else {
                // Mode direct simplifié
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
        Some(("init", sub_matches)) => {
            let force = sub_matches.get_flag("force");
            init_setup(force)?;
        }
        _ => {
            println!("Utilisez 'xsshend --help' pour voir les commandes disponibles");
        }
    }

    Ok(())
}

/// Fonction d'initialisation pour configurer xsshend
fn init_setup(force: bool) -> Result<()> {
    use dirs::home_dir;
    use std::fs;

    println!("🚀 Initialisation de xsshend");
    println!();

    // Vérifier le répertoire home
    let home =
        home_dir().ok_or_else(|| anyhow::anyhow!("Impossible de trouver le répertoire home"))?;
    let ssh_dir = home.join(".ssh");

    // 1. Créer le répertoire .ssh s'il n'existe pas
    if !ssh_dir.exists() {
        println!("📁 Création du répertoire ~/.ssh");
        fs::create_dir_all(&ssh_dir)?;
        // Permissions sécurisées pour .ssh
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&ssh_dir)?.permissions();
            perms.set_mode(0o700);
            fs::set_permissions(&ssh_dir, perms)?;
        }
        println!("✅ Répertoire ~/.ssh créé avec les bonnes permissions");
    } else {
        println!("📁 Répertoire ~/.ssh existe déjà");
    }

    // 2. Vérifier les clés SSH existantes
    println!();
    println!("🔑 Vérification des clés SSH...");

    let key_types = ["id_ed25519", "id_rsa", "id_ecdsa"];
    let mut existing_keys = Vec::new();

    for key_type in &key_types {
        let key_path = ssh_dir.join(key_type);
        if key_path.exists() {
            existing_keys.push(key_type);
            println!("✅ Clé trouvée: {}", key_type);
        }
    }

    if existing_keys.is_empty() {
        println!("⚠️ Aucune clé SSH trouvée");
        println!();
        println!("💡 Pour créer une nouvelle clé SSH Ed25519 (recommandée), exécutez:");
        println!("   ssh-keygen -t ed25519 -C \"votre.email@example.com\"");
        println!();
        println!("💡 Pour créer une clé RSA compatible (si Ed25519 n'est pas supporté):");
        println!("   ssh-keygen -t rsa -b 4096 -C \"votre.email@example.com\"");
        println!();

        // Demander si l'utilisateur veut créer une clé maintenant
        if confirm_action("Voulez-vous créer une clé SSH Ed25519 maintenant ?") {
            create_ssh_key(&ssh_dir)?;
        }
    } else {
        let keys_str: Vec<String> = existing_keys.iter().map(|s| s.to_string()).collect();
        println!(
            "✅ {} clé(s) SSH trouvée(s): {}",
            existing_keys.len(),
            keys_str.join(", ")
        );
    }

    // 3. Configurer hosts.json
    println!();
    println!("📋 Configuration du fichier hosts.json...");

    let hosts_config_path = ssh_dir.join("hosts.json");
    let config_exists = hosts_config_path.exists();

    if config_exists && !force {
        println!(
            "✅ Fichier hosts.json existe déjà: {}",
            hosts_config_path.display()
        );
        println!("   Utilisez --force pour le remplacer");
    } else {
        if config_exists {
            println!("🔄 Remplacement du fichier hosts.json existant");
        } else {
            println!("📝 Création du fichier hosts.json");
        }

        HostsConfig::create_default_config()?;
        println!(
            "✅ Fichier hosts.json créé: {}",
            hosts_config_path.display()
        );
        println!();
        println!("📝 Éditez ce fichier pour ajouter vos serveurs:");
        println!("   nano ~/.ssh/hosts.json");
        println!("   ou");
        println!("   code ~/.ssh/hosts.json");
    }

    // 4. Informations sur ssh-agent
    println!();
    println!("🔧 Configuration SSH recommandée:");
    println!();

    if std::env::var("SSH_AUTH_SOCK").is_ok() {
        println!("✅ ssh-agent est actif");
    } else {
        println!("⚠️ ssh-agent n'est pas actif");
        println!("💡 Pour démarrer ssh-agent, ajoutez à votre ~/.bashrc ou ~/.zshrc:");
        println!("   eval \"$(ssh-agent -s)\"");
        println!("   ssh-add ~/.ssh/id_ed25519  # ou votre clé préférée");
    }

    // 5. Conseils finaux
    println!();
    println!("🎯 Prochaines étapes:");
    println!("1. Éditez ~/.ssh/hosts.json avec vos serveurs");
    println!("2. Copiez vos clés publiques sur vos serveurs:");
    println!("   ssh-copy-id user@votre-serveur.com");
    println!("3. Testez la connexion:");
    println!("   xsshend upload fichier-test.txt --env Production --dry-run");
    println!();
    println!("✅ Initialisation terminée !");

    Ok(())
}

/// Créer une nouvelle clé SSH Ed25519
fn create_ssh_key(ssh_dir: &std::path::Path) -> Result<()> {
    use std::io::{self, Write};

    print!("📧 Entrez votre adresse email pour la clé SSH: ");
    io::stdout().flush()?;

    let mut email = String::new();
    io::stdin().read_line(&mut email)?;
    let email = email.trim();

    if email.is_empty() {
        println!("⚠️ Email vide, utilisation d'un commentaire par défaut");
    }

    let key_path = ssh_dir.join("id_ed25519");
    let comment = if email.is_empty() {
        "xsshend-generated-key".to_string()
    } else {
        email.to_string()
    };

    println!("🔑 Création de la clé SSH Ed25519...");

    let output = std::process::Command::new("ssh-keygen")
        .args([
            "-t",
            "ed25519",
            "-f",
            key_path.to_str().unwrap(),
            "-C",
            &comment,
            "-N",
            "", // Pas de passphrase pour simplifier
        ])
        .output()?;

    if output.status.success() {
        println!("✅ Clé SSH créée: {}", key_path.display());
        println!("✅ Clé publique: {}.pub", key_path.display());

        // Afficher la clé publique
        if let Ok(pub_key) = std::fs::read_to_string(format!("{}.pub", key_path.display())) {
            println!();
            println!("📋 Votre clé publique (à copier sur vos serveurs):");
            println!("{}", pub_key.trim());
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Erreur lors de la création de la clé SSH: {}", stderr);
    }

    Ok(())
}

/// Demander confirmation à l'utilisateur
fn confirm_action(message: &str) -> bool {
    use std::io::{self, Write};

    print!("{} (y/N): ", message);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    matches!(
        input.trim().to_lowercase().as_str(),
        "y" | "yes" | "o" | "oui"
    )
}
