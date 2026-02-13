#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEST_DIR="${ROOT_DIR}/.tools/original/seonbi-0.5.0"
ARCHIVE_DIR="${ROOT_DIR}/.tools/original"
VERSION="0.5.0"

os="$(uname -s)"
arch="$(uname -m)"

case "${os}/${arch}" in
  Darwin/arm64) asset="seonbi-${VERSION}.macos-arm64.tar.bz2" ;;
  Darwin/x86_64) asset="seonbi-${VERSION}.macos-x86_64.tar.bz2" ;;
  Linux/aarch64) asset="seonbi-${VERSION}.linux-arm64.tar.bz2" ;;
  Linux/x86_64) asset="seonbi-${VERSION}.linux-x86_64.tar.bz2" ;;
  *)
    echo "Unsupported platform: ${os}/${arch}" >&2
    exit 1
    ;;
esac

url="https://github.com/dahlia/seonbi/releases/download/${VERSION}/${asset}"
archive_path="${ARCHIVE_DIR}/${asset}"

mkdir -p "${ARCHIVE_DIR}"
rm -rf "${DEST_DIR}"
mkdir -p "${DEST_DIR}"

echo "Downloading ${url}"
curl -sSfL -o "${archive_path}" "${url}"
tar -xjf "${archive_path}" -C "${DEST_DIR}"
chmod +x "${DEST_DIR}/seonbi" "${DEST_DIR}/seonbi-api" || true

echo "Downloaded original binaries:"
echo "  ${DEST_DIR}/seonbi"
echo "  ${DEST_DIR}/seonbi-api"
echo
echo "Use it for comparative E2E:"
echo "  export SEONBI_ORIGINAL_BIN=\"${DEST_DIR}/seonbi\""
echo "  cargo test -p seonbi-cli --test e2e_compare"
