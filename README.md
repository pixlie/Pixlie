# Pixlie

**Smart Entity Analysis for Hacker News Discussions**

Pixlie is an intelligent data analysis platform that extracts, analyzes, and provides insights from Hacker News discussions about startups, founders, products, and investors. Using advanced NLP and machine learning models, Pixlie helps you understand what the tech community is saying about key entities in the startup ecosystem.

## 🚀 Features

- **Real-time Data Ingestion**: Continuously fetches data from the Hacker News Firebase API
- **Entity Extraction**: Identifies startups, founders, products, investors, and other key entities using GLiNER
- **Sentiment Analysis**: Analyzes community sentiment towards different entities
- **Content Categorization**: Classifies discussions as suggestions, recommendations, complaints, or general mentions
- **Advanced Search**: Query entities with complex filters like "find startups invested by Techstars in the last 12 months"
- **Historical Tracking**: Monitor entity sentiment and discussion trends over time
- **SOTA LLM Integration**: Leverages state-of-the-art language models for nuanced analysis

## 🏗️ Architecture

```
pixlie/
├── Cargo.toml
├── src/
│   ├── main.rs
│
webapp/
├── package.json
├── src/
│   ├── App.tsx
│
```

## 🛠️ Tech Stack

**Backend (Rust)**
- **Framework**: Actix Web for high-performance async web server
- **Database**: SQLite, MySQL or PostgreSQL with SQLx for type-safe queries
- **ML Models**: gline-rs for named entity recognition
- **LLM Integration**: mistral.rs for local LLMs (LLama, Gemma, etc.)

**Frontend (React)**
- **Framework**: React for reactive UI
- **Styling**: Tailwind CSS for utility-first styling
- **Component Library**: shadcn/ui for pre-built components

**Data Sources**
- **Hacker News API**: Firebase-based REST API

**Development & CI**
- **CI/CD**: GitHub Actions for automated testing and building
- **Code Quality**: Rust clippy, cargo fmt, ESLint, TypeScript compiler
- **Pre-commit Hooks**: Automated code quality checks before commits
- **Package Management**: Cargo for Rust, pnpm for Node.js

## 🚀 Quick Start

### Prerequisites
- Rust 1.70+ with cargo
- Node.js 18+ with pnpm
- Git

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

3. **Start the backend**
   ```bash
   cd pixlie
   cargo run
   ```

4. **Start the frontend** (in a new terminal)
   ```bash
   cd webapp
   pnpm dev
   ```

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
```

**Webapp Commands:**
```bash
make check-webapp  # TypeScript check and lint
make build-webapp  # Build the frontend
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
