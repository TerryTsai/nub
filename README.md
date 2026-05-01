# nub

Minimal Docker/Podman control plane node. One static binary, mobile-shaped JSON+WebSocket API.

## Status

Implemented:

- Standalone HTTP server (`/op`) for unary ops
- WebSocket transport (`/ws`) with framed request/response/stream protocol
- Bearer-token auth (constant-time compare) on all endpoints
- Ops: `host_info`, `list_containers`, `stream_logs`, `stream_stats`, `exec`

Not yet: TLS, hub mode, exec/stats/inspect/actions, image/volume/network ops, constrained create.

## Build & run

```sh
cargo build --release
./target/release/nub --config ./nub.toml
```

## `nub.toml`

```toml
bind = "127.0.0.1:8080"
token = "replace-with-a-long-random-string"

# Optional. Recognized by the loader but not yet wired into serving;
# nub will warn and serve plaintext if these are set.
# tls_cert = "/etc/nub/cert.pem"
# tls_key  = "/etc/nub/key.pem"
```

If `--config` is omitted, nub looks for `./nub.toml` then `/etc/nub/config.toml`.

## HTTP usage

Unary ops via POST to `/op`:

```sh
TOKEN=$(awk -F'"' '/^token/{print $2}' nub.toml)

curl -sS http://127.0.0.1:8080/op \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"op":"host_info"}'

curl -sS http://127.0.0.1:8080/op \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"op":"list_containers","all":true}'
```

A streaming op over `/op` returns 400 — use `/ws`.

## WebSocket usage

`websocat` example (any client that lets you set headers works):

```sh
TOKEN=$(awk -F'"' '/^token/{print $2}' nub.toml)

websocat -H "Authorization: Bearer $TOKEN" ws://127.0.0.1:8080/ws
```

Then send framed JSON requests, one per line:

```json
{"kind":"request","id":1,"op":{"op":"host_info"}}
{"kind":"request","id":2,"op":{"op":"list_containers","all":true}}
{"kind":"request","id":3,"op":{"op":"stream_logs","id":"<container-id>","follow":true,"tail":100}}
{"kind":"request","id":4,"op":{"op":"stream_stats","id":"<container-id>"}}
{"kind":"request","id":5,"op":{"op":"exec","id":"<container-id>","cmd":["sh","-c","echo hi"],"tty":false}}
```

Replies:

- Unary: one `{"kind":"response","id":N,"result":...}`.
- Stream: one `{"kind":"response","id":N,"result":{"type":"stream_started"}}`,
  followed by zero or more `{"kind":"stream","id":N,"chunk":...}` frames,
  terminated by `{"kind":"stream","id":N,"chunk":{"type":"end","ok":true}}`.
- Backpressure: if the writer can't keep up, dropped chunks are summarized as
  `{"chunk":{"type":"lagging","dropped":N}}`.
- For `exec`, the client may send `Frame::Stream` upstream:
  `{"kind":"stream","id":N,"chunk":{"type":"stdin","data":"ls\n"}}` for
  keystrokes, and `{"chunk":{"type":"stdin_close"}}` to send EOF.

## Design

See the project handoff brief for philosophy and the `OpHandler` seam. Short version:
the wrapper is the security boundary; list endpoints are compact, detail endpoints
are full; one trait sits between transports and Docker so HTTP, WS, and the future
hub-mode dial-out all share the same handler.
