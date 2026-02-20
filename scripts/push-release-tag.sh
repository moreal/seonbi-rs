#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

log() {
  echo "[push-release-tag] $*"
}

usage() {
  cat <<EOF
Usage: $(basename "$0") [OPTIONS] TAG

Create and push an annotated git tag with message "Release TAG".

Arguments:
  TAG                  Tag name to create and push (e.g. v0.1.0-alpha.3)

Options:
  -r, --remote NAME    Remote to push to (default: origin)
  -m, --message TEXT   Tag message (default: "Release TAG")
  -n, --dry-run        Print commands without executing
  -h, --help           Show this help
EOF
}

REMOTE="origin"
MESSAGE=""
DRY_RUN=false
TAG=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    -h|--help)
      usage
      exit 0
      ;;
    -r|--remote)
      if [[ $# -lt 2 ]]; then
        echo "missing value for $1" >&2
        exit 1
      fi
      REMOTE="$2"
      shift 2
      ;;
    -m|--message)
      if [[ $# -lt 2 ]]; then
        echo "missing value for $1" >&2
        exit 1
      fi
      MESSAGE="$2"
      shift 2
      ;;
    -n|--dry-run)
      DRY_RUN=true
      shift
      ;;
    -*)
      echo "unknown option: $1" >&2
      echo "run with --help for usage" >&2
      exit 1
      ;;
    *)
      if [[ -n "${TAG}" ]]; then
        echo "tag is already set to ${TAG}" >&2
        echo "run with --help for usage" >&2
        exit 1
      fi
      TAG="$1"
      shift
      ;;
  esac
done

if [[ -z "${TAG}" ]]; then
  echo "tag is required" >&2
  echo "run with --help for usage" >&2
  exit 1
fi

if [[ -z "${MESSAGE}" ]]; then
  MESSAGE="Release ${TAG}"
fi

log "repository: ${ROOT_DIR}"
log "tag: ${TAG}"
log "message: ${MESSAGE}"
log "remote: ${REMOTE}"
log "checking whether tag exists locally..."
if git rev-parse -q --verify "refs/tags/${TAG}" >/dev/null; then
  echo "tag already exists locally: ${TAG}" >&2
  exit 1
fi

log "checking whether tag exists on remote..."
if git ls-remote --exit-code --tags "${REMOTE}" "refs/tags/${TAG}" >/dev/null 2>&1; then
  echo "tag already exists on remote ${REMOTE}: ${TAG}" >&2
  exit 1
fi

if ${DRY_RUN}; then
  log "dry-run mode enabled; no changes will be made."
  echo "[dry-run] git tag -a \"${TAG}\" -m \"${MESSAGE}\""
  echo "[dry-run] git push \"${REMOTE}\" \"${TAG}\""
  exit 0
fi

log "creating annotated tag..."
git tag -a "${TAG}" -m "${MESSAGE}"
log "pushing tag to remote..."
git push "${REMOTE}" "${TAG}"

log "created and pushed tag: ${TAG}"
