# Changelog

All notable changes to nub. The wire surface (`Op` request shapes,
`OpResult` response variants, `Frame` envelope, `StreamChunk` chunks,
JWT claim layout, scope grammar) is the public API; changes there
are flagged. Internal Rust naming is not.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
nub is pre-1.0; minor wire shifts can land in any release until then.

## [Unreleased]

## [0.0.63] — 2026-05-06

### Changed

- **Token presets renamed.** `--preset phone` → `--preset operator`;
  same scope contents, neutral name. Devices aren't a codified role —
  the wire surface doesn't know who's holding a token.
- New `--preset deploy` for CI runners: stack lifecycle plus the
  composing sub-ops, no `containers:exec`, no `secrets:put`.
- Docs reorganized: new `docs/quickstart.md`, `docs/compose.md`,
  `docs/troubleshooting.md`. `docs/security.md` and `docs/api.md`
  rewritten for accuracy (op catalog scope names, container-create
  rules).

### Refactored (internal; no wire change)

- Consolidated three iso8601 formatters and two `valid_name`
  validators into `ops::util`.
- Shared tmpfs runtime helpers between `ops::secrets::runtime` and
  `ops::configs::runtime`.
- Stack runtime no longer talks the engine wire directly; routed
  through `ops::networks`/`ops::volumes`/`ops::images::pull` with
  internal label-aware variants. `ops::stacks::engine` removed.
- Single `auth::introspect` helper for the Whoami short-circuit
  shared by HTTP and WebSocket transports.
- `Policy::from_config(&Config)` constructor used by both `nub run`
  and the in-process CLI commands.

## [0.0.62] — 2026-04 (UI polish)

`0.0.50` through `0.0.62` are a sustained mobile-first UI polish
arc: touch targets, scroll behavior, copy affordances, density,
progressive disclosure. Highlights:

- Detail pages: sections collapse by default, every row tap-to-copy,
  Combobox replaces the picker zoo (`0.0.59`–`0.0.62`)
- Subnav pills aligned and stable across pages (`0.0.54`–`0.0.60`)
- iOS zoom defeated via literal 16px on inputs (`0.0.45`, `0.0.50`)
- Form-as-detail: create screens mirror their detail page exactly
  (`0.0.31`–`0.0.34`)
- Drop client-side permission gating; server is the source of truth
  (`0.0.33`)
- Color rule codified — `mono`+amber for engine identifiers, plain
  for classifications, `dim` for defaults (`0.0.36`)

Per-version notes: `git log --oneline`.

## [0.0.57] — 2026-03 (pure auth layer)

**Wire change.** Container actions split into per-verb scopes
(`containers:start`, `containers:stop`, `containers:restart`,
`containers:kill`, `containers:remove`) replacing the prior
combined `containers:action`. Each `Op` declares exactly one
required scope; stack handlers gate every composed sub-action
against the caller's claims.

`force` flags retired in favor of explicit scope grants. The
"no implicit pull/create" rule on `create_container` is enforced
here: image must be local, named volumes must already exist.

## [0.0.30] — 2026-02 (rootless secrets)

Compose `secrets:` and `configs:` materialization paths fixed for
rootless engines. Files land at `/tmp/nub-<USER>/secrets/...` (mode
0444) with parent dirs 0755 — traversable by mapped sub-UIDs while
still requiring an explicit bind-mount to read.

## [0.0.28] — 2026-02 (compose configs)

Compose `configs:` with `content:` (inline plaintext) supported.
`file:`/`external:`/`environment:` rejected on parse with a pointed
message.

## [0.0.27] — 2026-01 (positioning + binary trim)

Single-host scope made explicit. README lifted internals into
`docs/`. Release binary trimmed from 6.5 MB to 5.0 MB by switching
to musl-static + LTO.

## [0.0.26] — 2026-01 (secret rehydrate, stack CLI)

Daemon re-materializes every stack's referenced secrets at
startup so `restart: always` containers come back cleanly across
reboots. Per-stack failures logged and skipped.

`nub stack {deploy|ls|rm|redeploy|logs}` shipped.

## Earlier

`0.0.1` through `0.0.25`: foundation. JWT auth, scope grammar,
single-engine adapter (Docker or Podman), HTTP unary + WebSocket
streaming on one handler trait, embedded UI feature, install
script, age-encrypted secrets, compose subset translator. Full
detail in `git log --oneline`.

## Deferred

Acknowledged but deliberately not yet shipped:

- Token revoke (today: rotate the issuer key to invalidate everything)
- Rotate-on-put for secrets (re-encrypt to a new identity)
- Hot config reload (today: `nub restart`)
- `nub stack rehydrate-all` oneshot ordered before the engine
  (closes the boot-race window for `restart: always` containers
  with `secrets:` references)
- Network aliases (lets stack DNS work without `container_name:`)
- Podman-native secret mechanism (eliminates the host-readable
  exposure window during deploy)
- OpenAPI spec generation
- `nub doctor` (engine probe, scope audit, allowlist sanity check)
- Audit log for state-changing ops

[Unreleased]: https://github.com/TerryTsai/nub/compare/v0.0.63...HEAD
[0.0.63]: https://github.com/TerryTsai/nub/releases/tag/v0.0.63
[0.0.62]: https://github.com/TerryTsai/nub/releases/tag/v0.0.62
[0.0.57]: https://github.com/TerryTsai/nub/releases/tag/v0.0.57
[0.0.30]: https://github.com/TerryTsai/nub/releases/tag/v0.0.30
[0.0.28]: https://github.com/TerryTsai/nub/releases/tag/v0.0.28
[0.0.27]: https://github.com/TerryTsai/nub/releases/tag/v0.0.27
[0.0.26]: https://github.com/TerryTsai/nub/releases/tag/v0.0.26
