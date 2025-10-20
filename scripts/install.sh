#!/usr/bin/env bash
# Installation script for qnote
# Usage: curl -sSL https://raw.githubusercontent.com/adi-sen/qnote/master/scripts/install.sh | bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
REPO="adi-sen/qnote"
BINARY_NAME="qnote"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Detect OS and architecture
detect_platform() {
    local os arch

    # Detect OS
    case "$(uname -s)" in
        Linux*)     os="Linux" ;;
        Darwin*)    os="Darwin" ;;
        MINGW*|MSYS*|CYGWIN*)
            echo -e "${RED}Error: Windows installation via this script is not supported${NC}"
            echo "Please download the .exe from https://github.com/${REPO}/releases"
            exit 1
            ;;
        *)
            echo -e "${RED}Error: Unsupported operating system${NC}"
            exit 1
            ;;
    esac

    # Detect architecture
    case "$(uname -m)" in
        x86_64|amd64)   arch="x86_64" ;;
        aarch64|arm64)  arch="aarch64" ;;
        armv7l)         arch="armv7" ;;
        *)
            echo -e "${RED}Error: Unsupported architecture: $(uname -m)${NC}"
            exit 1
            ;;
    esac

    echo "${os}-${arch}"
}

# Get latest release version from GitHub API
get_latest_version() {
    local version
    version=$(curl -sSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

    if [ -z "$version" ]; then
        echo -e "${RED}Error: Could not determine latest version${NC}"
        exit 1
    fi

    echo "$version"
}

# Download and install binary
install_binary() {
    local platform version download_url temp_dir

    platform=$(detect_platform)
    version=$(get_latest_version)

    echo -e "${GREEN}Installing qnote ${version} for ${platform}${NC}"

    # Construct download URL
    download_url="https://github.com/${REPO}/releases/download/${version}/${BINARY_NAME}-${platform}"

    # Create temporary directory
    temp_dir=$(mktemp -d)
    trap 'rm -rf "$temp_dir"' EXIT

    # Download binary
    echo "Downloading from ${download_url}..."
    if ! curl -sSL -f "$download_url" -o "${temp_dir}/${BINARY_NAME}"; then
        echo -e "${RED}Error: Failed to download binary${NC}"
        echo "URL: ${download_url}"
        echo -e "${YELLOW}This platform might not have pre-built binaries.${NC}"
        echo "Try installing from source: cargo install qnote"
        exit 1
    fi

    # Make binary executable
    chmod +x "${temp_dir}/${BINARY_NAME}"

    # Create install directory if it doesn't exist
    mkdir -p "$INSTALL_DIR"

    # Move binary to install directory
    if mv "${temp_dir}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"; then
        echo -e "${GREEN}Successfully installed to ${INSTALL_DIR}/${BINARY_NAME}${NC}"
    else
        echo -e "${RED}Error: Failed to install binary${NC}"
        exit 1
    fi

    # Check if install directory is in PATH
    if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
        echo ""
        echo -e "${YELLOW}Warning: ${INSTALL_DIR} is not in your PATH${NC}"
        echo "Add this to your shell profile (.bashrc, .zshrc, etc.):"
        echo "  export PATH=\"\$PATH:${INSTALL_DIR}\""
    fi

    echo ""
    echo -e "${GREEN}Installation complete!${NC}"
    echo "Run 'qnote --help' to get started"
}

# Main installation flow
main() {
    echo "qnote installer"
    echo "==============="
    echo ""

    # Check for required commands
    for cmd in curl grep sed; do
        if ! command -v "$cmd" &> /dev/null; then
            echo -e "${RED}Error: Required command '$cmd' not found${NC}"
            exit 1
        fi
    done

    install_binary
}

main "$@"
