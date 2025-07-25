# LLM-Enabled CLI Data Analysis Tool - Technical Design

## Overview

A Rust-based CLI application that leverages Large Language Models (LLMs) to perform intelligent data analysis on SQLite databases. The system uses a tool-based architecture where the LLM can interact with various tools (SQL execution, user interaction) to iteratively analyze data and answer complex questions.

## Architecture

### Core Components

#### 1. CLI Interface
- **Entry Point**: Main CLI application built with `clap` crate
- **Arguments**:
  - `--database <path>`: Path to SQLite database
  - `--objective <text>`: Analysis objective/question
  - `--model <name>`: LLM model selection (optional)
  - `--max-iterations <n>`: Maximum query iterations (default: 10)

#### 2. Tool System
- **Tool Trait**: Generic interface for all tools
- **Tool Registry**: Dynamic registration and discovery of tools
- **Tool Execution**: Sandboxed execution with result serialization

#### 3. LLM Integration
- **Provider Abstraction**: Support for multiple LLM providers (OpenAI, Anthropic, local models)
- **JSON Schema Generation**: Auto-generate tool schemas for LLM function calling
- **Response Parsing**: Structured JSON response handling

#### 4. Type System
- **TypeScript Generation**: Using `ts-rs` crate for tool interface definitions
- **Schema Management**: Automated type generation and validation
- **Tool Documentation**: Auto-generated documentation from Rust types

## Tool Architecture

### Base Tool Interface

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn parameters_schema(&self) -> serde_json::Value;
    async fn execute(&self, params: serde_json::Value) -> Result<ToolResult, ToolError>;
}
```

### Built-in Tools

#### 1. SQLite Tool
- **Purpose**: Execute SQL queries against the target database
- **Capabilities**:
  - Schema introspection
  - Query execution with results
  - Error handling and validation
  - Query performance metrics

```rust
#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SqlQueryParams {
    pub query: String,
    pub limit: Option<u32>,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SqlQueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<serde_json::Value>,
    pub row_count: usize,
    pub execution_time_ms: u64,
}
```

#### 2. User Interaction Tool
- **Purpose**: Ask clarifying questions or present findings to user
- **Capabilities**:
  - Display formatted results
  - Ask yes/no questions
  - Request additional context
  - Present multiple choice options

```rust
#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UserPromptParams {
    pub message: String,
    pub prompt_type: UserPromptType,
    pub options: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub enum UserPromptType {
    Information,
    YesNo,
    MultipleChoice,
    FreeText,
}
```

#### 3. Database Schema Tool
- **Purpose**: Inspect database structure and metadata
- **Capabilities**:
  - List tables and views
  - Describe table schemas
  - Show indexes and constraints
  - Sample data preview

```rust
#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SchemaInspectionResult {
    pub tables: Vec<TableInfo>,
    pub views: Vec<ViewInfo>,
    pub indexes: Vec<IndexInfo>,
}
```

## LLM Integration

### Request Format

```rust
#[derive(Serialize, Deserialize)]
pub struct LlmRequest {
    pub objective: String,
    pub context: AnalysisContext,
    pub available_tools: Vec<ToolDefinition>,
    pub previous_actions: Vec<ToolExecution>,
    pub max_iterations: u32,
}
```

### Response Format

```rust
#[derive(Serialize, Deserialize)]
pub struct LlmResponse {
    pub reasoning: String,
    pub next_action: NextAction,
    pub confidence: f32,
    pub requires_user_input: bool,
}

#[derive(Serialize, Deserialize)]
pub enum NextAction {
    ExecuteTool {
        tool_name: String,
        parameters: serde_json::Value,
    },
    RequestUserInput {
        message: String,
        input_type: UserInputType,
    },
    ProvideAnswer {
        answer: String,
        supporting_data: Option<serde_json::Value>,
    },
    RequestMoreIterations {
        reason: String,
    },
}
```

## Implementation Details

### Project Structure

```
src/
├── main.rs                 # CLI entry point
├── lib.rs                  # Library exports
├── config/
│   ├── mod.rs
│   └── settings.rs         # Configuration management
├── tools/
│   ├── mod.rs
│   ├── base.rs            # Tool trait and registry
│   ├── sqlite.rs          # SQLite tool implementation
│   ├── user_interaction.rs # User prompt tool
│   └── schema.rs          # Database schema tool
├── llm/
│   ├── mod.rs
│   ├── provider.rs        # LLM provider abstraction
│   ├── openai.rs          # OpenAI integration
│   ├── anthropic.rs       # Anthropic integration
│   └── local.rs           # Local model support
├── analysis/
│   ├── mod.rs
│   ├── engine.rs          # Analysis orchestration
│   ├── context.rs         # Analysis context management
│   └── results.rs         # Result formatting
└── types/
    ├── mod.rs
    └── generated/          # Auto-generated TypeScript types
        ├── tools/
        │   ├── sqlite.ts
        │   ├── user_interaction.ts
        │   └── schema.ts
        └── core.ts
