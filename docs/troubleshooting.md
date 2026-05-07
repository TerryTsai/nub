# Troubleshooting

Common errors and what to do about them. If something here is wrong
or missing, file an issue.

## Engine socket not found

```
no docker or podman socket found.

if podman is installed, enable its socket (it's daemonless and not started by default):
  rootless: systemctl --user enable --now podman.socket
  rootful:  sudo systemctl enable --now podman.socket

if docker is installed, ensure the daemon is running.

override with DOCKER_HOST.
```

The error already tells you what to do. Common cases:

- **Podman, rootless** — the socket isn't started by default. Run
  `systemctl --user enable --now podman.socket`.
- **Docker, fresh install** — the daemon needs starting:
  `sudo systemctl enable --now docker`.
- **Custom socket path** — set `DOCKER_HOST=unix:///path/to/socket`
  before running nub.

`nub status` shows the engine address it resolved to once nub can
talk to it.

## 401 Unauthorized

The bearer token is missing, expired, or signed by the wrong key.

- **Expired** — mint a new one. Default TTL is 90 days. Check exp
  by decoding the JWT (any `jq`-friendly JWT decoder will do).
- **Audience mismatch** — the `aud` claim must equal nub's `id`. If
  you minted with `--aud foo` but nub's `id` is `bar`, every request
  401s. Pass `--aud <id>` matching nub's id, or omit `--aud` (the
  CLI uses `nub`'s hostname by default).
- **Signature invalid** — the issuer key was rotated. Re-mint, or
  recover the prior key from backup.

To inspect what nub thinks: `nub status` prints the `id` it's
advertising.

## 403 Forbidden

The token is valid, but its `scope` claim doesn't include the op
you're calling. Two paths:

```sh
# What does my token actually allow?
curl http://<host>:8080/api/op \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"op":"whoami"}'
```

```sh
# What's the full vocabulary I could ask for?
nub token scopes
```

Re-mint with the right preset or scope list. See
[security.md → Presets](security.md#presets).

## Image not local

```
image 'nginx:foo' not local — pull it first (images:pull)
```

`create_container` doesn't auto-pull — by design. The "no implicit
resource creation" rule keeps a `containers:create` token from
turning into a registry-pull token.

For a stack: `nub stack pull <name>` pulls every service's image,
then proceeds. For a one-off container: pull explicitly with the
`pull_image` op (or `docker pull` from the shell).

## Volume not found

```
volume 'data' not found — create it first (volumes:create)
```

Same pattern. Named volume mounts must reference an existing volume.
The stack runtime creates declared `volumes:` entries automatically
(unless `external: true`); for ad-hoc `create_container` calls, run
`create_volume` first.

## Port already in use

```
listen tcp 0.0.0.0:8080: bind: address already in use
```

Change `listen` in `nub.toml` (or pass `--listen 0.0.0.0:8081`) and
restart. To find what's holding it: `ss -ltnp | grep 8080`.

## TLS misconfiguration

```
tls_cert and tls_key must be set together (got one of two)
```

Half-configured TLS is a startup error rather than a silent fall-back
to plaintext. Set both fields or neither.

```
parsing certs from /etc/nub/cert.pem
```

PEM didn't parse. Check the file is a real cert chain (and that the
key file is the matching private key). `mkcert` produces both side
by side; see [security.md → TLS](security.md#tls).

## Stack deploy hangs / times out

Most often: the engine is pulling a large image. `nub stack logs
<name> --follow` doesn't show pull progress (that streams over the
WebSocket as `pull_progress` chunks). To watch pull progress, use
the UI or call `pull_image` directly.

If the engine is stuck for unrelated reasons, check `nub status`
(engine reachable?) and the engine's own logs (`journalctl --user
-u podman.socket` or equivalent).

## Compose YAML rejected at parse

The most common parse rejections:

- `secret 'x': must declare external: true` — store the value with
  `nub secret put` first, then reference it as `external: true`.
- `secret 'x': file: source not supported` — same fix.
- `config 'x': must declare a content: source` — inline the value
  in the YAML; nub doesn't read external config files.
- `${FOO}: is not set` — variable substitution found a reference
  with no default. Use `${FOO:-fallback}` or pre-substitute.

Full mapping: [compose.md → Hard-rejected](compose.md#hard-rejected).

## Service-to-service DNS doesn't work

By default the stack runtime creates one user-defined bridge per
stack and attaches every container to it. Container-name DNS
resolution is what makes `web` reach `db`. Two gotchas:

- The default container name is `<stack>_<service>` — so `web` would
  resolve `db` as `<stack>_db`, not `db`. Set
  `container_name: db` explicitly until network aliases ship.
- If `network_mode:` is set on the service, it overrides the stack
  network — your service won't be on the bridge.

## Lost the admin token

If `$XDG_DATA_HOME/nub/admin.jwt` is gone and you don't have it saved
elsewhere:

```sh
rm $XDG_DATA_HOME/nub/admin.jwt
nub restart      # or `systemctl --user restart nub`
nub url          # prints the new connect URL
```

nub re-mints the admin token on startup if the file is missing. The
issuer key is unchanged, so any other tokens you minted with `nub
token mint` still work.

To invalidate **all** tokens (admin + everything you minted), rotate
the issuer key:

```sh
nub key rotate
nub restart
```

## nub stops when I log out

User-level systemd units stop on user logout unless lingering is
enabled. The installer asks for sudo to do this; if it was skipped:

```sh
sudo loginctl enable-linger $USER
```

`loginctl show-user $USER | grep Linger` confirms (`Linger=yes`).

## Where do I find the logs?

```sh
journalctl --user -u nub -f          # user-level install
sudo journalctl -u nub -f            # system-level install
```

`nub run` (foreground) prints to stdout; useful for debugging.
