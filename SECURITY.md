# Security Policy

## Reporting Security Vulnerabilities

If you discover a security vulnerability in Pixlie, please report it privately to our security team at: **security@pixlie.ai**

Please do not report security vulnerabilities through public GitHub issues.

## Supported Versions

We actively maintain security updates for the following versions:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Known Security Advisories

### Current Status

Our automated security scanning has identified the following known issues in transitive dependencies:

#### RUSTSEC-2023-0071 - RSA Marvin Attack
- **Severity**: Medium (5.9 CVSS)
- **Affected Package**: `rsa` v0.9.8 (transitive dependency via sqlx-mysql)
- **Impact**: Potential key recovery through timing sidechannels
- **Status**: No fix available upstream
- **Mitigation**: This vulnerability affects RSA cryptographic operations. Pixlie uses SQLite exclusively and does not perform RSA operations, making this vulnerability non-exploitable in our use case.

#### RUSTSEC-2024-0436 - Unmaintained paste crate
- **Severity**: Warning (unmaintained)
- **Affected Package**: `paste` v1.0.15 (transitive dependency via tokenizers/gline-rs)
- **Impact**: No security impact, but crate is no longer maintained
- **Status**: Functional but unmaintained
- **Mitigation**: The crate remains functional. We are monitoring for maintained alternatives.

### Security Measures

1. **Automated Scanning**: GitHub Actions runs daily security audits
2. **Dependency Updates**: Regular dependency updates to address vulnerabilities
3. **Minimal Dependencies**: We use SQLite-only features to reduce attack surface
4. **Local-First Architecture**: No external network dependencies for core functionality

### Audit Configuration

The project includes a `.cargo/audit.toml` configuration that documents known issues and their mitigation status. These configurations are regularly reviewed and updated.

## Security Best Practices

When using Pixlie:

1. **Network Security**: Run Pixlie behind a reverse proxy or firewall in production
2. **Data Protection**: Use appropriate file system permissions for your data directory
3. **Updates**: Keep Pixlie updated to the latest version
4. **Access Control**: Limit network access to the Pixlie server to trusted clients only

## Security Development Lifecycle

- **Code Review**: All code changes require review before merging
- **Automated Testing**: Security-focused tests in CI/CD pipeline
- **Dependency Monitoring**: Automated monitoring of dependency vulnerabilities
- **Regular Audits**: Periodic security audits of the codebase

## Contact

For security-related questions or concerns, contact: **security@pixlie.ai**

For general questions about Pixlie, please use GitHub Issues or Discussions.