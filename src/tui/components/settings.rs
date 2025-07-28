use crate::config::{ConfigManager, UiConfig, SessionConfig, LlmConfig, DatabaseConfig, ShortcutsConfig};
use crate::tui::{App, SettingsTab, Layout};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Tabs, Clear, Paragraph, Wrap};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct SettingsModal;

impl SettingsModal {
    pub fn render(
        frame: &mut Frame<'_>,
        app: &App,
        config_manager: Arc<RwLock<ConfigManager>>,
        area: Rect,
    ) {
        let settings_chunks = Layout::settings_layout(area);
        
        // Clear the background
        frame.render_widget(Clear, area);
        
        // Render main settings block
        let settings_block = Block::default()
            .title("Settings (Ctrl+, to toggle)")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue));
        frame.render_widget(settings_block, Layout::centered_rect(80, 70, area));

        // Render tabs
        Self::render_tabs(frame, app, settings_chunks[0]);
        
        // Render content based on selected tab
        tokio::spawn(async move {
            let _config = config_manager.read().await;
            // Note: In a real implementation, we'd need to structure this differently
            // to avoid async in sync context. For now, this shows the structure.
        });
        
        // For now, render placeholder content
        Self::render_placeholder_content(frame, app, settings_chunks[1]);
        
        // Render action buttons
        Self::render_actions(frame, app, settings_chunks[2]);
    }

    fn render_tabs(frame: &mut Frame<'_>, app: &App, area: Rect) {
        let tab_titles: Vec<&str> = vec!["UI", "Session", "LLM", "Database", "Shortcuts"];
        let selected_tab = match app.settings_tab() {
            SettingsTab::Ui => 0,
            SettingsTab::Session => 1,
            SettingsTab::Llm => 2,
            SettingsTab::Database => 3,
            SettingsTab::Shortcuts => 4,
        };

        let tabs = Tabs::new(tab_titles)
            .block(Block::default().borders(Borders::BOTTOM))
            .style(Style::default().fg(Color::Gray))
            .highlight_style(Style::default().fg(Color::Yellow).bold())
            .select(selected_tab);

        frame.render_widget(tabs, area);
    }

    fn render_placeholder_content(frame: &mut Frame<'_>, app: &App, area: Rect) {
        let content = match app.settings_tab() {
            SettingsTab::Ui => Self::render_ui_settings_placeholder(area),
            SettingsTab::Session => Self::render_session_settings_placeholder(area),
            SettingsTab::Llm => Self::render_llm_settings_placeholder(area),
            SettingsTab::Database => Self::render_database_settings_placeholder(area),
            SettingsTab::Shortcuts => Self::render_shortcuts_settings_placeholder(area),
        };
        
        frame.render_widget(content, area);
    }

    fn render_ui_settings_placeholder(_area: Rect) -> Paragraph<'static> {
        let text = vec![
            Line::from("Theme Settings:"),
            Line::from(""),
            Line::from("● Dark   ○ Light   ○ Auto"),
            Line::from(""),
            Line::from("Layout:"),
            Line::from("○ Compact   ● Comfortable   ○ Spacious"),
            Line::from(""),
            Line::from("Animation Speed: [████████──] 200ms"),
            Line::from("Show Line Numbers: ☑"),
            Line::from("Word Wrap: ☑"),
        ];

        Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("UI Settings"))
            .wrap(Wrap { trim: true })
    }

    fn render_session_settings_placeholder(_area: Rect) -> Paragraph<'static> {
        let text = vec![
            Line::from("Chat History: 1000 messages"),
            Line::from("Auto-save: Every 30 seconds"),
            Line::from("Backup Count: 5 files"),
            Line::from(""),
            Line::from("Workspace Settings:"),
            Line::from("Auto-detect workspace: ☑"),
            Line::from("Load last workspace: ☑"),
        ];

        Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Session Settings"))
            .wrap(Wrap { trim: true })
    }

    fn render_llm_settings_placeholder(_area: Rect) -> Paragraph<'static> {
        let text = vec![
            Line::from("Default Model: gpt-3.5-turbo"),
            Line::from("Max Iterations: 10"),
            Line::from("Temperature: 0.7"),
            Line::from(""),
            Line::from("Provider Settings:"),
            Line::from("OpenAI API Key: [Set]"),
            Line::from("Streaming: ☑"),
            Line::from("Timeout: 30 seconds"),
        ];

        Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("LLM Settings"))
            .wrap(Wrap { trim: true })
    }

    fn render_database_settings_placeholder(_area: Rect) -> Paragraph<'static> {
        let text = vec![
            Line::from("Connection Settings:"),
            Line::from("Read-only Mode: ☑"),
            Line::from("Query Timeout: 30 seconds"),
            Line::from("Max Query Time: 300 seconds"),
            Line::from(""),
            Line::from("Performance:"),
            Line::from("Cache Results: ☑"),
            Line::from("Max Cache Size: 100 MB"),
        ];

        Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Database Settings"))
            .wrap(Wrap { trim: true })
    }

    fn render_shortcuts_settings_placeholder(_area: Rect) -> Paragraph<'static> {
        let text = vec![
            Line::from("Navigation:"),
            Line::from("Quit:           Ctrl+Q"),
            Line::from("Settings:       Ctrl+,"),
            Line::from("New Objective:  Ctrl+N"),
            Line::from("Delete Obj:     Ctrl+D"),
            Line::from(""),
            Line::from("Interface:"),
            Line::from("Toggle History: Ctrl+H"),
            Line::from("Save Session:   Ctrl+S"),
        ];

        Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Keyboard Shortcuts"))
            .wrap(Wrap { trim: true })
    }

    fn render_actions(frame: &mut Frame<'_>, app: &App, area: Rect) {
        let action_chunks = Layout::settings_actions_layout(area);
        
        let apply_style = if app.settings_modified() {
            Style::default().fg(Color::Green).bold()
        } else {
            Style::default().fg(Color::Gray)
        };

        let actions = vec![
            ("Apply", apply_style),
            ("Reset", Style::default().fg(Color::Red)),
            ("Export", Style::default().fg(Color::Blue)),
            ("Import", Style::default().fg(Color::Blue)),
            ("Close", Style::default().fg(Color::White)),
        ];

        for (i, (text, style)) in actions.iter().enumerate() {
            if i < action_chunks.len() {
                let button = Paragraph::new(*text)
                    .style(*style)
                    .block(Block::default().borders(Borders::ALL))
                    .alignment(Alignment::Center);
                frame.render_widget(button, action_chunks[i]);
            }
        }
    }

    // Placeholder for future implementation
    pub async fn render_with_config(
        frame: &mut Frame<'_>,
        app: &App,
        config_manager: Arc<RwLock<ConfigManager>>,
        area: Rect,
    ) {
        let config = config_manager.read().await;
        
        match app.settings_tab() {
            SettingsTab::Ui => {
                let ui_config = config.effective_ui_config();
                Self::render_ui_settings_with_config(frame, &ui_config, area);
            }
            SettingsTab::Session => {
                let session_config = config.effective_session_config();
                Self::render_session_settings_with_config(frame, &session_config, area);
            }
            SettingsTab::Llm => {
                let llm_config = config.effective_llm_config();
                Self::render_llm_settings_with_config(frame, &llm_config, area);
            }
            SettingsTab::Database => {
                let db_config = config.effective_database_config();
                Self::render_database_settings_with_config(frame, &db_config, area);
            }
            SettingsTab::Shortcuts => {
                let shortcuts_config = config.effective_shortcuts_config();
                Self::render_shortcuts_settings_with_config(frame, &shortcuts_config, area);
            }
        }
    }

    fn render_ui_settings_with_config(_frame: &mut Frame<'_>, _config: &UiConfig, _area: Rect) {
        // TODO: Implement actual UI settings rendering with real config values
    }

    fn render_session_settings_with_config(_frame: &mut Frame<'_>, _config: &SessionConfig, _area: Rect) {
        // TODO: Implement actual session settings rendering with real config values
    }

    fn render_llm_settings_with_config(_frame: &mut Frame<'_>, _config: &LlmConfig, _area: Rect) {
        // TODO: Implement actual LLM settings rendering with real config values
    }

    fn render_database_settings_with_config(_frame: &mut Frame<'_>, _config: &DatabaseConfig, _area: Rect) {
        // TODO: Implement actual database settings rendering with real config values
    }

    fn render_shortcuts_settings_with_config(_frame: &mut Frame<'_>, _config: &ShortcutsConfig, _area: Rect) {
        // TODO: Implement actual shortcuts settings rendering with real config values
    }
}