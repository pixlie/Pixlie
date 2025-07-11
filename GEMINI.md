# Gemini Project Helper

This document outlines the structure, conventions, and commands for the Pixlie project to ensure effective collaboration with the Gemini assistant.

## Development Workflow
- Create a new branch for each task
- Branch names should start with chore/ or feature/ or fix/
- Please add tests for any new features added
- Please run formatters, linters and tests before committing changes
- When finished please commit and push to the new branch
- Please mention GitHub issue if provided

## Project Overview

Pixlie is a data analysis platform that extracts and analyzes entities (startups, founders, products, etc.) from Hacker News discussions. It features a Rust backend for data processing and a React frontend for data visualization.

## Architecture

The project is a monorepo with two main components:

1.  `pixlie/`: A Rust-based backend.
2.  `webapp/`: A React-based frontend.

---

## Backend (`pixlie/`)

The backend is responsible for fetching data from the Hacker News API, performing NLP/ML analysis, storing it in a database, and serving a web API.

-   **Language**: Rust
-   **Framework**: Actix Web
-   **Database**: SQLx (supports SQLite, MySQL, PostgreSQL)
-   **CLI**: Clap for command-line operations.
-   **Key Crates**:
    -   `actix-web`: For the web server.
    -   `sqlx`: For database interaction.
    -   `reqwest`: For making HTTP requests to the HN API.
    -   `serde`: For serialization/deserialization.
    -   `gline-rs`: For Named Entity Recognition.
    -   `clap`: For the CLI.

### Common Commands

-   **Run the development server**: `cargo run`
-   **Run tests**: `cargo test`
-   **Build for production**: `cargo build --release`
-   **Check code**: `cargo check`

---

## Frontend (`webapp/`)

The frontend is a single-page application (SPA) for visualizing the data analyzed by the backend.

-   **Framework**: React
-   **Language**: TypeScript
-   **Build Tool**: Vite
-   **Styling**: Tailwind CSS
-   **UI Components**: shadcn/ui (inferred from dependencies like `tailwind-merge`, `clsx`, `lucide-react`).
-   **Routing**: React Router (`react-router-dom`).

### Common Commands

The `pnpm-lock.yaml` file indicates `pnpm` is the preferred package manager.

-   **Install dependencies**: `pnpm install`
-   **Run the development server**: `pnpm dev`
-   **Build for production**: `pnpm build`
-   **Lint files**: `pnpm lint`

## General Instructions

-   When working on the backend, navigate to the `pixlie/` directory.
-   When working on the frontend, navigate to the `webapp/` directory.
-   Configuration is managed via a `.env` file in the root directory.
-   Adhere to the existing coding style and conventions in each part of the codebase.
