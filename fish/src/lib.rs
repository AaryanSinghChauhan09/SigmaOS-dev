// SPDX-License-Identifier: MIT OR Apache-2.0
#![forbid(unsafe_code)]

//! SigmaOS Fish Shell Configuration & Plugin Manager
//!
//! Inspired by the Omarchy opinionated Linux setup, Hyprland tiling compositor,
//! and the Catppuccin Mocha colour palette — this module delivers a fully
//! declarative model for managing fish shell configuration: functions,
//! abbreviations, keybindings, plugin lifecycle (install/update/remove via URL),
//! theme selection, and PATH manipulation.

use std::collections::HashMap;
use std::fmt;

// ── Error types ──────────────────────────────────────────────────────────────

/// All errors that can arise within the fish configuration domain.
#[derive(Debug, PartialEq)]
pub enum FishError {
    /// A required field was empty or contained only whitespace.
    EmptyField(String),
    /// A URL failed basic structural validation.
    InvalidUrl(String),
    /// The requested plugin was not found in the registry.
    PluginNotFound(String),
    /// An abbreviation expansion must not be blank.
    EmptyExpansion(String),
    /// A keybinding sequence string was malformed.
    InvalidBinding(String),
    /// A PATH entry was rejected (e.g. absolute path required).
    InvalidPathEntry(String),
    /// Theme name not recognised.
    UnknownTheme(String),
}

impl fmt::Display for FishError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FishError::EmptyField(field) => write!(f, "field '{}' must not be empty", field),
            FishError::InvalidUrl(url) => write!(f, "invalid plugin URL: '{}'", url),
            FishError::PluginNotFound(name) => write!(f, "plugin '{}' not found", name),
            FishError::EmptyExpansion(abbr) => {
                write!(f, "expansion for abbreviation '{}' must not be empty", abbr)
            }
            FishError::InvalidBinding(seq) => {
                write!(f, "keybinding sequence '{}' is malformed", seq)
            }
            FishError::InvalidPathEntry(p) => {
                write!(f, "PATH entry '{}' must be an absolute path", p)
            }
            FishError::UnknownTheme(t) => write!(f, "unknown theme '{}'", t),
        }
    }
}

impl std::error::Error for FishError {}

pub type Result<T> = std::result::Result<T, FishError>;

// ── Themes ───────────────────────────────────────────────────────────────────

/// Colour themes available in SigmaOS Fish, all matching Catppuccin Mocha
/// base hex values for consistency with the Hyprland / GTK layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Theme {
    /// Catppuccin Mocha (default, dark).
    CatppuccinMocha,
    /// Catppuccin Latte (light variant).
    CatppuccinLatte,
    /// A fully custom theme identified by name.
    Custom(String),
}

impl Theme {
    /// Parse a theme name string into a `Theme` variant.
    pub fn from_str(s: &str) -> Result<Self> {
        match s.trim() {
            "catppuccin-mocha" | "mocha" => Ok(Theme::CatppuccinMocha),
            "catppuccin-latte" | "latte" => Ok(Theme::CatppuccinLatte),
            other if !other.is_empty() => Ok(Theme::Custom(other.to_string())),
            _ => Err(FishError::UnknownTheme(s.to_string())),
        }
    }

    /// Return the canonical name used in fish theme config files.
    pub fn canonical_name(&self) -> &str {
        match self {
            Theme::CatppuccinMocha => "catppuccin-mocha",
            Theme::CatppuccinLatte => "catppuccin-latte",
            Theme::Custom(n) => n.as_str(),
        }
    }
}

impl fmt::Display for Theme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.canonical_name())
    }
}

// ── Plugin lifecycle ──────────────────────────────────────────────────────────

/// Status of a managed fish plugin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginStatus {
    Installed,
    PendingUpdate,
    Removed,
}

impl fmt::Display for PluginStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            PluginStatus::Installed => "installed",
            PluginStatus::PendingUpdate => "pending-update",
            PluginStatus::Removed => "removed",
        };
        write!(f, "{}", s)
    }
}

/// A single managed fish plugin.
#[derive(Debug, Clone)]
pub struct Plugin {
    pub name: String,
    pub url: String,
    pub status: PluginStatus,
}

impl Plugin {
    /// Validate and create a new plugin record.
    pub fn new(name: &str, url: &str) -> Result<Self> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(FishError::EmptyField("plugin name".into()));
        }
        let url = url.trim().to_string();
        if !url.starts_with("https://") && !url.starts_with("http://") {
            return Err(FishError::InvalidUrl(url));
        }
        Ok(Plugin {
            name,
            url,
            status: PluginStatus::Installed,
        })
    }

    /// Mark the plugin as needing an update.
    pub fn mark_pending_update(&mut self) {
        self.status = PluginStatus::PendingUpdate;
    }

    /// Mark the plugin as removed.
    pub fn remove(&mut self) {
        self.status = PluginStatus::Removed;
    }
}

