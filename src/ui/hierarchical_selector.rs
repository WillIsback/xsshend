use crate::config::hosts::{HostEntry, HostsConfig};
use crate::ui::theme::{ThemeColors, ratatui_theme};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Text}, // Removed unused Span
    widgets::{Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub key: String,
    pub display_name: String,
    pub level: usize,
    pub node_type: NodeType,
    // pub is_expanded: bool, // Unused field
    pub host_entry: Option<HostEntry>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Environment,
    Region,
    ServerType,
    Host,
}

#[derive(Debug)]
pub struct HierarchicalServerSelector {
    pub tree_nodes: Vec<TreeNode>,
    pub filtered_nodes: Vec<usize>, // Indices des nœuds visibles après filtrage
    pub selection_cursor: usize,
    pub selected_hosts: HashMap<String, HostEntry>,
    pub search_query: String,
    pub search_mode: bool,
    pub expanded_paths: HashSet<String>,
}

impl HierarchicalServerSelector {
    pub fn new(hosts_config: &HostsConfig) -> Result<Self> {
        let mut selector = Self {
            tree_nodes: Vec::new(),
            filtered_nodes: Vec::new(),
            selection_cursor: 0,
            selected_hosts: HashMap::new(),
            search_query: String::new(),
            search_mode: false,
            expanded_paths: HashSet::new(),
        };

        selector.build_tree(hosts_config)?;
        selector.update_filtered_nodes();
        Ok(selector)
    }

    /// Constructeur avec vérification de connectivité
    pub fn new_with_connectivity(hosts_config: &HostsConfig, timeout_secs: u64) -> Result<Self> {
        log::info!("🔍 Filtrage des serveurs par connectivité...");

        let online_hosts = hosts_config.get_online_hosts_sync(timeout_secs);
        let online_host_paths: HashSet<String> =
            online_hosts.iter().map(|(path, _)| path.clone()).collect();

        let mut selector = Self {
            tree_nodes: Vec::new(),
            filtered_nodes: Vec::new(),
            selection_cursor: 0,
            selected_hosts: HashMap::new(),
            search_query: String::new(),
            search_mode: false,
            expanded_paths: HashSet::new(),
        };

        selector.build_tree_with_filter(hosts_config, &online_host_paths)?;
        selector.update_filtered_nodes();

        if online_hosts.is_empty() {
            log::warn!("⚠️  Aucun serveur en ligne disponible pour la sélection");
        } else {
            log::info!("✅ {} serveurs en ligne disponibles", online_hosts.len());
        }

        Ok(selector)
    }

