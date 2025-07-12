# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Workflow
- Create a new branch for each task
- Branch names should start with chore/ or feature/ or fix/
- Please add tests for any new features added
- Please run formatters, linters and tests before committing changes
- When finished please commit and push to the new branch
- Please mention GitHub issue if provided

## Project Overview

Pixlie is a dual-language application for smart entity analysis of Hacker News discussions. The project consists of:

- **Backend**: Rust application (`pixlie/`) - uses Actix Web, SQLx, gline-rs for entity recognition and relation extraction
- **Frontend**: React application (`webapp/`) - uses Vite, Tailwind CSS, shadcn/ui, and TypeScript

## Development Commands

### Rust Backend (`pixlie/`)
```bash
# Build and run the Rust application
cd pixlie
cargo build
cargo run

# Start server on specific port
cargo run -- --port 8080

# Run tests
cargo test

# Generate TypeScript types for frontend
cargo run --bin export_types

# Check for linting issues
cargo clippy --all-targets --all-features -- -D warnings

# Format code
cargo fmt --all -- --check
```

### Frontend (`webapp/`)
```bash
# Install dependencies
cd webapp
pnpm install

# Start development server (runs on port 3000)
pnpm dev

# Build for production
pnpm build

# Preview production build
pnpm preview

# Run linting
pnpm lint
```

## Architecture Notes

### Current Implementation Status
- **✅ Rust backend**: Full Actix Web API with entity extraction and relation detection
- **✅ Database layer**: SQLite with entity storage, relations, and reference tracking
- **✅ HN API integration**: Firebase API client for downloading stories and items
- **✅ Entity extraction**: Mock entity recognition for persons, companies, technologies
- **✅ Relation extraction**: 12 relation types (founded, acquired, works_at, etc.)
- **✅ Frontend**: React setup with modern UI components and routing
- **⏳ Entity visualization**: Planned interactive browser and graph visualization (Issues #22-24)

### Implemented Features
- **Data ingestion**: Download HN stories, recent items, and user data
- **Entity extraction**: Extract and store unique entities with references to source items
- **Relation detection**: Extract relationships between entities with confidence scoring
- **REST API**: Complete CRUD operations for items, entities, and relations
- **TypeScript integration**: Auto-generated types from Rust structs
- **Testing**: Unit tests for entity extraction and database operations

### API Endpoints
```
GET    /api/config              - Get application configuration
POST   /api/data-folder         - Set data storage folder
POST   /api/download/start      - Start HN data download
POST   /api/download/stop       - Stop data download
GET    /api/download/status     - Get download progress
GET    /api/models              - List available ML models
POST   /api/models/download     - Download ML model
POST   /api/extraction/start    - Start entity extraction
POST   /api/extraction/stop     - Stop entity extraction
GET    /api/extraction/status   - Get extraction progress
GET    /api/items               - Get paginated HN items
GET    /api/entities            - Get paginated entities
GET    /api/relations           - Get paginated entity relations
```

### Database Schema
- **hn_items**: Hacker News stories, comments, and metadata
- **hn_users**: User profiles and activity data
- **entities**: Unique entities (persons, companies, technologies)
- **entity_references**: Links entities to specific text locations in items
- **entity_relations**: Relationships between entities (founded, acquired, etc.)
- **entity_relation_references**: Links relations to source text
- **download_log**: Track data ingestion sessions
- **extraction_log**: Track entity extraction sessions

### Key Technologies
- **Rust**: Edition 2024 with Actix Web, SQLx, gline-rs, reqwest
- **React**: v19.1.0 with TypeScript support and React Router v7
- **Tailwind CSS**: v4.1.11 with Vite plugin for styling
- **shadcn/ui**: Component library with CVA, clsx, tailwind-merge, and lucide-react
- **Vite**: v7.0.4 for frontend tooling and development
- **SQLite**: Database with foreign keys and indexing for performance
- **ts-rs**: Auto-generation of TypeScript types from Rust structs

### Development Environment
- **Backend**: `pixlie/` directory with Cargo workspace
- **Frontend**: `webapp/` directory with Vite + React setup
- **API server**: Runs on localhost:8080 by default
- **Frontend dev server**: Runs on localhost:3000 by default
- **Testing**: Cargo test with tokio async runtime and tempfile for isolation
- **Type generation**: Automatic TypeScript type export to `webapp/src/types/`

### Next Development Priorities
1. **Entity Browser** (Issue #22): Backend APIs for entity search and detailed views
2. **Interactive Visualization** (Issue #23): React frontend with graph visualization
3. **Analytics Dashboard** (Issue #24): Insights and trending analysis

### Testing Strategy
- **Unit tests**: Entity extraction logic and database operations
- **Integration tests**: API endpoints and data flow
- **Mock data**: Predefined entity patterns for consistent testing
- **Async testing**: Full tokio runtime with proper cleanup
