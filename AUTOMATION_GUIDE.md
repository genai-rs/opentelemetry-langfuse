# Automation Development Instructions

## Important: Development Workflow

**NEVER push directly to main branch!** Always follow this workflow:

1. Create a feature branch
2. Make changes on the feature branch
3. Create a pull request
4. Wait for CI checks and review
5. Only merge after approval

## Git Workflow Commands

```bash
# Create and switch to a new branch
git checkout -b feature/description-of-change

# Make your changes, then commit
git add .
git commit -m "type: description following conventional commits"

# Push the branch
git push origin feature/description-of-change

# Create a pull request using GitHub CLI
gh pr create --title "type: description" --body "Description of changes"
```

## Conventional Commits

Follow the conventional commits specification:
- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation changes
- `style:` Code style changes (formatting, etc.)
- `refactor:` Code refactoring
- `test:` Test changes
- `chore:` Maintenance tasks
- `ci:` CI/CD changes

## AI Developer Guide

You MUST follow the AI Developer Guide at all times:
- Read it at: https://github.com/dwmkerr/ai-developer-guide
- Local copy available at: /Users/tim.van.wassenhove/src/github/ai-developer-guide

### Key Principles from the Guide:

1. **Plan / Implement / Review Approach**
   - Phase 1: Planning - Design and discuss before implementing
   - Phase 2: Implementation - Only implement agreed changes
   - Phase 3: Review - Verify changes and discuss improvements

2. **Never make assumptions** about improvements beyond what was discussed

3. **Always check existing code** before implementing new features

4. **Documentation and tests** must be part of the plan

## Project-Specific Rules

1. **No direct commits to main** - Always use feature branches and PRs
2. **All CI checks must pass** before merging
3. **Follow Rust idioms** and existing project patterns
4. **Keep changes focused** - One PR should address one issue
5. **Update documentation** when changing functionality

## Development Process

When asked to make changes:

1. **STOP** and discuss the plan first
2. Create a feature branch for the work
3. Implement only what was agreed
4. Create a PR for review
5. Wait for CI and human review
6. Only merge after approval

## Testing Locally Before Creating PR

**IMPORTANT: Always run CI checks before committing and pushing!** This speeds up CI checks and avoids ping-pong due to formatting issues.

Run these checks in order before every commit:
```bash
# 1. Format the code
cargo fmt

# 2. Run linter
cargo clippy --all-targets --all-features -- -D warnings

# 3. Run tests
cargo test --all-features

# 4. Check documentation
cargo doc --no-deps --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
```

Or run all checks in one command:
```bash
cargo fmt && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-features && cargo doc --no-deps --all-features
```

## Remember

- This is an open-source project under the genai-rs organization
- Quality and process matter more than speed
- Always create PRs for review, even for small changes
- Follow the established patterns in the codebase
