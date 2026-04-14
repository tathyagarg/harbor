#!/usr/bin/env bash
set -e

ARCH=$(uname -m)
OS=$(uname -s)

case "$OS-$ARCH" in
"Darwin-arm64") TARGET=aarch64-apple-darwin ;;
"Darwin-x86_64") TARGET=x86_64-apple-darwin ;;
"Linux-x86_64") TARGET=x86_64-unknown-linux-musl ;;
"Linux-aarch64") TARGET=aarch64-unknown-linux-musl ;;
*)
  echo "Unsupported platform: $OS-$ARCH" >&2
  exit 1
  ;;
esac

VERSION=$(curl -s https://harbor.arson.dev/version | grep -o '"version":[^,}]*' | sed 's/"version":"\([^"]*\)"/\1/')
echo $VERSION

BASE="https://github.com/tathyagarg/harbor/releases/download/${VERSION}"
RES_URL="${BASE}/res.tar.gz"
HARBOR_URL="${BASE}/harbor_${TARGET}"

# Download and extract res
echo "Downloading res from $RES_URL..."
curl -L "$RES_URL" | tar xz

# Download harbor binary
echo "Downloading harbor binary from $HARBOR_URL..."
curl -L -o harbor "$HARBOR_URL"
