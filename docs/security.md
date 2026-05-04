# Security

The Docker socket is root-equivalent. Anyone with a valid token can do
everything Docker can within their `scope` allowlist. nub layers two
defenses on top of that reality: per-token scopes, and a constrained
container-create surface.

## Per-token scopes

Each JWT carries a `scope` claim — a space-separated list of granular
`<resource>:<action>` strings. Wildcards are limited to `*` (everything)
and `<resource>:*` (every action on one resource). Audience and expiry
are enforced; mismatches are 401, scope-misses are 403.

### Resources and actions

Resources: `containers`, `images`, `volumes`, `networks`, `dockerfiles`,
`stacks`, `secrets`. Each op declares exactly one required scope —
`list_containers` needs `containers:list`, `delete_image` needs
`images:delete`, and so on. `host_info` and `whoami` are introspection
ops and require no scope.

### Presets

Presets are CLI sugar — they expand to explicit scope lists at mint
time, so the JWT contents are always auditable end-to-end. Definitions
live in [`src/auth/scope/presets.rs`](../src/auth/scope/presets.rs).

| Preset | Grants |
|---|---|
| `admin` | `*` |
| `phone` | Day-to-day operations: list/get/logs/stats, container actions, exec, image pull/delete, stack deploy/redeploy/update/delete/logs/pull, secret put/list/delete |
| `readonly` | `:list` and `:get` across every resource |

```sh
nub token mint --sub me  --preset admin
nub token mint --sub box --preset phone
nub token mint --sub ci  --scope containers:list,stacks:deploy,images:pull
nub token scopes
```

`--preset` and `--scope` are mutually exclusive. Default (neither) is
`--preset admin`.

### `secrets:reveal` is admin-only

The `secrets:reveal` scope authorizes reading a secret's plaintext
value back over the wire. It is **not** in any preset. Phones and
generated tokens cannot read secrets; only `*` (admin) authorizes
reveal. Operators read values via `nub secret get` on the host.

## Constrained container-create

Even with `containers:create` allowed, `create_container` rejects the
following — independent of token scope:

- `network = "host"` and `network = "container:..."`
- Bind-mount sources outside the configured `allowed_binds` list
  (default empty — only named volumes work out of the box)

Several engine flags are not exposed in the wire format at all:
`Privileged`, `PidMode`, `IpcMode`, `UTSMode`, `CapAdd`, `CapDrop`,
`SecurityOpt`, `Sysctls`, `Devices`. If you need any of these, nub is
the wrong tool — by design.

To allow specific host paths as bind sources:

```toml
allowed_binds = ["/data/nub", "/var/lib/nub"]
```

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
