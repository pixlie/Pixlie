# Pixlie - LLM-Enabled CLI Data Analysis Tool

## Overview
Rust CLI app using LLMs to analyze SQLite databases through tool-based architecture.

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
./data-analyzer --database <path> --objective <question> [--model <name>] [--max-iterations <n>]
```

## Architecture
- **CLI Interface**: clap-based with database path and analysis objective
- **Tool System**: Generic trait with registry for SQL execution, user interaction, schema inspection
- **LLM Integration**: Multi-provider support (OpenAI, Anthropic, local models) with JSON schema generation
- **Type System**: Auto-generated TypeScript definitions via ts-rs

## Core Tools
1. **SQLite Tool**: Execute queries, schema introspection, performance metrics
2. **User Interaction Tool**: Ask questions, present results, multiple choice prompts
3. **Database Schema Tool**: List tables, describe schemas, sample data

## Analysis Flow
1. Database schema inspection
2. LLM plans approach based on objective
3. Iterative SQL querying and analysis
4. User interaction for clarification
5. Result synthesis and final answer

## Project Structure
```
src/
├── main.rs
├── tools/          # Tool implementations
├── llm/            # LLM provider abstractions
├── analysis/       # Analysis orchestration
└── types/          # Generated TypeScript types
```

## Dependencies
- Core: clap, tokio, serde, rusqlite, async-trait, ts-rs
- LLM: reqwest, openai-api-rs
- UX: colored, indicatif, dialoguer

## Commands
- `cargo run --bin typegen` - Generate TypeScript definitions
- Default read-only database access with SQL injection prevention