```

### Type Generation System

#### Management Command

```bash
cargo run --bin typegen
```

This command will:
1. Scan all types marked with `#[derive(TS)]`
2. Generate TypeScript definitions
3. Organize by tool/module
4. Create index files for easy imports
5. Validate generated schemas

#### Generated Directory Structure

```
types/
├── index.ts               # Main exports
├── core/
│   ├── index.ts
│   ├── analysis.ts
│   └── llm.ts
└── tools/
    ├── index.ts
    ├── sqlite.ts
    ├── user_interaction.ts
    └── schema.ts
```

### Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("LLM provider error: {0}")]
    LlmProvider(String),
    
    #[error("Tool execution error: {tool}: {error}")]
    ToolExecution { tool: String, error: String },
    
    #[error("Maximum iterations exceeded: {max}")]
    MaxIterationsExceeded { max: u32 },
    
    #[error("User cancelled analysis")]
    UserCancelled,
}
```

## Usage Flow

### 1. Initialization
```bash
./data-analyzer --database ./hackernews.db --objective "What % of startup posts get replies from founders?"
```

### 2. Analysis Process
1. **Database Inspection**: Tool examines schema and sample data
2. **Initial Analysis**: LLM plans approach based on objective and schema
3. **Iterative Querying**: Execute SQL queries, analyze results, refine approach
4. **User Interaction**: Ask clarifying questions when needed
5. **Result Synthesis**: Combine findings into final answer

### 3. Example Interaction Flow

```
[1] Tool: schema_inspection
    Result: Found tables: posts, comments, users
    
[2] Tool: sql_query
    Query: "SELECT COUNT(*) FROM posts WHERE title LIKE '%startup%'"
    Result: 1,247 startup-related posts
    
[3] Tool: user_prompt
    Message: "I found posts mentioning 'startup'. Should I also include posts with 'founder', 'entrepreneur', etc.?"
    Response: "Yes, include those terms"
    
[4] Tool: sql_query
    Query: "SELECT p.id, p.title, p.author_id FROM posts p WHERE ..."
    Result: 2,156 relevant posts identified
    
[5] Tool: sql_query
    Query: "SELECT p.id, COUNT(c.id) as reply_count FROM posts p LEFT JOIN comments c..."
    Result: Reply statistics calculated
    
[6] Final Answer: "23.4% of startup-related posts receive replies from founders or team members"
```

## Dependencies

### Core Dependencies
```toml
[dependencies]
clap = { version = "4.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
rusqlite = { version = "0.29", features = ["bundled"] }
async-trait = "0.1"
thiserror = "1.0"
anyhow = "1.0"
ts-rs = "7.0"

# LLM Providers
reqwest = { version = "0.11", features = ["json"] }
openai-api-rs = "4.0"

# CLI and UX
colored = "2.0"
indicatif = "0.17"
dialoguer = "0.10"
```

### Development Dependencies
```toml
[dev-dependencies]
tempfile = "3.0"
tokio-test = "0.4"
```

## Future Enhancements

### Phase 2 Features
- **Multi-database Support**: PostgreSQL, MySQL, ClickHouse
- **Visualization Tools**: Generate charts and graphs
- **Export Tools**: CSV, JSON, PDF report generation
- **Caching System**: Query result caching for performance

### Phase 3 Features
- **Plugin System**: Custom tool development
- **Web Interface**: Browser-based analysis dashboard
- **Collaborative Features**: Share analyses and results
- **Advanced Analytics**: Statistical analysis tools

## Security Considerations

- **SQL Injection Prevention**: Parameterized queries and validation
- **Sandboxed Execution**: Isolated tool execution environment
- **API Key Management**: Secure credential storage
- **Database Permissions**: Read-only access by default
- **User Input Validation**: Sanitize all user inputs

## Performance Considerations

- **Connection Pooling**: Efficient database connection management
- **Query Optimization**: Automatic query analysis and suggestions
- **Concurrent Execution**: Parallel tool execution where possible
- **Memory Management**: Streaming for large result sets
- **Caching Strategy**: Intelligent caching of repeated queries