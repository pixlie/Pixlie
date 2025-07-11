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
