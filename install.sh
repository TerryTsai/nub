#!/bin/sh
# nub installer.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/TerryTsai/nub/main/install.sh | sh
#
# Env overrides:
#   NUB_VERSION=v0.0.1            # pin a specific release (default: latest)
#   NUB_PREFIX=/usr/local/bin     # install destination (default: $HOME/.local/bin
#                                  for normal users, /usr/local/bin for root)
#   NUB_SERVICE=user|system|none  # systemd unit setup (default: auto — system
#                                  if running as root, else user)
#
# Sudo is not used unless the destination needs it (e.g. NUB_PREFIX is
# /usr/local/bin and you're not root) or you opt into a system-level
# systemd unit. Single sudo prompt is left for `loginctl enable-linger`,
# which is needed for nub to survive logout — opportunistic; skipped
# with a warning if sudo isn't available.

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
  # Default destination: $HOME/.local/bin for normal users, /usr/local/bin for root.
  # Personal-tool shape — sudo isn't asked for unless the operator explicitly
  # opts in by setting NUB_PREFIX=/usr/local/bin (or running the script as root).
  if [ -n "${NUB_PREFIX:-}" ]; then
    PREFIX="${NUB_PREFIX}"
  elif [ "$(id -u)" = "0" ]; then
    PREFIX="/usr/local/bin"
  else
    PREFIX="${HOME}/.local/bin"
    mkdir -p "${PREFIX}"
  fi

  if [ -w "${PREFIX}" ] || [ "$(id -u)" = "0" ]; then
    SUDO=""
  elif command -v sudo >/dev/null 2>&1; then
    SUDO="sudo"
    info "writing to ${PREFIX} requires sudo"
  else
    err "${PREFIX} is not writable and sudo isn't available; try NUB_PREFIX=\$HOME/.local/bin"
  fi
}

install_binary() {
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
}

# Ensure a starter config exists by delegating to nub itself.
seed_config() {
  cfg_dir="${XDG_CONFIG_HOME:-$HOME/.config}/nub"
  if [ ! -f "${cfg_dir}/nub.toml" ]; then
    info "generating starter config via 'nub init'"
    "${PREFIX}/nub" init >/dev/null
  fi
}

choose_service_level() {
  if ! command -v systemctl >/dev/null 2>&1; then
    SERVICE_LEVEL="none"
    info "systemctl not found; skipping daemon setup"
    return
  fi
  case "${NUB_SERVICE:-auto}" in
    none|user|system) SERVICE_LEVEL="${NUB_SERVICE}" ;;
    auto)
      if [ "$(id -u)" = "0" ]; then
        SERVICE_LEVEL="system"
      else
        SERVICE_LEVEL="user"
      fi
      ;;
    *) err "NUB_SERVICE must be one of: user, system, none, auto" ;;
  esac
}

setup_systemd_user() {
  if command -v loginctl >/dev/null 2>&1; then
    linger=$(loginctl show-user "${USER}" -p Linger 2>/dev/null | cut -d= -f2 || true)
    if [ "${linger}" != "yes" ]; then
      info "enabling user lingering (so nub starts at boot, not login)"
      sudo_cmd=""
      if [ "$(id -u)" != "0" ] && command -v sudo >/dev/null 2>&1; then
        sudo_cmd="sudo"
      fi
      ${sudo_cmd} loginctl enable-linger "${USER}" || \
        info "warning: enable-linger failed; nub will only run while you're logged in"
    fi
  fi

  "${PREFIX}/nub" install systemd --user
}

setup_systemd_system() {
  if [ "$(id -u)" = "0" ]; then
    "${PREFIX}/nub" install systemd --system
  elif command -v sudo >/dev/null 2>&1; then
    sudo "${PREFIX}/nub" install systemd --system
  else
    err "system-level install needs root or sudo"
  fi
}

main() {
  need curl
  need tar
  need uname

  detect_target
  resolve_version
  choose_prefix
  install_binary
  seed_config
  choose_service_level

  case "${SERVICE_LEVEL}" in
    user)   setup_systemd_user ;;
    system) setup_systemd_system ;;
    none)
      info ""
      info "installed: ${PREFIX}/nub"
      info "to run: ${PREFIX}/nub"
      info "to set up as a service later: ${PREFIX}/nub install systemd --user"
      ;;
  esac

  case ":${PATH}:" in
    *:"${PREFIX}":*) ;;
    *) info "warning: ${PREFIX} is not in PATH; add it or use the full path" ;;
  esac
}

main "$@"
