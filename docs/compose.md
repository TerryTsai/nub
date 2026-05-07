# Compose support

nub translates a curated subset of compose YAML directly into its
container/network/volume primitives. No `docker compose` shell-out,
no orchestrator — the YAML is parsed in-process, validated, and
mapped onto nub ops. The trade-off is feature surface: nub takes
the parts of compose you'd actually trust on a homelab and refuses
the parts you wouldn't.

This page is the contract.

## Translated

The fields below behave the way you'd expect from compose.

### Top-level

| Key | Notes |
|---|---|
| `services` | Required. Map of service name → spec. |
| `volumes` | Map. Only `external: true/false` is read — `external: true` means "use a volume that already exists." |
| `secrets` | Map. Only `external: true` (referencing a value stored via `nub secret put`). The compose spec's `name:` override is honored. |
| `configs` | Map. Only `content:` (inline plaintext). |
| `version` | Read and ignored. |

### Service-level

| Key | Notes |
|---|---|
| `image` | Required. The image must already be local — nub never auto-pulls during create. Stack deploys explicitly pull missing images first. |
| `container_name` | Optional. Defaults to `<stack>_<service>`. |
| `command`, `entrypoint` | String (shell-split) or list. |
| `environment` | KV map or `KEY=VALUE` list. |
| `ports` | `container`, `host:container`, or `ip:host:container`. Bare numbers default to `/tcp`. |
| `volumes` | `target`, `source:target`, or `source:target:ro`. Named volume sources resolve to `<stack>_<volume>`. Host-path sources must be in `allowed_binds`. Anonymous volumes (no source) are rejected. |
| `network_mode` | Plain string. `host` and `container:...` are rejected by `create_container`. |
| `restart` | `no`, `always`, `unless-stopped`, `on-failure`, `on-failure:N`. |
| `working_dir`, `user`, `hostname` | Strings. |
| `labels` | KV map or list. nub adds `nub.stack=<name>` and `nub.service=<svc>` automatically. |
| `healthcheck` | `test`, `interval`, `timeout`, `retries`, `start_period`, `disable`. Durations accept compose syntax (`1h30m`, `500ms`, etc.). |
| `cap_add`, `cap_drop` | Lists. |
| `privileged` | Boolean. The wire surface accepts it; the bind allowlist is the security boundary. |
| `extra_hosts` | List of `host:ip` lines. |
| `init` | Boolean. |
| `expose` | List of container ports. |
| `secrets` | Refs to top-level `secrets:`. Short form (name string) or long form (`source`/`target`/`mode`/`uid`/`gid`). `mode`/`uid`/`gid` are parsed but advisory — files are always 0444 on the host. |
| `configs` | Same shape as `secrets`. |

## Flagged-unsupported

The parser accepts these without erroring, but doesn't translate them.
Each surfaces in `get_stack` under `unsupported` (top-level) or
`service_unsupported` (per-service) so the UI can call them out.

Top-level: anything not in the list above. Common examples: `name`,
`x-extensions`.

Service-level: `build`, `depends_on`, `deploy`, `profiles`, `dns`,
`devices`, `tmpfs`, `shm_size`, `ulimits`, `sysctls`, `memory`,
`cpu_shares`, `mem_limit`, `pid`, `ipc`, `userns_mode`, anything
under `x-`. (Some of these are present on the underlying engine
request — `devices`, `tmpfs`, `shm_size`, `ulimits`, `sysctls`,
`memory_limit`, `cpu_shares` — but the YAML shape isn't translated
yet. Use the `create_container` op directly if you need them today.)

## Hard-rejected

These fail at parse time with a pointed error message:

| Construct | Why |
|---|---|
| `secrets:` entry without `external: true` | nub-managed sources only — paste-and-deploy stacks must declare the secret was stored on the host already. |
| `secrets:` with `file:` | nub doesn't read files at parse time; store the value with `nub secret put` instead. |
| `secrets:` with `environment:` | Same reason — values come from `nub secret put`. |
| `configs:` without `content:` | Inline-only; no file/external/environment sources. |
| `configs:` with `file:`, `external:`, or `environment:` | Same reason. |

## Variable substitution

Compose-style `$VAR`, `${VAR}`, `${VAR:-default}`, `${VAR-default}`,
and `$$` (literal `$`) are honored. Undefined variables without a
default raise a parse error rather than silently emitting empty
string.

The substitution environment is **empty** by default — stack ops
don't read `os.environ`. Use `${VAR:-default}` if you want to keep
substitution syntax in your YAML but provide fallback values.

## Runtime caveats

These aren't YAML gotchas — they're what the slice-2 stack runtime
does and doesn't do:

- **No `depends_on` ordering.** Services start in parallel. If your
  app needs to wait for the database, use a healthcheck-aware client
  inside the app container, not a startup ordering directive.
- **No healthcheck-conditional startup.** `healthcheck:` is plumbed
  through to the engine (which then surfaces health state in
  `list_containers`), but it doesn't gate startup.
- **Always-recreate on redeploy.** Brief downtime per stack. No
  rolling updates today; not planned at the homelab scale.
- **Service-name DNS requires `container_name:`.** Every container
  in a stack joins one user-defined network named after the stack,
  which gives container-name DNS resolution. Until network aliases
  ship, set `container_name:` explicitly when one service needs to
  reach another by short name. Otherwise resolve via the
  `<stack>_<service>` default.

## Mapping summary

| Compose construct | Becomes |
|---|---|
| `services.<name>` | A container with `nub.stack=<stack>`, `nub.service=<name>` labels |
| `services.<name>.network_mode` (unset) | The stack network (`<stack>` bridge) |
| `services.<name>.volumes` named source | `<stack>_<source>` volume |
| `volumes.<name>` (`external: false`) | A volume created at deploy with `nub.stack=<stack>` |
| `volumes.<name>` (`external: true`) | Reference to an existing volume by exact name |
| `secrets.<name>` (`external: true`) | Decrypted to tmpfs, bind-mounted at `/run/secrets/<name>` |
| `configs.<name>` (`content: ...`) | Plaintext to tmpfs, bind-mounted at `/<name>` |

## When nub is the wrong tool

If your compose.yml uses any of these heavily, reach for
`docker compose` or a real orchestrator:

- `depends_on` with healthcheck conditions for startup ordering
- `deploy.replicas`, rolling updates, blue-green
- `profiles` for selective service activation
- `build` (multi-stage builds within compose)
- `extends` or YAML anchors for cross-file reuse

nub is a control plane for **one** container host running a small
number of stacks where redeploys can pause for a couple of seconds.
Outside that envelope, you'll feel the trim.
