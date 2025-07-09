use crossterm::style::Color;
use std::time::Duration;

/// Détecte le thème du terminal (clair/sombre)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TerminalTheme {
    Light,
    Dark,
    Unknown,
}

/// Configuration des couleurs selon le thème
#[derive(Debug, Clone)]
pub struct ThemeColors {
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_accent: Color,
    pub text_success: Color,
    pub text_warning: Color,
    pub text_error: Color,
    pub background_primary: Color,
    #[allow(dead_code)]
    pub background_secondary: Color,
    pub border: Color,
    pub border_title: Color,     // Nouvelle couleur pour les titres de bordures
    pub selection: Color,
    pub title_primary: Color,    // Nouvelle couleur pour les titres principaux
    pub title_secondary: Color,  // Nouvelle couleur pour les titres secondaires
}

impl ThemeColors {
    /// Couleurs pour thème sombre
    pub fn dark() -> Self {
        Self {
            text_primary: Color::White,
            text_secondary: Color::Grey,
            text_accent: Color::Cyan,
            text_success: Color::Green,
            text_warning: Color::Yellow,
            text_error: Color::Red,
            background_primary: Color::Black,
            background_secondary: Color::DarkGrey,
            border: Color::DarkGrey,
            border_title: Color::Cyan,        // Titres de bordures en cyan visible
            selection: Color::Blue,
            title_primary: Color::Yellow,     // Titres principaux en jaune
            title_secondary: Color::Cyan,     // Titres secondaires en cyan
        }
    }

    /// Couleurs pour thème clair
    pub fn light() -> Self {
        Self {
            text_primary: Color::Black,
            text_secondary: Color::Rgb { r: 60, g: 60, b: 60 }, // Gris foncé pour meilleur contraste
            text_accent: Color::DarkBlue,
            text_success: Color::DarkGreen,
            text_warning: Color::Rgb { r: 184, g: 134, b: 11 }, // Orange foncé pour warning
            text_error: Color::DarkRed,
            background_primary: Color::White,
            background_secondary: Color::Rgb { r: 245, g: 245, b: 245 }, // Gris très clair
            border: Color::Rgb { r: 128, g: 128, b: 128 }, // Gris moyen pour bordures
            border_title: Color::DarkBlue,    // Titres de bordures en bleu foncé
            selection: Color::DarkBlue,
            title_primary: Color::DarkRed,    // Titres principaux en rouge foncé
            title_secondary: Color::DarkBlue, // Titres secondaires en bleu foncé
        }
    }
}

/// Détecte le thème du terminal
pub fn detect_terminal_theme() -> TerminalTheme {
    log::debug!("🎨 Détection du thème du terminal...");

    // Utiliser termbg pour la détection
    match termbg::theme(Duration::from_millis(50)) {
        Ok(termbg::Theme::Light) => {
            log::debug!("🎨 Thème détecté par termbg: Light");
            TerminalTheme::Light
        }
        Ok(termbg::Theme::Dark) => {
            log::debug!("🎨 Thème détecté par termbg: Dark");
            TerminalTheme::Dark
        }
        Err(e) => {
            log::debug!("🎨 Impossible de détecter le thème: {}", e);
            // Vérifier les variables d'environnement
            if let Ok(colorfgbg) = std::env::var("COLORFGBG") {
                log::debug!("🎨 Variable COLORFGBG trouvée: {}", colorfgbg);
                // Format typique: "15;0" où le premier est foreground, le second background
                if let Some(bg) = colorfgbg.split(';').nth(1) {
                    if let Ok(bg_color) = bg.parse::<u8>() {
                        let theme = if bg_color < 8 { TerminalTheme::Dark } else { TerminalTheme::Light };
                        log::debug!("🎨 Thème détecté via COLORFGBG: {:?}", theme);
                        return theme;
                    }
                }
            }

            // Dernière tentative: vérifier TERM_PROGRAM
            if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
                match term_program.as_str() {
                    "vscode" => {
                        // VS Code a souvent un thème sombre par défaut
                        log::debug!("🎨 VS Code détecté, utilisation du thème sombre par défaut");
                        TerminalTheme::Dark
                    }
                    _ => {
                        log::debug!("🎨 Terminal {} détecté, thème inconnu", term_program);
                        TerminalTheme::Unknown
                    }
                }
            } else {
                log::debug!("🎨 Impossible de déterminer le thème, utilisation du défaut");
                TerminalTheme::Unknown
            }
        }
    }
}

/// Obtient les couleurs appropriées pour le thème du terminal
pub fn get_theme_colors() -> ThemeColors {
    match detect_terminal_theme() {
        TerminalTheme::Light => {
            log::info!("🎨 Utilisation du thème clair");
            ThemeColors::light()
        }
        TerminalTheme::Dark => {
            log::info!("🎨 Utilisation du thème sombre");
            ThemeColors::dark()
        }
        TerminalTheme::Unknown => {
            log::info!("🎨 Thème inconnu, utilisation du thème sombre par défaut");
            ThemeColors::dark()
        }
    }
}

/// Applique les couleurs du thème aux composants ratatui
pub mod ratatui_theme {
    use super::*;
    use ratatui::prelude::*;
    use ratatui::widgets::{Block, Borders};
    use crossterm::style::Color as CrosstermColor;

