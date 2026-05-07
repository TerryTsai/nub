# Configuration

Config can come from a TOML file, CLI flags, or neither — sane defaults
fill in. CLI flags override file values.

## File lookup

With `--config <path>` given, that path is used. Otherwise nub searches
in this order, taking the first that exists:

1. `$XDG_CONFIG_HOME/nub/nub.toml` (typically `~/.config/nub/nub.toml`)
2. `./nub.toml`
3. `/etc/nub/config.toml`

With no file in any of those, defaults apply. `nub init` materializes a
starter file at path #1. `nub config show` prints the file nub is
actually loading (or "(no config file; using compiled defaults)").

## Schema

```toml
id     = "host1"                            # also: --id host1
listen = "127.0.0.1:8080"                   # also: --listen 127.0.0.1:8080
# tls_cert = "/etc/nub/cert.pem"            # also: --tls-cert
# tls_key  = "/etc/nub/key.pem"             # also: --tls-key
# allowed_binds = ["/data/nub"]             # host paths usable as bind-mount sources
# dockerfiles   = "/srv/nub/dockerfiles"    # default: $XDG_DATA_HOME/nub/dockerfiles
# stacks        = "/srv/nub/stacks"         # default: $XDG_DATA_HOME/nub/stacks
# secrets       = "/srv/nub/secrets"        # default: $XDG_DATA_HOME/nub/secrets
# trusted_issuer = "<base64url ed25519 pubkey>"  # external token issuer
```

## Field reference

| Field | Default | Notes |
|---|---|---|
| `id` | `/etc/hostname` (else `nub`) | Identifier this nub advertises. Set per-host so a token issued for one box can't authenticate against another (it's the JWT `aud`). Also `--id`. |
| `listen` | `0.0.0.0:8080` | Listen address. Also `--listen`. |
| `tls_cert` | (off) | PEM cert path. Pair with `tls_key`. Also `--tls-cert`. |
| `tls_key` | (off) | PEM key path. Pair with `tls_cert`. Also `--tls-key`. |
| `allowed_binds` | `[]` | Host paths usable as bind-mount sources in `create_container`. Empty = bind mounts denied. |
| `dockerfiles` | `$XDG_DATA_HOME/nub/dockerfiles` | Flat directory of Dockerfile texts. |
| `stacks` | `$XDG_DATA_HOME/nub/stacks` | Compose-stack manifests. See [docs/compose.md](compose.md) for what's translated and what's flagged-unsupported. |
| `secrets` | `$XDG_DATA_HOME/nub/secrets` | age-encrypted secrets + per-host identity. |
| `trusted_issuer` | (self-managed) | Base64url Ed25519 pubkey. When set, nub validates JWTs against this key and never mints. |

## Auth modes

**Self-managed (default).** On first start, nub generates an Ed25519
keypair at `$XDG_DATA_HOME/nub/issuer.key` (mode 0600) and mints a
long-lived admin token at `$XDG_DATA_HOME/nub/admin.jwt`. Subsequent
starts re-print the same admin token, so paired clients stay paired
across restarts.

```sh
nub token mint --sub box --preset operator --expires 1y
```

Rotate the issuer key (invalidates everything): `nub key rotate`.

**External issuer.** Set `trusted_issuer = "<base64url pubkey>"`. nub
becomes verify-only — no auto-admin, no `nub token mint`. The private
key lives elsewhere (your laptop, a CI signer, an external service).
Any token validly signed by the configured key is accepted; scope
drives authorization the same way.

## File layout

nub follows the [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html):

| Purpose | Path | Override |
|---|---|---|
| Config | `$XDG_CONFIG_HOME/nub/nub.toml` | `--config <path>` |
| Issuer keypair | `$XDG_DATA_HOME/nub/issuer.key` | `trusted_issuer` (verify-only) |
| Admin token | `$XDG_DATA_HOME/nub/admin.jwt` | none — delete + restart to re-mint |
| Stacks | `$XDG_DATA_HOME/nub/stacks/<name>/compose.yml` | `stacks = "<path>"` |
| Secrets | `$XDG_DATA_HOME/nub/secrets/<name>.age` | `secrets = "<path>"` |
| Dockerfiles | `$XDG_DATA_HOME/nub/dockerfiles/<name>` | `dockerfiles = "<path>"` |

To wipe all state, run `nub uninstall` (prompts unless `--yes`). The
binary itself stays put — `rm` it manually if you also want it gone.
