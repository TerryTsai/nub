# nub

[![release](https://img.shields.io/github/actions/workflow/status/TerryTsai/nub/release.yml?label=release)](https://github.com/TerryTsai/nub/actions/workflows/release.yml)
[![ci](https://img.shields.io/github/actions/workflow/status/TerryTsai/nub/ci.yml?label=ci&branch=main)](https://github.com/TerryTsai/nub/actions/workflows/ci.yml)
[![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#license)

A control plane for one container host.

Runs on your box. Drives from any browser — phone, tablet, desktop —
or the API directly. Manages containers, stacks, and secrets without
making you reach for `docker run`.

## What you do with it

**See what's running.** A clean list with the state, status, and health
you actually want — no 30-field JSON dumps. Tap a row for full detail.
Tap a stack to see every container under it.

**Deploy stacks by paste.** Drop your `compose.yml` into the deploy box
and hit go. Edit and redeploy from the same screen. nub deploys the
parts of compose you'd actually trust on a homelab and tells you
clearly when it won't deploy the parts you wouldn't.

**Watch things live.** Logs, stats, and pull progress stream as they
happen. Open a shell in any container without SSHing the host first.

**Stash secrets without thinking.** Add a value once on the host,
reference it by name from any stack. Encrypted at rest with age.
Decrypted only while a stack uses it. Non-admin tokens can write
secrets but can't read them back over the network.

**Hand out limited access.** Mint scoped tokens — `admin`, `operator`,
`deploy`, `readonly`, or any explicit scope list. Each token says
exactly what it can and can't do. Rotate by deleting it.

**One host on purpose.** No fleet, no cluster, no orchestrator. Run
nub on each box you have and switch between them in the UI. The
single-host scope is what lets it stay simple.

**Stays out of your way.** ~5 MB binary, single-digit MiB RAM at idle,
zero idle CPU.[^1] Works against Docker or Podman. Doesn't fight your
other services for resources.

[^1]: Measured on Linux x86_64 musl, 0.0.62, idle. Your numbers will
vary with workload.

## Install

```sh
curl -fsSL https://raw.githubusercontent.com/TerryTsai/nub/main/install.sh | sh
```

Installs `~/.local/bin/nub`, sets up a user-level systemd unit, and
prints a connect URL. Source builds, system-wide installs, and env
overrides (`NUB_VERSION`, `NUB_PREFIX`, `NUB_SERVICE`):
[docs/install.md](docs/install.md).

**First five minutes:** [docs/quickstart.md](docs/quickstart.md) walks
you from fresh install to first stack deployed.

## CLI

`nub` manages itself. Container ops happen in the UI.

| Surface | Commands |
|---|---|
| **Lifecycle** | `nub run` · `nub restart` · `nub update` |
| **Setup** | `nub init` · `nub install systemd` · `nub uninstall` |
| **Inspect** | `nub status` · `nub config show` |
| **Connect** | `nub url` · `nub qr` |
| **Stacks** | `nub stack deploy\|ls\|rm\|redeploy\|logs` |
| **Secrets** | `nub secret put\|ls\|rm\|get` |
| **Tokens** | `nub token mint --preset {admin\|operator\|deploy\|readonly}` · `nub token scopes` |
| **Keys** | `nub key gen\|rotate` |
| **Bind allowlist** | `nub bind list\|allow\|deny` |

Bare `nub <noun>` prints the noun's verb list. Per-command help:
`nub <cmd> --help`. Shell completion: `nub completions {bash,zsh,fish}`.

## Usage

Once the daemon is running, open the connect URL on any device. Print
it any time:

```sh
nub url      # text URL
nub qr       # scannable QR code
```

Paste-and-deploy a stack from a terminal:

```sh
cat > compose.yml <<'EOF'
services:
  web:
    image: nginx:alpine
    ports:
      - "8080:80"
    restart: unless-stopped
EOF

nub stack deploy myapp ./compose.yml
nub stack logs myapp --follow
```

Compose feature support (what's translated, flagged, or rejected):
[docs/compose.md](docs/compose.md).

## API

The CLI uses a stable JSON+WebSocket API; you can hit it directly from
scripts or CI. Get a token from `nub url` (the `#t=...` fragment) or
mint one with `nub token mint`.

```sh
TOKEN=$(awk -F'#t=' '/#t=/{print $2}' <<<"$(nub url)")

curl http://127.0.0.1:8080/api/op \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"op":"list_containers","all":true}'
```

Streaming ops (logs, stats, exec, image pulls) run over `ws://.../api/ws`.
Frame format and full op catalog: [docs/api.md](docs/api.md).

## Configuration

nub reads `nub.toml` from the first match of: `--config <path>`,
`$XDG_CONFIG_HOME/nub/nub.toml`, `./nub.toml`, `/etc/nub/config.toml`.
With no file, sane defaults apply (`listen = 0.0.0.0:8080`,
hostname-derived `id`).

Common fields: `id`, `listen`, `tls_cert`/`tls_key`, `allowed_binds`,
`trusted_issuer`. Generate a starter file: `nub init`. Full schema:
[docs/configuration.md](docs/configuration.md).

## Authentication

nub uses Ed25519-signed JWTs as bearer tokens. Each token's `scope`
claim is a space-separated list of `<resource>:<action>` strings —
scopes are granular and never all-or-nothing. The presets:

| Preset | Grants |
|---|---|
| `admin` | `*` (everything) |
| `operator` | Day-to-day operations: list/get/logs/stats; create/start/stop/restart/remove/exec containers; pull/delete images; create/delete volumes and networks; deploy/redeploy/update/delete/logs/pull stacks; secrets put/list/delete |
| `deploy` | Stack delivery from CI: stack lifecycle plus the composing sub-ops (`images:pull`, `networks:*`, `volumes:*`, `containers:create`/`start`/`stop`/`remove`). No exec, no secret writes |
| `readonly` | `:list` and `:get` across every resource |

```sh
nub token mint --sub me   --preset admin
nub token mint --sub box  --preset operator
nub token mint --sub ci   --preset deploy
nub token mint --sub mon  --preset readonly
nub token mint --sub fine --scope containers:list,stacks:get
nub token scopes                         # full vocabulary
```

Use a minted token from a script: `Authorization: Bearer <token>`.
Use it from a browser: open the connect URL nub printed at startup
(or `nub url`/`nub qr`) — the token is in the `#t=` fragment, the UI
strips it from history once paired.

`secrets:reveal` (reading a secret's plaintext over the wire) is
admin-only by policy and isn't in any preset. Threat model and full
scope reference: [docs/security.md](docs/security.md).

## Secrets

`nub secret put` stores a value encrypted with age; reference it from
any stack with `external: true`:

```yaml
services:
  db:
    image: postgres
    secrets:
      - db_password
secrets:
  db_password:
    external: true
```

The container sees the value at `/run/secrets/db_password`. Plaintext
is decrypted to a tmpfs file at deploy time and never leaves the host
over the network. `file:` and `environment:` sources are rejected on
parse — use `nub secret put` instead. Internals (threat model, rehydrate
ordering, identity rotation): [docs/security.md](docs/security.md#secrets).

## TLS

Set `tls_cert` and `tls_key` to PEM file paths and nub serves HTTPS
(and `wss://` for streaming). rustls + ring; TLS 1.2 minimum, 1.3
preferred. Bring your own files; provisioning is out of scope.

For a homelab, [mkcert](https://github.com/FiloSottile/mkcert) is the
shortest path:

```sh
mkcert -install
mkcert -cert-file ~/.config/nub/cert.pem -key-file ~/.config/nub/key.pem nub.local 127.0.0.1
```

```toml
tls_cert = "/home/you/.config/nub/cert.pem"
tls_key  = "/home/you/.config/nub/key.pem"
```

## Troubleshooting

Common pitfalls. Full list: [docs/troubleshooting.md](docs/troubleshooting.md).

- **`no docker or podman socket found`** — start the engine. For
  Podman: `systemctl --user enable --now podman.socket` (rootless) or
  `sudo systemctl enable --now podman.socket` (rootful). Override
  with `DOCKER_HOST` if the socket lives somewhere unusual.
- **`401 Unauthorized`** — token expired (mint a new one), audience
  mismatch (`--aud` at mint must equal nub's `id`), or signature
  invalid (the issuer key was rotated).
- **`403 Forbidden`** — token is valid but its `scope` claim doesn't
  cover the op. Call `whoami` to see what your token can actually do.
- **`image 'foo' not local — pull it first`** — `create_container`
  doesn't auto-pull. `nub stack pull <name>` (or `images:pull`) first.
- **Port already in use** — change `listen` in `nub.toml` or pass
  `--listen 0.0.0.0:8081`.

## File layout

| Purpose | Path |
|---|---|
| Config | `$XDG_CONFIG_HOME/nub/nub.toml` |
| Issuer keypair | `$XDG_DATA_HOME/nub/issuer.key` |
| Admin token | `$XDG_DATA_HOME/nub/admin.jwt` |
| Stacks | `$XDG_DATA_HOME/nub/stacks/<name>/compose.yml` |
| Secrets | `$XDG_DATA_HOME/nub/secrets/<name>.age` |
| Dockerfiles | `$XDG_DATA_HOME/nub/dockerfiles/<name>` |

Wipe all state: `nub uninstall`.

## Status

Active. Single-host scope is intentional and not a phase. Acknowledged
deferred items (token revoke, rotate-on-put for secrets, hot config
reload) live in [CHANGELOG.md](CHANGELOG.md) under "Deferred." Release
history: [GitHub releases](https://github.com/TerryTsai/nub/releases).

## Maintainers

[@TerryTsai](https://github.com/TerryTsai)

## Contributing

Issues and PRs welcome. House rules the build enforces:

- 250 lines per file (`build.rs` fails the build over)
- `cargo fmt` and `cargo clippy` clean (strict lints)
- `unsafe` is forbidden
- New deps need justification — the stack is locked

## License

Dual-licensed under either of [MIT](LICENSE-MIT) or
[Apache-2.0](LICENSE-APACHE) at your option.
