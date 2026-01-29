#!/bin/sh
# Contui installer script
# Usage: curl -LsSf https://raw.githubusercontent.com/aycandv/contui/main/scripts/install.sh | sh

set -e

# Configuration
REPO="aycandv/contui"
BINARY_NAME="contui"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Colors
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m' # No Color

# Spinner frames (braille dots)
SPINNER_FRAMES="⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏"

# Global spinner PID
SPINNER_PID=""

# Delay between phases (0.5 seconds)
delay() {
    sleep 0.5
}

# Start spinner with message
start_spinner() {
    local msg="$1"

    # Only run spinner if we have a terminal
    if [ -t 1 ]; then
        (
            while true; do
                for frame in $SPINNER_FRAMES; do
                    printf "\r  ${CYAN}%s${NC} %s" "$frame" "$msg"
                    sleep 0.08
                done
            done
        ) &
        SPINNER_PID=$!
    else
        printf "  ... %s\n" "$msg"
    fi
}

# Stop spinner with success message
stop_spinner_success() {
    local msg="$1"
    if [ -n "$SPINNER_PID" ]; then
        kill "$SPINNER_PID" 2>/dev/null || true
        wait "$SPINNER_PID" 2>/dev/null || true
        SPINNER_PID=""
    fi
    printf "\r  ${GREEN}✓${NC} ${GREEN}%s${NC}                              \n" "$msg"
}

# Stop spinner with error message
stop_spinner_error() {
    local msg="$1"
    if [ -n "$SPINNER_PID" ]; then
        kill "$SPINNER_PID" 2>/dev/null || true
        wait "$SPINNER_PID" 2>/dev/null || true
        SPINNER_PID=""
    fi
    printf "\r  ${RED}✗${NC} ${RED}%s${NC}                              \n" "$msg"
}

# Print success checkmark
print_success() {
    printf "  ${GREEN}✓${NC} ${GREEN}%s${NC}\n" "$1"
}

# Print error
print_error() {
    printf "\n  ${RED}${BOLD}✗ %s${NC}\n" "$1" >&2
    if [ -n "$2" ]; then
        printf "\n     %s\n" "$2" >&2
    fi
}

# Print ASCII banner
print_banner() {
    echo ""
    printf "       ${CYAN}██████╗ ██████╗ ███╗   ██╗${NC}\n"
    printf "      ${CYAN}██╔════╝██╔═══██╗████╗  ██║${NC}\n"
    printf "      ${CYAN}██║     ██║   ██║██╔██╗ ██║${NC}\n"
    printf "      ${CYAN}██║     ██║   ██║██║╚██╗██║${NC}\n"
    printf "      ${CYAN}╚██████╗╚██████╔╝██║ ╚████║${NC}\n"
    printf "       ${CYAN}╚═════╝ ╚═════╝ ╚═╝  ╚═══╝${NC}\n"
    printf "         ${BOLD}CONTUI INSTALLER${NC}\n"
    echo ""
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

# Get target triple
get_target() {
    local os="$1"
    local arch="$2"

    case "${os}_${arch}" in
        linux_x86_64)   echo "x86_64-unknown-linux-gnu";;
        linux_aarch64)  echo "aarch64-unknown-linux-gnu";;
        macos_x86_64)   echo "x86_64-apple-darwin";;
        macos_aarch64)  echo "aarch64-apple-darwin";;
        windows_x86_64) echo "x86_64-pc-windows-msvc";;
        *)              echo "unknown";;
    esac
}

# Get latest release version from GitHub
get_latest_version() {
    curl -s "https://api.github.com/repos/${REPO}/releases/latest" | \
        grep '"tag_name":' | \
        sed -E 's/.*"([^"]+)".*/\1/'
}

# Download binary with progress
download_binary() {
    local version="$1"
    local target="$2"
    local dest="$3"

    local url="https://github.com/${REPO}/releases/download/${version}/${BINARY_NAME}-${target}.tar.gz"

    if command -v curl >/dev/null 2>&1; then
        if [ -t 1 ]; then
            # Terminal: show progress bar
            curl -fSL --progress-bar "$url" -o "$dest"
        else
            # Non-terminal: silent
            curl -fsSL "$url" -o "$dest"
        fi
    elif command -v wget >/dev/null 2>&1; then
        if [ -t 1 ]; then
            wget --progress=bar:force "$url" -O "$dest" 2>&1
        else
            wget -q "$url" -O "$dest"
        fi
    else
        print_error "Neither curl nor wget found" "Please install curl or wget and try again."
        exit 1
    fi
}

