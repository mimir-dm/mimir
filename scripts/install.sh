#!/bin/sh
# Mimir Install Script
# Usage: curl -sSL https://raw.githubusercontent.com/mimir-dm/mimir/main/scripts/install.sh | sh
#
# Options:
#   --version X.Y.Z    Install specific version (default: latest)

set -e

REPO="mimir-dm/mimir"
GITHUB_API="https://api.github.com/repos/${REPO}"
GITHUB_RELEASES="https://github.com/${REPO}/releases"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info() {
    printf "${BLUE}[INFO]${NC} %s\n" "$1"
}

success() {
    printf "${GREEN}[OK]${NC} %s\n" "$1"
}

warn() {
    printf "${YELLOW}[WARN]${NC} %s\n" "$1"
}

error() {
    printf "${RED}[ERROR]${NC} %s\n" "$1" >&2
    exit 1
}

# Check for required tools
check_dependencies() {
    for cmd in curl tar; do
        if ! command -v "$cmd" >/dev/null 2>&1; then
            error "Required command '$cmd' not found. Please install it and try again."
        fi
    done
}

# Detect operating system
detect_os() {
    OS=$(uname -s)
    case "$OS" in
        Darwin)
            echo "darwin"
            ;;
        Linux)
            echo "linux"
            ;;
        *)
            error "Unsupported operating system: $OS"
            ;;
    esac
}

# Detect architecture
detect_arch() {
    ARCH=$(uname -m)
    case "$ARCH" in
        x86_64|amd64)
            echo "x64"
            ;;
        arm64|aarch64)
            echo "aarch64"
            ;;
        *)
            error "Unsupported architecture: $ARCH"
            ;;
    esac
}

# Get latest version from GitHub API
get_latest_version() {
    VERSION=$(curl -sL "${GITHUB_API}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"tag_name": *"app-v([^"]+)".*/\1/')
    if [ -z "$VERSION" ]; then
        error "Failed to determine latest version. Check your internet connection or specify --version"
    fi
    echo "$VERSION"
}

# Parse command line arguments
parse_args() {
    VERSION=""
    while [ $# -gt 0 ]; do
        case "$1" in
            --version)
                VERSION="$2"
                shift 2
                ;;
            --help|-h)
                echo "Mimir Install Script"
                echo ""
                echo "Usage: curl -sSL https://raw.githubusercontent.com/${REPO}/main/scripts/install.sh | sh"
                echo "       curl -sSL ... | sh -s -- --version X.Y.Z"
                echo ""
                echo "Options:"
                echo "  --version X.Y.Z    Install specific version (default: latest)"
                echo "  --help, -h         Show this help message"
                exit 0
                ;;
            *)
                warn "Unknown option: $1"
                shift
                ;;
        esac
    done
}

# Install on macOS
install_macos() {
    local version="$1"
    local arch="$2"
    local install_dir="$HOME/Applications"
    local app_name="Mimir.app"
    local tmp_dir=$(mktemp -d)

    # Prefer tar.gz (simpler, no mount needed)
    # Tauri produces: Mimir_aarch64.app.tar.gz or Mimir_x64.app.tar.gz
    local filename="Mimir_${arch}.app.tar.gz"
    local download_url="${GITHUB_RELEASES}/download/app-v${version}/${filename}"

    info "Downloading Mimir v${version} for macOS (${arch})..."
    info "URL: ${download_url}"

    if ! curl -fSL "${download_url}" -o "${tmp_dir}/mimir.tar.gz"; then
        rm -rf "$tmp_dir"
        error "Failed to download Mimir. URL: ${download_url}"
    fi

    # Extract tar.gz
    info "Extracting..."
    tar -xzf "${tmp_dir}/mimir.tar.gz" -C "${tmp_dir}"

    # Create install directory if needed
    mkdir -p "$install_dir"

    # Remove existing installation
    if [ -d "${install_dir}/${app_name}" ]; then
        info "Removing existing installation..."
        rm -rf "${install_dir}/${app_name}"
    fi

    # Move app to Applications
    info "Installing to ${install_dir}..."
    mv "${tmp_dir}/"*.app "${install_dir}/${app_name}" 2>/dev/null || \
    mv "${tmp_dir}/Mimir.app" "${install_dir}/${app_name}" 2>/dev/null || \
    error "Failed to find .app bundle in download"

    # Remove quarantine attribute (Gatekeeper)
    info "Removing Gatekeeper quarantine..."
    xattr -rd com.apple.quarantine "${install_dir}/${app_name}" 2>/dev/null || true

    # Cleanup
    rm -rf "$tmp_dir"

    success "Mimir v${version} installed to ${install_dir}/${app_name}"

    # Install mimir-mcp CLI tool
    install_mcp_cli "${install_dir}/${app_name}" "$version" "$arch"

    echo ""
    info "You can now open Mimir from:"
    info "  - Finder: ~/Applications/Mimir.app"
    info "  - Terminal: open ~/Applications/Mimir.app"
}

# Install mimir-mcp CLI for Claude Code/Desktop integration
install_mcp_cli() {
    local app_path="$1"
    local version="$2"
    local arch="$3"
    local bin_dir="$HOME/.local/bin"
    local mcp_binary="${app_path}/Contents/MacOS/mimir-mcp"
    local skills_source="${app_path}/Contents/Resources/skills"
    local skills_dest="$HOME/.claude/skills"

    # Check if mimir-mcp is bundled with the app
    if [ -f "$mcp_binary" ]; then
        info "Installing mimir-mcp CLI..."
        mkdir -p "$bin_dir"

        # Create symlink to the bundled binary
        ln -sf "$mcp_binary" "${bin_dir}/mimir-mcp"
        success "mimir-mcp installed to ${bin_dir}/mimir-mcp"

        # Check if bin_dir is in PATH
        case ":$PATH:" in
            *":$bin_dir:"*)
                ;;
            *)
                echo ""
                warn "${bin_dir} is not in your PATH"
                info "Add it to enable Claude Code integration:"
                info "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.zshrc"
                info "  source ~/.zshrc"
                ;;
        esac

        # Install Mimir skill for Claude Code
        if [ -d "$skills_source" ]; then
            info "Installing Mimir skill for Claude Code..."
            mkdir -p "$skills_dest"
            cp -r "$skills_source/mimir-campaign" "$skills_dest/"
            success "Mimir skill installed to ${skills_dest}/mimir-campaign"
        fi

        echo ""
        info "To connect Claude Code to Mimir, run:"
        info "  claude mcp add mimir -- mimir-mcp"
    else
        warn "mimir-mcp CLI not found in app bundle (optional feature)"
    fi
}

