#!/bin/bash
set -e

REPO="4shil/agent-sandbox"
BINARY="abox"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Detect OS and arch
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$ARCH" in
    x86_64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="aarch64" ;;
    *) echo "❌ Unsupported architecture: $ARCH"; exit 1 ;;
esac

case "$OS" in
    linux) OS="unknown-linux-gnu" ;;
    darwin) OS="apple-darwin" ;;
    *) echo "❌ Unsupported OS: $OS"; exit 1 ;;
esac

TARGET="${ARCH}-${OS}"

echo "🛡️  Installing abox..."
echo "   OS: $OS"
echo "   Arch: $ARCH"
echo "   Target: $TARGET"

# Get latest release tag (with Accept header to avoid API issues)
LATEST=$(curl -s -H "Accept: application/vnd.github.v3+json" \
    "https://api.github.com/repos/${REPO}/releases/latest" | \
    grep '"tag_name":' | head -1 | sed -E 's/.*"([^"]+)".*/\1/')

# Fallback: try fetching from GitHub tags if releases fails
if [ -z "$LATEST" ]; then
    LATEST=$(curl -s "https://api.github.com/repos/${REPO}/tags" | \
        grep '"name":' | head -1 | sed -E 's/.*"([^"]+)".*/\1/')
fi

if [ -z "$LATEST" ]; then
    echo "⚠️  Could not determine latest release, using v0.3.0"
    LATEST="v0.3.0"
fi

echo "   Version: $LATEST"

# Download URL
URL="https://github.com/${REPO}/releases/download/${LATEST}/${BINARY}-${TARGET}.tar.gz"

echo "📥 Downloading from $URL..."

# Download and extract
TMP_DIR=$(mktemp -d)
curl -sL "$URL" -o "${TMP_DIR}/${BINARY}.tar.gz"
tar -xzf "${TMP_DIR}/${BINARY}.tar.gz" -C "$TMP_DIR"

# Install
if [ -w "$INSTALL_DIR" ]; then
    mv "${TMP_DIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
else
    sudo mv "${TMP_DIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
fi

chmod +x "${INSTALL_DIR}/${BINARY}"

# Cleanup
rm -rf "$TMP_DIR"

echo "✅ Installed to ${INSTALL_DIR}/${BINARY}"
echo ""
echo "Quick start:"
echo "  $ abox init my-project"
echo "  $ abox run --agent claude \"Build a todo API\""
echo "  $ abox replay my-project"
