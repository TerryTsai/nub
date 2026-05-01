# nub

> Manage containers from your phone. One small Rust binary, mobile-shaped API.

`nub` runs on each Docker or Podman host. It exposes a deliberately tiny
JSON+WebSocket API designed to be driven from a phone: list containers, tail
logs, stream stats, exec a shell, pull images, create containers under a strict
policy. Run it standalone or wire it into a fleet behind a hub.

It's an experiment in suckless minimalism — every endpoint earns its place,
list views are compact, detail views are full, and streaming things stream.
The whole node is about 1.6k lines of Rust.

## Table of Contents

- [Background](#background)
- [Install](#install)
- [Usage](#usage)
- [Modes](#modes)
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

`nub` builds with stable Rust:

```sh
git clone https://github.com/TerryTsai/nub
cd nub
cargo build --release
sudo install -m 0755 target/release/nub /usr/local/bin/nub
```

The binary uses rustls, not OpenSSL — drop it on any Linux host with Docker
or Podman running.

## Usage

Create `nub.toml`:

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

### Talking to nub

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

## Modes

The same binary runs in three modes, picked by config:

**Standalone.** Bind locally, phone connects directly. Dev / home lab.

```toml
bind  = "127.0.0.1:8080"
token = "..."
```

**Fleet node.** Dial out to a hub. No inbound port. The phone talks to the
hub; the hub multiplexes to registered nodes. The node trusts the hub fully.

```toml
[hub]
url        = "wss://hub.example.com/node"
node_token = "long-lived-token-from-enrollment"
```

**Hub.** Public-facing endpoint that holds the node registry and routes phone
requests to nodes. Same binary; runs on a separate host with no Docker socket.

```toml
[hub_server]
bind        = "0.0.0.0:8443"
phone_token = "..."

[[hub_server.nodes]]
id    = "host-a"
token = "..."   # nodes present this token in their dial-out
```

Phone hits `GET /nodes` to see which configured nodes are currently online,
and `POST /nodes/{id}/op` to proxy a unary op to a specific node:

```sh
curl -sS https://hub.example.com/nodes/host-a/op \
  -H "Authorization: Bearer $PHONE_TOKEN" \
  -d '{"op":"list_containers","all":true}'
```

WebSocket streaming via the hub (logs / stats / exec / pull progress) is on
the roadmap; for now the hub proxies only unary ops.

You can set both `bind` and `[hub]` to run both transports against the same
local Docker socket.

## Security

The Docker socket is root-equivalent. Anyone with the token can do everything
Docker can. The wrapper deliberately exposes less than Docker's full API to
shrink the blast radius:

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
into serving — `nub` will warn and serve plaintext if those are set. For now,
bind to localhost behind an SSH tunnel, or terminate TLS at a reverse proxy.
Hub-mode dial-out uses TLS via `wss://` natively.

## Status

Early. The node side of v1 is implemented end-to-end:

- 15 ops covering containers, images, volumes, networks, exec, host info
- HTTP + WebSocket transports against the same handler trait
- Hub-mode dial-out with exponential backoff and heartbeat
- Constrained `create_container` with allowlisted bind mounts

The hub itself and an official phone client are in progress. The wire format
may still shift before 1.0.

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