    /// Convertit une couleur Crossterm en couleur Ratatui avec plus de précision
    pub fn crossterm_to_ratatui(color: CrosstermColor) -> ratatui::style::Color {
        match color {
            CrosstermColor::Black => ratatui::style::Color::Black,
            CrosstermColor::DarkGrey => ratatui::style::Color::DarkGray,
            CrosstermColor::Red => ratatui::style::Color::Red,
            CrosstermColor::DarkRed => ratatui::style::Color::Red,
            CrosstermColor::Green => ratatui::style::Color::Green,
            CrosstermColor::DarkGreen => ratatui::style::Color::Green,
            CrosstermColor::Yellow => ratatui::style::Color::Yellow,
            CrosstermColor::DarkYellow => ratatui::style::Color::Yellow,
            CrosstermColor::Blue => ratatui::style::Color::Blue,
            CrosstermColor::DarkBlue => ratatui::style::Color::Blue,
            CrosstermColor::Magenta => ratatui::style::Color::Magenta,
            CrosstermColor::DarkMagenta => ratatui::style::Color::Magenta,
            CrosstermColor::Cyan => ratatui::style::Color::Cyan,
            CrosstermColor::DarkCyan => ratatui::style::Color::Cyan,
            CrosstermColor::White => ratatui::style::Color::White,
            CrosstermColor::Grey => ratatui::style::Color::Gray,
            CrosstermColor::Rgb { r, g, b } => ratatui::style::Color::Rgb(r, g, b),
            CrosstermColor::AnsiValue(val) => ratatui::style::Color::Indexed(val),
            _ => ratatui::style::Color::White,
        }
    }

    /// Crée un style pour le texte principal
    pub fn text_style(colors: &ThemeColors) -> Style {
        Style::default().fg(crossterm_to_ratatui(colors.text_primary))
    }

    /// Crée un style pour le texte secondaire
    pub fn text_secondary_style(colors: &ThemeColors) -> Style {
        Style::default().fg(crossterm_to_ratatui(colors.text_secondary))
    }

    /// Crée un style pour le texte accentué
    pub fn text_accent_style(colors: &ThemeColors) -> Style {
        Style::default().fg(crossterm_to_ratatui(colors.text_accent)).add_modifier(Modifier::BOLD)
    }

    /// Crée un style pour les éléments sélectionnés
    pub fn selection_style(colors: &ThemeColors) -> Style {
        Style::default()
            .fg(crossterm_to_ratatui(colors.background_primary))
            .bg(crossterm_to_ratatui(colors.selection))
            .add_modifier(Modifier::BOLD)
    }

    /// Crée un style pour les bordures
    pub fn border_style(colors: &ThemeColors) -> Style {
        Style::default().fg(crossterm_to_ratatui(colors.border))
    }

    /// Crée un style pour les messages de succès
    pub fn success_style(colors: &ThemeColors) -> Style {
        Style::default().fg(crossterm_to_ratatui(colors.text_success)).add_modifier(Modifier::BOLD)
    }

    /// Crée un style pour les messages d'avertissement
    pub fn warning_style(colors: &ThemeColors) -> Style {
        Style::default().fg(crossterm_to_ratatui(colors.text_warning)).add_modifier(Modifier::BOLD)
    }

    /// Crée un style pour les messages d'erreur
    pub fn error_style(colors: &ThemeColors) -> Style {
        Style::default().fg(crossterm_to_ratatui(colors.text_error)).add_modifier(Modifier::BOLD)
    }

    /// Crée un style pour les titres de bordures (plus visibles)
    pub fn border_title_style(colors: &ThemeColors) -> Style {
        Style::default().fg(crossterm_to_ratatui(colors.border_title)).add_modifier(Modifier::BOLD)
    }

    /// Crée un style pour les titres principaux
    pub fn title_primary_style(colors: &ThemeColors) -> Style {
        Style::default().fg(crossterm_to_ratatui(colors.title_primary)).add_modifier(Modifier::BOLD)
    }

    /// Crée un style pour les titres secondaires
    pub fn title_secondary_style(colors: &ThemeColors) -> Style {
        Style::default().fg(crossterm_to_ratatui(colors.title_secondary)).add_modifier(Modifier::BOLD)
    }

    /// Crée un style pour les éléments non sélectionnés dans les listes (contraste amélioré)
    pub fn unselected_item_style(colors: &ThemeColors) -> Style {
        // Utilise text_primary plutôt que text_secondary pour un meilleur contraste
        Style::default().fg(crossterm_to_ratatui(colors.text_primary))
    }

    /// Crée un style pour le panneau d'aide avec contraste amélioré
    pub fn help_text_style(colors: &ThemeColors) -> Style {
        // Utilise text_primary pour un meilleur contraste dans l'aide
        Style::default().fg(crossterm_to_ratatui(colors.text_primary))
    }

    /// Crée un bloc avec des couleurs de thème optimisées
    pub fn themed_block<'a>(colors: &ThemeColors, title: &'a str) -> Block<'a> {
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style(colors))
            .title(title)
            .title_style(border_title_style(colors))
    }

    /// Crée un bloc avec un titre principal
    pub fn primary_block<'a>(colors: &ThemeColors, title: &'a str) -> Block<'a> {
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style(colors))
            .title(title)
            .title_style(title_primary_style(colors))
    }

    /// Crée un bloc avec un titre secondaire
    pub fn secondary_block<'a>(colors: &ThemeColors, title: &'a str) -> Block<'a> {
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style(colors))
            .title(title)
            .title_style(title_secondary_style(colors))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_detection() {
        let theme = detect_terminal_theme();
        // Le test ne peut pas prédire le thème mais ne doit pas paniquer
        assert!(matches!(theme, TerminalTheme::Light | TerminalTheme::Dark | TerminalTheme::Unknown));
    }

    #[test]
    fn test_theme_colors() {
        let dark = ThemeColors::dark();
        let light = ThemeColors::light();
        
        // Vérifier que les thèmes ont des couleurs différentes
        assert_ne!(dark.text_primary, light.text_primary);
        assert_ne!(dark.background_primary, light.background_primary);
    }
}
