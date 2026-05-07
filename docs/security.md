# Security

The Docker socket is root-equivalent. Anyone with a valid token can do
everything the engine permits within their `scope` allowlist. nub
layers two defenses on top of that reality: per-token scopes, and a
constrained `create_container` surface.

## Per-token scopes

Each JWT carries a `scope` claim — a space-separated list of granular
`<resource>:<action>` strings. Wildcards are limited to `*` (everything)
and `<resource>:*` (every action on one resource). Audience and expiry
are enforced; mismatches are 401, scope-misses are 403.

### Resources and actions

Resources: `host`, `auth`, `containers`, `images`, `volumes`,
`networks`, `dockerfiles`, `stacks`, `secrets`. Each op declares
exactly one required scope — `list_containers` needs `containers:list`,
`delete_image` needs `images:delete`, and so on. Full op-to-scope
mapping in [API → Op catalog](api.md#op-catalog).

`whoami` is auth-layer introspection: any valid token may call it
regardless of its scope claim. Every other op gates on scope.

### Presets

Presets are CLI sugar — they expand to explicit scope lists at mint
time, so the JWT contents are always auditable end-to-end. Presets are
general-purpose roles, not device-specific.

| Preset | Grants |
|---|---|
| `admin` | `*` |
| `operator` | Day-to-day operations: list/get/logs/stats; create/start/stop/restart/remove/exec containers; pull/delete images; create/delete volumes and networks; deploy/redeploy/update/delete/logs/pull stacks; secrets put/list/delete |
| `deploy` | Stack delivery: list/get for every resource the stack runtime touches, plus stack lifecycle (create/update/delete/redeploy/pull/logs) and the composing sub-ops (`images:pull`, `networks:*`, `volumes:*`, `containers:create`/`start`/`stop`/`remove`). No exec, no secret writes |
| `readonly` | `:list` and `:get` across every resource. No state changes; secret values not included |

```sh
nub token mint --sub me   --preset admin     --expires 1y
nub token mint --sub box  --preset operator  --expires 90d
nub token mint --sub ci   --preset deploy    --expires 90d
nub token mint --sub mon  --preset readonly  --expires 90d
nub token mint --sub fine --scope containers:list,stacks:get
nub token scopes
```

`--preset` and `--scope` are mutually exclusive. Default (neither) is
`--preset admin`. Definitions: `src/auth/scope/presets.rs`.

### `secrets:reveal` is admin-only

The `secrets:reveal` scope authorizes reading a secret's plaintext
value back over the wire. It is **not** in any preset. Only `*`
(admin) authorizes reveal. Non-admin tokens can write and delete
secrets but never read plaintext over the wire — operators read
values via `nub secret get` on the host.

## Constrained `create_container`

The wire surface accepts most engine flags including `privileged`,
`cap_add`/`cap_drop`, `devices`, `sysctls`, `tmpfs`, `ulimits`,
`shm_size`, `init`, and `extra_hosts`. Treat the scope grant and the
bind allowlist below as the actual security boundary; `create_container`
is not a sandbox.

That said, even with `containers:create` granted, the handler rejects
the following — independent of token scope:

- `network = "host"` and `network = "container:..."`
- Bind-mount sources outside the configured `allowed_binds` list
  (default empty — only named volumes and nub-managed tmpfs paths
  work out of the box)
- Anonymous volume mounts (`[{"source":"","target":"/data"}]`) — name
  the volume and `create_volume` first
- Image references that aren't already local — pull explicitly with
  `images:pull` before creating
- Named volume mounts whose volume doesn't exist — create it first
  with `create_volume`

The "no implicit resource creation" rules let you mint a token with
`containers:create` and know it can't trigger background pulls, name
volumes, or grow the host attack surface in ways the caller didn't ask
for. Stack ops compose these primitives explicitly, with each
sub-action gated against the caller's scope.

To allow specific host paths as bind sources:

```toml
allowed_binds = ["/data/nub", "/var/lib/nub"]
```

Engine flags not modeled in nub's wire format today (no scope, no
field): `PidMode`, `IpcMode`, `UTSMode`, `SecurityOpt`. Adding these
is a wire change and would need a release note.

## Secrets

