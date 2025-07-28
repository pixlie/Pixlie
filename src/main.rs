use clap::Parser;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use pixlie::tui::components::SettingsModal;
use pixlie::tui::{App, Event, EventHandler};
use pixlie::{
    init_logging, log_error, ConfigManager, ErrorContext, ErrorSeverity, LoggingConfig,
    PixlieError, Result,
};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use std::io;
use std::process;
use tracing::{debug, info};

#[derive(Parser)]
#[command(name = "pixlie")]
#[command(about = "LLM-enabled TUI data analysis tool for SQLite databases")]
#[command(version = "0.1.0")]
pub struct Args {
    /// Optional workspace path to open on startup
    #[arg(value_name = "WORKSPACE")]
    pub workspace: Option<String>,
}

async fn run_application(args: Args) -> Result<()> {
    let _context = ErrorContext::new().with_context("Application startup");

    info!("🚀 Pixlie TUI Data Analyzer v0.1.0");

    // Initialize configuration manager
    let mut config_manager = ConfigManager::new()?;

    // Load workspace if specified
    if let Some(workspace_path) = &args.workspace {
        info!(workspace = workspace_path, "📁 Loading specified workspace");
        load_workspace_configuration(&mut config_manager, workspace_path).await?;
    } else {
        // Auto-detect workspace in current directory
        if let Some(detected_workspace) = detect_workspace_in_current_dir().await? {
            info!(workspace = detected_workspace, "📁 Auto-detected workspace");
            load_workspace_configuration(&mut config_manager, &detected_workspace).await?;
        } else {
            info!("📁 No workspace detected, using default configuration");
        }
    }

    // Load configuration from environment and files (no CLI args to override)
    load_basic_configuration(&mut config_manager).await?;

    // Get effective configuration for TUI startup
    let ui_config = config_manager.effective_ui_config();
    let _session_config = config_manager.effective_session_config();

    debug!("Configuration loaded for TUI startup");
    debug!("UI theme: {}", ui_config.theme);
    debug!("UI layout: {}", ui_config.layout);

    // Display startup information
    if let Some(workspace) = &args.workspace {
        info!(workspace = workspace, "📁 Workspace");

        if let Some(workspace_config) = &config_manager.workspace {
            if let Some(name) = &workspace_config.metadata.name {
                info!(workspace_name = name, "📁 Workspace name");
            }

            let pinned_count = workspace_config.workspace.pinned_objectives.len();
            if pinned_count > 0 {
                info!(
                    pinned_objectives = pinned_count,
                    "📌 Pinned objectives available"
                );
            }
        }
    }

    info!("🚀 Starting TUI interface...");

    // Launch actual TUI interface
    start_tui(config_manager).await
}

/// Start the TUI interface
async fn start_tui(config_manager: ConfigManager) -> Result<()> {
    // Setup terminal
    enable_raw_mode().map_err(|e| {
        PixlieError::session(
            format!("Failed to enable raw mode: {}", e),
            ErrorContext::new().with_context("TUI initialization"),
        )
    })?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(|e| {
        PixlieError::session(
            format!("Failed to enter alternate screen: {}", e),
            ErrorContext::new().with_context("TUI initialization"),
        )
    })?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| {
        PixlieError::session(
            format!("Failed to create terminal: {}", e),
            ErrorContext::new().with_context("TUI initialization"),
        )
    })?;

    // Create app and event handler
    let mut app = App::new(config_manager);

    // If no workspace is loaded, open the workspace picker
    if app.mode() == &pixlie::tui::AppMode::WorkspacePicker {
        app.open_workspace_picker().await;
    }

    let mut event_handler = EventHandler::new();

    let result = run_tui_loop(&mut terminal, &mut app, &mut event_handler).await;

    // Restore terminal
    disable_raw_mode().map_err(|e| {
        PixlieError::session(
            format!("Failed to disable raw mode: {}", e),
            ErrorContext::new().with_context("TUI cleanup"),
        )
    })?;

    execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(|e| {
        PixlieError::session(
            format!("Failed to leave alternate screen: {}", e),
            ErrorContext::new().with_context("TUI cleanup"),
        )
    })?;

    terminal.show_cursor().map_err(|e| {
        PixlieError::session(
            format!("Failed to show cursor: {}", e),
            ErrorContext::new().with_context("TUI cleanup"),
        )
    })?;

    result
}

/// Run the main TUI event loop
async fn run_tui_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    event_handler: &mut EventHandler,
) -> Result<()> {
    loop {
        // Render the UI
        terminal
            .draw(|frame| {
                render_ui(frame, app);
            })
            .map_err(|e| {
                PixlieError::session(
                    format!("Failed to draw terminal: {}", e),
                    ErrorContext::new().with_context("TUI rendering"),
                )
            })?;

        // Handle events
        if let Some(event) = event_handler.next().await {
            match event {
                Event::Key(key_event) => {
                    // Handle special key combinations
                    if key_event.code == crossterm::event::KeyCode::F(12) {
                        app.toggle_settings();
                    } else if key_event.code == crossterm::event::KeyCode::F(11) {
                        app.open_workspace_manager().await;
                    } else {
                        app.handle_key(key_event.code).await?;
                    }
                }
                Event::Resize(_, _) => {
                    // Terminal was resized, will be handled on next draw
                }
                Event::Quit => {
                    break;
                }
            }
        }

        if app.should_quit() {
            break;
        }
    }

    Ok(())
}