    /// Construit l'arbre hiérarchique à partir de la configuration
    fn build_tree(&mut self, config: &HostsConfig) -> Result<()> {
        self.tree_nodes.clear();

        for (env_name, regions) in &config.environments {
            // Nœud environnement
            let env_path = env_name.clone();
            self.tree_nodes.push(TreeNode {
                key: env_path.clone(),
                display_name: env_name.clone(),
                level: 0,
                node_type: NodeType::Environment,
                host_entry: None,
            });

            for (region_name, server_types) in regions {
                // Nœud région
                let region_path = format!("{}/{}", env_name, region_name);
                self.tree_nodes.push(TreeNode {
                    key: region_path.clone(),
                    display_name: region_name.clone(),
                    level: 1,
                    node_type: NodeType::Region,
                    host_entry: None,
                });

                for (type_name, hosts) in server_types {
                    // Nœud type de serveur
                    let type_path = format!("{}/{}/{}", env_name, region_name, type_name);
                    self.tree_nodes.push(TreeNode {
                        key: type_path.clone(),
                        display_name: type_name.clone(),
                        level: 2,
                        node_type: NodeType::ServerType,
                        host_entry: None,
                    });

                    for (host_name, host_entry) in hosts {
                        // Nœud serveur
                        let host_path =
                            format!("{}/{}/{}/{}", env_name, region_name, type_name, host_name);
                        self.tree_nodes.push(TreeNode {
                            key: host_path,
                            display_name: host_name.clone(),
                            level: 3,
                            node_type: NodeType::Host,
                            host_entry: Some(host_entry.clone()),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Construit l'arbre en filtrant seulement les hôtes en ligne
    fn build_tree_with_filter(
        &mut self,
        config: &HostsConfig,
        online_paths: &HashSet<String>,
    ) -> Result<()> {
        self.tree_nodes.clear();

        for (env_name, regions) in &config.environments {
            let mut env_has_online_hosts = false;

            // D'abord, vérifier s'il y a des hôtes en ligne dans cet environnement
            for (region_name, server_types) in regions {
                for (type_name, hosts) in server_types {
                    for server_name in hosts.keys() {
                        let host_path = format!(
                            "{} > {} > {} > {}",
                            env_name, region_name, type_name, server_name
                        );
                        if online_paths.contains(&host_path) {
                            env_has_online_hosts = true;
                            break;
                        }
                    }
                    if env_has_online_hosts {
                        break;
                    }
                }
                if env_has_online_hosts {
                    break;
                }
            }

            // N'ajouter l'environnement que s'il contient des hôtes en ligne
            if !env_has_online_hosts {
                continue;
            }

            // Nœud environnement
            let env_path = env_name.clone();
            self.tree_nodes.push(TreeNode {
                key: env_path.clone(),
                display_name: format!("{} 🟢", env_name), // Indicateur en ligne
                level: 0,
                node_type: NodeType::Environment,
                host_entry: None,
            });

            for (region_name, server_types) in regions {
                let mut region_has_online_hosts = false;

                // Vérifier s'il y a des hôtes en ligne dans cette région
                for (type_name, hosts) in server_types {
                    for server_name in hosts.keys() {
                        let host_path = format!(
                            "{} > {} > {} > {}",
                            env_name, region_name, type_name, server_name
                        );
                        if online_paths.contains(&host_path) {
                            region_has_online_hosts = true;
                            break;
                        }
                    }
                    if region_has_online_hosts {
                        break;
                    }
                }

                if !region_has_online_hosts {
                    continue;
                }

                // Nœud région
                let region_path = format!("{}/{}", env_name, region_name);
                self.tree_nodes.push(TreeNode {
                    key: region_path.clone(),
                    display_name: format!("{} 🟢", region_name),
                    level: 1,
                    node_type: NodeType::Region,
                    host_entry: None,
                });

                for (type_name, hosts) in server_types {
                    let mut type_has_online_hosts = false;

                    // Vérifier s'il y a des hôtes en ligne dans ce type
                    for server_name in hosts.keys() {
                        let host_path = format!(
                            "{} > {} > {} > {}",
                            env_name, region_name, type_name, server_name
                        );
                        if online_paths.contains(&host_path) {
                            type_has_online_hosts = true;
                            break;
                        }
                    }

                    if !type_has_online_hosts {
                        continue;
                    }

                    // Nœud type de serveur
                    let type_path = format!("{}/{}/{}", env_name, region_name, type_name);
                    self.tree_nodes.push(TreeNode {
                        key: type_path.clone(),
                        display_name: format!("{} 🟢", type_name),
                        level: 2,
                        node_type: NodeType::ServerType,
                        host_entry: None,
                    });

                    // Nœuds serveurs individuels (seulement ceux en ligne)
                    for (server_name, host_entry) in hosts {
                        let host_path = format!(
                            "{} > {} > {} > {}",
                            env_name, region_name, type_name, server_name
                        );

                        if online_paths.contains(&host_path) {
                            let host_full_path = format!(
                                "{}/{}/{}/{}",
                                env_name, region_name, type_name, server_name
                            );
                            self.tree_nodes.push(TreeNode {
                                key: host_full_path,
                                display_name: format!("🟢 {} ({})", server_name, host_entry.alias),
                                level: 3,
                                node_type: NodeType::Host,
                                host_entry: Some(host_entry.clone()),
                            });
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Met à jour la liste des nœuds visibles selon le filtrage et l'expansion
    fn update_filtered_nodes(&mut self) {
        self.filtered_nodes.clear();

        for (index, node) in self.tree_nodes.iter().enumerate() {
            // Filtrage par recherche
            if !self.search_query.is_empty() {
                let query_lower = self.search_query.to_lowercase();
                let matches = node.display_name.to_lowercase().contains(&query_lower)
                    || node.key.to_lowercase().contains(&query_lower);

                if !matches {
                    continue;
                }
            }

            // Vérifier si le nœud doit être visible selon l'état d'expansion
            if self.should_show_node(node) {
                self.filtered_nodes.push(index);
            }
        }

        // Ajuster le curseur si nécessaire
        if self.selection_cursor >= self.filtered_nodes.len() && !self.filtered_nodes.is_empty() {
            self.selection_cursor = self.filtered_nodes.len() - 1;
        }
    }

    /// Détermine si un nœud doit être affiché selon l'état d'expansion de ses parents
    fn should_show_node(&self, node: &TreeNode) -> bool {
        if node.level == 0 {
            return true; // Les environnements sont toujours visibles
        }

        // Vérifier que tous les parents sont expansés
        let path_parts: Vec<&str> = node.key.split('/').collect();

        for i in 1..path_parts.len() {
            let parent_path = path_parts[0..i].join("/");
            if !self.expanded_paths.contains(&parent_path) {
                return false;
            }
        }

        true
    }

    /// Gère l'événement clavier
    pub fn handle_key_event(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('/') if !self.search_mode => {
                self.search_mode = true;
                self.search_query.clear();
                Ok(true)
            }
            KeyCode::Esc if self.search_mode => {
                self.search_mode = false;
                self.search_query.clear();
                self.update_filtered_nodes();
                Ok(true)
            }
            KeyCode::Char(c) if self.search_mode => {
                self.search_query.push(c);
                self.update_filtered_nodes();
                Ok(true)
            }
            KeyCode::Backspace if self.search_mode => {
                self.search_query.pop();
                self.update_filtered_nodes();
                Ok(true)
            }
            KeyCode::Enter if self.search_mode => {
                self.search_mode = false;
                Ok(true)
            }
            KeyCode::Up => {
                if !self.filtered_nodes.is_empty() && self.selection_cursor > 0 {
                    self.selection_cursor -= 1;
                }
                Ok(true)
            }
            KeyCode::Down => {
                if !self.filtered_nodes.is_empty()
                    && self.selection_cursor < self.filtered_nodes.len() - 1
                {
                    self.selection_cursor += 1;
                }
                Ok(true)
            }
            KeyCode::Right | KeyCode::Enter if !self.search_mode => {
                self.toggle_expansion();
                Ok(true)
            }
            KeyCode::Left if !self.search_mode => {
                self.collapse_current();
                Ok(true)
            }
            KeyCode::Char(' ') if !self.search_mode => {
                self.toggle_selection();
                Ok(true)
            }
            KeyCode::Char('a') if !self.search_mode => {
                self.select_all_visible();
                Ok(true)
            }
            KeyCode::Char('c') if !self.search_mode => {
                self.clear_selection();
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// Bascule l'expansion du nœud sélectionné
    fn toggle_expansion(&mut self) {
        if let Some(&node_index) = self.filtered_nodes.get(self.selection_cursor) {
            let node = &self.tree_nodes[node_index];

            if node.node_type != NodeType::Host {
                if self.expanded_paths.contains(&node.key) {
                    self.expanded_paths.remove(&node.key);
                } else {
                    self.expanded_paths.insert(node.key.clone());
                }
                self.update_filtered_nodes();
            } else {
                // Pour un host, on sélectionne
                self.toggle_selection();
            }
        }
    }

    /// Réduit le nœud courant
    fn collapse_current(&mut self) {
        if let Some(&node_index) = self.filtered_nodes.get(self.selection_cursor) {
            let node = &self.tree_nodes[node_index];

            if node.node_type != NodeType::Host {
                self.expanded_paths.remove(&node.key);
                self.update_filtered_nodes();
            } else if node.level > 0 {
                // Remonter au parent et le fermer
                let path_parts: Vec<&str> = node.key.split('/').collect();
                if path_parts.len() > 1 {
                    let parent_path = path_parts[0..path_parts.len() - 1].join("/");
                    self.expanded_paths.remove(&parent_path);
                    self.update_filtered_nodes();
                }
            }
        }
    }

    /// Bascule la sélection du serveur courant
    fn toggle_selection(&mut self) {
        if let Some(&node_index) = self.filtered_nodes.get(self.selection_cursor) {
            let node = &self.tree_nodes[node_index];

            if let Some(host_entry) = &node.host_entry {
                if self.selected_hosts.contains_key(&node.display_name) {
                    self.selected_hosts.remove(&node.display_name);
                } else {
                    self.selected_hosts
                        .insert(node.display_name.clone(), host_entry.clone());
                }
            }
        }
    }

    /// Sélectionne tous les serveurs visibles
    fn select_all_visible(&mut self) {
        for &node_index in &self.filtered_nodes {
            let node = &self.tree_nodes[node_index];
            if let Some(host_entry) = &node.host_entry {
                self.selected_hosts
                    .insert(node.display_name.clone(), host_entry.clone());
            }
        }
    }

    /// Vide la sélection
    fn clear_selection(&mut self) {
        self.selected_hosts.clear();
    }

    /// Rendu de l'interface (version compatible sans thème)
    #[allow(dead_code)]
    pub fn render(&self, f: &mut Frame, area: Rect) {
        // Utiliser les couleurs par défaut (pour compatibilité)
        let theme_colors = crate::ui::theme::get_theme_colors();
        self.render_with_theme(f, area, &theme_colors);
    }

    pub fn render_with_theme(&self, f: &mut Frame, area: Rect, theme_colors: &ThemeColors) {
        // Diviser la zone : arbre + sélectionnés + aide
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),   // Arbre
                Constraint::Length(6), // Sélectionnés
                Constraint::Length(4), // Aide
            ])
            .split(area);

        self.render_tree_with_theme(f, chunks[0], theme_colors);
        self.render_selected_with_theme(f, chunks[1], theme_colors);
        self.render_help_with_theme(f, chunks[2], theme_colors);

        // Overlay de recherche si actif
        if self.search_mode {
            self.render_search_overlay_with_theme(f, area, theme_colors);
        }
    }

    /// Rendu de l'arbre hiérarchique (version compatible sans thème)
    #[allow(dead_code)]
    fn render_tree(&self, f: &mut Frame, area: Rect) {
        // Utiliser les couleurs par défaut (pour compatibilité)
        let theme_colors = crate::ui::theme::get_theme_colors();
        self.render_tree_with_theme(f, area, &theme_colors);
    }

    /// Rendu de l'arbre hiérarchique avec thème
    fn render_tree_with_theme(&self, f: &mut Frame, area: Rect, theme_colors: &ThemeColors) {
        let mut items = Vec::new();

        for &node_index in &self.filtered_nodes {
            let node = &self.tree_nodes[node_index];

            // Indentation selon le niveau
            let indent = "  ".repeat(node.level);

            // Icône selon le type et l'état
            let icon = match node.node_type {
                NodeType::Environment => {
                    if self.expanded_paths.contains(&node.key) {
                        "📂"
                    } else {
                        "📁"
                    }
                }
                NodeType::Region => {
                    if self.expanded_paths.contains(&node.key) {
                        "🌐"
                    } else {
                        "🗺️"
                    }
                }
                NodeType::ServerType => {
                    if self.expanded_paths.contains(&node.key) {
                        "📊"
                    } else {
                        "📋"
                    }
                }
                NodeType::Host => {
                    if self.selected_hosts.contains_key(&node.display_name) {
                        "✅"
                    } else {
                        "🖥️"
                    }
                }
            };

            // Détails supplémentaires pour les hosts
            let display_text = if let Some(host_entry) = &node.host_entry {
                format!(
                    "{}{} {} ({})",
                    indent, icon, node.display_name, host_entry.alias
                )
            } else {
                format!("{}{} {}", indent, icon, node.display_name)
            };

            // Style selon l'état avec thème
            let style = match node.node_type {
                NodeType::Environment => ratatui_theme::title_primary_style(theme_colors),
                NodeType::Region => ratatui_theme::title_secondary_style(theme_colors),
                NodeType::ServerType => ratatui_theme::text_accent_style(theme_colors),
                NodeType::Host => {
                    if self.selected_hosts.contains_key(&node.display_name) {
                        ratatui_theme::success_style(theme_colors)
                    } else {
                        ratatui_theme::unselected_item_style(theme_colors)
                    }
                }
            };

            items.push(ListItem::new(display_text).style(style));
        }

        let title = if !self.search_query.is_empty() {
            format!("🔍 Serveurs (recherche: '{}')", self.search_query)
        } else {
            "🌳 Serveurs Hiérarchiques".to_string()
        };

        let tree_list = List::new(items)
            .block(ratatui_theme::themed_block(theme_colors, &title))
            .highlight_style(ratatui_theme::selection_style(theme_colors));

        let mut list_state = ListState::default();
        list_state.select(Some(self.selection_cursor));
        f.render_stateful_widget(tree_list, area, &mut list_state);
    }

    /// Rendu des serveurs sélectionnés (version compatible sans thème)
    #[allow(dead_code)]
    fn render_selected(&self, f: &mut Frame, area: Rect) {
        // Utiliser les couleurs par défaut (pour compatibilité)
        let theme_colors = crate::ui::theme::get_theme_colors();
        self.render_selected_with_theme(f, area, &theme_colors);
    }

    /// Rendu des serveurs sélectionnés avec thème
    fn render_selected_with_theme(&self, f: &mut Frame, area: Rect, theme_colors: &ThemeColors) {
        let selected_text = if self.selected_hosts.is_empty() {
            Text::from("Aucun serveur sélectionné")
        } else {
            let mut lines = vec![Line::from(format!(
                "🖥️ {} serveur(s) sélectionné(s):",
                self.selected_hosts.len()
            ))];
            for (name, entry) in &self.selected_hosts {
                lines.push(Line::from(format!("  ✅ {} → {}", name, entry.alias)));
            }
            Text::from(lines)
        };

        let selected_servers = Paragraph::new(selected_text)
            .style(ratatui_theme::success_style(theme_colors))
            .block(ratatui_theme::secondary_block(theme_colors, "Sélectionnés"))
            .wrap(Wrap { trim: true });
        f.render_widget(selected_servers, area);
    }

    /// Rendu de l'aide (version compatible sans thème)
    #[allow(dead_code)]
    fn render_help(&self, f: &mut Frame, area: Rect) {
        // Utiliser les couleurs par défaut (pour compatibilité)
        let theme_colors = crate::ui::theme::get_theme_colors();
        self.render_help_with_theme(f, area, &theme_colors);
    }

    /// Rendu de l'aide avec thème
    fn render_help_with_theme(&self, f: &mut Frame, area: Rect, theme_colors: &ThemeColors) {
        let help_text = if self.search_mode {
            "🔍 Mode recherche: Tapez pour filtrer | Entrée: Valider | Esc: Annuler"
        } else {
            "🌳 Navigation: ↑↓ | ←→ Déplier/Réduire | Espace: Sélectionner | /: Rechercher | a: Tout | c: Vider | Tab: Continuer"
        };

        let help = Paragraph::new(help_text)
            .style(ratatui_theme::help_text_style(theme_colors))
            .block(ratatui_theme::themed_block(theme_colors, "Aide"))
            .wrap(Wrap { trim: true });
        f.render_widget(help, area);
    }

    /// Overlay de recherche (version compatible sans thème)
    #[allow(dead_code)]
    fn render_search_overlay(&self, f: &mut Frame, area: Rect) {
        // Utiliser les couleurs par défaut (pour compatibilité)
        let theme_colors = crate::ui::theme::get_theme_colors();
        self.render_search_overlay_with_theme(f, area, &theme_colors);
    }

    fn render_search_overlay_with_theme(
        &self,
        f: &mut Frame,
        area: Rect,
        theme_colors: &ThemeColors,
    ) {
        let popup_area = {
            let vertical = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(40),
                    Constraint::Length(3),
                    Constraint::Percentage(60),
                ])
                .split(area);

            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Percentage(60),
                    Constraint::Percentage(20),
                ])
                .split(vertical[1])[1]
        };

        f.render_widget(Clear, popup_area);

        let search_text = format!("🔍 Recherche: {}", self.search_query);
        let search_popup = Paragraph::new(search_text)
            .style(ratatui_theme::selection_style(theme_colors))
            .block(ratatui_theme::primary_block(theme_colors, "Rechercher"))
            .alignment(Alignment::Left);
        f.render_widget(search_popup, popup_area);
    }

    /// Retourne les serveurs sélectionnés dans le format attendu
    pub fn get_selected_hosts(&self) -> Vec<(String, HostEntry)> {
        self.selected_hosts
            .iter()
            .map(|(name, entry)| (name.clone(), entry.clone()))
            .collect()
    }

    /// Réinitialise complètement la sélection
    pub fn reset_selection(&mut self) {
        self.selected_hosts.clear();
        self.selection_cursor = 0;
        self.search_query.clear();
        self.search_mode = false;
        self.expanded_paths.clear();
        self.update_filtered_nodes();
    }

    // Unused method - commented out for optimization
    // pub fn has_selection(&self) -> bool {
    //     !self.selected_hosts.is_empty()
    // }
}
