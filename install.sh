#!/usr/bin/env bash
# Rotiv installer — downloads the latest release binary and installs it to /usr/local/bin
# Usage: curl -fsSL https://github.com/rotiv-dev/rotiv/releases/latest/download/install.sh | bash

set -euo pipefail

REPO="rotiv-dev/rotiv"
INSTALL_DIR="${ROTIV_INSTALL_DIR:-/usr/local/bin}"
BINARY_NAME="rotiv"

# --- Platform detection ---
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64) ARTIFACT="rotiv-linux-x86_64" ;;
      aarch64|arm64) ARTIFACT="rotiv-linux-aarch64" ;;
      *)
        echo "Unsupported architecture: $ARCH" >&2
        exit 1
        ;;
    esac
    ;;
  Darwin)
    case "$ARCH" in
      arm64) ARTIFACT="rotiv-macos-arm64" ;;
      x86_64) ARTIFACT="rotiv-macos-x86_64" ;;
      *)
        echo "Unsupported architecture: $ARCH" >&2
        exit 1
        ;;
    esac
    ;;
  *)
    echo "Unsupported OS: $OS" >&2
    echo "For Windows, download rotiv-windows-x64.exe from:" >&2
    echo "  https://github.com/${REPO}/releases/latest" >&2
    exit 1
    ;;
esac

# --- Resolve latest release URL ---
DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${ARTIFACT}"

echo "Rotiv installer"
echo "  platform: ${OS}/${ARCH}"
echo "  artifact: ${ARTIFACT}"
echo "  destination: ${INSTALL_DIR}/${BINARY_NAME}"
echo ""

# --- Download ---
TMP_FILE="$(mktemp)"
trap 'rm -f "$TMP_FILE"' EXIT

echo "Downloading ${DOWNLOAD_URL} ..."
if command -v curl &>/dev/null; then
  curl -fsSL --progress-bar "$DOWNLOAD_URL" -o "$TMP_FILE"
elif command -v wget &>/dev/null; then
  wget -q --show-progress "$DOWNLOAD_URL" -O "$TMP_FILE"
else
  echo "Error: curl or wget is required" >&2
  exit 1
fi

# --- Install ---
chmod +x "$TMP_FILE"

if [ -w "$INSTALL_DIR" ]; then
  mv "$TMP_FILE" "${INSTALL_DIR}/${BINARY_NAME}"
else
  echo "Installing to ${INSTALL_DIR} requires sudo..."
  sudo mv "$TMP_FILE" "${INSTALL_DIR}/${BINARY_NAME}"
fi

# --- Verify ---
if command -v rotiv &>/dev/null; then
  VERSION="$(rotiv --version 2>/dev/null || echo 'unknown')"
  echo ""
  echo "✓ Rotiv installed successfully: ${VERSION}"
  echo "  Run 'rotiv new myapp' to create your first project."
else
  echo ""
  echo "✓ Installed to ${INSTALL_DIR}/${BINARY_NAME}"
  echo "  Make sure ${INSTALL_DIR} is in your PATH."
fi
