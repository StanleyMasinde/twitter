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
1. Download the latest binary from [GitHub releases](https://github.com/StanleyMasinde/twitter/releases)
2. Move it to your PATH:
```bash
sudo mv twitter_cli /usr/local/bin/
sudo chmod +x /usr/local/bin/twitter_cli
```

## Usage

### CLI Mode
```bash
# Tweet
twitter_cli --tweet "Building something cool today"
```

### Server Mode
```bash
# Start local server (default port 3000)
twitter_cli --serve

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
