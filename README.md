# Twitter CLI
> Tweet without going to twitter.com


![GitHub Downloads (all assets, all releases)](https://img.shields.io/github/downloads/StanleyMasinde/twitter/total)
[![Build and test](https://github.com/StanleyMasinde/twitter/actions/workflows/rust.yml/badge.svg)](https://github.com/StanleyMasinde/twitter/actions/workflows/rust.yml)

## What it does
I love creating content on Twitter but twitter.com leads to doomscrolling. This is my way of fighting that.

Simple CLI for posting to Twitter using their API v2. No OAuth flow - just configure once and tweet.

## Installation

### Quick install
Install the latest version with the following command. If you prefer, review the script first.

```shell
curl -fsSL https://raw.githubusercontent.com/StanleyMasinde/twitter/main/install.sh | sh
```
This will automatically
- Detect your operating system and architecture
- Download the correct binary
- Verify the SHA256 checksum for security
- Install to /usr/local/bin
- Clean up temporary files

Review the installer before running:
```shell
curl -fsSL https://raw.githubusercontent.com/StanleyMasinde/twitter/main/install.sh
```

### Install a specific version
```shell
curl -fsSL https://raw.githubusercontent.com/StanleyMasinde/twitter/main/install.sh | sh -s v1.5.0
```

### Custom install location
```shell
curl -fsSL https://raw.githubusercontent.com/StanleyMasinde/twitter/main/install.sh | TWITTER_INSTALL=~/.local/bin sh
```

### Updating
```shell
sudo twitter update
```
`twitter update` downloads the latest release, writes a temporary binary in the install directory, then atomically replaces the current executable.  
If `twitter` is installed in `/usr/local/bin` (default install location), that directory is root-owned, so updating requires elevated privileges.

If you installed to a user-writable location (for example with `TWITTER_INSTALL=~/.local/bin`), `sudo` is not required.

On Windows, run:
```powershell
twitter update
```
`twitter update` schedules replacement of the current `twitter.exe` after the running process exits. Run an elevated terminal only if the executable is in a protected directory (for example `C:\Program Files`).

### ArchLinux
ArchLinux users can install the community maintained AUR binary [package](https://aur.archlinux.org/packages/twitter-cli) using yay or any other AUR helper:
```bash
yay -S twitter-cli
```
>[!WARNING]
> The AUR package is managed by the community. 
> Please use the install script for other Unix based systems.

After installation the executable is available as `twitter`

You can also download the appropriate binary for your machine from [releases](https://github.com/StanleyMasinde/twitter/releases/latest):

### Windows x64 Via PowerShell
> Administrator privileges are only needed when writing to a protected directory.
> Ensure your default shell is PowerShell. If you are not sure, search for PowerShell and run it.
```powershell
# Download and extract
Invoke-WebRequest -Uri https://github.com/StanleyMasinde/twitter/releases/latest/download/twitter-windows-x86_64.zip -OutFile twitter.zip
Expand-Archive -Force twitter.zip -DestinationPath .

# Move exe to a user bin directory
$dest = "$env:USERPROFILE\bin"
New-Item -ItemType Directory -Force -Path $dest | Out-Null
Move-Item -Force ".\twitter.exe" "$dest\twitter.exe"

# Add bin directory to PATH (persistent, per-user)
$old = [Environment]::GetEnvironmentVariable("Path","User")
if ($old -notlike "*$dest*") {
  [Environment]::SetEnvironmentVariable("Path","$old;$dest","User")
}

# Make it available in this session
$env:Path += ";$dest"
```

>[!NOTE]
> For the CLI to run on Windows, ensure you have installed the latest C++ [redistributable runtime](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist?view=msvc-170#latest-supported-redistributable-version) for your architecture.
> After that installation, open the Windows Terminal and use Twitter CLI by typing twitter. 

## Configuration
1. Create a Twitter developer account at [developer.twitter.com](https://developer.twitter.com)
2. Create a new app and get your API credentials

### Interactive Setup (Recommended)
> [!WARNING]
> This will override your existing config. Only run it on setup.

```bash
twitter config --init
```

### Quick Edit
```bash
twitter config --edit
```
Opens your default editor (`$EDITOR`) with the config file. Creates `~/.config/twitter_cli/config.toml` if it doesn't exist.

Expected format:
```toml
# The account that will be used to tweet
# Please note, current account uses 0 based index.
# This means the first account is 0
current_account = 0

# Account 1
[[accounts]]
consumer_key = "your_consumer_key"
consumer_secret = "your_consumer_secret"
access_token = "your_access_token"
access_secret = "your_access_secret"
bearer_token = "your_bearer_token"

# Account 2
[[accounts]]
consumer_key = "your_consumer_key"
consumer_secret = "your_consumer_secret"
access_token = "your_access_token"
access_secret = "your_access_secret"
bearer_token = "your_bearer_token"
```

### Manual Configuration
Create config file at `~/.config/twitter_cli/config.toml` with the format above. Keep this file private since it contains API secrets.

### Validation
```bash
twitter config --show # Visual preview
twitter config --validate # Check for issues
```

## Update App Permissions
If you face a 403 error when tweeting:

1. In the Twitter Developer Portal, go to your App → **User authentication settings**
2. Set **App permissions** to **Read and write**
3. Save changes, then regenerate your Access Token & Secret
4. Update your config with the new values

> **NB:** Regenerate tokens after updating permissions, otherwise old tokens remain read-only.

## Service Unavailable (503) when posting
If media upload works but `POST /2/tweets` fails with:

```json
{"title":"Service Unavailable","detail":"Service Unavailable","type":"about:blank","status":503}
```

check the following:

1. In the X Developer Portal (`developer.x.com` / `console.x.com`), confirm your app has active billing and available credits.
2. Ensure your app permissions are still **Read and write**, then regenerate Access Token and Secret after permission changes.
3. Retry a minimal text-only tweet first:
   ```bash
   twitter tweet --body "test"
   ```
4. If text-only works, retry with media:
   ```bash
   twitter tweet --body "test with image" --image ~/path/to/image.png
   ```
5. Check your X API usage dashboard and logs to confirm write calls are not blocked by billing, limits, or temporary platform incidents.

> **Note:** Some 503 responses are transient. If configuration and billing are correct, wait a few minutes and retry.

## Usage

### Tweet in CLI Mode
#### Tweet
```bash
twitter tweet --body "Building something cool today"
```

#### Piped input
```bash
echo "I love CLIs" | twitter tweet
```
#### From text files
```bash
cat drafts.txt | twitter tweet
```
#### Edit tweet in an editor
Omit --body and it will launch your default terminal editor. Like Vim or Nano.
```bash
twitter tweet
```

#### Add Media
The parameter is image because I do not see the point of uploading video from the terminal.
```bash
twitter tweet --image ~/Downloads/image.png
```

### Tweet a thread
Threads are created whenever input contains `---` separators, regardless of input mode.
```bash
twitter tweet --editor
```
This will launch your default terminal editor. Separate your threads with --- like this.
```txt
This is a very long thread. This being the intro.
---
The three dashes indicate the boundary between tweets so this will be tweet 2.
---
This will be tweet 3
```

#### Thread via stdin
Pipe thread content with `---` separators.
```bash
cat thread.txt | twitter tweet
```

### Schedule tweets
#### Add a scheduled tweet
Use either `--on` or `--at`.
```bash
twitter schedule new --body "Ship update at 5:06pm" --at "17:06"
twitter schedule new --body "Ship update on Tuesday" --on "Tuesday"
```

#### List scheduled tweets
```bash
twitter schedule list
twitter schedule list --filter failed
twitter schedule list --filter sent
```
If no rows match your filter, the CLI prints:
```text
No scheduled tweets were found.
```

#### Run pending scheduled tweets
```bash
twitter schedule run
```
If none are due, the CLI prints:
```text
No pending scheduled tweets to run.
```

#### Pair with your OS scheduler
`schedule run` is intended to be executed regularly by your OS scheduler.
Run scheduler jobs as the same user who ran `twitter config --init`.
First get your installed binary path:
```bash
command -v twitter
```

Linux (cron):
```bash
crontab -e
```
```cron
* * * * * /usr/local/bin/twitter schedule run >> /tmp/twitter-schedule.log 2>&1
```
If your binary is not in `/usr/local/bin/twitter`, replace that path with the output of `command -v twitter`.

macOS (launchd):
```xml
<!-- ~/Library/LaunchAgents/com.twitter.schedule.plist -->
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
  <dict>
    <key>Label</key><string>com.twitter.schedule</string>
    <key>ProgramArguments</key>
    <array>
      <string>/usr/local/bin/twitter</string>
      <string>schedule</string>
      <string>run</string>
    </array>
    <key>StartInterval</key><integer>60</integer>
    <key>StandardOutPath</key><string>/tmp/twitter-schedule.log</string>
    <key>StandardErrorPath</key><string>/tmp/twitter-schedule.log</string>
  </dict>
</plist>
```
If your binary is not in `/usr/local/bin/twitter`, replace that path with the output of `command -v twitter`.
Load it (current user):
```bash
launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/com.twitter.schedule.plist
launchctl kickstart -k gui/$(id -u)/com.twitter.schedule
```

Windows (Task Scheduler):
Use `where twitter` in PowerShell to find the full path to `twitter.exe`.

Create a task that runs every minute with:
```powershell
schtasks /Create /SC MINUTE /MO 1 /TN "TwitterScheduleRunner" /TR "\"C:\Users\<you>\bin\twitter.exe\" schedule run" /F
```
If your binary is not in `C:\Users\<you>\bin\twitter.exe`, replace that path with the output of `where twitter`.

#### Clear scheduled tweets
```bash
twitter schedule clear
```
The CLI prints how many records were removed, for example:
```text
Cleared 3 scheduled tweets.
```

**API Response:**
```text
Tweet Id: 2006409743426818416
Tweet body: Hello, world
```

## Show usage
You can show the API usage via the usage subcommand.
```shell
twitter usage
```
The above command will show the API usage like the one below.
```text
Daily project usage: 0/100
```

## Tech Stack
- Rust
- Twitter API v2
