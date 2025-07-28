use crate::config::ConfigManager;
use crate::tui::components::{WorkspaceManager, WorkspacePicker};
use crate::{ErrorContext, PixlieError, Result};
use crossterm::event::KeyCode;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    WorkspacePicker,
    Normal,
    Settings,
    WorkspaceManager,
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
    workspace_picker: Option<WorkspacePicker>,
    workspace_manager: Option<WorkspaceManager>,
    needs_workspace_picker: bool,
}

impl App {
    pub fn new(config_manager: ConfigManager) -> Self {
        let needs_workspace_picker = config_manager.workspace.is_none();
        let initial_mode = if needs_workspace_picker {
            AppMode::WorkspacePicker
        } else {
            AppMode::Normal
        };

        Self {
            config_manager: Arc::new(RwLock::new(config_manager)),
            mode: initial_mode,
            settings_tab: SettingsTab::Ui,
            should_quit: false,
            settings_modified: false,
            workspace_picker: None,
            workspace_manager: None,
            needs_workspace_picker,
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
            AppMode::WorkspacePicker => AppMode::Settings,
            AppMode::WorkspaceManager => AppMode::Settings,
        };
    }

    pub fn open_settings(&mut self) {
        self.mode = AppMode::Settings;
    }

    pub fn close_settings(&mut self) {
        self.mode = AppMode::Normal;
        self.settings_modified = false;
    }

    pub async fn open_workspace_picker(&mut self) {
        if self.workspace_picker.is_none() {
            self.workspace_picker = Some(WorkspacePicker::new());
        }

        if let Some(picker) = &mut self.workspace_picker {
            let _ = picker.load_recent_workspaces().await;
        }

        self.mode = AppMode::WorkspacePicker;
    }

    pub fn close_workspace_picker(&mut self) {
        self.mode = AppMode::Normal;
        self.workspace_picker = None;
    }

    pub async fn open_workspace_manager(&mut self) {
        if self.workspace_manager.is_none() {
            self.workspace_manager = Some(WorkspaceManager::new());
        }

        if let Some(manager) = &mut self.workspace_manager {
            let _ = manager.load_workspaces().await;

            // Set current workspace info if available
            let config = self.config_manager.read().await;
            if let Some(workspace_config) = &config.workspace {
                if let Some(workspace_path) = &config.paths.workspace_config {
                    if let Some(parent_path) = workspace_path.parent() {
                        let workspace_info =
                            crate::tui::components::workspace_picker::WorkspaceInfo {
                                name: workspace_config.metadata.name.clone().unwrap_or_else(|| {
                                    parent_path
                                        .file_name()
                                        .unwrap_or_default()
                                        .to_string_lossy()
                                        .to_string()
                                }),
                                path: parent_path.to_path_buf(),
                                description: workspace_config.metadata.description.clone(),
                                last_modified: workspace_config.metadata.last_modified,
                                objectives_count: workspace_config
                                    .workspace
                                    .pinned_objectives
                                    .len(),
                            };
                        manager.set_current_workspace(workspace_info);
                    }
                }
            }
        }

        self.mode = AppMode::WorkspaceManager;
    }

    pub fn close_workspace_manager(&mut self) {
        self.mode = AppMode::Normal;
        self.workspace_manager = None;
    }

    pub fn next_settings_tab(&mut self) {
        self.settings_tab = self.settings_tab.next();
    }

    pub fn previous_settings_tab(&mut self) {
        self.settings_tab = self.settings_tab.previous();
    }

    pub async fn handle_key(&mut self, key: KeyCode) -> Result<()> {
        match self.mode {
            AppMode::WorkspacePicker => self.handle_workspace_picker_key(key).await,
            AppMode::Normal => self.handle_normal_mode_key(key).await,
            AppMode::Settings => self.handle_settings_mode_key(key).await,
            AppMode::WorkspaceManager => self.handle_workspace_manager_key(key).await,
        }
    }

    async fn handle_workspace_picker_key(&mut self, key: KeyCode) -> Result<()> {
        let workspace_path = if let Some(picker) = &mut self.workspace_picker {
            match key {
                KeyCode::Up => {
                    picker.previous();
                    return Ok(());
                }
                KeyCode::Down => {
                    picker.next();
                    return Ok(());
                }
                KeyCode::Enter => {
                    if let Some(selected_workspace) = picker.selected_workspace() {
                        Some(selected_workspace.path.clone())
                    } else {
                        return Ok(());
                    }
                }
                KeyCode::Char('b') | KeyCode::Char('B') => {
                    picker.toggle_browse();
                    return Ok(());
                }
                KeyCode::Char('n') | KeyCode::Char('N') => {
                    // TODO: Open workspace creation wizard
                    return Ok(());
                }
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                    self.quit();
                    return Ok(());
                }
                _ => return Ok(()),
            }
        } else {
            return Ok(());
        };

