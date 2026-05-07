# API

One JSON+WebSocket surface, one auth model, one op catalog. The CLI
uses this; you can hit it from scripts, CI, or your own client.

## Endpoints

| Path | Use |
|---|---|
| `POST /api/op` | Unary ops (list, get, action, create, delete) |
| `GET  /api/ws` | Streaming ops (logs, stats, exec, image pulls, builds) |

Both require a bearer token. Unary returns one `OpResult`; the
WebSocket multiplexes any number of concurrent ops by `id`.

## Authentication

```
Authorization: Bearer <jwt>
```

Browsers can't set `Authorization` on `new WebSocket()`. The standard
workaround: send the bearer as a subprotocol named `bearer.<jwt>`:

```js
new WebSocket(`ws://${host}/api/ws`, ['nub', `bearer.${token}`]);
```

The server echoes the `nub` subprotocol back. See
[Authentication](../README.md#authentication) for token minting and
the scope grammar.

## Unary round-trip

Request:

```sh
curl http://127.0.0.1:8080/api/op \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"op":"host_info"}'
```

Response (HTTP 200):

```json
{
  "type": "host_info",
  "data": {
    "nub": "0.0.62+abc123",
    "engine": "podman",
    "version": "5.2.0",
    "os": "fedora",
    "arch": "x86_64",
    "kernel": "6.10.7-200.fc40.x86_64",
    "cpus": 8,
    "mem_total": 33651957760,
    "containers_running": 4,
    "containers_total": 11,
    "images": 23
  }
}
```

Op-level errors come back HTTP 200 with the err variant:

```json
{ "type": "err", "data": { "message": "image 'nginx:foo' not local — pull it first (images:pull)" } }
```

Transport-level errors (auth, malformed body) come back as 4xx with
the same envelope.

## Streaming

```sh
websocat -H "Authorization: Bearer $TOKEN" \
  --header "Sec-WebSocket-Protocol: nub, bearer.$TOKEN" \
  ws://127.0.0.1:8080/api/ws
```

Send framed JSON requests, one per line:

```json
{"kind":"request","id":1,"op":{"op":"stream_logs","id":"<container_id>","follow":true,"tail":100}}
{"kind":"request","id":2,"op":{"op":"stream_stats","id":"<container_id>"}}
{"kind":"request","id":3,"op":{"op":"pull_image","reference":"alpine:latest"}}
{"kind":"request","id":4,"op":{"op":"exec","id":"<container_id>","cmd":["sh"],"tty":true}}
```

`<container_id>` is a 12-char short id from `list_containers`.

## Frame shapes

| Shape | When |
|---|---|
| `{"kind":"response","id":N,"result":{"type":"...","data":...}}` | Unary op completed |
| `{"kind":"response","id":N,"result":{"type":"stream_started"}}` | Stream began; expect `stream` frames |
| `{"kind":"stream","id":N,"chunk":{"type":"log","stderr":false,"data":"..."}}` | Stream chunk |
| `{"kind":"stream","id":N,"chunk":{"type":"end","ok":true}}` | Stream finished |
| `{"kind":"stream","id":N,"chunk":{"type":"lagging","dropped":N}}` | Backpressure summary; client fell behind |

Exec sends `stdin` chunks upstream over the same WebSocket:

```json
{"kind":"stream","id":4,"chunk":{"type":"stdin","data":"ls\n"}}
{"kind":"stream","id":4,"chunk":{"type":"stdin_close"}}
```

## Op catalog

Every op declares exactly one required scope. `whoami` is auth-layer
introspection — any valid token may call it regardless of scope claim.

### Containers

| Op | Scope |
|---|---|
| `list_containers` | `containers:list` |
| `get_container` | `containers:get` |
| `start_container` | `containers:start` |
| `stop_container` | `containers:stop` |
| `restart_container` | `containers:restart` |
| `kill_container` | `containers:kill` |
| `remove_container` | `containers:remove` |
| `create_container` | `containers:create` |
| `stream_logs` | `containers:logs` |
| `stream_stats` | `containers:stats` |
| `exec` | `containers:exec` |

### Images

| Op | Scope |
|---|---|
| `list_images` | `images:list` |
| `get_image` | `images:get` |
| `pull_image` *(streams)* | `images:pull` |
| `build_image` *(streams)* | `images:build` |
| `delete_image` | `images:delete` |

### Volumes

| Op | Scope |
|---|---|
| `list_volumes` | `volumes:list` |
| `get_volume` | `volumes:get` |
| `create_volume` | `volumes:create` |
| `delete_volume` | `volumes:delete` |

### Networks

| Op | Scope |
|---|---|
| `list_networks` | `networks:list` |
| `get_network` | `networks:get` |
| `create_network` | `networks:create` |
| `delete_network` | `networks:delete` |

### Stacks

| Op | Scope |
|---|---|
| `list_stacks` | `stacks:list` |
| `get_stack` | `stacks:get` |
| `create_stack` | `stacks:create` |
| `update_stack` | `stacks:update` |
| `delete_stack` | `stacks:delete` |
| `redeploy_stack` | `stacks:redeploy` |
| `pull_stack` | `stacks:pull` |
| `stream_stack_logs` | `stacks:logs` |

Stack ops compose other resource ops at runtime (network create, image
pull, container create, etc.). The composing scopes are gated against
the caller's token in addition to the top-level `stacks:*` scope.

### Dockerfiles

CRUD on text files in a configured flat directory; stored build
inputs for `build_image`. Not compose, not orchestration.

| Op | Scope |
|---|---|
| `list_dockerfiles` | `dockerfiles:list` |
| `get_dockerfile` | `dockerfiles:get` |
| `put_dockerfile` | `dockerfiles:put` |
| `delete_dockerfile` | `dockerfiles:delete` |

### Secrets

| Op | Scope |
|---|---|
| `list_secrets` | `secrets:list` |
| `put_secret` | `secrets:put` |
| `delete_secret` | `secrets:delete` |
| `get_secret` *(returns plaintext)* | `secrets:reveal` *(admin-only)* |

`secrets:reveal` is intentionally not in any preset. Non-admin
tokens — regardless of where they're held — can write and delete
secrets but never read plaintext back over the wire.

### Host / introspection

| Op | Scope |
|---|---|
| `host_info` | `host:info` |
| `whoami` | *(none — any valid token)* |

## Error semantics

| HTTP | Cause |
|---|---|
| 401 | Missing/expired token, audience mismatch, signature invalid |
| 403 | Token valid but its scope claim doesn't include the op's required scope |
| 4xx | Malformed request body |
| 200 + `{"type":"err",...}` | Op reached the handler but failed (engine error, invalid argument, etc.) |

Streaming op failures arrive as a final `End` chunk with `ok: false`
and an `err` message — no separate transport-level error.
