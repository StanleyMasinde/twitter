# Twitter CLI

Tweet from your terminal without opening twitter.com. This Rust CLI also supports threads, scheduled tweets, and other Twitter API v2 actions.

[Documentation](https://twitter.stanleymasinde.com) · [Releases](https://github.com/StanleyMasinde/twitter/releases/latest) · [Contributing](CONTRIBUTING.md)

## Get started

On macOS or Linux, install the latest release:

```sh
curl -fsSL https://raw.githubusercontent.com/StanleyMasinde/twitter/main/install.sh | sh
```

On Windows, install the latest release in PowerShell 7 or later:

```powershell
irm https://raw.githubusercontent.com/StanleyMasinde/twitter/main/install.ps1 | iex
```

For other installation options, see the [installation guide](https://twitter.stanleymasinde.com).

Create a [Twitter developer app](https://developer.twitter.com), then configure the CLI with its credentials:

```sh
twitter config --init
twitter tweet --body "Hello from the terminal"
```

For setup details, commands, and troubleshooting, see the [full documentation](https://twitter.stanleymasinde.com).
