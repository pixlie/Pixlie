# Pixlie - LLM-Enabled TUI Data Analysis Tool

## Overview

A Rust-based Terminal User Interface (TUI) application that leverages Large Language Models (LLMs) to perform intelligent data analysis on SQLite databases. Similar to Claude Code, Pixlie provides an interactive chat interface where users can manage multiple analysis objectives simultaneously, maintain conversation history, and collaborate with AI to explore their data.

## Key Features

- **Interactive TUI**: Modern terminal interface with keyboard shortcuts and intuitive navigation
- **Multiple Concurrent Objectives**: Run and manage several data analysis tasks at the same time
- **Persistent Chat History**: Full conversation history with context preservation across sessions
- **Real-time Collaboration**: Chat-based interaction with AI for iterative data exploration
- **Tool-based Architecture**: Extensible system with SQL execution, schema inspection, and user interaction tools
- **Multi-LLM Support**: Works with OpenAI, Anthropic, and local models

## Architecture

### TUI Components

#### 1. Main Interface Layout
```
┌─────────────────────────────────────────────────────────────────┐
│ Pixlie Data Analyzer v0.2.0                    [Ctrl+Q to quit] │
├─────────────────────────────────────────────────────────────────┤
│ Active Objectives (3)        │ Chat History                     │
│ ┌─────────────────────────┐   │ ┌─────────────────────────────┐ │
│ │ 📊 Sales Analysis       │   │ │ User: What are top products?│ │
│ │ 🔍 Customer Insights    │   │ │ AI: Let me analyze...       │ │
│ │ 📈 Revenue Trends       │   │ │ User: Include Q4 data too   │ │
│ └─────────────────────────┘   │ │ AI: Sure, querying...       │ │
│                               │ └─────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│ > Tell me about customer retention rates by segment_            │
└─────────────────────────────────────────────────────────────────┘
```

#### 2. Navigation and Controls
- **Tab**: Switch between objectives, chat, and input areas
- **Ctrl+N**: Create new objective
- **Ctrl+D**: Delete current objective
- **Ctrl+H**: Toggle chat history visibility
- **Ctrl+S**: Save session and history
- **Ctrl+L**: Load previous session
- **Ctrl+Q**: Quit application

#### 3. Session Management
- **Workspace Persistence**: Automatically saves objectives and chat history
- **Context Switching**: Seamlessly switch between different analysis contexts
- **History Search**: Full-text search through conversation history
- **Export Options**: Save conversations and results to files

### Core Components

#### 1. TUI Framework
- **Library**: Built with `ratatui` for modern terminal interfaces
- **Event Handling**: Async input processing with crossterm
- **Layout System**: Responsive design that adapts to terminal size
- **State Management**: Centralized application state with message passing

#### 2. Session Management
- **Objective Tracking**: Multiple concurrent analysis objectives
- **Chat History**: Persistent conversation storage with SQLite
- **Context Preservation**: Maintain analysis state across restarts
- **Workspace Isolation**: Separate contexts for different projects

#### 3. Tool System (Unchanged)
- **Tool Trait**: Generic interface for all tools
- **Tool Registry**: Dynamic registration and discovery of tools
- **Tool Execution**: Sandboxed execution with result serialization

#### 4. Enhanced LLM Integration
- **Streaming Responses**: Real-time response rendering in chat
- **Context Awareness**: Multi-objective context management
- **Provider Abstraction**: Support for multiple LLM providers
- **Rate Limiting**: Intelligent request throttling and queuing

## Usage

### Starting Pixlie
```bash
# Launch TUI with database
./pixlie --database ./sales.db

# Launch with specific workspace
./pixlie --workspace ./analysis-workspace/

# Launch and load previous session
./pixlie --database ./sales.db --load-session
```

### TUI Interaction Flow

#### 1. Creating Objectives
1. Press `Ctrl+N` to create new objective
2. Enter objective description (e.g., "Analyze customer churn patterns")
3. Objective appears in the active objectives panel
4. Switch between objectives using `Tab` or arrow keys

#### 2. Chat-based Analysis
1. Type questions in the input area at the bottom
2. AI responds with analysis, SQL queries, and follow-up questions
3. Full conversation history is maintained and searchable
4. Context is preserved within each objective

#### 3. Managing Multiple Objectives
- Each objective maintains independent chat history
- Switch contexts without losing analysis progress
- Visual indicators show active/pending objectives
- Parallel analysis execution where possible