nub stores per-host secrets as
[age](https://age-encryption.org/v1)-encrypted blobs at
`$XDG_DATA_HOME/nub/secrets/<name>.age`, encrypted to a per-host X25519
identity at `secrets/.identity` (mode 0600).

### Compose integration

Reference a stored secret from any stack with `external: true`:

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

At deploy time nub decrypts each referenced secret to a tmpfs file
under `/tmp/nub-<USER>/secrets/<stack>/<svc>/<name>` (mode 0444, parent
dirs 0755) and bind-mounts it read-only into the container at
`/run/secrets/<name>`. Override the in-container path with the
long-form ref:

```yaml
    secrets:
      - source: db_password
        target: /etc/postgres.pw
```

`file:` and `environment:` sources in compose `secrets:` blocks are
rejected on parse — use `nub secret put` + `external: true`.

### Reboot rehydrate

`/tmp/nub-<USER>/secrets/` is tmpfs on systemd-default Linux distros
(Fedora, Arch); on Debian/Ubuntu `/tmp` is on disk by default but the
cleanup-on-stack-delete + rehydrate-on-boot behavior keeps stale
plaintext bounded. The daemon re-materializes every stack's referenced
secrets on startup, before serving, so containers with `restart: always`
come back cleanly. Per-stack failures are logged and skipped — one
broken stack must not block boot. Lives in
`ops::stacks::rehydrate::rehydrate_all`.

A small race remains: if the engine starts containers before nub's
daemon comes up (because nub's systemd unit is `After=docker.service`),
those first-attempt starts may fail. The fix is a oneshot
`nub stack rehydrate-all` ordered `Before=docker.service`; not yet
shipped, deliberately deferred until someone reports the bite.

### Threat model

The threat model is split deliberately by lifecycle phase.

**At rest** (the encrypted `.age` blob in `$XDG_DATA_HOME/nub/secrets/`):
mode 0600, owned by the user running nub. Protects against:

- Backup leaks
- Accidental `git add` of `$XDG_DATA_HOME`
- Filesystem reads by other host users

**During deploy** (the materialized plaintext under `/tmp/nub-<USER>/secrets/`):
mode 0444 inside a 0755 parent dir. **Readable by any local host user
while the stack is up.** This matches what docker compose, Kubernetes
pod-mounted secrets, and Vault Agent do — the materialized copy needs
to be reachable by container UIDs after rootless engine userns mapping,
which means it can't be locked to the owner. nub's threat model
assumes a single trusted operator account per host; if you run nub on
a multi-user box and care about isolation between local user accounts,
this is a real exposure window.

**Out of scope, all phases:** root on the host. Same posture as Docker
Swarm workers, Kubernetes nodes, and Vault Agent on a host. Root can
read the identity file and decrypt any secret. nub does not try to
defend against host-root and will not add a fake "secure enclave"
story.

Realistic future hardening (deferred until asked):

- TPM2 sealing of the identity (at-rest)
- Linux kernel keyring (in-memory only)
- Passphrase on `nub run`
- Use podman's native `--secret` mechanism when the engine is podman,
  so plaintext lives in a tmpfs the kernel scopes to the container's
  userns — eliminates the host-readable-during-deploy exposure

## TLS

Set `tls_cert` and `tls_key` to PEM file paths and nub serves HTTPS
(and `wss://` for the WebSocket transport). rustls + ring; TLS 1.2
minimum, 1.3 preferred. Both fields must be set together — a
half-configured TLS fails at startup rather than silently serving
plaintext.

The cert and key are loaded once at startup; rotation requires a
restart. Provisioning is out of scope: bring your own files from
wherever (Let's Encrypt, mkcert, your CA). Without TLS configured,
nub serves plaintext.

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

Restart nub and the connect URL switches to `https://`.

## Glossary

- **Audience (`aud`)**: the host id a token is for. Tokens minted for
  one nub won't authenticate against another even with the same issuer
  key.
- **Issuer**: the Ed25519 keypair that signs tokens. nub manages its
  own by default; set `trusted_issuer = "<pubkey>"` to delegate
  signing to an external service (nub becomes verify-only).
- **Scope**: a `<resource>:<action>` string in a token's `scope`
  claim. Each op declares one required scope.
- **Stack**: a directory of compose YAML deployed as a labeled set of
  containers, networks, and volumes — `nub.stack=<name>` is the
  invariant that ties them together.
- **Stack network**: the user-defined bridge nub creates per stack so
  service-name DNS works between its containers.
- **Rehydrate**: re-materialize a stack's compose `secrets:` /
  `configs:` to tmpfs on daemon startup, so containers with
  `restart: always` find their files after a reboot.
