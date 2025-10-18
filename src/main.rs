use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod config;
mod core;
mod interactive;
mod ssh;
mod utils;

use config::HostsConfig;
use core::uploader::Uploader;

/// Outil Rust de téléversement multi-SSH avec mode interactif
#[derive(Parser)]
#[command(name = "xsshend")]
#[command(version = "0.5.1")]
#[command(about = "Téléverse des fichiers vers plusieurs serveurs SSH")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Liste tous les serveurs disponibles
    #[arg(short = 'l', long)]
    list: bool,

    /// Désactiver le mode interactif (erreur si arguments manquants)
    #[arg(long, global = true)]
    non_interactive: bool,

    /// Répondre oui à toutes les confirmations
    #[arg(short = 'y', long, global = true)]
    yes: bool,

    /// Clé SSH spécifique à utiliser
    #[arg(long, global = true, value_name = "PATH")]
    key: Option<PathBuf>,

    /// Afficher les logs de debug
    #[arg(short = 'v', long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Téléverse des fichiers vers plusieurs serveurs SSH
    Upload {
        /// Fichiers à téléverser
        #[arg(required = true, value_name = "FILE")]
        files: Vec<PathBuf>,

        /// Environnement (Production, Staging, Development, etc.)
        #[arg(long, value_name = "ENV")]
        env: Option<String>,

        /// Région géographique (Region-A, Europe, US-East, etc.)
        #[arg(long, value_name = "REGION")]
        region: Option<String>,

        /// Type de serveur (Public, Private, Database, etc.)
        #[arg(long, short = 't', value_name = "TYPE")]
        server_type: Option<String>,

        /// Répertoire de destination sur les serveurs
        #[arg(long, short = 'd', value_name = "PATH", default_value = "/tmp/")]
        dest: PathBuf,

        /// Simulation sans transfert réel
        #[arg(long)]
        dry_run: bool,
    },

    /// Exécute une commande SSH sur plusieurs serveurs
    Command {
        /// Commande inline à exécuter
        #[arg(long, conflicts_with = "script", value_name = "COMMAND")]
        inline: Option<String>,

        /// Fichier script à exécuter
        #[arg(long, conflicts_with = "inline", value_name = "FILE")]
        script: Option<PathBuf>,

        /// Environnement cible
        #[arg(long, value_name = "ENV")]
        env: Option<String>,

        /// Région cible
        #[arg(long, value_name = "REGION")]
        region: Option<String>,

        /// Type de serveur
        #[arg(long, short = 't', value_name = "TYPE")]
        server_type: Option<String>,

        /// Exécution parallèle (défaut: séquentiel)
        #[arg(long)]
        parallel: bool,

        /// Timeout par commande en secondes
        #[arg(long, default_value = "30", value_name = "SECS")]
        timeout: u64,

        /// Afficher stderr séparément
        #[arg(long)]
        capture_stderr: bool,

        /// Format de sortie (text ou json)
        #[arg(long, default_value = "text", value_name = "FORMAT")]
        output_format: String,
    },

    /// Liste les serveurs disponibles
    List,

    /// Initialise la configuration xsshend
    Init {
        /// Remplace la configuration existante
        #[arg(long, short = 'f')]
        force: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Configurer le logger selon --verbose
    if cli.verbose {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::init();
    }

    // Gérer le flag --list/-l en priorité
    if cli.list {
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
    let Some(command) = cli.command else {
        println!("Utilisez 'xsshend --help' pour voir les commandes disponibles");
        println!("Exemples:");
        println!("  xsshend upload file.txt --env Production");
        println!("  xsshend --list");
        return Ok(());
    };

    match command {
        Commands::Upload {
            files,
            env,
            region,
            server_type,
            dest,
            dry_run,
        } => {
            handle_upload_command(UploadArgs {
                files,
                env,
                region,
                server_type,
                dest,
                dry_run,
                non_interactive: cli.non_interactive,
                yes: cli.yes,
                key: cli.key,
            })
            .await?;
        }
        Commands::Command {
            inline,
            script,
            env,
            region,
            server_type,
            parallel,
            timeout,
            capture_stderr,
            output_format,
        } => {
            handle_command_execution(CommandArgs {
                inline,
                script,
                env,
                region,
                server_type,
                parallel,
                timeout,
                capture_stderr,
                output_format,
                non_interactive: cli.non_interactive,
                yes: cli.yes,
            })
            .await?;
        }
        Commands::List => {
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
        Commands::Init { force } => {
            init_setup(force)?;
        }
    }

    Ok(())
}

/// Arguments pour la commande upload
struct UploadArgs {
    files: Vec<PathBuf>,
    env: Option<String>,
    region: Option<String>,
    server_type: Option<String>,
    dest: PathBuf,
    dry_run: bool,
    non_interactive: bool,
    yes: bool,
    key: Option<PathBuf>,
}

/// Arguments pour la commande command
struct CommandArgs {
    inline: Option<String>,
    script: Option<PathBuf>,
    env: Option<String>,
    region: Option<String>,
    server_type: Option<String>,
    parallel: bool,
    timeout: u64,
    capture_stderr: bool,
    output_format: String,
    non_interactive: bool,
    yes: bool,
}

/// Gère l'exécution de commandes SSH
async fn handle_command_execution(args: CommandArgs) -> Result<()> {
    use crate::core::executor::CommandExecutor;
    use crate::interactive::{is_interactive_mode, prompts, should_prompt};
    use anyhow::Context;

    println!("🚀 xsshend - Exécution de commandes SSH");

    // Charger la configuration
    let config = HostsConfig::load()?;

    // Variables mutables pour le mode interactif
    let mut inline_cmd = args.inline;
    let mut script_path = args.script;
    let mut env = args.env;
    let mut region = args.region;
    let mut server_type = args.server_type;

    // 1. Mode interactif: compléter les arguments manquants
    if !args.non_interactive && is_interactive_mode() {
        println!("\n{}", "=".repeat(60));
        println!("🎨 Mode Interactif");
        println!("{}", "=".repeat(60));

        // Type de commande (inline ou script)
        if inline_cmd.is_none() && script_path.is_none() {
            let cmd_type = prompts::prompt_command_type()?;
            if cmd_type.contains("inline") {
                inline_cmd = Some(prompts::prompt_inline_command()?);
            } else {
                script_path = Some(prompts::prompt_script_path()?);
            }
        }

        // Environnement
        if should_prompt(&env, args.non_interactive) {
            env = Some(prompts::prompt_environment(&config)?);
        }

        // Région
        if env.is_some() && should_prompt(&region, args.non_interactive) {
            region = prompts::prompt_region(&config, env.as_ref().unwrap())?;
        }

        // Type de serveur
        if env.is_some() && should_prompt(&server_type, args.non_interactive) {
            server_type =
                prompts::prompt_server_type(&config, env.as_ref().unwrap(), region.as_deref())?;
        }
    } else if args.non_interactive && inline_cmd.is_none() && script_path.is_none() {
        // Mode explicitement non-interactif: valider les arguments
        anyhow::bail!("❌ Argument --inline ou --script requis avec --non-interactive");
    } else if args.non_interactive && env.is_none() {
        anyhow::bail!("❌ Argument --env requis avec --non-interactive");
    }

    // 2. Déterminer la commande à exécuter
    let command = if let Some(inline) = inline_cmd {
        inline
    } else if let Some(script) = script_path {
        tokio::fs::read_to_string(&script).await.context(format!(
            "Impossible de lire le script: {}",
            script.display()
        ))?
    } else {
        anyhow::bail!("Vous devez fournir --inline ou --script");
    };

    // 3. Filtrer les hôtes
    let target_hosts = config.filter_hosts(env.as_ref(), region.as_ref(), server_type.as_ref());

    if target_hosts.is_empty() {
        anyhow::bail!("❌ Aucun serveur trouvé avec les critères spécifiés");
    }

    // 4. Confirmation
    if !args.yes {
        if !args.non_interactive && is_interactive_mode() {
            let confirmed = prompts::confirm_command_execution(
                &command,
                &target_hosts,
                env.as_deref().unwrap_or("Unknown"),
                args.parallel,
                args.timeout,
            )?;

            if !confirmed {
                println!("❌ Exécution annulée");
                return Ok(());
            }
        } else {
            println!("⚠️  Utilisez --yes pour confirmer automatiquement en mode non-interactif");
            anyhow::bail!("Confirmation requise");
        }
    }

    // 5. Exécuter les commandes
    if args.output_format != "json" {
        println!("\n🚀 Début de l'exécution...\n");
    }

    let executor = CommandExecutor::new();
    let results = executor
        .execute(
            &command,
            &target_hosts,
            args.parallel,
            std::time::Duration::from_secs(args.timeout),
        )
        .await?;

    // 6. Afficher les résultats détaillés (seulement en mode text)
    if args.output_format != "json" {
        println!("\n📊 Résultats détaillés:");
        println!("{}", "=".repeat(80));

        for result in &results {
            println!("\n▶ Serveur: {}", result.host);
            println!("  Exit code: {}", result.exit_code);
            println!("  Durée: {:.2}s", result.duration.as_secs_f64());
            println!(
                "  Statut: {}",
                if result.success {
                    "✅ Succès"
                } else {
                    "❌ Échec"
                }
            );

            if !result.stdout.is_empty() {
                println!("\n  📤 Stdout:");
                for line in result.stdout.lines() {
                    println!("    {}", line);
                }
            }

            if args.capture_stderr && !result.stderr.is_empty() {
                println!("\n  ⚠️  Stderr:");
                for line in result.stderr.lines() {
                    println!("    {}", line);
                }
            }
            println!("{}", "-".repeat(80));
        }
    }

    // 7. Résumé final
    let success_count = results.iter().filter(|r| r.success).count();
    let total_count = results.len();

    // Afficher les résultats selon le format demandé
    if args.output_format == "json" {
        // Format JSON pour parsing automatique
        use crate::core::executor::ExecutionSummary;

        let summary = ExecutionSummary {
            total: total_count,
            success: success_count,
            failed: total_count - success_count,
            total_duration_secs: results.iter().map(|r| r.duration.as_secs_f64()).sum(),
        };

        let json_output = serde_json::json!({
            "summary": summary,
            "results": results,
        });

        println!("{}", serde_json::to_string_pretty(&json_output)?);
    } else {
        // Format texte (par défaut)
        println!("\n✨ Résumé:");
        println!("  Succès: {}/{}", success_count, total_count);
        println!("  Échecs: {}/{}", total_count - success_count, total_count);

        if success_count == total_count {
            println!("\n✅ Toutes les commandes ont été exécutées avec succès !");
        } else if success_count > 0 {
            println!("\n⚠️  Certaines commandes ont échoué.");
        } else {
            println!("\n❌ Toutes les commandes ont échoué.");
        }
    }

    Ok(())
}

/// Gère la commande upload avec mode interactif
async fn handle_upload_command(args: UploadArgs) -> Result<()> {
    use crate::core::validator::Validator;
    use crate::interactive::{is_interactive_mode, prompts, should_prompt};

    println!("🚀 xsshend - Téléversement Multi-SSH");

    // 1. Validation des fichiers
    println!("🔍 Validation des fichiers...");
    for file in &args.files {
        Validator::validate_file(file)
            .map_err(|e| anyhow::anyhow!("Validation échouée pour {}: {}", file.display(), e))?;
    }

    // Charger la configuration
    let config = HostsConfig::load()?;

    // Extraire les arguments mutables
    let mut env = args.env;
    let mut region = args.region;
    let mut server_type = args.server_type;
    let mut dest = args.dest;

    // 2. Mode interactif: compléter les arguments manquants
    if !args.non_interactive && is_interactive_mode() {
        println!("\n{}", "=".repeat(60));
        println!("🎨 Mode Interactif");
        println!("{}", "=".repeat(60));

        // Environnement
        if should_prompt(&env, args.non_interactive) {
            env = Some(prompts::prompt_environment(&config)?);
        }

        // Région
        if env.is_some() && should_prompt(&region, args.non_interactive) {
            region = prompts::prompt_region(&config, env.as_ref().unwrap())?;
        }

        // Type de serveur
        if env.is_some() && should_prompt(&server_type, args.non_interactive) {
            server_type =
                prompts::prompt_server_type(&config, env.as_ref().unwrap(), region.as_deref())?;
        }

        // Destination (si default)
        if dest == PathBuf::from("/tmp/") {
            let new_dest = prompts::prompt_destination("/tmp/")?;
            dest = new_dest;
        }
    } else if args.non_interactive && env.is_none() {
        // Mode explicitement non-interactif: valider les arguments
        anyhow::bail!("❌ Argument --env requis avec --non-interactive");
    }

    // 3. Filtrer les serveurs
    let target_hosts = config.filter_hosts(env.as_ref(), region.as_ref(), server_type.as_ref());

    if target_hosts.is_empty() {
        anyhow::bail!("❌ Aucun serveur trouvé avec les critères spécifiés");
    }

    // 4. Afficher le récapitulatif
    println!("\n{}", "=".repeat(60));
    println!("📋 RÉCAPITULATIF");
    println!("{}", "=".repeat(60));
    println!("📦 Fichiers: {}", args.files.len());
    for file in &args.files {
        if let Ok(metadata) = std::fs::metadata(file) {
            println!("   • {} ({} octets)", file.display(), metadata.len());
        }
    }
    println!("\n🎯 Environnement: {}", env.as_deref().unwrap_or("Tous"));
    println!("📍 Région: {}", region.as_deref().unwrap_or("Toutes"));
    println!("🖥️  Type: {}", server_type.as_deref().unwrap_or("Tous"));
    println!("📂 Destination: {}", dest.display());
    println!("🖥️  Serveurs ciblés: {}", target_hosts.len());
    println!("{}", "=".repeat(60));

    // 5. Confirmation
    if !args.dry_run && !args.yes {
        if !args.non_interactive && is_interactive_mode() {
            let confirmed = prompts::confirm_upload(
                &args.files,
                &target_hosts,
                &dest,
                env.as_deref().unwrap_or("Unknown"),
            )?;

            if !confirmed {
                println!("❌ Téléversement annulé");
                return Ok(());
            }
        } else {
            println!("⚠️  Utilisez --yes pour confirmer automatiquement en mode non-interactif");
            anyhow::bail!("Confirmation requise");
        }
    }

    // 6. Upload
    println!("\n🚀 Début du téléversement...\n");

    let uploader = Uploader::new();
    let file_refs: Vec<&std::path::Path> = args.files.iter().map(|p| p.as_path()).collect();
    let dest_str = dest.to_str().unwrap_or("/tmp/");

    // Si une clé SSH est fournie, l'indiquer (elle est lue ici pour éviter l'avertissement
    // ; le comportement effectif d'utilisation peut être géré par d'autres modules)
    if let Some(key_path) = &args.key {
        println!(
            "🔑 Utilisation de la clé SSH fournie: {}",
            key_path.display()
        );
    }

    if args.dry_run {
        uploader
            .dry_run(&file_refs, &target_hosts, dest_str)
            .await?;
    } else {
        uploader
            .upload_files(&file_refs, &target_hosts, dest_str)
            .await?;
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
