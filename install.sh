#!/bin/sh
# nub installer.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/TerryTsai/nub/main/install.sh | sh
#
# Env overrides:
#   NUB_VERSION=v0.0.1   # pin a specific release (default: latest)
#   NUB_PREFIX=$HOME/.local/bin   # install destination (default: /usr/local/bin
#                                  with sudo fallback, else $HOME/.local/bin)

set -eu

REPO="TerryTsai/nub"

err() { printf 'error: %s\n' "$1" >&2; exit 1; }
info() { printf '%s\n' "$1"; }

need() {
  command -v "$1" >/dev/null 2>&1 || err "$1 not found in PATH"
}

detect_target() {
  os=$(uname -s | tr '[:upper:]' '[:lower:]')
  arch=$(uname -m)
  case "${os}/${arch}" in
    linux/x86_64)              TARGET=x86_64-unknown-linux-musl ;;
    linux/aarch64|linux/arm64) TARGET=aarch64-unknown-linux-musl ;;
    darwin/*) err "macOS binaries not built yet; build from source: https://github.com/${REPO}#install" ;;
    *) err "unsupported platform: ${os}/${arch}" ;;
  esac
}

resolve_version() {
  if [ -n "${NUB_VERSION:-}" ]; then
    VERSION="${NUB_VERSION}"
    return
  fi
  VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | sed -nE 's/.*"tag_name":[[:space:]]*"([^"]+)".*/\1/p' | head -n1)
  [ -n "${VERSION}" ] || err "could not resolve latest release; set NUB_VERSION"
}

choose_prefix() {
  if [ -n "${NUB_PREFIX:-}" ]; then
    PREFIX="${NUB_PREFIX}"
    SUDO=""
    return
  fi
  if [ -w /usr/local/bin ] || [ "$(id -u)" = "0" ]; then
    PREFIX="/usr/local/bin"
    SUDO=""
  elif command -v sudo >/dev/null 2>&1; then
    PREFIX="/usr/local/bin"
    SUDO="sudo"
  else
    PREFIX="${HOME}/.local/bin"
    mkdir -p "${PREFIX}"
    SUDO=""
  fi
}

main() {
  need curl
  need tar
  need uname

  detect_target
  resolve_version
  choose_prefix

  base="nub-${VERSION}-${TARGET}"
  url="https://github.com/${REPO}/releases/download/${VERSION}/${base}.tar.gz"

  info "installing nub ${VERSION} (${TARGET}) to ${PREFIX}"

  tmp=$(mktemp -d)
  trap 'rm -rf "${tmp}"' EXIT

  curl -fsSL "${url}" | tar -xz -C "${tmp}" \
    || err "download or extract failed: ${url}"

  if [ -n "${SUDO}" ]; then
    ${SUDO} install -m 0755 "${tmp}/nub" "${PREFIX}/nub"
  else
    install -m 0755 "${tmp}/nub" "${PREFIX}/nub"
  fi

  case ":${PATH}:" in
    *:"${PREFIX}":*) ;;
    *) info "warning: ${PREFIX} is not in PATH; add it or use the full path" ;;
  esac

  info ""
  info "installed: ${PREFIX}/nub"
  info "try: nub --id host1 --bind 127.0.0.1:8080"
}

main "$@"