### Example Session
```
Objective 1: "Customer Segmentation Analysis"
├─ User: "What are our main customer segments?"
├─ AI: "Let me analyze your customer data..."
├─ Tool: schema_inspection → Found customers, orders, products tables
├─ Tool: sql_query → "SELECT customer_type, COUNT(*) FROM customers..."
├─ AI: "I found 4 main segments: Enterprise (23%), SMB (45%), Individual (32%)"
├─ User: "Which segment has highest lifetime value?"
└─ AI: "Analyzing LTV by segment..."

Objective 2: "Sales Performance Q4"
├─ User: "How did Q4 sales compare to Q3?"
├─ AI: "I'll compare quarterly performance..."
└─ Tool: sql_query → "SELECT quarter, SUM(revenue) FROM orders..."
```

## Project Structure

```
src/
├── main.rs                 # TUI entry point
├── lib.rs                  # Library exports
├── tui/
│   ├── mod.rs
│   ├── app.rs             # Main application state
│   ├── components/        # UI components
│   │   ├── chat.rs        # Chat history display
│   │   ├── objectives.rs  # Objectives panel
│   │   ├── input.rs       # Input handling
│   │   └── status.rs      # Status bar
│   ├── events.rs          # Event handling
│   └── layout.rs          # UI layout management
├── session/
│   ├── mod.rs
│   ├── manager.rs         # Session persistence
│   ├── workspace.rs       # Workspace management
│   └── history.rs         # Chat history storage
├── analysis/
│   ├── mod.rs
│   ├── objective.rs       # Objective management
│   ├── context.rs         # Analysis context
│   └── coordinator.rs     # Multi-objective coordination
├── tools/                 # Tool implementations (unchanged)
│   ├── mod.rs
│   ├── base.rs
│   ├── sqlite.rs
│   ├── user_interaction.rs
│   └── schema.rs
├── llm/                   # LLM providers (enhanced)
│   ├── mod.rs
│   ├── provider.rs
│   ├── streaming.rs       # Streaming response handling
│   └── context.rs         # Multi-objective context
└── types/
    ├── mod.rs
    └── generated/         # Auto-generated TypeScript types
```

## New Dependencies

### TUI and Interface
```toml
[dependencies]
# Existing dependencies...
ratatui = "0.24"
crossterm = "0.27"
tokio-util = "0.7"

# Session management
dirs = "5.0"
uuid = { version = "1.0", features = ["v4"] }

# Enhanced database
rusqlite = { version = "0.29", features = ["bundled", "chrono"] }
chrono = { version = "0.4", features = ["serde"] }
```

## Configuration

### Settings File (~/.pixlie/config.toml)
```toml
[ui]
theme = "dark"
show_line_numbers = true
chat_history_limit = 1000

[llm]
default_provider = "openai"
default_model = "gpt-4"
max_concurrent_requests = 3

[session]
auto_save_interval = 30  # seconds
max_objectives = 10
workspace_path = "~/.pixlie/workspaces"

[database]
default_timeout = 30
max_query_results = 1000
```

## Migration from CLI

The original CLI functionality is preserved in `data-analyzer` binary:
- Existing scripts and automation continue to work
- CLI mode available via `--cli` flag
- TUI is the new default interface
- All tool functionality remains identical

## Future Enhancements

### Phase 2: Advanced TUI Features
- **Visualization Panel**: Inline charts and graphs in terminal
- **Split Screen Mode**: Multiple objectives visible simultaneously  
- **Collaboration**: Share workspaces with team members
- **Plugin System**: Custom TUI components and tools

### Phase 3: Extended Capabilities
- **Export Dashboard**: Generate reports and presentations
- **Scheduled Analysis**: Automated recurring analysis
- **Integration Hub**: Connect to external data sources
- **AI Suggestions**: Proactive analysis recommendations

## Performance Considerations

- **Async Architecture**: Non-blocking UI with background processing
- **Streaming Updates**: Real-time chat and result updates
- **Memory Management**: Efficient chat history and session storage
- **Concurrent Analysis**: Parallel objective processing where possible
- **Caching Strategy**: Intelligent query and result caching

## Security

- **Session Encryption**: Encrypted workspace and history storage
- **API Key Management**: Secure credential storage per workspace
- **Audit Logging**: Track all database queries and AI interactions
- **Sandboxed Execution**: Isolated tool execution environment