# Install

The README's one-liner covers the common case. This page covers everything
else: source builds, system-wide installs, env overrides, manual systemd
unit refresh.

## Quick install

```sh
curl -fsSL https://raw.githubusercontent.com/TerryTsai/nub/main/install.sh | sh
```

The script downloads the binary, runs `nub init` to drop a starter config
in `$XDG_CONFIG_HOME/nub/nub.toml`, and (when systemd is present) installs
a unit file, enables lingering, and starts nub as a managed service.

**Defaults.** Installs to `$HOME/.local/bin/nub`, runs as the invoking user
with a user-level unit (`~/.config/systemd/user/nub.service`). When invoked
as root: installs to `/usr/local/bin/nub` and uses a system-level unit
(`/etc/systemd/system/nub.service`).

**Sudo behavior.** The script doesn't ask for sudo unless you explicitly
opt into a system destination (`NUB_PREFIX=/usr/local/bin`) or system-level
systemd (`NUB_SERVICE=system`). One opportunistic sudo prompt is left in
the user-level path: `loginctl enable-linger` so nub survives your SSH
session ending — skipped with a warning if sudo isn't available.

**Upgrade.** Re-run the script. It replaces the binary and `systemctl
restart` picks up the new version. Plain binary install with no daemon
setup: `NUB_SERVICE=none curl … | sh`.

The binary uses rustls (not OpenSSL) and is statically linked against
musl on Linux — works on any glibc/musl distro.

## Env overrides

| Variable | Default | Purpose |
|---|---|---|
| `NUB_VERSION` | latest | Pin a specific release (e.g. `v0.0.27`) |
| `NUB_PREFIX` | `$HOME/.local/bin` | Install destination |
| `NUB_SERVICE` | auto (`user` if unprivileged, `system` if root) | Force `user`, `system`, or `none` |

## From source (development)

```sh
git clone https://github.com/TerryTsai/nub
cd nub
cd ui && npm install && npm run build && cd ..
cargo build --release --features embed-ui
sudo install -m 0755 target/release/nub /usr/local/bin/nub
```

Drop `--features embed-ui` (and the `npm` step) if you don't want the UI
baked in — the binary is API-only and any web server can serve `ui/dist`.

## Refresh the systemd unit

For an existing install, regenerate the unit any time without re-running
the installer:

```sh
nub install systemd --user --print > ~/.config/systemd/user/nub.service && \
  systemctl --user daemon-reload && \
  systemctl --user restart nub
```

(Use `--system` for `/etc/systemd/system/nub.service`.)

## macOS / BSD

Not built for macOS today. The Rust code is portable; `nub install
systemd` is Linux-only by definition. Run `nub run` directly under
launchd or as a foreground process while platform support catches up.
