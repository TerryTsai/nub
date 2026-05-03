# API

nub speaks JSON+WebSocket on the same handler trait. The CLI uses this
API; you can hit it from scripts, CI, or your own client. There's no
hand-rolled HTTP — every op flows through the `Op` enum in `src/proto/`.

## Endpoints

| Path | Use |
|---|---|
| `POST /api/op` | Unary ops (list, get, action, create, delete) |
| `GET  /api/ws` | Streaming ops (logs, stats, exec, image pulls, builds) |

Both require a bearer token. The unary endpoint returns one
`OpResult` JSON; the WebSocket multiplexes any number of concurrent
ops by `id`.

## Unary example

```sh
curl http://127.0.0.1:8080/api/op \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"op":"list_containers","all":true}'
```

```sh
curl http://127.0.0.1:8080/api/op \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"op":"host_info"}'
```

## Streaming example

```sh
websocat -H "Authorization: Bearer $TOKEN" ws://127.0.0.1:8080/api/ws
```

Send framed JSON requests, one per line:

```json
{"kind":"request","id":1,"op":{"op":"stream_logs","id":"<id>","follow":true,"tail":100}}
{"kind":"request","id":2,"op":{"op":"stream_stats","id":"<id>"}}
{"kind":"request","id":3,"op":{"op":"pull_image","reference":"alpine:latest"}}
{"kind":"request","id":4,"op":{"op":"exec","id":"<id>","cmd":["sh"],"tty":true}}
```

## Frame shapes

| Shape | When |
|---|---|
| `{"kind":"response","id":N,"result":{"type":"...","data":...}}` | Unary op completed |
| `{"kind":"response","id":N,"result":{"type":"stream_started"}}` | Stream began; expect `stream` frames |
| `{"kind":"stream","id":N,"chunk":{"type":"log","stderr":false,"data":"..."}}` | Stream chunk |
| `{"kind":"stream","id":N,"chunk":{"type":"end","ok":true}}` | Stream finished |
| `{"chunk":{"type":"lagging","dropped":N}}` | Backpressure summary; client fell behind |

Exec sends `stdin` chunks upstream over the same WebSocket:

```json
{"kind":"stream","id":N,"chunk":{"type":"stdin","data":"ls\n"}}
```

## Op catalog

Each op declares one required scope (see
[Authentication](../README.md#authentication)). The `host_info` and
`whoami` introspection ops require no scope.

### Containers
| Op | Scope |
|---|---|
| `list_containers` | `containers:list` |
| `get_container` | `containers:get` |
| `create_container` | `containers:create` |
| `container_action` (start/stop/restart/kill/remove) | `containers:action` |
| `stream_logs` | `containers:logs` |
| `stream_stats` | `containers:stats` |
| `exec` | `containers:exec` |

### Images
| Op | Scope |
|---|---|
| `list_images` / `get_image` / `delete_image` | `images:{list,get,delete}` |
| `pull_image` (streams) | `images:pull` |
| `build_image` (streams) | `images:build` |

### Volumes / Networks
| Op | Scope |
|---|---|
| `list_volumes` / `get_volume` / `delete_volume` | `volumes:{list,get,delete}` |
| `list_networks` / `get_network` / `create_network` / `delete_network` | `networks:{...}` |

### Stacks
| Op | Scope |
|---|---|
| `list_stacks` / `get_stack` | `stacks:{list,get}` |
| `create_stack` / `update_stack` / `delete_stack` | `stacks:{create,update,delete}` |
| `redeploy_stack` / `pull_stack` | `stacks:{redeploy,pull}` |
| `stream_stack_logs` | `stacks:logs` |

### Dockerfiles
| Op | Scope |
|---|---|
| `list_dockerfiles` / `get_dockerfile` / `put_dockerfile` / `delete_dockerfile` | `dockerfiles:{list,get,put,delete}` |

CRUD on text files in a configured flat directory; stored build inputs
for `build_image`. Not compose, not orchestration.

### Secrets
| Op | Scope |
|---|---|
| `list_secrets` / `put_secret` / `delete_secret` | `secrets:{list,put,delete}` |
| `get_secret` (returns plaintext) | `secrets:reveal` (admin-only) |

`secrets:reveal` is intentionally not in any preset. Phones can write
and delete secrets but never read them back over the wire.

### Host / introspection
| Op | Scope |
|---|---|
| `host_info` | none |
| `whoami` | none |

## Permission errors

- 401: missing/expired token, audience mismatch, signature invalid.
- 403: token valid but scope doesn't include the requested op.
- 4xx response body: `{"type":"err","data":{"message":"..."}}`.
