# Twitter CLI
> Tweet without going to twitter.com

## What it does
I love creating content on Twitter but twitter.com leads to doomscrolling. This is my way of fighting that.

Simple CLI for posting to Twitter using their API v2. No authentication flow - just configure once and tweet.

## Installation

### Quick install
Install the latest version with the following command.

```shell
curl -fsSL https://raw.githubusercontent.com/StanleyMasinde/twitter/main/install.sh | sh
```
This will automatically
- Detect your operating system and architecture
- Download the correct binary
- Verify the SHA256 checksum for security
- Install to /usr/local/bin
- Clean up temporary files

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
twitter update
```

### ArchLinux
ArchLinux users can install the community maintained AUR binary [package](https://aur.archlinux.org/packages/twitter-cli) using yay or any other AUR helper:
```bash
yay -S twitter-cli
```
>[!WARNING]
> The AUR package is managed by the community. 
> Please use the install script for Unix based systems.

After installation the executable is available as `twitter`

You can also download the appropriate binary for your machine from [releases](https://github.com/StanleyMasinde/twitter/releases/latest):

### Windows x64 Via powershell
> Run the Windows terminal as Administrator. 
> Ensure your default shell is Powershell if you are not sure, search for pwershell and run it.
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
> After that the above installation, open the Windows Terminal and use Twitter CLI by typing twitter. 

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
#### Tweet
```bash
twitter tweet --body "Building something cool today"
```

#### Piped input
```bash
echo "I love CLIs" | twitter tweet
```


##### From text files
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

**API Response:**
```json
{
  "id": "1234567890",
  "text": "Building in public without the scroll trap"
}
```

## Show usage
You can show the API usage via the usage subcommand.
```shell
twitter usage
```
The above command will show the API usage.

## Future Plans
- Thread support via stdin piping
- Media attachments
- Multiple account profiles

## Tech Stack
- Rust + Axum
- Twitter API v2
