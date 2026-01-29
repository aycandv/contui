#!/bin/sh
# Contui installer script
# Usage: curl -LsSf https://raw.githubusercontent.com/aycandv/contui/main/scripts/install.sh | sh

set -e

# Configuration
REPO="aycandv/contui"
BINARY_NAME="contui"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Colors (if terminal supports it)
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Print functions
info() {
    printf "${GREEN}info:${NC} %s\n" "$1"
}

warn() {
    printf "${YELLOW}warn:${NC} %s\n" "$1"
}

error() {
    printf "${RED}error:${NC} %s\n" "$1" >&2
}

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Linux*)     echo "linux";;
        Darwin*)    echo "macos";;
        CYGWIN*|MINGW*|MSYS*) echo "windows";;
        *)          echo "unknown";;
    esac
}

# Detect architecture
detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)   echo "x86_64";;
        arm64|aarch64)  echo "aarch64";;
        *)              echo "unknown";;
    esac
}

# Get latest release version from GitHub
get_latest_version() {
    curl -s "https://api.github.com/repos/${REPO}/releases/latest" | \
        grep '"tag_name":' | \
        sed -E 's/.*"([^"]+)".*/\1/'
}

# Download binary
download_binary() {
    local version="$1"
    local os="$2"
    local arch="$3"
    local dest="$4"
    
    # Map OS and arch to release artifact names
    local target
    case "${os}_${arch}" in
        linux_x86_64)   target="x86_64-unknown-linux-gnu";;
        linux_aarch64)  target="aarch64-unknown-linux-gnu";;
        macos_x86_64)   target="x86_64-apple-darwin";;
        macos_aarch64)  target="aarch64-apple-darwin";;
        windows_x86_64) target="x86_64-pc-windows-msvc";;
        *)
            error "Unsupported platform: ${os}_${arch}"
            exit 1
            ;;
    esac
    
    local url="https://github.com/${REPO}/releases/download/${version}/${BINARY_NAME}-${target}.tar.gz"
    
    info "Downloading ${BINARY_NAME} ${version} for ${target}..."
    
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$url" -o "$dest"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$url" -O "$dest"
    else
        error "Neither curl nor wget found. Please install one of them."
        exit 1
    fi
}

# Main installation
main() {
    info "Installing ${BINARY_NAME}..."
    
    # Detect platform
    OS=$(detect_os)
    ARCH=$(detect_arch)
    
    if [ "$OS" = "unknown" ] || [ "$ARCH" = "unknown" ]; then
        error "Unsupported platform: $(uname -s) $(uname -m)"
        error "Please build from source or download manually from GitHub releases."
        exit 1
    fi
    
    info "Detected platform: ${OS} ${ARCH}"
    
    # Get latest version
    VERSION=$(get_latest_version)
    if [ -z "$VERSION" ]; then
        error "Failed to get latest version. GitHub API may be rate limited."
        error "Please try again later or download manually from:"
        error "  https://github.com/${REPO}/releases"
        exit 1
    fi
    
    info "Latest version: ${VERSION}"
    
    # Create temp directory
    TMP_DIR=$(mktemp -d)
    trap "rm -rf $TMP_DIR" EXIT
    
    # Download
    ARCHIVE="${TMP_DIR}/${BINARY_NAME}.tar.gz"
    download_binary "$VERSION" "$OS" "$ARCH" "$ARCHIVE"
    
    # Extract
    info "Extracting..."
    tar -xzf "$ARCHIVE" -C "$TMP_DIR"
    
    # Install
    if [ ! -d "$INSTALL_DIR" ]; then
        info "Creating directory: ${INSTALL_DIR}"
        mkdir -p "$INSTALL_DIR"
    fi
    
    local binary_path="${TMP_DIR}/${BINARY_NAME}"
    if [ "$OS" = "windows" ]; then
        binary_path="${binary_path}.exe"
    fi
    
    info "Installing to: ${INSTALL_DIR}/${BINARY_NAME}"
    cp "$binary_path" "${INSTALL_DIR}/${BINARY_NAME}"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
    
    # Verify installation
    if command -v "$BINARY_NAME" >/dev/null 2>&1; then
        INSTALLED_VERSION=$($BINARY_NAME --version 2>/dev/null || echo "unknown")
        info "Successfully installed ${INSTALLED_VERSION}!"
    elif [ -f "${INSTALL_DIR}/${BINARY_NAME}" ]; then
        info "Successfully installed ${BINARY_NAME}!"
        warn "${INSTALL_DIR} is not in your PATH"
        warn "Add the following to your shell profile:"
        warn "  export PATH=\"${INSTALL_DIR}:\$PATH\""
    else
        error "Installation failed"
        exit 1
    fi
    
    # Post-install message
    echo ""
    info "To get started:"
    echo "  ${BINARY_NAME} --help"
    echo ""
    info "For more information, visit:"
    echo "  https://github.com/${REPO}"
}

# Run main function
main "$@"
