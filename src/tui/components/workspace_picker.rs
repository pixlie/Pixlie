use crate::config::WorkspaceConfig;
use crate::{ErrorContext, PixlieError, Result};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct WorkspaceInfo {
    pub name: String,
    pub path: PathBuf,
    pub description: Option<String>,
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
    pub objectives_count: usize,
}

pub struct WorkspacePicker {
    workspaces: Vec<WorkspaceInfo>,
    selected: ListState,
    show_browse: bool,
    browse_path: String,
}

impl WorkspacePicker {
    pub fn new() -> Self {
        let mut selected = ListState::default();
        selected.select(Some(0));

        Self {
            workspaces: Vec::new(),
            selected,
            show_browse: false,
            browse_path: String::new(),
        }
    }

    pub async fn load_recent_workspaces(&mut self) -> Result<()> {
        let context = ErrorContext::new().with_context("Loading recent workspaces");

        // For now, we'll look in common directories for workspace files
        // In the future, this could be tracked in a recent workspaces file
        let mut workspaces = Vec::new();

        // Check current directory and subdirectories
        let current_dir = std::env::current_dir().map_err(|e| {
            PixlieError::session(
                format!("Failed to get current directory: {}", e),
                context.clone(),
            )
        })?;

        self.scan_directory_for_workspaces(&current_dir, &mut workspaces)
            .await?;

        // Check user's home directory
        if let Some(home_dir) = dirs::home_dir() {
            self.scan_directory_for_workspaces(&home_dir, &mut workspaces)
                .await?;
        }

        // Sort by last modified date (most recent first)
        workspaces.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));

        self.workspaces = workspaces;

        // Select first workspace if available
        if !self.workspaces.is_empty() {
            self.selected.select(Some(0));
        }

        Ok(())
    }

    async fn scan_directory_for_workspaces(
        &self,
        dir: &PathBuf,
        workspaces: &mut Vec<WorkspaceInfo>,
    ) -> Result<()> {
        let workspace_file = dir.join(".pixlie-workspace.toml");

        if workspace_file.exists() {
            if let Ok(workspace_info) = self.load_workspace_info(&workspace_file).await {
                workspaces.push(workspace_info);
            }
        }

        // Scan subdirectories (but not too deep to avoid performance issues)
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && !path.starts_with(".") {
                    let workspace_file = path.join(".pixlie-workspace.toml");
                    if workspace_file.exists() {
                        if let Ok(workspace_info) = self.load_workspace_info(&workspace_file).await
                        {
                            workspaces.push(workspace_info);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn load_workspace_info(&self, config_path: &PathBuf) -> Result<WorkspaceInfo> {
        let content = tokio::fs::read_to_string(config_path).await.map_err(|e| {
            PixlieError::session(
                format!("Failed to read workspace config: {}", e),
                ErrorContext::new(),
            )
        })?;

        let config: WorkspaceConfig = toml::from_str(&content).map_err(|e| {
            PixlieError::configuration(
                format!("Failed to parse workspace config: {}", e),
                ErrorContext::new(),
            )
        })?;

        let workspace_path = config_path.parent().unwrap().to_path_buf();
        let name = config.metadata.name.clone().unwrap_or_else(|| {
            workspace_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        });

        Ok(WorkspaceInfo {
            name,
            path: workspace_path,
            description: config.metadata.description.clone(),
            last_modified: config.metadata.last_modified,
            objectives_count: config.workspace.pinned_objectives.len(),
        })
    }

    pub fn next(&mut self) {
        let i = match self.selected.selected() {
            Some(i) => {
                if i >= self.workspaces.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.selected.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.selected.selected() {
            Some(i) => {
                if i == 0 {
                    self.workspaces.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.selected.select(Some(i));
    }

    pub fn selected_workspace(&self) -> Option<&WorkspaceInfo> {
        self.selected
            .selected()
            .and_then(|i| self.workspaces.get(i))
    }

    pub fn toggle_browse(&mut self) {
        self.show_browse = !self.show_browse;
        if self.show_browse && self.browse_path.is_empty() {
            if let Ok(current_dir) = std::env::current_dir() {
                self.browse_path = current_dir.to_string_lossy().to_string();
            }
        }
    }

    pub fn set_browse_path(&mut self, path: String) {
        self.browse_path = path;
    }

    pub fn browse_path(&self) -> &str {
        &self.browse_path
    }

    pub fn is_browsing(&self) -> bool {
        self.show_browse
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        // Clear the background
        frame.render_widget(Clear, area);

        let popup_area = crate::tui::layout::Layout::centered_rect(80, 70, area);

        let main_block = Block::default()
            .title("Workspace Picker")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        frame.render_widget(main_block, popup_area);

        let inner_area = popup_area.inner(&Margin {
            horizontal: 1,
            vertical: 1,
        });

        if self.show_browse {
            self.render_browse_mode(frame, inner_area);
        } else {
            self.render_workspace_list(frame, inner_area);
        }
    }

    fn render_workspace_list(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = ratatui::layout::Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Instructions
                Constraint::Min(0),    // Workspace list
                Constraint::Length(3), // Actions
            ])
            .split(area);

        // Instructions
        let instructions = Paragraph::new(vec![
            Line::from("Select a workspace to continue:"),
            Line::from("↑/↓: Navigate • Enter: Select • B: Browse • N: New • Q: Quit"),
        ])
        .block(Block::default().borders(Borders::BOTTOM))
        .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(instructions, chunks[0]);

        // Workspace list
        if self.workspaces.is_empty() {
            let empty_msg = Paragraph::new("No workspaces found. Press 'N' to create a new workspace or 'B' to browse for one.")
                .block(Block::default().title("Recent Workspaces"))
                .wrap(ratatui::widgets::Wrap { trim: true });
            frame.render_widget(empty_msg, chunks[1]);
        } else {
            let items: Vec<ListItem> = self
                .workspaces
                .iter()
                .map(|workspace| {
                    let mut lines = vec![Line::from(Span::styled(
                        workspace.name.clone(),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ))];

                    if let Some(desc) = &workspace.description {
                        lines.push(Line::from(Span::styled(
                            format!("  {}", desc),
                            Style::default().fg(Color::Gray),
                        )));
                    }

                    let mut info_parts = vec![format!("📁 {}", workspace.path.display())];

                    if workspace.objectives_count > 0 {
                        info_parts.push(format!("📌 {} objectives", workspace.objectives_count));
                    }

                    if let Some(modified) = workspace.last_modified {
                        let duration = chrono::Utc::now().signed_duration_since(modified);
                        let time_ago = if duration.num_days() > 0 {
                            format!("{} days ago", duration.num_days())
                        } else if duration.num_hours() > 0 {
                            format!("{} hours ago", duration.num_hours())
                        } else {
                            "Recently".to_string()
                        };
                        info_parts.push(format!("⏰ {}", time_ago));
                    }

                    lines.push(Line::from(Span::styled(
                        format!("  {}", info_parts.join(" • ")),
                        Style::default().fg(Color::DarkGray),
                    )));

                    ListItem::new(lines)
                })
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .title("Recent Workspaces")
                        .borders(Borders::ALL),
                )
                .highlight_style(Style::default().bg(Color::DarkGray))
                .highlight_symbol("❯ ");

            frame.render_stateful_widget(list, chunks[1], &mut self.selected);
        }

        // Actions
        let actions =
            Paragraph::new("B: Browse for workspace • N: Create new workspace • Q: Cancel")
                .block(Block::default().borders(Borders::TOP))
                .style(Style::default().fg(Color::Gray));

        frame.render_widget(actions, chunks[2]);
    }

    fn render_browse_mode(&self, frame: &mut Frame, area: Rect) {
        let chunks = ratatui::layout::Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Instructions
                Constraint::Length(3), // Path input
                Constraint::Min(0),    // Directory contents
                Constraint::Length(3), // Actions
            ])
            .split(area);

        // Instructions
        let instructions = Paragraph::new("Browse for a workspace directory:")
            .block(Block::default().borders(Borders::BOTTOM));
        frame.render_widget(instructions, chunks[0]);

        // Path input
        let path_input = Paragraph::new(self.browse_path.clone())
            .block(Block::default().title("Path").borders(Borders::ALL))
            .style(Style::default().fg(Color::White).bg(Color::DarkGray));
        frame.render_widget(path_input, chunks[1]);

        // Directory contents (placeholder for now)
        let contents = Paragraph::new("Directory browsing functionality coming soon...").block(
            Block::default()
                .title("Directory Contents")
                .borders(Borders::ALL),
        );
        frame.render_widget(contents, chunks[2]);

        // Actions
        let actions =
            Paragraph::new("Enter: Select current path • Esc: Back to workspace list • Q: Cancel")
                .block(Block::default().borders(Borders::TOP))
                .style(Style::default().fg(Color::Gray));
        frame.render_widget(actions, chunks[3]);
    }
}