# Install on Linux
install_linux() {
    local version="$1"
    local arch="$2"

    # Tauri produces AppImage for Linux
    local filename="mimir_${version}_amd64.AppImage"
    local download_url="${GITHUB_RELEASES}/download/app-v${version}/${filename}"
    local install_dir="$HOME/.local/bin"

    info "Downloading Mimir v${version} for Linux..."

    mkdir -p "$install_dir"

    if ! curl -fSL "${download_url}" -o "${install_dir}/mimir"; then
        # Try alternative naming
        filename="Mimir_${version}_amd64.AppImage"
        download_url="${GITHUB_RELEASES}/download/app-v${version}/${filename}"
        if ! curl -fSL "${download_url}" -o "${install_dir}/mimir"; then
            error "Failed to download Mimir. URL: ${download_url}"
        fi
    fi

    chmod +x "${install_dir}/mimir"

    success "Mimir v${version} installed to ${install_dir}/mimir"

    # Check if install_dir is in PATH
    case ":$PATH:" in
        *":$install_dir:"*)
            info "Run 'mimir' to start the application"
            ;;
        *)
            echo ""
            warn "${install_dir} is not in your PATH"
            info "Add it to your shell config:"
            info "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc"
            info "  source ~/.bashrc"
            ;;
    esac
}

# Main
main() {
    echo ""
    echo "  __  __ _           _      "
    echo " |  \\/  (_)_ __ ___ (_)_ __ "
    echo " | |\\/| | | '_ \` _ \\| | '__|"
    echo " | |  | | | | | | | | | |   "
    echo " |_|  |_|_|_| |_| |_|_|_|   "
    echo ""
    echo " D&D Campaign Assistant"
    echo ""

    parse_args "$@"
    check_dependencies

    OS=$(detect_os)
    ARCH=$(detect_arch)

    if [ -z "$VERSION" ]; then
        info "Fetching latest version..."
        VERSION=$(get_latest_version)
    fi

    info "Installing Mimir v${VERSION} for ${OS}/${ARCH}"
    echo ""

    case "$OS" in
        darwin)
            install_macos "$VERSION" "$ARCH"
            ;;
        linux)
            install_linux "$VERSION" "$ARCH"
            ;;
    esac

    echo ""
    success "Installation complete!"
}

main "$@"
