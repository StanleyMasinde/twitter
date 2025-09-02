# Twitter CLI
> Tweet without going to twitter.com

## What it does
I love creating content on Twitter but twitter.com leads to doomscrolling. This is my way of fighting that.

Simple CLI for posting to Twitter using their API v2. No authentication flow - just configure once and tweet.

## Setup
1. Create a Twitter developer account at [developer.twitter.com](https://developer.twitter.com)
2. Create a new app and get your API credentials
3. Create config file at `~/.config/twitter_cli/config.toml`:
```toml
consumer_key = "your_consumer_key"
consumer_secret = "your_consumer_secret"
access_token = "your_access_token"
access_secret = "your_access_secret"
```

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

## Usage

### CLI Mode
```bash
# Tweet
twitter --tweet "Building something cool today"
```

### Server Mode
```bash
# Start local server (default port 3000)
twitter --serve

# Custom port
twitter --serve --port 8080

# Post via HTTP
curl -X POST http://localhost:3000/api/tweet \
  -H "Content-Type: application/json" \
  -d '{"text": "Building in public without the scroll trap"}'
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

## Tech Stack
- Rust + Axum
- Twitter API v2
