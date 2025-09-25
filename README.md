# Twitter CLI
> Tweet without going to twitter.com

## What it does
I love creating content on Twitter but twitter.com leads to doomscrolling. This is my way of fighting that.

Simple CLI for posting to Twitter using their API v2. No authentication flow - just configure once and tweet.

## Installation
Download the appropriate binary from [releases](https://github.com/StanleyMasinde/twitter/releases/latest):

### Linux (x64)
```bash
wget https://github.com/StanleyMasinde/twitter/releases/latest/download/twitter-linux-x64.tar.gz && tar -xzf twitter-linux-x64.tar.gz && rm twitter-linux-x64.tar.gz
sudo mv twitter /usr/local/bin/
sudo chmod +x /usr/local/bin/twitter
```

### macOS (Intel)
```bash
curl -L https://github.com/StanleyMasinde/twitter/releases/latest/download/twitter-darwin-x64.tar.gz | tar -xz
sudo mv twitter /usr/local/bin/
sudo chmod +x /usr/local/bin/twitter
```

### macOS (Apple Silicon)
```bash
curl -L https://github.com/StanleyMasinde/twitter/releases/latest/download/twitter-darwin-arm64.tar.gz | tar -xz
sudo mv twitter /usr/local/bin/
sudo chmod +x /usr/local/bin/twitter
```

### Windows x64 Via powershell
```powershell
# Download and extract
Invoke-WebRequest -Uri https://github.com/StanleyMasinde/twitter/releases/latest/download/twitter-windows-x64.zip -OutFile twitter.zip
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

## Configuration
1. Create a Twitter developer account at [developer.twitter.com](https://developer.twitter.com)
2. Create a new app and get your API credentials

### Interactive Setup (Recommended)
> [!WARNING]
> This will override your existing config Only run it on setup.

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
consumer_key = "your_consumer_key"
consumer_secret = "your_consumer_secret"
access_token = "your_access_token"
access_secret = "your_access_secret"
```

### Manual Configuration
Create config file at `~/.config/twitter_cli/config.toml` with the format above.

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

## Usage

### Tweet in CLI Mode
```bash
# Tweet
twitter tweet --body "Building something cool today"

# Piped input
echo "I love CLIs" | twitter tweet

# From text files
cat drafts.txt | twitter tweet

# Edit tweet in an editor
# Omit --body and it will launch your default terminal editor. Like Vim or Nano.
twitter tweet
```

### Tweet in Server Mode
```bash
# Start local server (default port 3000)
twitter serve

# Custom port
twitter serve --port 8080

# Post via HTTP
curl -X POST http://localhost:3000/api/tweet \
  -H "Content-Type: application/json" \
  -d '{"text": "Building in public without the scroll trap"}'
```

### Tweet a thread
```bash
twitter tweet
```
This will launch your default terminal editor. Separate your threads with --- like this.
```txt
This is a very long thread. This being the intro.
---
The three dashes indicate the boundary between tweets so this will be tweet 2.
---
This will be tweet 3
```

**API Response:**
```json
{
  "id": "1234567890",
  "text": "Building in public without the scroll trap"
}
```

## Future Plans
- Thread support via stdin piping
- Media attachments
- Multiple account profiles

## Tech Stack
- Rust + Axum
- Twitter API v2
