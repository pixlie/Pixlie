# Pixlie - LLM-Enabled TUI Data Analysis Tool

## Overview
Rust TUI app using LLMs to analyze SQLite databases through tool-based architecture. Interactive terminal interface supporting multiple concurrent objectives and persistent chat history, similar to Claude Code.

## Development Workflow
- Create a new branch for each task
- Branch names should start with chore/ or feature/ or fix/
- Please add tests for any new features added, particularly integration tests
- Please run formatters, linters and tests before committing changes
- When finished please commit and push to the new branch
- Please mention GitHub issue if provided
- After working on an issue from GitHub, update issue's tasks and open PR

## Usage
```bash
# Launch TUI interface (default)
./pixlie --database <path>

# Create/load workspace
./pixlie --workspace <workspace-path>

# Legacy CLI mode (preserved for automation)
./data-analyzer --database <path> --objective <question> [--model <name>] [--max-iterations <n>]
```

## Architecture
- **TUI Interface**: ratatui-based with chat interface, multiple objectives panel, and session management
- **Session Management**: Persistent workspaces with chat history and objective tracking
- **Tool System**: Generic trait with registry for SQL execution, user interaction, schema inspection
- **LLM Integration**: Multi-provider support with streaming responses and context awareness
- **Type System**: Auto-generated TypeScript definitions via ts-rs

## Core Features
1. **Multiple Concurrent Objectives**: Run several analysis tasks simultaneously with independent chat contexts
2. **Persistent Chat History**: Full conversation history with search and export capabilities
3. **Interactive Sessions**: Real-time collaboration with AI through chat interface
4. **Workspace Management**: Save and restore analysis sessions across restarts
5. **Legacy CLI Support**: Maintain backward compatibility for existing automation

## Core Tools
1. **SQLite Tool**: Execute queries, schema introspection, performance metrics
2. **User Interaction Tool**: Ask questions, present results, multiple choice prompts
3. **Database Schema Tool**: List tables, describe schemas, sample data

## TUI Interaction Flow
1. Launch TUI and select/create workspace
2. Create analysis objectives (Ctrl+N)
3. Chat with AI for each objective independently
4. Switch between objectives to manage multiple analysis threads
5. Session auto-saves progress and chat history
6. Export results and conversations as needed

## Project Structure
```
src/
├── main.rs                 # TUI entry point
├── tui/                    # TUI components and event handling
│   ├── app.rs             # Main application state
│   ├── components/        # UI components (chat, objectives, input)
│   ├── events.rs          # Event handling
│   └── layout.rs          # UI layout management
├── session/               # Session and workspace management
│   ├── manager.rs         # Session persistence
│   ├── workspace.rs       # Workspace management
│   └── history.rs         # Chat history storage
├── analysis/              # Multi-objective analysis coordination
│   ├── objective.rs       # Objective management
│   ├── context.rs         # Analysis context
│   └── coordinator.rs     # Multi-objective coordination
├── tools/                 # Tool implementations
├── llm/                   # Enhanced LLM providers with streaming
└── types/                 # Generated TypeScript types
```

## Dependencies
- Core: clap, tokio, serde, rusqlite, async-trait, ts-rs
- TUI: ratatui, crossterm, tokio-util
- Session: dirs, uuid, chrono
- LLM: reqwest, openai-api-rs
- UX: colored, indicatif, dialoguer (CLI mode)

## Commands
- `cargo run` - Launch TUI interface
- `cargo run --bin data-analyzer` - Launch legacy CLI mode
- `cargo run --bin typegen` - Generate TypeScript definitions
- Default read-only database access with SQL injection prevention

## TUI Controls
- **Tab**: Switch between objectives, chat, and input areas
- **Ctrl+N**: Create new objective
- **Ctrl+D**: Delete current objective  
- **Ctrl+H**: Toggle chat history visibility
- **Ctrl+S**: Save session and history
- **Ctrl+L**: Load previous session
- **Ctrl+Q**: Quit application
