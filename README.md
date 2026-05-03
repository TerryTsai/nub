# nub

[![release](https://img.shields.io/github/actions/workflow/status/TerryTsai/nub/release.yml?label=release)](https://github.com/TerryTsai/nub/actions/workflows/release.yml)
[![ci](https://img.shields.io/github/actions/workflow/status/TerryTsai/nub/ci.yml?label=ci&branch=main)](https://github.com/TerryTsai/nub/actions/workflows/ci.yml)
[![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#license)

> Manage containers from your phone. One small Rust binary, mobile-shaped API.

`nub` runs on a Docker or Podman host and exposes a deliberately tiny
JSON+WebSocket API designed for a phone client to drive: list containers,
tail logs, stream stats, exec a shell, pull images, create containers under a
strict policy. Per-token permissions; no orchestration; no fleet management
yet — just a server with a trust list.

It's an experiment in suckless minimalism. Every endpoint earns its place,
list views are compact, detail views are full, and streaming things stream.

## Table of Contents

- [Background](#background)
- [Footprint](#footprint)
- [Install](#install)
- [Usage](#usage)
- [Configuration](#configuration)
- [Security](#security)
- [Status](#status)
- [Maintainers](#maintainers)
- [Contributing](#contributing)
- [License](#license)

## Background

Docker's HTTP API is sprawling. `/containers/json` returns ~30 fields per
container; every endpoint exposes every Docker knob. Driving that from a
phone over a flaky connection is miserable.

`nub` takes the opposite approach:

- **Mobile-shaped responses.** Lists return ~6 fields per item, not 30. Detail
  views return everything a detail screen actually wants. One round trip per
  screen.
- **Streams over polling.** Logs, stats, exec, image pull progress all stream
  over one multiplexed WebSocket.
- **Curated surface.** No users-and-roles, no orchestration, no compose. Just
  container primitives over the wire.
- **Wrapper as boundary.** `create_container` rejects host networking, host
  bind mounts outside an allowlist, privileged mode. The phone-driven create
  path is intentionally smaller than Docker's.

If you want full Docker, run Docker. `nub` is for the 80% of operations a
human does on their phone at 11pm.

## Footprint

A single statically-linked binary with the UI embedded as static assets.
Numbers from a `--release` build, idle, on linux x86_64:

- **Binary**: 3.4 MiB stripped (1.4 MiB packaged). 2.6 MiB without the UI.
- **RSS at idle**: ≈ 4 MiB. Flat — there is no background poller.
- **Idle CPU**: 0%. Event-driven: a stream stays open while you're
  watching it, otherwise `nub` is asleep on the socket.

That's roughly two orders of magnitude lighter than the typical
container-management UI server.

## Install

Pre-built binaries (Linux x86_64 / aarch64; macOS not built yet — see below):

```sh
curl -fsSL https://raw.githubusercontent.com/TerryTsai/nub/main/install.sh | sh
```

The installer downloads the binary, runs `nub init` to drop a starter config
in `$XDG_CONFIG_HOME/nub/nub.toml`, and (when systemd is present) installs a
unit file, enables lingering, and starts nub as a managed service.

Defaults — runs as the invoking user with a user-level unit
(`~/.config/systemd/user/nub.service`); falls back to a system-level unit
(`/etc/systemd/system/nub.service`) when invoked as root.

Env overrides:

- `NUB_VERSION=v0.0.1` — pin a specific release (default: latest)
- `NUB_PREFIX=$HOME/.local/bin` — install destination
- `NUB_SERVICE=user|system|none` — service-level override (default: auto)

Re-run the script to upgrade — it replaces the binary and `systemctl restart`
picks up the new version. Plain binary install with no daemon setup:
`NUB_SERVICE=none curl … | sh`.

The binary uses rustls (not OpenSSL) and is statically linked against musl on
Linux — works on any glibc/musl distro.

From source (for development):

```sh
git clone https://github.com/TerryTsai/nub
cd nub
cd ui && npm install && npm run build && cd ..
cargo build --release --features embed-ui
sudo install -m 0755 target/release/nub /usr/local/bin/nub
```

Drop `--features embed-ui` (and the `npm` step) if you don't want the UI
baked in — the binary is API-only and any web server can serve `ui/dist`.

For an existing install, refresh the systemd unit any time without
re-running the installer:

```sh
nub systemd-unit --user > ~/.config/systemd/user/nub.service && \
  systemctl --user daemon-reload && \
  systemctl --user restart nub
```

(`--system` for `/etc/systemd/system/nub.service`.)

## Usage

Zero-arg startup uses sane defaults — `id` from `/etc/hostname`, `bind`
`0.0.0.0:8080`:

```sh
$ nub
admin token: 7a9e4b...c4f1   (regenerates each restart, allows everything)
nub m73a listening on 0.0.0.0:8080
```

When you want a real config, `nub init` writes a starter:

```sh
$ nub init                            # writes ~/.config/nub/nub.toml; prints the path
$ nub init /etc/nub/config.toml       # write somewhere specific (sudo if needed)
$ nub init -                          # write to stdout (for piping)
$ nub init --force                    # overwrite if exists
```

The generated file has the live defaults filled in plus commented examples
for `[engine]` and `[[trust]]`. Edit it; nub re-reads on restart and never
overwrites it.

Copy the admin token from stdout, then poke the host:

```sh
TOKEN=7a9e4b...c4f1

curl -sS http://127.0.0.1:8080/op \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"op":"host_info"}'

curl -sS http://127.0.0.1:8080/op \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"op":"list_containers","all":true}'
```

The admin token is regenerated on every restart and authorizes everything —
it's there so the operator on the host can always poke their own nub for
debugging. For phones and other long-lived clients, define `[[trust]]` entries
in a config file (see below).

### Streaming ops over WebSocket

```sh
websocat -H "Authorization: Bearer $TOKEN" ws://127.0.0.1:8080/ws
```

Then send framed JSON requests, one per line:

```json
{"kind":"request","id":1,"op":{"op":"stream_logs","id":"<id>","follow":true,"tail":100}}
{"kind":"request","id":2,"op":{"op":"stream_stats","id":"<id>"}}
{"kind":"request","id":3,"op":{"op":"pull_image","reference":"alpine:latest"}}
{"kind":"request","id":4,"op":{"op":"exec","id":"<id>","cmd":["sh"],"tty":true}}
```

Replies come back framed:

- **Unary:** one `{"kind":"response","id":N,"result":{"type":"...","data":...}}`
- **Stream:** one `{"kind":"response","id":N,"result":{"type":"stream_started"}}`,
  then zero or more `{"kind":"stream","id":N,"chunk":...}` frames, terminated
  by an `end` chunk.
- **Backpressure:** if the client can't keep up, dropped chunks are summarized
  as `{"chunk":{"type":"lagging","dropped":N}}`.
- **Exec:** send `stdin` chunks upstream over the same WebSocket
  (`{"kind":"stream","id":N,"chunk":{"type":"stdin","data":"ls\n"}}`).

### What you can do

- **Host** — `host_info`
- **Containers** — `list_containers`, `inspect_container`, `container_action`
  (start / stop / restart / kill / remove), `create_container`, `stream_logs`,
  `stream_stats`, `exec`
- **Images** — `list_images`, `inspect_image`, `remove_image`, `pull_image`
  (streams progress), `build_image` (streams /build progress; reads a stored
  Dockerfile, applies a tag)
- **Volumes** — `list_volumes`, `inspect_volume`, `remove_volume`
- **Networks** — `list_networks`, `inspect_network`, `create_network`,
  `remove_network`
- **Dockerfiles** — `list_dockerfiles`, `read_dockerfile`, `write_dockerfile`,
  `delete_dockerfile` (CRUD on text files in a configured flat directory; not
  compose, not orchestration — just stored build inputs for `build_image`)

## Configuration

Config can come from a TOML file, CLI flags, or neither (sane defaults
fill in). CLI flags override file values.

### File lookup

With `--config` given, that path is used. Otherwise `nub` searches in this
order, taking the first that exists:

1. `$XDG_CONFIG_HOME/nub/nub.toml` (typically `~/.config/nub/nub.toml`)
2. `./nub.toml`
3. `/etc/nub/config.toml`

With no file in any of those, defaults apply: `id` from `/etc/hostname`,
`bind` `0.0.0.0:8080`, `dockerfiles` at `$XDG_DATA_HOME/nub/dockerfiles`
(typically `~/.local/share/nub/dockerfiles`). `nub init` materializes a
starter file at path #1.

### Schema

```toml
id   = "host1"                              # also: --id host1
bind = "127.0.0.1:8080"                     # also: --bind 127.0.0.1:8080
# tls_cert = "/etc/nub/cert.pem"            # also: --tls-cert
# tls_key  = "/etc/nub/key.pem"             # also: --tls-key
# allowed_binds = ["/data/nub"]             # host paths usable as bind-mount sources
# dockerfiles   = "/srv/nub/dockerfiles"    # default: $XDG_DATA_HOME/nub/dockerfiles
# trusted_issuer = "<base64url ed25519 pubkey>"  # external token issuer; default: nub mints its own
```

### Authentication

nub uses **Ed25519-signed JWTs** as bearer tokens (RFC 7519 + RFC 8037).
Authorization derives from each token's `scope` claim per OAuth 2.0
(RFC 6749 §3.3). There's no user database — only a single trusted issuer
key against which every presented token is validated.

Two modes:

- **Self-managed (default).** On first start, nub generates an Ed25519
  keypair at `$XDG_DATA_HOME/nub/issuer.key` (mode 0600) and mints a
  long-lived admin token at `$XDG_DATA_HOME/nub/admin.jwt`. Subsequent
  starts re-print the same admin token, so phones paired once stay paired
  across restarts. Adding a device:
  ```
  nub mint --sub phone-1 --scope '*' --expires 1y
  ```
  Paste/scan the output to the phone. Rotating to invalidate everything:
  `nub keygen --rotate`.

- **External issuer.** Set `trusted_issuer = "<base64url pubkey>"` in
  config. nub becomes verify-only — no auto-admin, no `nub mint` (the
  private key lives elsewhere: your laptop, latch, a CI signer). Any
  token validly signed by the configured key is accepted; scope drives
  authorization the same way.

Tokens carry `sub` (caller identity), `aud` (which host the token is for),
`exp` (expiry), and `scope` (space-separated op names; `*` is wildcard).
Mismatched audience or expired tokens are 401. Valid token requesting
an op not in scope is 403.

### File layout

`nub` follows the [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html):

| Purpose | Path | Override |
|---------|------|----------|
| Config file | `$XDG_CONFIG_HOME/nub/nub.toml` | `--config <path>` or any of the lookup paths above |
| Dockerfiles directory | `$XDG_DATA_HOME/nub/dockerfiles` | `dockerfiles = "<path>"` in the config |
| Issuer keypair | `$XDG_DATA_HOME/nub/issuer.key` | `trusted_issuer` in the config (verify-only) |
| Admin token | `$XDG_DATA_HOME/nub/admin.jwt` | none — delete + restart to re-mint |

To wipe all nub state on a host, run `nub uninstall` (prompts for
confirmation; pass `--yes` to skip). The binary itself stays put — `rm
/usr/local/bin/nub` if you also want the binary gone.

## Security

The Docker socket is root-equivalent. Anyone with a valid token can do
everything Docker can within their `allowed` list. Two layers of protection:

1. **Per-token op allowlist.** Each `[[trust]]` entry constrains which ops
   that token may invoke. Grant `host_info` to a read-only viewer; grant
   `["*"]` only to operators you'd give SSH to.
2. **Constrained `create_container`.** Even with `create_container` allowed,
   the binary rejects:
   - `network = "host"` and `network = "container:..."`
   - bind-mount sources outside the configured `allowed_binds` list (default
     empty — only named volumes work out of the box)
   - Never exposed in the wire format at all: `Privileged`, `PidMode`,
     `IpcMode`, `UTSMode`, `CapAdd`, `CapDrop`, `SecurityOpt`, `Sysctls`,
     `Devices`. If you need any of these, `nub` is the wrong tool.

To allow specific host paths as bind sources:

```toml
allowed_binds = ["/data/nub", "/var/lib/nub"]
```

### TLS

Set `tls_cert` and `tls_key` to PEM file paths and nub serves HTTPS (and
`wss://` for the WebSocket transport). rustls + ring; TLS 1.2 minimum,
1.3 preferred. Both fields must be set together — a half-configured TLS
fails at startup rather than silently serving plaintext.

The cert and key are loaded once at startup; rotation requires a restart.
Provisioning is out of scope: bring your own files from wherever (Let's
Encrypt, mkcert, your CA, …). Without TLS configured, nub serves
plaintext as before.

## Status

Early. End-to-end:

- 15 ops covering containers, images, volumes, networks, exec, host info
- HTTP + WebSocket transports against the same handler trait
- Per-token permission enforcement
- Random admin token at startup for first-run / debugging
- Constrained `create_container` with allowlisted bind mounts

Cut in the most recent slice (had been added then redesigned out): hub-of-many
fleet routing. The model coming next is "phone has a list of nubs it knows
about and talks to each directly." If multi-host aggregation through a single
endpoint becomes necessary, it'll come back as a deliberate design rather
than a static-config registry.

## Maintainers

[@TerryTsai](https://github.com/TerryTsai)

## Contributing

Issues and PRs welcome. Before opening either, please skim the
[Background](#background) — `nub` is opinionated and many "obvious" features
have been deliberately left out.

A few house rules the build enforces:

- 250-line-per-file limit (`build.rs` fails the build over)
- `cargo fmt` and `cargo clippy` clean (strict lints in `Cargo.toml [lints]`)
- `unsafe` is forbidden
- New dependencies need a justification — the stack is locked

## License

MIT OR Apache-2.0
