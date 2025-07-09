# 🎨 Gestion des thèmes et accessibilité

## Vue d'ensemble

xsshend offre une gestion automatique des thèmes pour optimiser la lisibilité sur tous les terminaux. Le système détecte automatiquement si votre terminal utilise un thème clair ou sombre et adapte les couleurs en conséquence.

## Détection automatique du thème

### Méthodes de détection

1. **API termbg** : Utilise la librairie `termbg` pour interroger directement le terminal
2. **Variables d'environnement** : Analyse `COLORFGBG` pour détecter les couleurs de fond
3. **Détection du terminal** : Reconnaissance des terminaux spécifiques (VS Code, etc.)

### Thèmes supportés

#### Thème sombre (défaut)
- Fond noir/sombre détecté
- Texte blanc/clair
- Accents colorés (cyan, jaune, vert)
- Sélections en bleu vif

#### Thème clair
- Fond blanc/clair détecté
- Texte noir/foncé avec contraste optimisé
- Couleurs adaptées pour fonds clairs
- Éléments non sélectionnés en gris foncé lisible

## Optimisations de lisibilité

### Problèmes résolus

- **Éléments non sélectionnés invisibles** : Utilisation de couleurs contrastées
- **Panneau d'aide illisible** : Texte principal au lieu de texte secondaire
- **Titres de panneaux peu visibles** : Couleurs dédiées pour les titres
- **Contraste insuffisant** : Respect des guidelines WCAG

### Couleurs par thème

#### Thème sombre
```rust
text_primary: Color::White,
text_secondary: Color::Grey,
selection: Color::Blue,
border_title: Color::Cyan,
title_primary: Color::Yellow,
title_secondary: Color::Cyan,
```

#### Thème clair
```rust
text_primary: Color::Black,
text_secondary: Color::Rgb { r: 60, g: 60, b: 60 }, // Gris foncé
selection: Color::DarkBlue,
border_title: Color::DarkBlue,
title_primary: Color::DarkRed,
title_secondary: Color::DarkBlue,
```

## Styles spécialisés

### Styles pour éléments d'interface

- `text_style()` : Texte principal
- `text_secondary_style()` : Texte secondaire (moins important)
- `unselected_item_style()` : Éléments non sélectionnés dans les listes
- `help_text_style()` : Texte d'aide dans les panneaux
- `selection_style()` : Éléments sélectionnés/surlignés

### Styles pour titres

- `title_primary_style()` : Titres principaux (écrans)
- `title_secondary_style()` : Titres secondaires (sections)
- `border_title_style()` : Titres de bordures de panneaux

### Helpers de blocs

- `themed_block()` : Bloc standard avec bordures
- `primary_block()` : Bloc avec titre principal
- `secondary_block()` : Bloc avec titre secondaire

## Configuration avancée

### Variables d'environnement

```bash
# Force la détection d'un thème spécifique (debug)
export XSSHEND_FORCE_THEME=light  # ou dark

# Désactive la détection automatique
export XSSHEND_NO_THEME_DETECTION=1
```

### Debugging

Pour diagnostiquer les problèmes de thème :

```bash
# Logs de détection du thème
RUST_LOG=debug xsshend

# Test avec différents terminaux
# - Terminal par défaut
# - VS Code terminal
# - Terminal avec thème personnalisé
```

## Compatibilité

### Terminaux testés

- ✅ **Gnome Terminal** (thème clair/sombre)
- ✅ **VS Code Terminal** (thème clair/sombre)  
- ✅ **iTerm2** (macOS)
- ✅ **Windows Terminal**
- ✅ **Alacritty**
- ✅ **Kitty**

### Fallback

Si la détection échoue, xsshend utilise le thème sombre par défaut qui reste lisible sur la plupart des terminaux.

## Résolution de problèmes

### Éléments illisibles

Si certains éléments restent difficiles à lire :

1. Vérifiez votre terminal supporte les couleurs
2. Testez avec `RUST_LOG=debug` pour voir la détection
3. Forcez un thème avec les variables d'environnement
4. Reportez le problème avec les détails de votre terminal

### Rapporter un problème

Incluez ces informations :

- Terminal utilisé
- Système d'exploitation
- Thème du terminal
- Sortie de `RUST_LOG=debug xsshend`
- Capture d'écran si possible
