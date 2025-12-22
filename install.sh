#!/usr/bin/env sh

set -e

REPO="StanleyMasinde/twitter"
VERSION="${1:-latest}"
INSTALL_DIR="${TWITTER_INSTALL:-/usr/local/bin}"

detect_platform() {
    local os arch
    
    os=$(uname -s | tr '[:upper:]' '[:lower:]')
    case "$os" in
        linux*) os="linux" ;;
        darwin*) os="darwin" ;;
        mingw*|msys*|cygwin*) os="windows" ;;
        *) echo "Error: Unsupported OS: $os" >&2; exit 1 ;;
    esac
    
    arch=$(uname -m)
    case "$arch" in
        x86_64|amd64) arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        armv7l|armv6l) arch="arm" ;;
        *) echo "Error: Unsupported architecture: $arch" >&2; exit 1 ;;
    esac
    
    echo "${os}-${arch}"
}

get_release_data() {
    local version="$1"
    local api_url
    
    if [ "$version" = "latest" ]; then
        api_url="https://api.github.com/repos/$REPO/releases/latest"
    else
        api_url="https://api.github.com/repos/$REPO/releases/tags/$version"
    fi
    
    curl -fsSL "$api_url" || {
        echo "Error: Could not fetch release data" >&2
        exit 1
    }
}

parse_version() {
    local json="$1"
    echo "$json" | grep -o '"tag_name"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | sed 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/'
}

parse_asset() {
    local json="$1"
    local filename="$2"
    local url="" digest=""
    
    local asset_block
    asset_block=$(echo "$json" | sed -n "/${filename}/,/\"browser_download_url\"/p")
    
    url=$(echo "$asset_block" | grep "browser_download_url" | sed 's/.*"browser_download_url"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/')
    
    digest=$(echo "$asset_block" | grep "digest" | sed 's/.*"sha256:\([^"]*\)".*/\1/')
    
    if [ -n "$url" ] && [ -n "$digest" ]; then
        echo "${url}|${digest}"
    fi
}

verify_checksum() {
    local file="$1"
    local expected_sha="$2"
    
    if [ -z "$expected_sha" ] || [ "$expected_sha" = "null" ]; then
        echo "Warning: No checksum available for this release"
        echo "Skipping verification (this is expected for older releases)"
        return 0
    fi
    
    echo "Verifying checksum..."
    
    local actual_sha
    if command -v sha256sum >/dev/null 2>&1; then
        actual_sha=$(sha256sum "$file" | awk '{print $1}')
    elif command -v shasum >/dev/null 2>&1; then
        actual_sha=$(shasum -a 256 "$file" | awk '{print $1}')
    else
        echo "Warning: Neither sha256sum nor shasum found"
        echo "Cannot verify checksum"
        return 0
    fi
    
    # Compare checksums
    if [ "$actual_sha" = "$expected_sha" ]; then
        echo "✓ Checksum verified: $expected_sha"
        return 0
    else
        echo "✗ Checksum verification failed!" >&2
        echo "  Expected: $expected_sha" >&2
        echo "  Got:      $actual_sha" >&2
        echo "" >&2
        echo "The downloaded file may be corrupted or tampered with." >&2
        echo "Please try downloading again or report this issue." >&2
        return 1
    fi
}

