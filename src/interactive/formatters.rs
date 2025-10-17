/// Formatage et stylisation des messages avec console
///
/// Ce module fournit des fonctions pour formater les messages
/// avec des couleurs et styles selon le contexte.
use console::style;

/// Formate le nom d'un environnement avec couleur appropriée
///
/// - Production: rouge et gras (danger)
/// - Staging: jaune (attention)
/// - Development: vert (safe)
/// - Autres: blanc
pub fn format_environment(env: &str) -> String {
    let styled = match env.to_lowercase().as_str() {
        "production" | "prod" => style(env).red().bold(),
        "staging" | "stage" => style(env).yellow(),
        "development" | "dev" => style(env).green(),
        _ => style(env).white(),
    };
    styled.to_string()
}

/// Formate la taille d'un fichier de manière lisible
///
/// Convertit les octets en KB, MB, GB selon la taille
pub fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Formate le nombre de serveurs avec style
pub fn format_server_count(count: usize) -> String {
    let styled = if count > 10 {
        style(count).yellow().bold()
    } else if count > 5 {
        style(count).cyan().bold()
    } else {
        style(count).green().bold()
    };

    format!("{} serveur{}", styled, if count > 1 { "s" } else { "" })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(500), "500 B");
        assert_eq!(format_file_size(1024), "1.00 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_format_server_count() {
        let result = format_server_count(1);
        assert!(result.contains("serveur"));

        let result = format_server_count(5);
        assert!(result.contains("serveurs"));
    }

    #[test]
    fn test_format_environment() {
        let prod = format_environment("Production");
        let dev = format_environment("Development");

        // Les tests de console peuvent ne pas afficher les couleurs en CI
        // On vérifie juste que les fonctions ne crashent pas
        assert!(!prod.is_empty());
        assert!(!dev.is_empty());
    }
}