// ── Abbreviation ─────────────────────────────────────────────────────────────

/// A fish shell abbreviation (short alias that expands on `<Space>` or `<Enter>`).
#[derive(Debug, Clone)]
pub struct Abbreviation {
    pub trigger: String,
    pub expansion: String,
}

impl Abbreviation {
    pub fn new(trigger: &str, expansion: &str) -> Result<Self> {
        let trigger = trigger.trim().to_string();
        if trigger.is_empty() {
            return Err(FishError::EmptyField("abbreviation trigger".into()));
        }
        let expansion = expansion.trim().to_string();
        if expansion.is_empty() {
            return Err(FishError::EmptyExpansion(trigger));
        }
        Ok(Abbreviation { trigger, expansion })
    }

    /// Render as a `abbr --add` fish command string.
    pub fn to_fish_command(&self) -> String {
        format!("abbr --add {} '{}'", self.trigger, self.expansion)
    }
}

// ── Keybinding ───────────────────────────────────────────────────────────────

/// A custom keybinding registered with `bind`.
#[derive(Debug, Clone)]
pub struct Keybinding {
    pub sequence: String,
    pub command: String,
}

impl Keybinding {
    pub fn new(sequence: &str, command: &str) -> Result<Self> {
        let sequence = sequence.trim().to_string();
        if sequence.is_empty() || (!sequence.starts_with("\\c") && !sequence.starts_with("\\e") && !sequence.contains('-')) {
            return Err(FishError::InvalidBinding(sequence));
        }
        let command = command.trim().to_string();
        if command.is_empty() {
            return Err(FishError::EmptyField("keybinding command".into()));
        }
        Ok(Keybinding { sequence, command })
    }

    /// Render as a `bind` fish command.
    pub fn to_fish_command(&self) -> String {
        format!("bind {} '{}'", self.sequence, self.command)
    }
}

// ── PATH manager ─────────────────────────────────────────────────────────────

/// Manages custom PATH entries prepended in `config.fish`.
#[derive(Debug, Clone, Default)]
pub struct PathManager {
    entries: Vec<String>,
}

impl PathManager {
    pub fn new() -> Self {
        PathManager::default()
    }

    /// Add an absolute path entry.  Rejects relative paths.
    pub fn add(&mut self, entry: &str) -> Result<()> {
        let entry = entry.trim().to_string();
        if !entry.starts_with('/') {
            return Err(FishError::InvalidPathEntry(entry));
        }
        if !self.entries.contains(&entry) {
            self.entries.push(entry);
        }
        Ok(())
    }

