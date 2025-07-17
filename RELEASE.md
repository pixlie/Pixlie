# Release Management Guide

This document outlines the release process for Pixlie using GitHub Actions.

## Automated Release Workflow

The project uses GitHub Actions to automate cross-platform builds and release creation. The workflow is triggered by pushing version tags.

### Release Process

1. **Update Version**
   ```bash
   ./scripts/update-version.sh 1.2.3
   ```

2. **Review and Commit Changes**
   ```bash
   git add -A
   git commit -m "chore: bump version to v1.2.3"
   ```

3. **Create and Push Tag**
   ```bash
   git tag v1.2.3
   git push origin main --tags
   ```

4. **Automatic Release Creation**
   - GitHub Actions automatically builds for all platforms
   - Creates release with generated changelog
   - Uploads all binaries and installers
   - Generates checksums for verification

## Build Targets

### Supported Platforms
- **Linux**: x86_64, aarch64 (ARM64)
- **macOS**: x86_64 (Intel), aarch64 (Apple Silicon)
- **Windows**: x86_64

### Release Artifacts
- **Binaries**: Compressed archives (tar.gz, zip)
- **Linux Packages**: .deb, .rpm
- **Checksums**: SHA256 hashes for all files

## Manual Release (Emergency)

If automated release fails, you can create a manual release:

1. **Build Locally**
   ```bash
   ./scripts/build-release.sh x86_64-unknown-linux-gnu
   ```

2. **Generate Checksums**
   ```bash
   ./scripts/generate-checksums.sh dist/
   ```

3. **Create GitHub Release**
   ```bash
   gh release create v1.2.3 dist/* --title "Pixlie v1.2.3" --notes "Release notes here"
   ```

## Workflow Configuration

### Main Workflows
- **`release.yml`**: Triggered by version tags, creates full release
- **`build.yml`**: Triggered by PRs and pushes, builds and tests
- **`security.yml`**: Daily security audits and dependency checks

### Environment Variables
- `CARGO_TERM_COLOR=always`: Enable colored cargo output
- `GITHUB_TOKEN`: Automatically provided by GitHub Actions

## Troubleshooting

### Build Failures
1. Check logs in GitHub Actions tab
2. Verify all targets compile locally
3. Ensure dependencies are up to date

### Cross-compilation Issues
- Linux ARM64: Requires `gcc-aarch64-linux-gnu`
- Windows: Uses MSVC toolchain
- macOS: Native compilation on macOS runners

### Release Asset Problems
1. Verify file permissions on scripts
2. Check artifact naming conventions
3. Ensure checksums are generated correctly

## Version Management

### Semantic Versioning
- **MAJOR**: Breaking changes
- **MINOR**: New features, backward compatible
- **PATCH**: Bug fixes, backward compatible

### Pre-releases
- **Alpha**: `v1.2.3-alpha.1`
- **Beta**: `v1.2.3-beta.1`
- **Release Candidate**: `v1.2.3-rc.1`

### File Synchronization
The `update-version.sh` script automatically updates:
- `pixlie/Cargo.toml`
- `webapp/package.json`
- `website/package.json` (if exists)

## Security

### Code Signing
- **macOS**: Requires Apple Developer certificate
- **Windows**: Requires code signing certificate
- **Linux**: GPG signing recommended

### Dependency Auditing
- Automated daily security scans
- `cargo audit` for Rust dependencies
- `npm audit` for Node.js dependencies

## Package Managers

### Future Distribution Channels
- **Homebrew**: macOS package manager
- **Chocolatey**: Windows package manager
- **Snap**: Universal Linux packages
- **Cargo**: Rust package registry
- **AUR**: Arch Linux User Repository

## Monitoring

### Build Metrics
- Build success rates
- Build duration tracking
- Artifact size monitoring
- Download statistics

### Release Quality
- Automated testing on multiple platforms
- Integration test verification
- Smoke test execution
- User feedback collection