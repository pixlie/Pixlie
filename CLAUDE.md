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

- **Backend**: Rust application (`pixlie/`) - planned to use Actix Web, SQLx, GLiNER for entity recognition, and mistral.rs for LLM inference
- **Frontend**: React application (`webapp/`) - uses Vite, Tailwind CSS, shadcn/ui, and TypeScript

## Development Commands

### Rust Backend (`pixlie/`)
```bash
# Build and run the Rust application
cd pixlie
cargo build
cargo run

# Run tests
cargo test

# Check for linting issues
cargo clippy

# Format code
cargo fmt
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
pnpm serve
```

## Architecture Notes

### Current State
- **Rust backend**: Currently minimal with just a "Hello, world!" in main.rs
- **Frontend**: React setup with Vite, Tailwind CSS, and shadcn/ui configured

### Planned Architecture
- Backend will handle data ingestion from Hacker News Firebase API
- Entity extraction using GLiNER for identifying startups, founders, products, investors
- Sentiment analysis and content categorization
- Local LLM integration via mistral.rs
- Database layer with SQLx (SQLite/MySQL/PostgreSQL)

### Key Technologies
- **Rust**: Edition 2024, no dependencies yet configured
- **React**: v19.1.0 with TypeScript support
- **Tailwind CSS**: v4.1.11 with Vite plugin
- **shadcn/ui**: Component library with CVA, clsx, tailwind-merge, and lucide-react
- **Vite**: v7.0.4 for frontend tooling

### Development Workflow
- Rust backend in `pixlie/` directory with standard Cargo project structure
- Frontend in `webapp/` directory with Vite-based React setup
- Frontend runs on port 3000 by default
- No testing framework currently configured in either project
