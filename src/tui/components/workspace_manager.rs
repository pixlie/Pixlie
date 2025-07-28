use super::workspace_picker::WorkspaceInfo;
use crate::config::WorkspaceConfig;
use crate::{ErrorContext, PixlieError, Result};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum WorkspaceManagerMode {
    List,
    Details,
    Create,
    Edit,
    Delete,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkspaceAction {
    Open,
    Switch,
    Rename,
    Edit,
    Delete,
    CreateNew,
    ShowDetails,
    Close,
}

pub struct WorkspaceManager {
    workspaces: Vec<WorkspaceInfo>,
    selected: ListState,
    mode: WorkspaceManagerMode,
    current_workspace: Option<WorkspaceInfo>,
    action: Option<WorkspaceAction>,

    // Edit state
    edit_name: String,
    edit_description: String,
    edit_path: String,

    // Create new workspace state
    create_name: String,
    create_path: String,
    create_description: String,
    create_step: usize, // 0-4 for the 5 steps in the wizard
}

impl WorkspaceManager {
    pub fn new() -> Self {
        let mut selected = ListState::default();
        selected.select(Some(0));

        Self {
            workspaces: Vec::new(),
            selected,
            mode: WorkspaceManagerMode::List,
            current_workspace: None,
            action: None,
            edit_name: String::new(),
            edit_description: String::new(),
            edit_path: String::new(),
            create_name: String::new(),
            create_path: String::new(),
            create_description: String::new(),
            create_step: 0,
        }
    }

    pub async fn load_workspaces(&mut self) -> Result<()> {
        let context = ErrorContext::new().with_context("Loading workspaces for manager");

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

        // Sort by name
        workspaces.sort_by(|a, b| a.name.cmp(&b.name));

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

        // Scan subdirectories
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

    pub fn set_current_workspace(&mut self, workspace: WorkspaceInfo) {
        self.current_workspace = Some(workspace);
    }

    pub fn next(&mut self) {
        let len = match self.mode {
            WorkspaceManagerMode::List => self.workspaces.len(),
            _ => return,
        };

        let i = match self.selected.selected() {
            Some(i) => {
                if i >= len - 1 {
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
        let len = match self.mode {
            WorkspaceManagerMode::List => self.workspaces.len(),
            _ => return,
        };

        let i = match self.selected.selected() {
            Some(i) => {
                if i == 0 {
                    len - 1
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

    pub fn enter_details_mode(&mut self) {
        if self.selected_workspace().is_some() {
            self.mode = WorkspaceManagerMode::Details;
        }
    }

    pub fn enter_create_mode(&mut self) {
        self.mode = WorkspaceManagerMode::Create;
        self.create_step = 0;
        self.create_name.clear();
        self.create_path.clear();
        self.create_description.clear();

        // Set default path to current directory
        if let Ok(current_dir) = std::env::current_dir() {
            self.create_path = current_dir.to_string_lossy().to_string();
        }
    }

    pub fn enter_edit_mode(&mut self) {
        let workspace_info = if let Some(workspace) = self.selected_workspace() {
            (workspace.name.clone(), 
             workspace.description.clone().unwrap_or_default(),
             workspace.path.to_string_lossy().to_string())
        } else {
            return;
        };
        
        self.mode = WorkspaceManagerMode::Edit;
        self.edit_name = workspace_info.0;
        self.edit_description = workspace_info.1;
        self.edit_path = workspace_info.2;
    }

    pub fn enter_delete_mode(&mut self) {
        if self.selected_workspace().is_some() {
            self.mode = WorkspaceManagerMode::Delete;
        }
    }

    pub fn return_to_list(&mut self) {
        self.mode = WorkspaceManagerMode::List;
    }

    pub fn next_create_step(&mut self) {
        if self.create_step < 4 {
            self.create_step += 1;
        }
    }

    pub fn previous_create_step(&mut self) {
        if self.create_step > 0 {
            self.create_step -= 1;
        }
    }

    pub fn mode(&self) -> &WorkspaceManagerMode {
        &self.mode
    }

    pub fn get_action(&mut self) -> Option<WorkspaceAction> {
        self.action.take()
    }

    pub fn set_action(&mut self, action: WorkspaceAction) {
        self.action = Some(action);
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        // Clear the background
        frame.render_widget(Clear, area);

        let popup_area = crate::tui::layout::Layout::centered_rect(90, 80, area);

        let main_block = Block::default()
            .title("Workspace Manager")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        frame.render_widget(main_block, popup_area);

        let inner_area = popup_area.inner(&Margin {
            horizontal: 1,
            vertical: 1,
        });

        match self.mode {
            WorkspaceManagerMode::List => self.render_list_mode(frame, inner_area),
            WorkspaceManagerMode::Details => self.render_details_mode(frame, inner_area),
            WorkspaceManagerMode::Create => self.render_create_mode(frame, inner_area),
            WorkspaceManagerMode::Edit => self.render_edit_mode(frame, inner_area),
            WorkspaceManagerMode::Delete => self.render_delete_mode(frame, inner_area),
        }
    }

    fn render_list_mode(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = ratatui::layout::Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4), // Current workspace + instructions
                Constraint::Min(0),    // Workspace list
                Constraint::Length(3), // Actions
            ])
            .split(area);

        // Current workspace and instructions
        let current_text = if let Some(current) = &self.current_workspace {
            format!("Current: {} ({})", current.name, current.path.display())
        } else {
            "Current: None".to_string()
        };

        let header = Paragraph::new(vec![
            Line::from(Span::styled(current_text, Style::default().fg(Color::Green))),
            Line::from(""),
            Line::from("↑/↓: Navigate • Enter: Open • S: Switch • D: Details • E: Edit • Del: Delete • N: New • Esc: Close"),
        ])
        .block(Block::default().borders(Borders::BOTTOM))
        .wrap(ratatui::widgets::Wrap { trim: true });

        frame.render_widget(header, chunks[0]);

        // Workspace list
        if self.workspaces.is_empty() {
            let empty_msg =
                Paragraph::new("No workspaces found. Press 'N' to create a new workspace.")
                    .block(Block::default().title("Available Workspaces"))
                    .wrap(ratatui::widgets::Wrap { trim: true });
            frame.render_widget(empty_msg, chunks[1]);
        } else {
            let items: Vec<ListItem> = self
                .workspaces
                .iter()
                .enumerate()
                .map(|(_i, workspace)| {
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

                    // Add current workspace indicator
                    if let Some(current) = &self.current_workspace {
                        if current.path == workspace.path {
                            lines.insert(
                                0,
                                Line::from(Span::styled(
                                    "● CURRENT",
                                    Style::default()
                                        .fg(Color::Green)
                                        .add_modifier(Modifier::BOLD),
                                )),
                            );
                        }
                    }

                    ListItem::new(lines)
                })
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .title("Available Workspaces")
                        .borders(Borders::ALL),
                )
                .highlight_style(Style::default().bg(Color::DarkGray))
                .highlight_symbol("❯ ");

            frame.render_stateful_widget(list, chunks[1], &mut self.selected);
        }

        // Actions
        let actions = Paragraph::new("N: New workspace • I: Import • G: Settings • Esc: Close")
            .block(Block::default().borders(Borders::TOP))
            .style(Style::default().fg(Color::Gray));

        frame.render_widget(actions, chunks[2]);
    }

    fn render_details_mode(&self, frame: &mut Frame, area: Rect) {
        if let Some(workspace) = self.selected_workspace() {
            let chunks = ratatui::layout::Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Title
                    Constraint::Min(0),    // Details
                    Constraint::Length(3), // Actions
                ])
                .split(area);

            // Title
            let title = Paragraph::new(format!("Workspace Details: {}", workspace.name))
                .block(Block::default().borders(Borders::BOTTOM))
                .style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                );
            frame.render_widget(title, chunks[0]);

            // Details
            let mut details = vec![
                Line::from(vec![
                    Span::styled("Name: ", Style::default().fg(Color::Cyan)),
                    Span::raw(workspace.name.clone()),
                ]),
                Line::from(vec![
                    Span::styled("Path: ", Style::default().fg(Color::Cyan)),
                    Span::raw(workspace.path.display().to_string()),
                ]),
            ];

            if let Some(desc) = &workspace.description {
                details.push(Line::from(vec![
                    Span::styled("Description: ", Style::default().fg(Color::Cyan)),
                    Span::raw(desc.clone()),
                ]));
            }

            details.push(Line::from(vec![
                Span::styled("Objectives: ", Style::default().fg(Color::Cyan)),
                Span::raw(workspace.objectives_count.to_string()),
            ]));

            if let Some(modified) = workspace.last_modified {
                details.push(Line::from(vec![
                    Span::styled("Last Modified: ", Style::default().fg(Color::Cyan)),
                    Span::raw(modified.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
                ]));
            }

            let details_widget = Paragraph::new(details)
                .block(Block::default().title("Details").borders(Borders::ALL))
                .wrap(ratatui::widgets::Wrap { trim: true });

            frame.render_widget(details_widget, chunks[1]);

            // Actions
            let actions =
                Paragraph::new("Enter: Open • S: Switch • E: Edit • Del: Delete • Esc: Back")
                    .block(Block::default().borders(Borders::TOP))
                    .style(Style::default().fg(Color::Gray));

            frame.render_widget(actions, chunks[2]);
        }
    }

    fn render_create_mode(&self, frame: &mut Frame, area: Rect) {
        let step_titles = vec![
            "Step 1: Location and Name",
            "Step 2: Description and Tags",
            "Step 3: Default Database",
            "Step 4: Import Settings",
            "Step 5: Create Objectives",
        ];

        let chunks = ratatui::layout::Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Progress
                Constraint::Min(0),    // Step content
                Constraint::Length(3), // Actions
            ])
            .split(area);

        // Progress indicator
        let progress = Paragraph::new(format!(
            "{} ({}/5)",
            step_titles[self.create_step],
            self.create_step + 1
        ))
        .block(Block::default().borders(Borders::BOTTOM))
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
        frame.render_widget(progress, chunks[0]);

        // Step content
        match self.create_step {
            0 => self.render_create_step_1(frame, chunks[1]),
            1 => self.render_create_step_2(frame, chunks[1]),
            2 => self.render_create_step_3(frame, chunks[1]),
            3 => self.render_create_step_4(frame, chunks[1]),
            4 => self.render_create_step_5(frame, chunks[1]),
            _ => {}
        }

        // Actions
        let actions = if self.create_step == 0 {
            "Tab: Next field • Enter: Next step • Esc: Cancel"
        } else if self.create_step == 4 {
            "Tab: Next field • Enter: Create workspace • Backspace: Previous step • Esc: Cancel"
        } else {
            "Tab: Next field • Enter: Next step • Backspace: Previous step • Esc: Cancel"
        };

        let actions_widget = Paragraph::new(actions)
            .block(Block::default().borders(Borders::TOP))
            .style(Style::default().fg(Color::Gray));

        frame.render_widget(actions_widget, chunks[2]);
    }

    fn render_create_step_1(&self, frame: &mut Frame, area: Rect) {
        let chunks = ratatui::layout::Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Name input
                Constraint::Length(3), // Path input
                Constraint::Min(0),    // Instructions
            ])
            .split(area);

        // Name input
        let name_input = Paragraph::new(self.create_name.clone())
            .block(
                Block::default()
                    .title("Workspace Name")
                    .borders(Borders::ALL),
            )
            .style(Style::default().fg(Color::White).bg(Color::DarkGray));
        frame.render_widget(name_input, chunks[0]);

        // Path input
        let path_input = Paragraph::new(self.create_path.clone())
            .block(
                Block::default()
                    .title("Workspace Path")
                    .borders(Borders::ALL),
            )
            .style(Style::default().fg(Color::White));
        frame.render_widget(path_input, chunks[1]);

        // Instructions
        let instructions = Paragraph::new(
            "Choose a name and location for your new workspace. The workspace name will be used for display and the path determines where workspace files are stored."
        )
        .block(Block::default().title("Instructions").borders(Borders::ALL))
        .wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(instructions, chunks[2]);
    }

    fn render_create_step_2(&self, frame: &mut Frame, area: Rect) {
        let chunks = ratatui::layout::Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5), // Description input
                Constraint::Min(0),    // Instructions
            ])
            .split(area);

        // Description input
        let desc_input = Paragraph::new(self.create_description.clone())
            .block(
                Block::default()
                    .title("Description (Optional)")
                    .borders(Borders::ALL),
            )
            .style(Style::default().fg(Color::White).bg(Color::DarkGray))
            .wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(desc_input, chunks[0]);

        // Instructions
        let instructions = Paragraph::new(
            "Add an optional description to help identify this workspace. You can also add tags for organization (coming soon)."
        )
        .block(Block::default().title("Instructions").borders(Borders::ALL))
        .wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(instructions, chunks[1]);
    }

    fn render_create_step_3(&self, frame: &mut Frame, area: Rect) {
        let instructions = Paragraph::new(
            "Configure a default database for this workspace (optional). You can always add databases later through the workspace settings."
        )
        .block(Block::default().title("Default Database Configuration").borders(Borders::ALL))
        .wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(instructions, area);
    }

    fn render_create_step_4(&self, frame: &mut Frame, area: Rect) {
        let instructions = Paragraph::new(
            "Import settings from an existing workspace or template (optional). This will copy configuration like UI preferences, shortcuts, and tool settings."
        )
        .block(Block::default().title("Import Settings").borders(Borders::ALL))
        .wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(instructions, area);
    }

    fn render_create_step_5(&self, frame: &mut Frame, area: Rect) {
        let instructions = Paragraph::new(
            "Create some pinned objectives for this workspace (optional). Objectives help organize your analysis tasks and can be auto-loaded when you open the workspace."
        )
        .block(Block::default().title("Create Pinned Objectives").borders(Borders::ALL))
        .wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(instructions, area);
    }

    fn render_edit_mode(&self, frame: &mut Frame, area: Rect) {
        let chunks = ratatui::layout::Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(3), // Name input
                Constraint::Length(5), // Description input
                Constraint::Min(0),    // Path (read-only)
                Constraint::Length(3), // Actions
            ])
            .split(area);

        // Title
        let title = Paragraph::new("Edit Workspace")
            .block(Block::default().borders(Borders::BOTTOM))
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );
        frame.render_widget(title, chunks[0]);

        // Name input
        let name_input = Paragraph::new(self.edit_name.clone())
            .block(Block::default().title("Name").borders(Borders::ALL))
            .style(Style::default().fg(Color::White).bg(Color::DarkGray));
        frame.render_widget(name_input, chunks[1]);

        // Description input
        let desc_input = Paragraph::new(self.edit_description.clone())
            .block(Block::default().title("Description").borders(Borders::ALL))
            .style(Style::default().fg(Color::White).bg(Color::DarkGray))
            .wrap(ratatui::widgets::Wrap { trim: true });
        frame.render_widget(desc_input, chunks[2]);

        // Path (read-only)
        let path_display = Paragraph::new(self.edit_path.clone())
            .block(
                Block::default()
                    .title("Path (Read-only)")
                    .borders(Borders::ALL),
            )
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(path_display, chunks[3]);

        // Actions
        let actions = Paragraph::new("Tab: Next field • Enter: Save changes • Esc: Cancel")
            .block(Block::default().borders(Borders::TOP))
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(actions, chunks[4]);
    }

    fn render_delete_mode(&self, frame: &mut Frame, area: Rect) {
        if let Some(workspace) = self.selected_workspace() {
            let chunks = ratatui::layout::Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Title
                    Constraint::Min(0),    // Warning
                    Constraint::Length(3), // Actions
                ])
                .split(area);

            // Title
            let title = Paragraph::new("Delete Workspace")
                .block(Block::default().borders(Borders::BOTTOM))
                .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
            frame.render_widget(title, chunks[0]);

            // Warning
            let warning = Paragraph::new(vec![
                Line::from(Span::styled(
                    "⚠️  WARNING: This action cannot be undone!",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(format!("You are about to delete the workspace:")),
                Line::from(format!("Name: {}", workspace.name)),
                Line::from(format!("Path: {}", workspace.path.display())),
                Line::from(""),
                Line::from(
                    "This will remove the workspace configuration file (.pixlie-workspace.toml).",
                ),
                Line::from(
                    "Your data files will NOT be deleted, only the workspace configuration.",
                ),
                Line::from(""),
                Line::from("Are you sure you want to continue?"),
            ])
            .block(
                Block::default()
                    .title("Confirmation Required")
                    .borders(Borders::ALL),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

            frame.render_widget(warning, chunks[1]);

            // Actions
            let actions = Paragraph::new("Y: Yes, delete workspace • N: No, cancel • Esc: Cancel")
                .block(Block::default().borders(Borders::TOP))
                .style(Style::default().fg(Color::Gray));

            frame.render_widget(actions, chunks[2]);
        }
    }
}
