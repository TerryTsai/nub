# Quick start

Fresh box → first stack deployed in five minutes. Linux with Docker
or Podman already installed.

## 1. Install

```sh
curl -fsSL https://raw.githubusercontent.com/TerryTsai/nub/main/install.sh | sh
```

The script drops the binary at `~/.local/bin/nub`, writes a starter
config, installs a user-level systemd unit, and starts nub. On
success it prints something like:

```
issuer key:  ed25519:DkPL...
admin token: eyJhbGciOiJFZERTQSI...
connect:     http://my-host:8080/add#t=eyJhbGciOiJFZERTQSI...
```

Save the connect URL — it's the only place the admin token is shown
end-to-end.

If you closed the terminal: `nub url` reprints it.

## 2. Confirm it's running

```sh
nub status
```

Expected: `engine` shows `docker (...)` or `podman (...)`, `systemd`
shows `active`. If `engine` reports unreachable, see
[Troubleshooting → engine socket](troubleshooting.md#engine-socket-not-found).

## 3. Open the UI

Either:

- Paste the connect URL into a browser on any device on the same
  network, or
- Run `nub qr` and scan the QR code from a phone.

The UI strips the token from the address bar once the device is
paired and stores it locally.

## 4. Deploy your first stack

From the UI: paste a compose.yml into the Deploy box and hit go.

From the shell:

```sh
cat > /tmp/web.yml <<'EOF'
services:
  web:
    image: nginx:alpine
    ports:
      - "8080:80"
    restart: unless-stopped
EOF

nub stack deploy demo /tmp/web.yml
```

`nub stack deploy` does the work of `images:pull`,
`networks:create`, `containers:create`, `containers:start` in order
and reports back. If the image isn't local, it's pulled first.

`http://<host>:8080` from the UI now shows the demo stack with one
container running. Visit `http://<host>:8080` (or whatever you
mapped) for nginx itself.

## 5. Tail logs

```sh
nub stack logs demo --follow
```

Or click the stack in the UI and the logs panel streams on its own.

## 6. Mint a scoped token for CI

```sh
nub token mint --sub ci --preset deploy --expires 90d
```

The output is the JWT. Hand it to the CI runner; configure the runner
to call:

```sh
curl http://<host>:8080/api/op \
  -H "Authorization: Bearer $NUB_TOKEN" \
  -d '{"op":"redeploy_stack","name":"demo"}'
```

`deploy` covers everything `stacks:*` needs to compose under the
hood. It does **not** grant `containers:exec` or `secrets:put`.
For those, mint an `operator` token.

## What to read next

- [Compose support](compose.md) — what nub translates, flags as
  unsupported, or rejects on parse
- [Authentication & security](security.md) — the scope grammar,
  presets, container-create rules, threat model
- [API](api.md) — full op catalog, frame format, streaming
- [Configuration](configuration.md) — every field in `nub.toml`
- [Troubleshooting](troubleshooting.md) — common errors with fixes
