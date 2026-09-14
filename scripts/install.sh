#!/usr/bin/env sh
# VoxForg Turnkey One-Line Installer for Linux & macOS
# Usage: curl -fsSL https://raw.githubusercontent.com/AMG555/VoxForg/main/scripts/install.sh | sh

set -e

REPO="AMG555/VoxForg"
INSTALL_DIR="${VOXFORG_INSTALL_DIR:-$HOME/.local/bin}"

printf "\033[1;33m"
cat << 'EOF'
  _    _            ______                 
 | |  | |          |  ____|                
 | |  | | _____  __| |__ ___  _ __ __ _    
 | |  | |/ _ \ \/ /|  __/ _ \| '__/ _` |   
  \ \/ / (_) >  < | | | (_) | | | (_| |   
   \__/ \___/_/\_\|_|  \___/|_|  \__, |   
                                  __/ |   
                                 |___/    
EOF
printf "\033[0m"
printf "\033[1;37mVoxForg Standalone Workstation Installer\033[0m\n\n"

# 1. Detect OS and Architecture
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS" in
    linux*)
        PLATFORM="linux"
        ;;
    darwin*)
        PLATFORM="macos"
        ;;
    *)
        printf "\033[1;31mError: Unsupported operating system: %s\033[0m\n" "$OS"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64|amd64)
        TARGET_ARCH="x86_64"
        ;;
    arm64|aarch64)
        TARGET_ARCH="aarch64"
        ;;
    *)
        printf "\033[1;31mError: Unsupported CPU architecture: %s\033[0m\n" "$ARCH"
        exit 1
        ;;
esac

BINARY_NAME="voxforg-${PLATFORM}-${TARGET_ARCH}"
if [ "$PLATFORM" = "windows" ]; then
    BINARY_NAME="${BINARY_NAME}.exe"
fi

printf "Detected Platform: \033[1;32m%s\033[0m (%s)\n" "$PLATFORM" "$TARGET_ARCH"

# 2. Determine target install directory
mkdir -p "$INSTALL_DIR"
TARGET_FILE="$INSTALL_DIR/voxforg"

# 3. Fetch latest release version
LATEST_TAG=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' || true)
if [ -z "$LATEST_TAG" ]; then
    LATEST_TAG="v0.1.0"
fi

DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/${BINARY_NAME}"

printf "Fetching release \033[1;36m%s\033[0m from GitHub...\n" "$LATEST_TAG"
if curl -fL --progress-bar "$DOWNLOAD_URL" -o "$TARGET_FILE" 2>/dev/null; then
    chmod +x "$TARGET_FILE"
    printf "\033[1;32mSuccessfully installed voxforg binary to %s\033[0m\n" "$TARGET_FILE"
else
    # Fallback to local source compilation if binary release asset is unavailable
    printf "\033[1;33mRelease binary not directly downloadable; attempting local cargo build...\033[0m\n"
    if command -v cargo >/dev/null 2>&1; then
        cargo install --git "https://github.com/${REPO}.git" voxforg-cli --root "$HOME/.local"
        printf "\033[1;32mSuccessfully compiled and installed via cargo.\033[0m\n"
    else
        printf "\033[1;31mDownload failed and cargo was not found. Please install Rust or check repository releases.\033[0m\n"
        exit 1
    fi
fi

# 4. PATH verification
case ":$PATH:" in
    *":$INSTALL_DIR:"*)
        ;;
    *)
        printf "\n\033[1;33mNote: %s is not in your current PATH.\033[0m\n" "$INSTALL_DIR"
        printf "Add it to your environment by running:\n"
        printf "  export PATH=\"%s:\$PATH\"\n\n" "$INSTALL_DIR"
        ;;
esac

# 5. Verify system hardware profile
printf "\nValidating installation...\n"
"$TARGET_FILE" hardware || true

printf "\n\033[1;32m=== VoxForg installation ready! ===\033[0m\n"
printf "Run visual workstation:  \033[1;37mvoxforg serve --open\033[0m\n"
printf "Run speech synthesis:    \033[1;37mvoxforg synth \"Hello from VoxForg\"\033[0m\n"
printf "Documentation & API:     \033[1;37mhttp://localhost:8080/docs\033[0m\n\n"
