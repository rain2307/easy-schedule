# Contributing to Easy Schedule

Thank you for your interest in contributing to Easy Schedule! This document provides guidelines and information for contributors.

## 🚀 Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/yourusername/easy-schedule.git`
3. Create a new branch: `git checkout -b feature/your-feature-name`
4. Make your changes
5. Run tests: `cargo test`
6. Commit your changes using [Conventional Commits](#conventional-commits)
7. Push to your fork and submit a pull request

## 🧪 Running Tests

```bash
# Run all tests
cargo test

# Run specific test files
cargo test --test skip_tests
cargo test --test task_tests
cargo test --test scheduler_tests

# Run examples
cargo run --example basic
cargo run --example skip_example
cargo run --example string_parsing
cargo run --example error_handling

# Check formatting and linting
cargo fmt --check
cargo clippy -- -D warnings
```

## 📝 Conventional Commits

This project uses [Conventional Commits](https://www.conventionalcommits.org/) for automatic versioning and changelog generation. Please follow this format:

### Commit Message Format

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Types

- **feat**: A new feature (triggers minor version bump)
- **fix**: A bug fix (triggers patch version bump)
- **perf**: Performance improvement
- **refactor**: Code refactoring without functionality changes
- **docs**: Documentation changes
- **test**: Adding or updating tests
- **ci**: CI/CD changes
- **chore**: Maintenance tasks

### Examples

```bash
# New feature
git commit -m "feat: add priority system for tasks"

# Bug fix
git commit -m "fix: resolve timezone calculation error"

# Breaking change (triggers major version bump)
git commit -m "feat!: redesign Task API for better ergonomics"

# With scope
git commit -m "feat(scheduler): add task batching support"

# With body and footer
git commit -m "fix: handle edge case in time range calculation

When time ranges cross midnight, the previous logic
would incorrectly skip valid execution times.

Fixes #123"
```

## 🔄 Automatic Releases

This project uses automated releases:

- **Commits to main branch** trigger release checks
- **Conventional commits** determine version bumps:
  - `fix:` → patch version (0.1.0 → 0.1.1)
  - `feat:` → minor version (0.1.0 → 0.2.0)  
  - `feat!:` or `BREAKING CHANGE:` → major version (0.1.0 → 1.0.0)
- **GitHub releases** are automatically created
- **Crates.io publishing** happens automatically

## 🐛 Reporting Issues

When reporting issues, please include:

- Rust version (`rustc --version`)
- Operating system
- Minimal example that reproduces the issue
- Expected vs actual behavior

## 📋 Pull Request Guidelines

- Include tests for new features
- Update documentation as needed
- Follow existing code style
- Write clear commit messages
- Keep PRs focused on a single change

## 💡 Development Tips

### Project Structure

```
src/
├── lib.rs          # Main library code
examples/           # Usage examples
tests/              # Integration tests
├── skip_tests.rs   # Skip functionality tests
├── task_tests.rs   # Task parsing tests
└── scheduler_tests.rs # Scheduler tests
```

### Adding New Features

1. Write tests first (TDD approach)
2. Implement the feature
3. Update documentation and examples
4. Ensure all CI checks pass

### Code Style

- Use `cargo fmt` for formatting
- Address all `cargo clippy` warnings
- Follow Rust naming conventions
- Add documentation for public APIs

## 🤝 Code of Conduct

Be respectful, inclusive, and constructive in all interactions. We want this to be a welcoming community for all contributors.

## ❓ Questions

Feel free to open an issue for questions or discussion about:
- Feature requests
- API design
- Performance considerations
- Documentation improvements

Thank you for contributing! 🎉