# Cleanup on exit
cleanup() {
    if [ -n "$SPINNER_PID" ]; then
        kill "$SPINNER_PID" 2>/dev/null || true
    fi
    if [ -n "$TMP_DIR" ] && [ -d "$TMP_DIR" ]; then
        rm -rf "$TMP_DIR"
    fi
}

# Main installation
main() {
    trap cleanup EXIT

    print_banner

    # Phase 1: Detect platform
    start_spinner "Detecting platform..."
    OS=$(detect_os)
    ARCH=$(detect_arch)
    TARGET=$(get_target "$OS" "$ARCH")
    delay

    if [ "$TARGET" = "unknown" ]; then
        stop_spinner_error "Unsupported platform"
        print_error "Unsupported platform: $(uname -s) $(uname -m)" \
            "Please build from source or download manually from GitHub releases."
        exit 1
    fi

    stop_spinner_success "Detected: $OS $ARCH"
    delay

    # Phase 2: Fetch latest version
    start_spinner "Fetching latest version..."
    VERSION=$(get_latest_version)
    delay

    if [ -z "$VERSION" ]; then
        stop_spinner_error "Failed to fetch version"
        print_error "Failed to get latest version" \
            "GitHub API may be rate limited. Try again later or download from:\n     https://github.com/${REPO}/releases"
        exit 1
    fi

    stop_spinner_success "Latest version: $VERSION"
    delay

    # Phase 3: Download
    TMP_DIR=$(mktemp -d)
    ARCHIVE="${TMP_DIR}/${BINARY_NAME}.tar.gz"

    printf "  📥 ${CYAN}Downloading contui %s...${NC}\n" "$VERSION"

    if ! download_binary "$VERSION" "$TARGET" "$ARCHIVE"; then
        print_error "Download failed" \
            "Could not download from GitHub releases.\n     Check your network connection and try again."
        exit 1
    fi

    delay

    # Phase 4: Extract
    start_spinner "Extracting archive..."
    tar -xzf "$ARCHIVE" -C "$TMP_DIR" 2>/dev/null
    delay
    stop_spinner_success "Extracted"
    delay

    # Phase 5: Install
    start_spinner "Installing to ${INSTALL_DIR}..."

    if [ ! -d "$INSTALL_DIR" ]; then
        mkdir -p "$INSTALL_DIR"
    fi

    local binary_path="${TMP_DIR}/${BINARY_NAME}"
    if [ "$OS" = "windows" ]; then
        binary_path="${binary_path}.exe"
    fi

    cp "$binary_path" "${INSTALL_DIR}/${BINARY_NAME}"
    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
    delay
    stop_spinner_success "Installed"
    delay

    # Phase 6: Verify
    start_spinner "Verifying..."
    delay

    if [ -x "${INSTALL_DIR}/${BINARY_NAME}" ]; then
        INSTALLED_VERSION=$("${INSTALL_DIR}/${BINARY_NAME}" --version 2>/dev/null || echo "contui")
        stop_spinner_success "Verified: $INSTALLED_VERSION"
    else
        stop_spinner_error "Verification failed"
        print_error "Installation verification failed"
        exit 1
    fi

    # Success message
    echo ""
    printf "  ${GREEN}${BOLD}✅ contui installed successfully!${NC}\n"
    echo ""
    printf "  Get started:\n"
    printf "    ${CYAN}\$ contui${NC}              Launch TUI\n"
    printf "    ${CYAN}\$ contui --help${NC}       Show help\n"
    echo ""
    printf "  📚 Docs: %bhttps://github.com/%s%b\n" "$CYAN" "$REPO" "$NC"
    echo ""

    # PATH warning if needed
    if ! command -v "$BINARY_NAME" >/dev/null 2>&1; then
        printf "  ${YELLOW}⚠${NC}  ${DIM}%s is not in your PATH${NC}\n" "$INSTALL_DIR"
        printf "     ${DIM}Add to your shell profile:${NC}\n"
        printf "     ${DIM}export PATH=\"%s:\$PATH\"${NC}\n\n" "$INSTALL_DIR"
    fi
}

# Run main function
main "$@"
