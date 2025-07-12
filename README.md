# Pixlie

**Smart Entity Analysis for Hacker News Discussions**

Pixlie is an intelligent data analysis platform that extracts, analyzes, and provides insights from Hacker News discussions about startups, founders, products, and investors. Using advanced NLP and machine learning models, Pixlie helps you understand what the tech community is saying about key entities in the startup ecosystem.

## 🚀 Features

### ✅ **Currently Implemented**
- **🔄 Real-time Data Ingestion**: Fetches data from the Hacker News Firebase API (stories, comments, users)
- **🏷️ Entity Extraction**: Identifies startups, founders, products, investors, and technologies
- **🔗 Relation Detection**: Extracts relationships between entities (founded, acquired, works_at, etc.)
- **💾 Smart Storage**: Deduplicated entity storage with reference tracking to source text
- **🌐 REST API**: Complete API for accessing items, entities, and relationships
- **📊 Data Management**: Download progress tracking and extraction session management
- **⚡ TypeScript Integration**: Auto-generated types for seamless frontend development

### 🚧 **In Development** 
- **🔍 Entity Browser**: Interactive search and filtering (Issue #22)
- **📈 Relationship Visualization**: Interactive graph of entity connections (Issue #23)  
- **📊 Analytics Dashboard**: Trending entities and insights (Issue #24)

### 🎯 **Planned Features**
- **📈 Sentiment Analysis**: Community sentiment towards different entities
- **🏷️ Content Categorization**: Discussion classification (suggestions, complaints, etc.)
- **🔍 Advanced Search**: Complex entity queries with temporal filters
- **📈 Historical Tracking**: Entity sentiment and discussion trends over time
- **🤖 Enhanced NLP**: Integration with production GLiNER models

## 🏗️ Architecture

```
pixlie/                         # Rust Backend
├── Cargo.toml                  # Dependencies: actix-web, sqlx, gline-rs
├── src/
│   ├── main.rs                 # Server entry point
│   ├── handlers.rs             # API route handlers  
│   ├── database.rs             # SQLite schema and queries
│   ├── entity_extraction.rs    # Entity and relation detection
│   ├── hn_api.rs              # Hacker News API client
│   ├── config.rs              # Configuration management
│   └── bin/
│       └── export_types.rs     # TypeScript type generation
│
webapp/                         # React Frontend  
├── package.json                # Dependencies: React 19, Vite, Tailwind
├── src/
│   ├── App.tsx                # Main application
│   ├── components/            # Reusable UI components
│   │   ├── ui/                # shadcn/ui base components
│   │   └── Settings.tsx       # App settings interface
│   ├── types/                 # Auto-generated TypeScript types
│   └── hooks/                 # Custom React hooks
│
data/                          # Application data (created at runtime)
├── hackernews.db              # SQLite database
└── models/                    # Downloaded ML models
```

## 🛠️ Tech Stack

**Backend (Rust)**
- **Framework**: Actix Web 4.5 for high-performance async web server
- **Database**: SQLite with SQLx 0.8 for type-safe queries and migrations
- **Entity Recognition**: gline-rs 1.0 for named entity recognition
- **HTTP Client**: reqwest 0.12 for Hacker News API integration
- **Type Safety**: ts-rs 8.0 for automatic TypeScript type generation

**Frontend (React)**
- **Framework**: React 19.1 with TypeScript 5.8
- **Build Tool**: Vite 7.0 for fast development and building
- **Styling**: Tailwind CSS 4.1 for utility-first styling
- **Components**: shadcn/ui with Radix primitives and CVA
- **Routing**: React Router 7.6 for client-side navigation
- **Icons**: Lucide React for consistent iconography

**Database Schema**
- **HN Data**: Items, users, and metadata from Hacker News
- **Entities**: Deduplicated entity storage with type classification
- **Relations**: Entity relationships with confidence scoring
- **References**: Links between entities/relations and source text
- **Sessions**: Download and extraction progress tracking

**Data Sources**
- **Hacker News API**: Firebase-based REST API for stories and comments

**Development & CI**
- **CI/CD**: GitHub Actions for automated testing and building
- **Code Quality**: Rust clippy, cargo fmt, ESLint, TypeScript compiler
- **Pre-commit Hooks**: Automated code quality checks before commits
- **Package Management**: Cargo for Rust, pnpm 9.12 for Node.js
- **Testing**: Cargo test with async support and database isolation

## 🚀 Quick Start

### Prerequisites
- **Rust** 1.70+ with cargo
- **Node.js** 18+ with pnpm
- **Git** for version control

### Setup Development Environment

1. **Clone the repository**
   ```bash
   git clone https://github.com/pixlie/Pixlie.git
   cd Pixlie
   ```

2. **Setup development environment**
   ```bash
   make setup
   ```
   This will:
   - Install git pre-commit hooks
   - Install webapp dependencies

3. **Start the backend** (API server on port 8080)
   ```bash
   cd pixlie
   cargo run
   ```

4. **Start the frontend** (in a new terminal - dev server on port 3000)
   ```bash
   cd webapp
   pnpm dev
   ```

### Getting Started with Data

1. **Configure data folder** (via Settings UI or API)
   ```bash
   curl -X POST http://localhost:8080/api/data-folder \
     -H "Content-Type: application/json" \
     -d '{"folder_path": "./data"}'
   ```

2. **Download Hacker News data**
   ```bash
   curl -X POST http://localhost:8080/api/download/start \
     -H "Content-Type: application/json" \
     -d '{"download_type": "stories", "limit": 100}'
   ```

3. **Extract entities and relations**
   ```bash
   curl -X POST http://localhost:8080/api/extraction/start \
     -H "Content-Type: application/json" \
     -d '{"batch_size": 50}'
   ```

4. **Explore the data**
   - View entities: `http://localhost:8080/api/entities`
   - View relations: `http://localhost:8080/api/relations`
   - Browse items: `http://localhost:8080/api/items`

### Development Commands

We provide a convenient Makefile for common development tasks:

```bash
# Setup and install hooks
make setup

# Run all checks (format, lint, test)
make check

# Build all projects
make build

# Run tests
make test

# Clean build artifacts
make clean
```

**Rust Backend Commands:**
```bash
make check-rust    # Format, clippy, and test
make build-rust    # Build the backend
make test-rust     # Run Rust tests

# Or use cargo directly:
cargo run --bin export_types  # Generate TypeScript types
cargo test                    # Run all tests
cargo clippy                  # Lint check
```

**Webapp Commands:**
```bash
make check-webapp  # TypeScript check and lint
make build-webapp  # Build the frontend

# Or use pnpm directly:
pnpm lint         # ESLint check
pnpm build        # Production build
```

### Git Hooks and CI

This project uses automated code quality checks:

- **Pre-commit hooks**: Run automatically before each commit
- **GitHub Actions**: Run on every push and pull request
- **Code formatting**: Rust `cargo fmt` and ESLint for TypeScript
- **Linting**: Rust `clippy` and ESLint
- **Testing**: Automated test execution

To bypass pre-commit hooks in emergencies:
```bash
git commit --no-verify
```

## 📊 Current Status & Roadmap

### ✅ **Phase 1: Core Infrastructure (Completed)**
- [x] Rust backend with Actix Web and SQLite
- [x] Hacker News API integration and data ingestion
- [x] Entity extraction system with 8+ entity types
- [x] Relation extraction with 12 relationship types
- [x] REST API with comprehensive endpoints
- [x] TypeScript type generation and integration
- [x] React frontend foundation with shadcn/ui

### 🚧 **Phase 2: Interactive Visualization (In Progress)**
- [ ] **Issue #22**: Entity browser with search and filtering APIs
- [ ] **Issue #23**: Interactive relationship graph visualization
- [ ] **Issue #24**: Analytics dashboard with trends and insights

### 🎯 **Phase 3: Advanced Features (Planned)**
- [ ] Production GLiNER model integration
- [ ] Real-time sentiment analysis
- [ ] Advanced search with temporal queries
- [ ] Entity influence and trending algorithms
- [ ] Export and API integrations
- [ ] Performance optimizations for large datasets

### 📈 **Metrics**
- **Database Schema**: 8 tables with proper indexing and foreign keys
- **API Endpoints**: 11 REST endpoints for complete data access
- **Entity Types**: 8 types (person, company, technology, etc.)
- **Relation Types**: 12 types (founded, acquired, works_at, etc.)
- **Test Coverage**: Unit and integration tests for core functionality
- **Type Safety**: 100% TypeScript coverage with auto-generated types

## 📄 License

This project is licensed under the GPL-3.0 License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Hacker News](https://news.ycombinator.com/) for providing the data
- [gline-rs](https://github.com/fbilhaut/gline-rs) for entity extraction, in Rust
- [GLiNER](https://github.com/urchade/GLiNER) for entity extraction (original in Python)
- [mistral.rs](https://github.com/EricLBuehler/mistral.rs) for LLM inference, in Rust
- [React](https://react.dev/) and [Rust](https://www.rust-lang.org/) communities

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/pixlie/Pixlie/issues)
- **Discussions**: [GitHub Discussions](https://github.com/pixlie/Pixlie/discussions)
- **Email**: support@pixlie.com

---

**Built with ❤️ by the Pixlie team**
