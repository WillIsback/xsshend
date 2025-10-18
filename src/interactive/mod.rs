/// Module pour gérer l'interactivité et les prompts utilisateur
///
/// Ce module fournit des fonctionnalités pour:
/// - Détecter si l'application est en mode interactif (TTY)
/// - Afficher des prompts pour compléter les arguments manquants
/// - Formater les messages avec des couleurs et styles
pub mod formatters;
pub mod prompts;

use std::io::IsTerminal;

/// Détermine si l'application est en mode interactif
///
/// Retourne `true` si stdin et stdout sont des terminaux (TTY)
pub fn is_interactive_mode() -> bool {
    std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}

/// Détermine si un prompt doit être affiché pour compléter une valeur manquante
///
/// # Arguments
/// * `cli_value` - La valeur fournie via CLI (None si manquante)
/// * `non_interactive` - Flag --non-interactive activé
///
/// # Returns
/// `true` si on doit afficher un prompt (valeur manquante + mode interactif)
pub fn should_prompt<T>(cli_value: &Option<T>, non_interactive: bool) -> bool {
    cli_value.is_none() && !non_interactive && is_interactive_mode()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_prompt_with_value() {
        let value = Some("test");
        assert!(!should_prompt(&value, false));
    }

    #[test]
    fn test_should_prompt_non_interactive() {
        let value: Option<String> = None;
        assert!(!should_prompt(&value, true));
    }
}
