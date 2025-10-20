/// Expansion des chemins avec variables d'environnement et tilde
///
/// Ce module gère l'expansion côté client des chemins contenant :
/// - Variables d'environnement ($HOME, $USER, etc.)
/// - Tilde (~user ou ~ seul)
use anyhow::Result;

/// Expanse un chemin contenant des variables d'environnement ou tilde
///
/// Utilise le répertoire HOME réel récupéré du serveur distant.
///
/// Exemples :
/// - `$HOME/work/tmp` → `/home/username/work/tmp` (avec HOME réel)
/// - `~/documents` → `/home/username/documents`
/// - `$WORK/files` → `$WORK/files` (variables inconnues preservées)
/// - `/absolute/path` → `/absolute/path` (inchangé)
pub fn expand_path(path: &str, target_user: &str, remote_home: Option<&str>) -> Result<String> {
    let mut expanded = path.to_string();

    // 1. Expansion du tilde (~)
    if expanded.starts_with("~/") {
        // ~ seul → HOME réel de l'utilisateur cible
        let fallback_home = format!("/home/{}", target_user);
        let home = remote_home.unwrap_or(&fallback_home);
        expanded = format!("{}{}", home, &expanded[1..]);
    } else if expanded.starts_with('~') && expanded.len() > 1 {
        // ~user → home de l'utilisateur spécifié
        if let Some(slash_pos_in_substring) = expanded[1..].find('/') {
            let actual_slash_pos = slash_pos_in_substring + 1; // Position réelle du slash dans expanded
            let user = &expanded[1..actual_slash_pos]; // De après ~ jusqu'au slash
            let rest = &expanded[actual_slash_pos + 1..]; // Après le slash
            expanded = format!("/home/{}/{}", user, rest);
        } else {
            // ~user sans slash final
            let user = &expanded[1..];
            expanded = format!("/home/{}", user);
        }
    }

    // 2. Expansion des variables d'environnement ($VAR)
    if expanded.contains('$') {
        expanded = expand_variables(&expanded, target_user, remote_home)?;
    }

    Ok(expanded)
}

/// Expanse les variables d'environnement dans un chemin
fn expand_variables(path: &str, target_user: &str, remote_home: Option<&str>) -> Result<String> {
    let mut result = path.to_string();

    // Variables communes à remplacer avec HOME réel
    let fallback_home = format!("/home/{}", target_user);
    let home = remote_home.unwrap_or(&fallback_home);
    let replacements = vec![
        ("$HOME", home.to_string()),
        ("$USER", target_user.to_string()),
        ("$USERNAME", target_user.to_string()),
    ];

    // Remplacer les variables connues de manière simple mais efficace
    for (var, value) in replacements {
        // Remplacement uniquement si suivi de '/' ou fin de chaîne
        if result == var {
            result = value; // Variable seule
        } else if result.contains(&format!("{}/", var)) {
            result = result.replace(&format!("{}/", var), &format!("{}/", value));
        }
    }

    Ok(result)
}
