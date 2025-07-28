use crate::config::ConfigManager;
use crate::{Result, PixlieError, ErrorContext};
use crossterm::event::KeyCode;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    Settings,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsTab {
    Ui,
    Session,
    Llm,
    Database,
    Shortcuts,
}

impl SettingsTab {
    pub fn next(&self) -> Self {
        match self {
            Self::Ui => Self::Session,
            Self::Session => Self::Llm,
            Self::Llm => Self::Database,
            Self::Database => Self::Shortcuts,
            Self::Shortcuts => Self::Ui,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            Self::Ui => Self::Shortcuts,
            Self::Session => Self::Ui,
            Self::Llm => Self::Session,
            Self::Database => Self::Llm,
            Self::Shortcuts => Self::Database,
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Self::Ui => "UI",
            Self::Session => "Session",
            Self::Llm => "LLM",
            Self::Database => "Database",
            Self::Shortcuts => "Shortcuts",
        }
    }
}

pub struct App {
    config_manager: Arc<RwLock<ConfigManager>>,
    mode: AppMode,
    settings_tab: SettingsTab,
    should_quit: bool,
    settings_modified: bool,
}

impl App {
    pub fn new(config_manager: ConfigManager) -> Self {
        Self {
            config_manager: Arc::new(RwLock::new(config_manager)),
            mode: AppMode::Normal,
            settings_tab: SettingsTab::Ui,
            should_quit: false,
            settings_modified: false,
        }
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn mode(&self) -> &AppMode {
        &self.mode
    }

    pub fn settings_tab(&self) -> &SettingsTab {
        &self.settings_tab
    }

    pub fn settings_modified(&self) -> bool {
        self.settings_modified
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn toggle_settings(&mut self) {
        self.mode = match self.mode {
            AppMode::Normal => AppMode::Settings,
            AppMode::Settings => AppMode::Normal,
        };
    }

    pub fn open_settings(&mut self) {
        self.mode = AppMode::Settings;
    }

    pub fn close_settings(&mut self) {
        self.mode = AppMode::Normal;
        self.settings_modified = false;
    }

    pub fn next_settings_tab(&mut self) {
        self.settings_tab = self.settings_tab.next();
    }

    pub fn previous_settings_tab(&mut self) {
        self.settings_tab = self.settings_tab.previous();
    }

    pub async fn handle_key(&mut self, key: KeyCode) -> Result<()> {
        match self.mode {
            AppMode::Normal => self.handle_normal_mode_key(key).await,
            AppMode::Settings => self.handle_settings_mode_key(key).await,
        }
    }

    async fn handle_normal_mode_key(&mut self, key: KeyCode) -> Result<()> {
        let quit_key = {
            let config = self.config_manager.read().await;
            let shortcuts = config.effective_shortcuts_config();
            shortcuts.quit.clone()
        };

        match key {
            KeyCode::Char('q') => {
                if quit_key == "q" {
                    self.quit();
                }
            }
            KeyCode::Char(',') => {
                // Ctrl+, is handled by the event handler
                // This is just the ',' character
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_settings_mode_key(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Esc => {
                if self.settings_modified {
                    // TODO: Show confirmation dialog
                }
                self.close_settings();
            }
            KeyCode::Tab => {
                self.next_settings_tab();
            }
            KeyCode::BackTab => {
                self.previous_settings_tab();
            }
            KeyCode::Char('q') => {
                self.close_settings();
            }
            _ => {
                // Handle setting-specific inputs
                self.handle_setting_input(key).await?;
            }
        }
        Ok(())
    }

    async fn handle_setting_input(&mut self, key: KeyCode) -> Result<()> {
        match self.settings_tab {
            SettingsTab::Ui => self.handle_ui_setting_input(key).await,
            SettingsTab::Session => self.handle_session_setting_input(key).await,
            SettingsTab::Llm => self.handle_llm_setting_input(key).await,
            SettingsTab::Database => self.handle_database_setting_input(key).await,
            SettingsTab::Shortcuts => self.handle_shortcuts_setting_input(key).await,
        }
    }

    async fn handle_ui_setting_input(&mut self, _key: KeyCode) -> Result<()> {
        // TODO: Implement UI setting input handling
        Ok(())
    }

    async fn handle_session_setting_input(&mut self, _key: KeyCode) -> Result<()> {
        // TODO: Implement session setting input handling
        Ok(())
    }

    async fn handle_llm_setting_input(&mut self, _key: KeyCode) -> Result<()> {
        // TODO: Implement LLM setting input handling
        Ok(())
    }

    async fn handle_database_setting_input(&mut self, _key: KeyCode) -> Result<()> {
        // TODO: Implement database setting input handling
        Ok(())
    }

    async fn handle_shortcuts_setting_input(&mut self, _key: KeyCode) -> Result<()> {
        // TODO: Implement shortcuts setting input handling
        Ok(())
    }

    pub async fn save_settings(&mut self) -> Result<()> {
        let config = self.config_manager.write().await;
        config.save().await.map_err(|e| {
            PixlieError::configuration(
                format!("Failed to save settings: {}", e),
                ErrorContext::new().with_context("Settings save"),
            )
        })?;
        self.settings_modified = false;
        Ok(())
    }

    pub async fn reset_settings_to_defaults(&mut self) -> Result<()> {
        let mut config = self.config_manager.write().await;
        match self.settings_tab {
            SettingsTab::Ui => {
                config.global.ui = Default::default();
            }
            SettingsTab::Session => {
                config.global.session = Default::default();
            }
            SettingsTab::Llm => {
                config.global.llm = Default::default();
            }
            SettingsTab::Database => {
                config.global.database = Default::default();
            }
            SettingsTab::Shortcuts => {
                config.global.shortcuts = Default::default();
            }
        }
        self.settings_modified = true;
        Ok(())
    }

    pub async fn get_config_manager(&self) -> Arc<RwLock<ConfigManager>> {
        self.config_manager.clone()
    }
}