/// Render the main UI
fn render_ui(frame: &mut Frame, app: &mut App) {
    let area = frame.size();

    match app.mode() {
        pixlie::tui::AppMode::WorkspacePicker => {
            if let Some(picker) = app.workspace_picker() {
                picker.render(frame, area);
            }
        }
        pixlie::tui::AppMode::Normal => {
            render_normal_mode(frame, app, area);
        }
        pixlie::tui::AppMode::Settings => {
            render_normal_mode(frame, app, area); // Render background

            // Get config manager for settings modal
            let config_manager = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async { app.get_config_manager().await })
            });

            SettingsModal::render(frame, app, config_manager, area);
        }
        pixlie::tui::AppMode::WorkspaceManager => {
            render_normal_mode(frame, app, area); // Render background

            if let Some(manager) = app.workspace_manager() {
                manager.render(frame, area);
            }
        }
    }
}

/// Render normal mode (main interface)
fn render_normal_mode(frame: &mut Frame, app: &App, area: Rect) {
    let workspace_info = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let config = app.get_config_manager().await;
            let config = config.read().await;

            if let Some(workspace_config) = &config.workspace {
                let workspace_name = workspace_config
                    .metadata
                    .name
                    .clone()
                    .unwrap_or_else(|| "Unknown Workspace".to_string());
                let objectives_count = workspace_config.workspace.pinned_objectives.len();

                Some((workspace_name, objectives_count))
            } else {
                None
            }
        })
    });

    let main_title = if let Some((workspace_name, _)) = &workspace_info {
        format!("Pixlie - {} Workspace", workspace_name)
    } else {
        "Pixlie - LLM Data Analysis Tool".to_string()
    };

    let main_block = Block::default().title(main_title).borders(Borders::ALL);

    let mut content_lines = vec![Line::from("Welcome to Pixlie!"), Line::from("")];

    if let Some((workspace_name, objectives_count)) = workspace_info {
        content_lines.extend(vec![
            Line::from(vec![
                Span::styled("Current Workspace: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    workspace_name,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Pinned Objectives: ", Style::default().fg(Color::Cyan)),
                Span::raw(objectives_count.to_string()),
            ]),
            Line::from(""),
        ]);
    } else {
        content_lines.extend(vec![
            Line::from(Span::styled(
                "No workspace loaded",
                Style::default().fg(Color::Red),
            )),
            Line::from(""),
        ]);
    }

    content_lines.extend(vec![
        Line::from("Keyboard Shortcuts:"),
        Line::from("• Ctrl+W: Open Workspace Manager"),
        Line::from("• Ctrl+,: Open Settings"),
        Line::from("• Ctrl+Q: Quit"),
        Line::from(""),
        Line::from("This is the main interface placeholder."),
        Line::from("The workspace management system is now ready!"),
    ]);

    let content = Paragraph::new(content_lines)
        .block(main_block)
        .wrap(ratatui::widgets::Wrap { trim: true });

    frame.render_widget(content, area);
}

/// Load workspace configuration if workspace exists
async fn load_workspace_configuration(
    config_manager: &mut ConfigManager,
    workspace_path: &str,
) -> Result<()> {
    use pixlie::ConfigLoader;

    let loader = ConfigLoader::new()?;

    // Check if workspace configuration exists
    if loader.workspace_config_exists(workspace_path) {
        debug!("Loading workspace configuration from: {}", workspace_path);
        if let Some(workspace_config) = loader.load_workspace_config(workspace_path).await? {
            config_manager.workspace = Some(workspace_config);
            config_manager.paths.workspace_config =
                Some(std::path::PathBuf::from(workspace_path).join(".pixlie-workspace.toml"));
        }
    } else {
        debug!("No workspace configuration found at: {}", workspace_path);
    }

    Ok(())
}

/// Load basic configuration (global config + environment variables)
async fn load_basic_configuration(config_manager: &mut ConfigManager) -> Result<()> {
    use pixlie::ConfigLoader;

    let loader = ConfigLoader::new()?;

    // Load global configuration
    config_manager.global = loader.load_global_config().await?;

    // Apply environment variable overrides
    loader.apply_environment_overrides(&mut config_manager.global)?;

    debug!("Basic configuration loaded");
    Ok(())
}

/// Detect if current directory or parent directories contain a workspace
async fn detect_workspace_in_current_dir() -> Result<Option<String>> {
    let current_dir = std::env::current_dir().map_err(|e| {
        PixlieError::session(
            format!("Failed to get current directory: {}", e),
            ErrorContext::new().with_context("Workspace detection"),
        )
    })?;

    // Look for .pixlie-workspace.toml in current directory and parents
    let mut dir = current_dir.as_path();

    loop {
        let workspace_config = dir.join(".pixlie-workspace.toml");
        if workspace_config.exists() {
            debug!("Found workspace configuration at: {:?}", dir);
            return Ok(Some(dir.to_string_lossy().to_string()));
        }

        // Move to parent directory
        match dir.parent() {
            Some(parent) => dir = parent,
            None => break, // Reached filesystem root
        }
    }

    debug!("No workspace configuration found in current directory or parents");
    Ok(None)
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Initialize basic logging system with defaults
    // The configuration system will be loaded inside run_application
    let logging_config = LoggingConfig {
        json_format: false,
        level: "info".to_string(),
        colored: true,
        file_path: None,
    };

    if let Err(e) = init_logging(logging_config) {
        eprintln!("Failed to initialize logging: {}", e);
        process::exit(1);
    }

    // Run the application and handle errors
    if let Err(e) = run_application(args).await {
        log_error(&e);

        // Also print user-friendly error to stderr
        eprintln!("Error: {}", e.user_message());

        // Exit with error code based on severity
        let exit_code = match e.severity() {
            ErrorSeverity::Low => 1,
            ErrorSeverity::Medium => 2,
            ErrorSeverity::High => 3,
            ErrorSeverity::Critical => 4,
        };

        process::exit(exit_code);
    }
}