install_twitter() {
    local version="$1"
    local platform="$2"
    
    local ext
    case "$platform" in
        windows-*) ext="zip" ;;
        *) ext="tar.gz" ;;
    esac
    
    local filename="twitter-${platform}.${ext}"
    
    echo "Twitter CLI Installer"
    echo ""
    echo "Fetching release information..."
    
    local release_json
    release_json=$(get_release_data "$version")
    
    if [ "$version" = "latest" ]; then
        version=$(parse_version "$release_json")
        if [ -z "$version" ]; then
            echo "Error: Could not parse version from API response" >&2
            exit 1
        fi
    fi
    
    echo "Version:  $version"
    echo "Platform: $platform"
    echo ""
    
    local asset_info
    asset_info=$(parse_asset "$release_json" "$filename")
    
    if [ -z "$asset_info" ]; then
        echo "Error: Could not find asset '$filename' in release" >&2
        echo "" >&2
        echo "Available platforms for this release:" >&2
        echo "$release_json" | grep -o '"name"[[:space:]]*:[[:space:]]*"twitter-[^"]*"' | sed 's/.*"twitter-\([^"]*\)".*/  - \1/' | sed 's/\.tar\.gz$//' | sed 's/\.zip$//' >&2
        exit 1
    fi
    
    local download_url sha256_digest
    download_url=$(echo "$asset_info" | cut -d'|' -f1)
    sha256_digest=$(echo "$asset_info" | cut -d'|' -f2)
    
    if [ -z "$download_url" ]; then
        echo "Error: Could not extract download URL from release data" >&2
        exit 1
    fi
    
    local tmp_dir
    tmp_dir=$(mktemp -d)
    cd "$tmp_dir"
    
    echo "Downloading from: $download_url"
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL --progress-bar -o "$filename" "$download_url" || {
            echo "Error: Download failed" >&2
            rm -rf "$tmp_dir"
            exit 1
        }
    elif command -v wget >/dev/null 2>&1; then
        wget -q --show-progress -O "$filename" "$download_url" || {
            echo "Error: Download failed" >&2
            rm -rf "$tmp_dir"
            exit 1
        }
    else
        echo "Error: Neither curl nor wget found" >&2
        exit 1
    fi
    
    echo ""
    
    verify_checksum "$filename" "$sha256_digest" || {
        rm -rf "$tmp_dir"
        exit 1
    }
    
    echo ""
    
    echo "Extracting..."
    case "$ext" in
        tar.gz)
            tar -xzf "$filename" || {
                echo "Error: Extraction failed" >&2
                rm -rf "$tmp_dir"
                exit 1
            }
            ;;
        zip)
            if command -v unzip >/dev/null 2>&1; then
                unzip -q "$filename" || {
                    echo "Error: Extraction failed" >&2
                    rm -rf "$tmp_dir"
                    exit 1
                }
            else
                echo "Error: unzip not found" >&2
                rm -rf "$tmp_dir"
                exit 1
            fi
            ;;
    esac
    
    if [ -f "twitter" ] || [ -f "twitter.exe" ]; then
        local binary_name="twitter"
        [ -f "twitter.exe" ] && binary_name="twitter.exe"
        
        chmod +x "$binary_name" 2>/dev/null || true
        
        echo "Installing to $INSTALL_DIR..."
        
        # Check if we need sudo
        if [ -w "$INSTALL_DIR" ]; then
            install -m 755 "$binary_name" "$INSTALL_DIR/twitter" || {
                echo "Error: Installation failed" >&2
                rm -rf "$tmp_dir"
                exit 1
            }
        else
            sudo install -sm 755 "$binary_name" "$INSTALL_DIR/twitter" || {
                echo "Error: Installation failed" >&2
                echo "Make sure you have sudo privileges or set TWITTER_INSTALL to a writable directory" >&2
                rm -rf "$tmp_dir"
                exit 1
            }
        fi
        
        cd - > /dev/null
        rm -rf "$tmp_dir"
        
        echo ""
        echo "✓ Twitter CLI was installed successfully to $INSTALL_DIR/twitter"
        echo ""
        echo "Run 'twitter --help' to get started"
    else
        echo "Error: Binary not found after extraction" >&2
        rm -rf "$tmp_dir"
        exit 1
    fi
}

if [ "$1" = "-h" ] || [ "$1" = "--help" ]; then
    cat <<EOF
Twitter CLI Installer

Usage: 
  curl -fsSL https://raw.githubusercontent.com/$REPO/main/install.sh | sh
  
Or with specific version:
  curl -fsSL https://raw.githubusercontent.com/$REPO/main/install.sh | sh -s v1.4.0

Environment Variables:
  TWITTER_INSTALL    Installation directory (default: /usr/local/bin)

Examples:
  # Install latest version
  curl -fsSL <installer-url> | sh
  
  # Install specific version
  curl -fsSL <installer-url> | sh -s v1.5.0
  
  # Install to custom location
  curl -fsSL <installer-url> | TWITTER_INSTALL=~/.local/bin sh

Supported Platforms:
  - Linux (x86_64, aarch64, arm)
  - macOS/Darwin (x86_64, aarch64)
  - Windows (x86_64, arm)

Security:
  - Downloads are verified using SHA256 checksums from GitHub API
  - Checksums are automatically generated by GitHub for all release assets
  - Installation will fail if checksum verification fails

Requirements:
  - curl or wget
  - sha256sum or shasum (for checksum verification)
  - tar (for Unix systems) or unzip (for Windows)
  - sudo (if installing to system directory)
EOF
    exit 0
fi

main() {
    local platform version
    
    platform=$(detect_platform)
    
    if [ -z "$VERSION" ] || [ "$VERSION" = "latest" ]; then
        version="latest"
    else
        version="$VERSION"
    fi
    
    install_twitter "$version" "$platform"
}

main
