# Contributing to Twitter CLI

Use this guide to set up a development checkout, run the project checks, and submit a pull request. You need Git and a Rust development environment.

## Code of conduct

Follow the [Code of Conduct](CODE_OF_CONDUCT.md). Report unacceptable behavior to the project maintainers.

## Set up your development environment

Install stable Rust with [rustup](https://rust-lang.github.io/rustup/installation/), including Clippy and rustfmt. If you already use rustup, select stable Rust and install the components:

```sh
rustup default stable
rustup component add clippy rustfmt
```

Before building, install the native tools for your platform:

- **macOS:** Xcode Command Line Tools.
- **Linux:** a C compiler, `pkg-config`, and the development packages for libcurl, OpenSSL, SQLite, and zlib. Package names depend on your distribution.
- **Windows:** PowerShell 7 or later, the Rust MSVC toolchain for your architecture, and Visual Studio C++ build tools with a Windows SDK. Follow the [vcpkg MSVC prerequisites](https://learn.microsoft.com/en-us/vcpkg/users/platforms/windows).

### Create a checkout

1. Fork [the CLI repository](https://github.com/StanleyMasinde/twitter) on GitHub.
2. Clone your fork:

   ```sh
   git clone https://github.com/GITHUB_USERNAME/twitter.git
   cd twitter
   ```

   Replace `GITHUB_USERNAME` with your GitHub username.

3. Create a branch for your change:

   ```sh
   git switch -c feature/your-feature-name
   ```

### Set up Windows native dependencies

Run these commands in PowerShell before building. They keep your working directory in the CLI checkout and install vcpkg in your user profile:

```powershell
git clone https://github.com/microsoft/vcpkg.git "$env:USERPROFILE\vcpkg"
& "$env:USERPROFILE\vcpkg\bootstrap-vcpkg.bat"
if ($LASTEXITCODE -ne 0) { throw "vcpkg bootstrap failed" }
$env:VCPKG_ROOT = "$env:USERPROFILE\vcpkg"
```

If you already have vcpkg, skip cloning and bootstrapping and set `VCPKG_ROOT` to its existing directory.

Choose the triplet that matches your Rust MSVC toolchain. For x64, run:

```powershell
$env:VCPKGRS_TRIPLET = "x64-windows-static-md"
```

For ARM64, run this instead:

```powershell
$env:VCPKGRS_TRIPLET = "arm64-windows-static-md"
```

Install the dependencies and configure static linking:

```powershell
$env:VCPKGRS_DYNAMIC = "0"
& "$env:VCPKG_ROOT\vcpkg.exe" install "sqlite3:$env:VCPKGRS_TRIPLET" "curl:$env:VCPKGRS_TRIPLET" "zlib:$env:VCPKGRS_TRIPLET"
if ($LASTEXITCODE -ne 0) { throw "Native dependency installation failed" }

$nativeLibDir = Join-Path $env:VCPKG_ROOT "installed/$env:VCPKGRS_TRIPLET/lib"
if (-not (Test-Path (Join-Path $nativeLibDir "zs.lib") -PathType Leaf)) {
    throw "Static zlib library not found in $nativeLibDir"
}
$env:CARGO_ENCODED_RUSTFLAGS = @(
    "-L", "native=$nativeLibDir", "-l", "static=zs", "-l", "iphlpapi"
) -join [char]0x1f
```

The static curl library needs zlib and Windows IP Helper linkage. zlib 1.3.2 uses `zs.lib` for its Windows static release library. These flags provide the same linkage as the Windows CI workflows. [Cargo's encoded flags](https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-reads) preserve library paths containing spaces.

Keep this PowerShell session open for builds and checks. In a new session, set `VCPKG_ROOT`, `VCPKGRS_TRIPLET`, `VCPKGRS_DYNAMIC`, and the linker flags again; you do not need to reinstall the dependencies.

### Build and verify the CLI

Build from the repository root:

```sh
cargo build --locked
```

On macOS or Linux, verify the executable:

```sh
./target/debug/twitter --version
./target/debug/twitter --help
```

On Windows, verify it in PowerShell:

```powershell
.\target\debug\twitter.exe --version
.\target\debug\twitter.exe --help
```

The first command prints the version; the second lists the available commands.

## Develop and run checks

Make your changes and add or update tests for the affected behavior. Format the code before running the checks:

```sh
cargo fmt
```

Run the same formatting, lint, and test checks as CI:

```sh
cargo fmt --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked
```

Build the Rust API documentation with warnings treated as errors. On macOS or Linux, run:

```sh
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps
```

On Windows, run:

```powershell
$env:RUSTDOCFLAGS = "-D warnings"
cargo doc --locked --no-deps
```

CI runs these checks on Ubuntu for pull requests to `main`. The smoke tests and full platform build matrix run when the CI workflow is manually dispatched. The release workflow builds Linux, macOS, and Windows x64/ARM64 binaries on version tags.

## Write commit messages

Use [Conventional Commits](https://www.conventionalcommits.org) with a concise subject and a body when the change needs explanation:

```text
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

Use these types:

- **feat:** add a feature.
- **fix:** fix a bug.
- **docs:** change documentation.
- **style:** change formatting without changing behavior.
- **refactor:** restructure code without adding a feature or fixing a bug.
- **perf:** improve performance.
- **test:** add or update tests.
- **ci:** change CI or release workflows.
- **build:** change dependencies or build configuration.
- **chore:** perform other maintenance.

For example:

```text
fix(ci): link static curl dependencies on Windows

Add the vcpkg library search path and link zs and iphlpapi for
Windows builds.
```

Conventional Commits makes the intent of each change clear. The release workflow publishes releases from version tags and generates GitHub release notes; it does not automatically bump the version in `Cargo.toml`.

## Submit a pull request

1. Update tests and documentation relevant to your change.
2. Run the checks in [Develop and run checks](#develop-and-run-checks).
3. Push your branch to your fork and open a pull request targeting `main`.
4. Fill in the [PR template](.github/pull_request_template.md), including the problem, changes, related issues, and verification steps.
5. Address maintainer feedback and ensure the required checks pass.

A maintainer reviews the pull request and decides when to merge it.

## Follow Rust coding standards

Use these conventions:

- Write idiomatic Rust and follow rustfmt's formatting.
- Use descriptive names and keep functions focused on one task.
- Explain complex logic with comments.
- Use `Result` and the project's error types for recoverable failures.
- Cover the changed behavior, including relevant edge cases and error paths, with tests.

Unit tests live alongside the source in `#[cfg(test)]` modules. The thread-splitting benchmark is in `benches/split_tweet.rs`. To run it, use:

```sh
cargo bench --locked --bench split_tweet
```

## Update documentation

Keep `README.md` concise and update it when installation or basic usage changes. User guides and the documentation site live in [StanleyMasinde/twitter-docs](https://github.com/StanleyMasinde/twitter-docs); submit changes there for user-facing command, setup, and troubleshooting instructions.

Update Rust API comments when you change public behavior. Use clear language and runnable examples, and check commands against the CLI's `--help` output and source.
