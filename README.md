# nub

> Manage containers from your phone. One small Rust binary, mobile-shaped API.

`nub` is a Docker/Podman control plane built around two roles: a **nub** runs
on each Docker host and exposes a deliberately tiny JSON+WebSocket API; a
**hub** sits in front of a fleet of nubs and routes phone traffic to them.
Both roles are the same binary, picked by config. A standalone host playing
both roles is a **hubnub** — perfect for a single home-lab box.

It's an experiment in suckless minimalism. Every endpoint earns its place,
list views are compact, detail views are full, and streaming things stream.
The whole codebase is about 1.7k lines of Rust.

## Table of Contents

- [Background](#background)
- [Install](#install)
- [Usage](#usage)
- [Roles](#roles)
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

## Install

Builds with stable Rust:

```sh
git clone https://github.com/TerryTsai/nub
cd nub
cargo build --release
sudo install -m 0755 target/release/nub /usr/local/bin/nub
```

The binary uses rustls, not OpenSSL — drop it on any Linux host with Docker
or Podman running.

## Usage

The smallest useful config — a hubnub serving a phone directly:

```toml
bind  = "127.0.0.1:8080"
token = "use-a-long-random-string"
```

Run it:

```sh
nub --config ./nub.toml
```

If `--config` is omitted, `nub` looks for `./nub.toml` then
`/etc/nub/config.toml`.

### Talking to a hubnub

Every request needs `Authorization: Bearer <token>`.

Unary ops go to `POST /op`:

```sh
TOKEN=$(awk -F'"' '/^token/{print $2}' nub.toml)

curl -sS http://127.0.0.1:8080/op \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"op":"host_info"}'

curl -sS http://127.0.0.1:8080/op \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"op":"container_action","id":"<id>","action":{"kind":"stop"}}'
```

Streaming ops upgrade to `/ws`. Any WebSocket client that lets you set headers
works (`websocat`, `wsta`, your phone client):

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
- **Images** — `list_images`, `remove_image`, `pull_image` (streams progress)
- **Volumes** — `list_volumes`, `remove_volume`
- **Networks** — `list_networks`, `remove_network`

## Roles

Three ways to run the binary, picked by config sections present.

**Hubnub** — single process, both roles collapsed. Phone connects directly.
Dev / home lab.

```toml
bind  = "127.0.0.1:8080"
token = "..."
```

**Fleet nub** — a worker on a Docker host that dials *out* to a hub. No
inbound port. The phone never talks to it directly; the hub does.

```toml
[nub]
hub_url   = "wss://hub.example.com/nub"
nub_token = "long-lived-token-from-enrollment"
```

**Hub** — public-facing router. Holds the registry of allowed nubs and
proxies phone traffic to them. Runs on its own host with no Docker socket.

```toml
[hub]
bind        = "0.0.0.0:8443"
phone_token = "..."

[[hub.nubs]]
id    = "host-a"
token = "..."   # the nub presents this in its dial-out

[[hub.nubs]]
id    = "host-b"
token = "..."
```

Phone hits `GET /nubs` to see which configured nubs are currently online,
and `POST /nubs/{id}/op` to proxy a unary op to a specific nub:

```sh
curl -sS https://hub.example.com/nubs/host-a/op \
  -H "Authorization: Bearer $PHONE_TOKEN" \
  -d '{"op":"list_containers","all":true}'
```

WebSocket streaming through the hub (logs / stats / exec / pull progress) is
on the roadmap; for now the hub proxies only unary ops.

You can mix sections — e.g. `bind` + `[hub]` to act as a hub that's also
addressable directly via its own Docker socket. Practically rare.

## Security

The Docker socket is root-equivalent. Anyone with a valid token can do
everything Docker can. The wrapper deliberately exposes less than Docker's
full API to shrink the blast radius:

- `create_container` rejects `network = "host"` and `network = "container:..."`.
- `create_container` rejects bind-mount sources outside the configured
  `allowed_binds` allowlist. Default is empty — only named volumes work out of
  the box.
- Never exposed in the wire format, no opt-in possible: `Privileged`,
  `PidMode`, `IpcMode`, `UTSMode`, `CapAdd`, `CapDrop`, `SecurityOpt`,
  `Sysctls`, `Devices`. If you need any of these, `nub` is the wrong tool.

To let specific host paths through:

```toml
allowed_binds = ["/data/nub", "/var/lib/nub"]
```

TLS support is recognized in config (`tls_cert`, `tls_key`) but not yet wired
into serving — the binary will warn and serve plaintext if those are set. For
now, bind to localhost behind an SSH tunnel or terminate TLS at a reverse
proxy. Nub-to-hub dial-out uses TLS via `wss://` natively.

## Status

Early. The nub side of v1 is implemented end-to-end:

- 15 ops covering containers, images, volumes, networks, exec, host info
- HTTP + WebSocket transports against the same handler trait
- Nub-to-hub dial-out with exponential backoff and heartbeat
- Constrained `create_container` with allowlisted bind mounts
- Hub: routes phones to nubs for unary ops (config-listed registry, no DB yet)

In progress: streaming through the hub, persistent nub registry, and the
official phone client. The wire format may still shift before 1.0.

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
