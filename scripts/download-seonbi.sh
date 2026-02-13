#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ARCHIVE_DIR="${ROOT_DIR}/.tools/original"
DEFAULT_VERSION="0.5.0"

usage() {
  cat <<EOF
Usage: $(basename "$0") [VERSION]
       $(basename "$0") --version VERSION

Download a seonbi release binary for the current platform.

Arguments:
  VERSION             Release version (default: ${DEFAULT_VERSION})

Options:
  -v, --version VER   Release version
  -h, --help          Show this help
EOF
}

VERSION="${DEFAULT_VERSION}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    -h|--help)
      usage
      exit 0
      ;;
    -v|--version)
      if [[ $# -lt 2 ]]; then
        echo "missing value for $1" >&2
        exit 1
      fi
      VERSION="$2"
      shift 2
      ;;
    -*)
      echo "unknown option: $1" >&2
      echo "run with --help for usage" >&2
      exit 1
      ;;
    *)
      if [[ "${VERSION}" != "${DEFAULT_VERSION}" ]]; then
        echo "version already provided: ${VERSION}" >&2
        echo "run with --help for usage" >&2
        exit 1
      fi
      VERSION="$1"
      shift
      ;;
  esac
done

VERSION="${VERSION#v}"
if [[ -z "${VERSION}" ]]; then
  echo "version must not be empty" >&2
  exit 1
fi

DEST_DIR="${ROOT_DIR}/.tools/original/seonbi-${VERSION}"

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
