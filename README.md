# nub

[![release](https://img.shields.io/github/actions/workflow/status/TerryTsai/nub/release.yml?label=release)](https://github.com/TerryTsai/nub/actions/workflows/release.yml)
[![ci](https://img.shields.io/github/actions/workflow/status/TerryTsai/nub/ci.yml?label=ci&branch=main)](https://github.com/TerryTsai/nub/actions/workflows/ci.yml)
[![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#license)

A control plane for one container host.

Runs on your box. Drives from any browser. Manages containers, stacks,
and secrets without making you reach for `docker run`.

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
Decrypted only while a stack uses it. Phones can write secrets but
can't read them back over the network.

**Hand out limited access.** Mint a token for your phone that does
phone things. Mint another for CI that only deploys stacks. Each token
says exactly what it can and can't do. Rotate by deleting it.

**One host on purpose.** No fleet, no cluster, no orchestrator. Run
nub on each box you have and switch between them in the UI. The
single-host scope is what lets it stay simple.

**Stays out of your way.** ~5 MB binary, ~4 MiB RAM at idle, zero idle
CPU. Works against Docker or Podman. Doesn't fight your other services
for resources.

## Install

```sh
curl -fsSL https://raw.githubusercontent.com/TerryTsai/nub/main/install.sh | sh
```

Installs `~/.local/bin/nub`, sets up a user-level systemd unit, and
prints a connect URL. Source builds, system-wide installs, and env
overrides (`NUB_VERSION`, `NUB_PREFIX`, `NUB_SERVICE`):
[docs/install.md](docs/install.md).

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
| **Tokens** | `nub token mint --preset {admin\|phone\|readonly}` · `nub token scopes` |
| **Keys** | `nub key gen\|rotate` |
| **Bind allowlist** | `nub bind list\|allow\|deny` |

Per-command help: `nub <cmd> --help`. Shell completion: `nub completions {bash,zsh,fish}`.

## Usage

Once the daemon is running, open the connect URL on any device — the
same UI works on phone, tablet, or desktop. Print it any time:

```sh
nub url      # text URL
nub qr       # scannable QR code
```

Paste-and-deploy a stack from a terminal:

```sh
nub stack deploy myapp ./compose.yml
nub stack logs myapp --follow
```

## API

The CLI uses a stable JSON+WebSocket API; you can hit it directly from
scripts or CI:

```sh
curl http://127.0.0.1:8080/api/op \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"op":"list_containers","all":true}'
```

Streaming ops (logs, stats, exec, image pulls) run over `ws://.../api/ws`.
Frame format and full op catalog: [docs/api.md](docs/api.md).

## Configuration

nub reads `nub.toml` from the first match of: `--config <path>`,
`$XDG_CONFIG_HOME/nub/nub.toml`, `./nub.toml`, `/etc/nub/config.toml`.
With no file, sane defaults apply (`listen = 0.0.0.0:8080`, hostname-derived
`id`).

| Field | Default | Notes |
|---|---|---|
| `id` | `/etc/hostname` | Identifier this nub advertises. Also `--id`. |
| `listen` | `0.0.0.0:8080` | Listen address. Also `--listen`. |
| `tls_cert` / `tls_key` | (off) | PEM paths. Both required to enable TLS. |
| `allowed_binds` | `[]` | Host paths usable as bind-mount sources. |
| `dockerfiles` | `$XDG_DATA_HOME/nub/dockerfiles` | Stored Dockerfile texts. |
| `stacks` | `$XDG_DATA_HOME/nub/stacks` | Compose-stack manifests. |
| `secrets` | `$XDG_DATA_HOME/nub/secrets` | age-encrypted secrets. |
| `trusted_issuer` | (self) | Base64url Ed25519 pubkey; verify-only mode. |

Generate a starter file: `nub init`. Full schema with examples:
[docs/configuration.md](docs/configuration.md).

## Authentication

nub uses Ed25519-signed JWTs as bearer tokens. Each token's `scope`
claim is a space-separated list of `<resource>:<action>` strings —
scopes are granular and never all-or-nothing. The presets:

| Preset | Grants |
|---|---|
| `admin` | `*` (everything) |
| `phone` | Day-to-day operations: list/get/logs/stats, container actions, exec, image pull/delete, stack deploy/redeploy/update/delete/logs/pull, secret put/list/delete |
| `readonly` | `:list` and `:get` across every resource |

```sh
nub token mint --sub me  --preset admin
nub token mint --sub box --preset phone
nub token mint --sub ci  --scope containers:list,stacks:deploy,images:pull
nub token scopes                         # full vocabulary
```

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
preferred. Bring your own files (Let's Encrypt, mkcert, your CA);
provisioning is out of scope.

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

Active. Single-host scope is intentional and not a phase. The roadmap
of acknowledged-but-deferred items (token revoke, rotate-on-put for
secrets, hot config reload) is tracked in-tree.

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