    /// Remove a path entry.  Returns `true` if it existed.
    pub fn remove(&mut self, entry: &str) -> bool {
        let entry = entry.trim();
        if let Some(pos) = self.entries.iter().position(|e| e == entry) {
            self.entries.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn entries(&self) -> &[String] {
        &self.entries
    }

    /// Render as `fish_add_path` statements.
    pub fn to_fish_commands(&self) -> Vec<String> {
        self.entries
            .iter()
            .map(|e| format!("fish_add_path {}", e))
            .collect()
    }
}

// ── FishConfig (top-level config struct) ─────────────────────────────────────

/// Top-level fish shell configuration, built via `FishConfigBuilder`.
#[derive(Debug)]
pub struct FishConfig {
    pub theme: Theme,
    pub abbreviations: Vec<Abbreviation>,
    pub keybindings: Vec<Keybinding>,
    pub path_manager: PathManager,
    plugins: HashMap<String, Plugin>,
}

impl FishConfig {
    fn new(theme: Theme) -> Self {
        FishConfig {
            theme,
            abbreviations: Vec::new(),
            keybindings: Vec::new(),
            path_manager: PathManager::new(),
            plugins: HashMap::new(),
        }
    }

    // ── Plugin operations ────────────────────────────────────────────────

    /// Install a plugin.
    pub fn install_plugin(&mut self, name: &str, url: &str) -> Result<()> {
        let plugin = Plugin::new(name, url)?;
        self.plugins.insert(plugin.name.clone(), plugin);
        Ok(())
    }

    /// Mark a plugin for update.
    pub fn update_plugin(&mut self, name: &str) -> Result<()> {
        self.plugins
            .get_mut(name)
            .ok_or_else(|| FishError::PluginNotFound(name.to_string()))
            .map(|p| p.mark_pending_update())
    }

    /// Remove a plugin.
    pub fn remove_plugin(&mut self, name: &str) -> Result<()> {
        self.plugins
            .get_mut(name)
            .ok_or_else(|| FishError::PluginNotFound(name.to_string()))
            .map(|p| p.remove())
    }

    /// List plugins with the given status.
    pub fn plugins_with_status(&self, status: &PluginStatus) -> Vec<&Plugin> {
        self.plugins
            .values()
            .filter(|p| &p.status == status)
            .collect()
    }

    /// Total plugin count.
    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    // ── Render ───────────────────────────────────────────────────────────

    /// Render a minimal `config.fish` body for the managed configuration.
    pub fn render_config_fish(&self) -> String {
        let mut lines: Vec<String> = vec![
            format!("# SigmaOS Fish — theme: {}", self.theme),
            String::new(),
        ];

        // PATH
        for cmd in self.path_manager.to_fish_commands() {
            lines.push(cmd);
        }
        if !self.path_manager.entries().is_empty() {
            lines.push(String::new());
        }

        // Abbreviations
        for abbr in &self.abbreviations {
            lines.push(abbr.to_fish_command());
        }
        if !self.abbreviations.is_empty() {
            lines.push(String::new());
        }

        // Keybindings
        for kb in &self.keybindings {
            lines.push(kb.to_fish_command());
        }

        lines.join("\n")
    }
}

// ── Builder ───────────────────────────────────────────────────────────────────

/// Builder for `FishConfig` with validation at each step.
#[derive(Default)]
pub struct FishConfigBuilder {
    theme: Option<Theme>,
    abbreviations: Vec<Abbreviation>,
    keybindings: Vec<Keybinding>,
    path_entries: Vec<String>,
}

impl FishConfigBuilder {
    pub fn new() -> Self {
        FishConfigBuilder::default()
    }

    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = Some(theme);
        self
    }

    pub fn abbreviation(mut self, trigger: &str, expansion: &str) -> Result<Self> {
        self.abbreviations.push(Abbreviation::new(trigger, expansion)?);
        Ok(self)
    }

    pub fn keybinding(mut self, sequence: &str, command: &str) -> Result<Self> {
        self.keybindings.push(Keybinding::new(sequence, command)?);
        Ok(self)
    }

    pub fn path_entry(mut self, entry: &str) -> Result<Self> {
        let entry = entry.trim().to_string();
        if !entry.starts_with('/') {
            return Err(FishError::InvalidPathEntry(entry));
        }
        self.path_entries.push(entry);
        Ok(self)
    }

    pub fn build(self) -> Result<FishConfig> {
        let theme = self.theme.unwrap_or(Theme::CatppuccinMocha);
        let mut cfg = FishConfig::new(theme);
        cfg.abbreviations = self.abbreviations;
        cfg.keybindings = self.keybindings;
        for entry in self.path_entries {
            cfg.path_manager.add(&entry)?;
        }
        Ok(cfg)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // 1. Theme parsing – known names
    #[test]
    fn test_theme_from_str_known() {
        // Arrange
        let inputs = [("mocha", Theme::CatppuccinMocha), ("latte", Theme::CatppuccinLatte)];
        for (s, expected) in inputs {
            // Act
            let result = Theme::from_str(s).unwrap();
            // Assert
            assert_eq!(result, expected);
        }
    }

    // 2. Theme parsing – custom name
    #[test]
    fn test_theme_from_str_custom() {
        // Arrange / Act
        let theme = Theme::from_str("nord").unwrap();
        // Assert
        assert_eq!(theme, Theme::Custom("nord".to_string()));
        assert_eq!(theme.canonical_name(), "nord");
    }

    // 3. Theme parsing – empty string returns error
    #[test]
    fn test_theme_from_str_empty_errors() {
        // Arrange / Act
        let result = Theme::from_str("   ");
        // Assert
        assert!(matches!(result, Err(FishError::UnknownTheme(_))));
    }

    // 4. Plugin creation – valid URL
    #[test]
    fn test_plugin_new_valid() {
        // Arrange / Act
        let plugin = Plugin::new("jorgebucaran/fisher", "https://github.com/jorgebucaran/fisher").unwrap();
        // Assert
        assert_eq!(plugin.name, "jorgebucaran/fisher");
        assert_eq!(plugin.status, PluginStatus::Installed);
    }

    // 5. Plugin creation – bad URL rejected
    #[test]
    fn test_plugin_new_invalid_url() {
        // Arrange / Act
        let result = Plugin::new("bad-plugin", "git@github.com:user/repo.git");
        // Assert
        assert!(matches!(result, Err(FishError::InvalidUrl(_))));
    }

    // 6. Plugin lifecycle transitions
    #[test]
    fn test_plugin_lifecycle() {
        // Arrange
        let mut plugin = Plugin::new("tide", "https://github.com/IlanCosman/tide").unwrap();
        assert_eq!(plugin.status, PluginStatus::Installed);

        // Act – mark pending update
        plugin.mark_pending_update();
        // Assert
        assert_eq!(plugin.status, PluginStatus::PendingUpdate);

        // Act – remove
        plugin.remove();
        // Assert
        assert_eq!(plugin.status, PluginStatus::Removed);
    }

    // 7. Abbreviation rendering
    #[test]
    fn test_abbreviation_to_fish_command() {
        // Arrange
        let abbr = Abbreviation::new("gs", "git status").unwrap();
        // Act
        let cmd = abbr.to_fish_command();
        // Assert
        assert_eq!(cmd, "abbr --add gs 'git status'");
    }

    // 8. Abbreviation – empty expansion error
    #[test]
    fn test_abbreviation_empty_expansion_errors() {
        // Arrange / Act
        let result = Abbreviation::new("gs", "   ");
        // Assert
        assert!(matches!(result, Err(FishError::EmptyExpansion(_))));
    }

    // 9. PathManager – valid entry added once (dedup)
    #[test]
    fn test_path_manager_dedup() {
        // Arrange
        let mut pm = PathManager::new();
        // Act
        pm.add("/usr/local/bin").unwrap();
        pm.add("/usr/local/bin").unwrap();
        // Assert
        assert_eq!(pm.entries().len(), 1);
    }

    // 10. PathManager – relative path rejected
    #[test]
    fn test_path_manager_relative_rejected() {
        // Arrange
        let mut pm = PathManager::new();
        // Act
        let result = pm.add("relative/path");
        // Assert
        assert!(matches!(result, Err(FishError::InvalidPathEntry(_))));
    }

    // 11. PathManager – remove entry
    #[test]
    fn test_path_manager_remove() {
        // Arrange
        let mut pm = PathManager::new();
        pm.add("/opt/bin").unwrap();
        // Act
        let removed = pm.remove("/opt/bin");
        // Assert
        assert!(removed);
        assert!(pm.entries().is_empty());
        assert!(!pm.remove("/opt/bin")); // second remove returns false
    }

    // 12. FishConfig install + query plugins
    #[test]
    fn test_fish_config_install_plugin() {
        // Arrange
        let mut cfg = FishConfigBuilder::new().build().unwrap();
        // Act
        cfg.install_plugin("tide", "https://github.com/IlanCosman/tide").unwrap();
        // Assert
        assert_eq!(cfg.plugin_count(), 1);
        let installed = cfg.plugins_with_status(&PluginStatus::Installed);
        assert_eq!(installed.len(), 1);
        assert_eq!(installed[0].name, "tide");
    }

    // 13. FishConfig – update_plugin on unknown name
    #[test]
    fn test_fish_config_update_unknown_plugin_errors() {
        // Arrange
        let mut cfg = FishConfigBuilder::new().build().unwrap();
        // Act
        let result = cfg.update_plugin("nonexistent");
        // Assert
        assert!(matches!(result, Err(FishError::PluginNotFound(_))));
    }

    // 14. Builder – produces rendered config.fish with expected sections
    #[test]
    fn test_builder_render_config_fish() {
        // Arrange
        let cfg = FishConfigBuilder::new()
            .theme(Theme::CatppuccinMocha)
            .abbreviation("gs", "git status").unwrap()
            .path_entry("/home/user/.local/bin").unwrap()
            .build().unwrap();

        // Act
        let rendered = cfg.render_config_fish();

        // Assert
        assert!(rendered.contains("catppuccin-mocha"), "theme header missing");
        assert!(rendered.contains("fish_add_path /home/user/.local/bin"), "PATH missing");
        assert!(rendered.contains("abbr --add gs 'git status'"), "abbreviation missing");
    }

    // 15. Error Display impls are non-empty
    #[test]
    fn test_error_display() {
        let errors: Vec<Box<dyn std::error::Error>> = vec![
            Box::new(FishError::EmptyField("x".into())),
            Box::new(FishError::InvalidUrl("bad".into())),
            Box::new(FishError::PluginNotFound("p".into())),
            Box::new(FishError::EmptyExpansion("a".into())),
            Box::new(FishError::InvalidBinding("b".into())),
            Box::new(FishError::InvalidPathEntry("c".into())),
            Box::new(FishError::UnknownTheme("t".into())),
        ];
        for e in errors {
            assert!(!e.to_string().is_empty());
        }
    }
}
