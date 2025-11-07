# Contributing to opentelemetry-langfuse

Thank you for your interest in contributing! This document provides guidelines for contributing to this project.

## Getting Started

### Prerequisites

- Rust 1.83 or later (MSRV)
- Cargo
- A Langfuse account for integration testing (optional but recommended)

### Setup

1. Fork and clone the repository
2. Install dependencies:
   ```bash
   cargo build
   ```
3. Set up environment variables for integration tests (optional):
   ```bash
   export LANGFUSE_PUBLIC_KEY="pk-lf-..."
   export LANGFUSE_SECRET_KEY="sk-lf-..."
   export LANGFUSE_HOST="https://cloud.langfuse.com"
   ```

## Development Workflow

### Running Tests

```bash
# Run unit tests
cargo test --lib

# Run all tests (including integration tests - requires Langfuse credentials)
cargo test

# Run specific test
cargo test test_name
```

### Code Quality

Before submitting a PR, ensure your code passes all checks:

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Build documentation
cargo doc --no-deps --all-features
```

### Running Examples

```bash
# Set environment variables
export LANGFUSE_PUBLIC_KEY="pk-lf-..."
export LANGFUSE_SECRET_KEY="sk-lf-..."
export LANGFUSE_HOST="https://cloud.langfuse.com"

# Run an example
cargo run --example async_batch
```

## Pull Request Process

1. **Create a feature branch** from `main`
2. **Make your changes** following the project's coding standards
3. **Add tests** for new functionality
4. **Update documentation** (README.md, rustdoc comments) as needed
5. **Run all checks** (format, clippy, tests, doc build)
6. **Create a pull request** with a clear description of your changes
7. **Wait for CI** to pass and address any review feedback

### PR Guidelines

- Write clear, descriptive commit messages
- Keep PRs focused on a single feature or fix
- Add tests for new functionality
- Update CHANGELOG.md is managed by release-plz (no manual updates needed)
- Ensure all CI checks pass
- Follow the existing code style

## Code Standards

### Rust Style

- Follow the official [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting (configured in `rustfmt.toml`)
- Address all `cargo clippy` warnings (configured in `.clippy.toml`)
- Write rustdoc comments for public APIs

### Testing

- Use `#[serial]` attribute from `serial_test` for tests that modify environment variables
- Clean up environment variables in tests to ensure isolation
- Integration tests should be self-contained and verifiable
- Add unit tests for all new functionality

### Documentation

- All public items must have rustdoc comments
- Include examples in rustdoc where appropriate
- Keep README.md up to date with new features
- Document breaking changes clearly

## Commit Messages

Follow conventional commit format:

- `feat:` for new features
- `fix:` for bug fixes
- `docs:` for documentation changes
- `test:` for test additions or changes
- `refactor:` for code refactoring
- `chore:` for maintenance tasks
- `ci:` for CI/CD changes

Example:
```
feat: add support for custom HTTP headers

Add with_header() and with_headers() methods to ExporterBuilder
to allow users to configure custom HTTP headers for the OTLP exporter.
```

## Reporting Issues

When reporting issues, please include:

- Rust version (`rustc --version`)
- Operating system
- Minimal reproducible example
- Expected vs actual behavior
- Relevant logs or error messages

## Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help create a welcoming environment for all contributors

## Questions?

Feel free to:
- Open an issue for questions
- Check existing issues and PRs
- Review the [README.md](README.md) for usage examples

## License

By contributing, you agree that your contributions will be licensed under the same license as the project (MIT OR Apache-2.0).
