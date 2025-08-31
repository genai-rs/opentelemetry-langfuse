# Contributing to opentelemetry-langfuse

Thank you for your interest in contributing to opentelemetry-langfuse! This guide will help you get started.

## Quick Start

1. Fork the repository
2. Clone your fork
3. Create a feature branch
4. Make your changes
5. Run tests
6. Submit a pull request

## Development Setup

### Prerequisites

- Rust 1.82 or later
- Git

### Building the Project

```bash
git clone https://github.com/YOUR_USERNAME/opentelemetry-langfuse.git
cd opentelemetry-langfuse
cargo build
```

### Running Tests

```bash
cargo test
```

### Code Formatting

```bash
cargo fmt --all -- --check
```

### Linting

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

## Pull Request Process

1. Ensure all tests pass
2. Update documentation if needed
3. Follow the existing code style
4. Write clear commit messages
5. Keep PRs focused on a single change

## Code of Conduct

Please be respectful and inclusive in all interactions.

## Questions?

Feel free to open an issue if you have questions or need help getting started.