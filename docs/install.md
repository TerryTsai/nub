# Install

The README's one-liner covers the common case. This page covers everything
else: source builds, system-wide installs, env overrides, upgrade paths,
manual systemd unit refresh, uninstall.

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
session ending — see below.

**`enable-linger` matters.** Without lingering, your user-level systemd
unit stops the moment you log out. The installer asks for sudo once to
run `loginctl enable-linger $USER` so nub keeps running on a remote box
you ssh into. If sudo isn't available, the installer warns and skips —
you can re-run `sudo loginctl enable-linger $USER` any time afterward.

The binary uses rustls (not OpenSSL) and is statically linked against
musl on Linux — works on any glibc/musl distro.

## Env overrides

| Variable | Default | Purpose |
|---|---|---|
| `NUB_VERSION` | latest | Pin a specific release (e.g. `v0.0.62`) |
| `NUB_PREFIX` | `$HOME/.local/bin` (root → `/usr/local/bin`) | Install destination |
| `NUB_SERVICE` | auto (`user` if unprivileged, `system` if root) | Force `user`, `system`, or `none` |

`NUB_SERVICE=none` installs the binary only — no systemd unit. Run
`nub run` directly or wrap it in your own service manager.

## Upgrade

For an existing install, prefer the in-binary updater:

```sh
nub update          # download latest, swap atomically, restart
nub update --check  # show "current vs latest" without changing anything
nub update --version v0.0.55   # pin
```

`nub update` requires `curl` and `tar` in PATH and write access to the
binary's directory. It restarts the systemd unit when one is present.

For unusual destinations or a fresh re-install, re-run the install
script. It replaces the binary and `systemctl restart` picks up the
new version.

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

House rules the build enforces (don't pretend they're suggestions):

- 250 lines per file (`build.rs` fails the build over)
- `cargo fmt` and `cargo clippy` clean (strict lints)
- `unsafe` is forbidden
- New deps need justification — the stack is locked

## Refresh the systemd unit

For an existing install, regenerate the unit any time without re-running
the installer:

```sh
nub install systemd --user --print > ~/.config/systemd/user/nub.service && \
  systemctl --user daemon-reload && \
  systemctl --user restart nub
```

(Use `--system` for `/etc/systemd/system/nub.service`.)

## Uninstall

```sh
nub uninstall          # prompts; --yes to skip
```

Removes `$XDG_CONFIG_HOME/nub`, `$XDG_DATA_HOME/nub`, and any nub
systemd unit nub itself wrote (stopping/disabling first). The binary
itself stays put — `rm $(which nub)` to also remove that.

## macOS / BSD

Not built for macOS today. The Rust code is portable; `nub install
systemd` is Linux-only by definition. Run `nub run` directly under
launchd or as a foreground process while platform support catches up.