        // Load the selected workspace
        if let Some(path) = workspace_path {
            self.load_workspace(&path).await?;
            self.close_workspace_picker();
        }
        
        Ok(())
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
            KeyCode::Char('w') => {
                // Ctrl+W is handled by the event handler for workspace manager
            }
            KeyCode::Char(',') => {
                // Ctrl+, is handled by the event handler
                // This is just the ',' character
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_workspace_manager_key(&mut self, key: KeyCode) -> Result<()> {
        let (mode, workspace_path) = if let Some(manager) = &mut self.workspace_manager {
            let mode = manager.mode().clone();
            let mut workspace_path = None;
            
            match mode {
                crate::tui::components::workspace_manager::WorkspaceManagerMode::List => {
                    match key {
                        KeyCode::Up => {
                            manager.previous();
                            return Ok(());
                        }
                        KeyCode::Down => {
                            manager.next();
                            return Ok(());
                        }
                        KeyCode::Enter => {
                            // Open selected workspace
                            if let Some(workspace) = manager.selected_workspace() {
                                workspace_path = Some(workspace.path.clone());
                            }
                        }
                        KeyCode::Char('s') | KeyCode::Char('S') => {
                            // Switch to selected workspace
                            if let Some(workspace) = manager.selected_workspace() {
                                workspace_path = Some(workspace.path.clone());
                            }
                        }
                        KeyCode::Char('d') | KeyCode::Char('D') => {
                            manager.enter_details_mode();
                            return Ok(());
                        }
                        KeyCode::Char('e') | KeyCode::Char('E') => {
                            manager.enter_edit_mode();
                            return Ok(());
                        }
                        KeyCode::Delete => {
                            manager.enter_delete_mode();
                            return Ok(());
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') => {
                            manager.enter_create_mode();
                            return Ok(());
                        }
                        KeyCode::Esc => {
                            self.close_workspace_manager();
                            return Ok(());
                        }
                        _ => return Ok(()),
                    }
                }
                crate::tui::components::workspace_manager::WorkspaceManagerMode::Details => {
                    match key {
                        KeyCode::Enter => {
                            if let Some(workspace) = manager.selected_workspace() {
                                workspace_path = Some(workspace.path.clone());
                            }
                        }
                        KeyCode::Char('s') | KeyCode::Char('S') => {
                            if let Some(workspace) = manager.selected_workspace() {
                                workspace_path = Some(workspace.path.clone());
                            }
                        }
                        KeyCode::Char('e') | KeyCode::Char('E') => {
                            manager.enter_edit_mode();
                            return Ok(());
                        }
                        KeyCode::Delete => {
                            manager.enter_delete_mode();
                            return Ok(());
                        }
                        KeyCode::Esc => {
                            manager.return_to_list();
                            return Ok(());
                        }
                        _ => return Ok(()),
                    }
                }
                crate::tui::components::workspace_manager::WorkspaceManagerMode::Create => {
                    match key {
                        KeyCode::Enter => {
                            manager.next_create_step();
                            return Ok(());
                        }
                        KeyCode::Backspace => {
                            manager.previous_create_step();
                            return Ok(());
                        }
                        KeyCode::Esc => {
                            manager.return_to_list();
                            return Ok(());
                        }
                        _ => return Ok(()),
                    }
                }
                crate::tui::components::workspace_manager::WorkspaceManagerMode::Edit => {
                    match key {
                        KeyCode::Enter => {
                            // TODO: Save changes
                            manager.return_to_list();
                            return Ok(());
                        }
                        KeyCode::Esc => {
                            manager.return_to_list();
                            return Ok(());
                        }
                        _ => return Ok(()),
                    }
                }
                crate::tui::components::workspace_manager::WorkspaceManagerMode::Delete => {
                    match key {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            // TODO: Delete workspace
                            manager.return_to_list();
                            return Ok(());
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            manager.return_to_list();
                            return Ok(());
                        }
                        _ => return Ok(()),
                    }
                }
            }
            
            (mode, workspace_path)
        } else {
            return Ok(());
        };

        // Load workspace if selected
        if let Some(path) = workspace_path {
            self.load_workspace(&path).await?;
            self.close_workspace_manager();
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

    pub fn workspace_picker(&mut self) -> Option<&mut WorkspacePicker> {
        self.workspace_picker.as_mut()
    }

    pub fn workspace_manager(&mut self) -> Option<&mut WorkspaceManager> {
        self.workspace_manager.as_mut()
    }

    async fn load_workspace(&mut self, workspace_path: &std::path::Path) -> Result<()> {
        use crate::config::loader::ConfigLoader;

        let loader = ConfigLoader::new()?;

        if let Some(workspace_config) = loader.load_workspace_config(workspace_path).await? {
            let mut config = self.config_manager.write().await;
            config.workspace = Some(workspace_config);
            config.paths.workspace_config = Some(workspace_path.join(".pixlie-workspace.toml"));
        }

        Ok(())
    }
}
