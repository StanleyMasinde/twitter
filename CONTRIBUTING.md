# Contributing to Twitter CLI

Thank you for considering contributing to Twitter CLI! This document outlines the guidelines and processes for contributing to this project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Process](#development-process)
- [Commit Guidelines](#commit-guidelines)
- [Pull Request Process](#pull-request-process)
- [Rust Coding Standards](#rust-coding-standards)
- [Testing](#testing)
- [Documentation](#documentation)

## Code of Conduct

This project follows our [Code of Conduct](./CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to the project maintainers.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/your-username/twitter.git
   cd twitter
   ```
3. **Set up the development environment**:
   ```bash
   # Install Rust (if you haven't already)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Install required dependencies
   cargo build
   ```

4. **Create a branch** for your work:
   ```bash
   git checkout -b feature/your-feature-name
   ```

## Development Process

1. Make your changes in your feature branch
2. Write or update tests as necessary
3. Run tests locally to ensure they pass:
   ```bash
   cargo test
   ```
4. Format your code using Rust's formatter:
   ```bash
   cargo fmt
   ```
5. Ensure your code passes the linter:
   ```bash
   cargo clippy
   ```

## Commit Guidelines

This project follows [Conventional Commits](https://www.conventionalcommits.org) for all commit messages. This helps maintain a clear and structured commit history and enables automatic versioning and changelog generation.

### Commit Format

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Types

- **feat**: A new feature
- **fix**: A bug fix
- **docs**: Documentation only changes
- **style**: Changes that do not affect the meaning of the code (formatting, etc)
- **refactor**: A code change that neither fixes a bug nor adds a feature
- **perf**: A code change that improves performance
- **test**: Adding missing tests or correcting existing tests
- **chore**: Changes to the build process or auxiliary tools

### Examples

```
feat: add thread support via stdin piping
```

```
fix(auth): correctly handle expired tokens

Resolves issue #42
```

```
docs: update README with server mode instructions
```

## Pull Request Process

1. Update the README.md or documentation with details of changes if appropriate
2. The PR should work on the main development branch
3. Include tests that cover your changes if applicable
4. Follow the commit message conventions
5. Your PR will be reviewed by at least one maintainer
6. Once approved, your PR will be merged

## Rust Coding Standards

- Follow Rust's official style guide and idiomatic practices
- Use meaningful variable and function names
- Add comments for complex logic
- Keep functions small and focused on a single task
- Use proper error handling with Result types

## Testing

- Write unit tests for new functionality
- Ensure all tests pass before submitting a PR
- Consider edge cases and error scenarios in your tests

## Documentation

- Update documentation for any new features or changes
- Use clear and concise language
- Include examples where appropriate

---

Thank you for contributing to Twitter CLI! Your efforts help make this tool better for everyone who wants to tweet without the distractions of twitter.com.
