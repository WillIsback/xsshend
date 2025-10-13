# GitHub Actions Workflows

This directory contains automated workflows for maintaining code quality and deployment.

## Workflows Overview

### 🔧 `test.yml` - Comprehensive Testing
- **Triggered on**: Push to main, Pull Requests, Manual dispatch
- **Purpose**: Complete test suite with multiple Rust versions and platforms
- **Features**:
  - Format checking with `cargo fmt`
  - Strict linting with `cargo clippy`
  - Full test suite execution
  - Security audit with `cargo audit`
  - Cross-platform build testing (Linux, Windows, macOS)
  - Documentation generation

### ⚡ `pre-commit.yml` - Quality Gate
- **Triggered on**: Every push (except gh-pages)
- **Purpose**: Enforce code quality standards before commit acceptance
- **Features**:
  - Automatic code formatting
  - Strict Clippy linting (warnings as errors)
  - Complete test suite execution
  - Release build verification
  - Documentation check
  - Auto-commit formatting fixes (main branch only)
  - Performance regression detection

### 📖 `deploy-docs.yml` - Documentation Deployment
- **Triggered on**: Changes to docs/ directory
- **Purpose**: Deploy documentation to GitHub Pages
- **Features**:
  - Jekyll-based GitHub Pages deployment
  - Automatic documentation updates

## Workflow Behavior

### Quality Gate Process (pre-commit.yml)
1. **Format**: Auto-format code with `cargo fmt`
2. **Lint**: Check with `cargo clippy --deny warnings`
3. **Test**: Run complete test suite (93 tests)
4. **Build**: Verify release build works
5. **Docs**: Check documentation builds without warnings
6. **Auto-fix**: Commit formatting changes if needed (main branch only)

### Commit Rejection Conditions
Commits will be **rejected** if:
- ❌ Code fails Clippy linting (warnings treated as errors)
- ❌ Any test fails in the test suite
- ❌ Release build fails
- ❌ Documentation generation fails

### Auto-formatting Behavior
- ✅ Code is automatically formatted on main branch pushes
- ✅ Formatting changes are committed with `[skip ci]` to prevent loops
- ✅ Pull requests show formatting check results without auto-commit

## Local Development

To ensure your commits pass the quality gate, run locally:

```bash
# Check formatting
cargo fmt --all -- --check

# Or auto-format
cargo fmt --all

# Check linting (strict)
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test --verbose --all-features

# Check documentation
cargo doc --no-deps --all-features --document-private-items

# Run all checks at once
cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings && cargo test --verbose
```

## Badge Integration

Add these badges to your README.md:

```markdown
[![Tests](https://github.com/willisback/xsshend/actions/workflows/test.yml/badge.svg)](https://github.com/willisback/xsshend/actions/workflows/test.yml)
[![Quality Gate](https://github.com/willisback/xsshend/actions/workflows/pre-commit.yml/badge.svg)](https://github.com/willisback/xsshend/actions/workflows/pre-commit.yml)
```

## Security Features

- **Dependency Audit**: `cargo audit` checks for known vulnerabilities
- **Minimal Permissions**: Workflows use least-privilege access
- **Secure Token Usage**: Uses `secrets.GITHUB_TOKEN` appropriately
- **Auto-commit Safety**: Only formats code, never changes logic

## Performance Monitoring

- **Benchmark Execution**: Tracks performance regression
- **Test Suite Timing**: Monitors test execution time
- **Build Performance**: Tracks compilation times across platforms