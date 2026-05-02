import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { ApiError, call, unwrap, type Host } from "@/api/client";
import { useHosts } from "@/state/hosts";
import { Button } from "@/components/Button";
import { Card } from "@/components/Card";
import { Field } from "@/components/Field";
import { Page } from "./Hosts";

// One-shot read of `#t=<token>` from the URL. The host URL is taken from
// `window.location.origin` since the user is already loading the UI from the
// nub they want to add. Strip the fragment so a refresh doesn't re-trigger
// and so the token doesn't linger in the address bar.
function consumeBootstrapFragment(): { url: string; token: string } | null {
  if (typeof window === "undefined") return null;
  const hash = window.location.hash.replace(/^#/, "");
  if (!hash) return null;
  const params = new URLSearchParams(hash);
  const t = params.get("t");
  if (!t) return null;
  history.replaceState(null, "", window.location.pathname + window.location.search);
  return { url: window.location.origin, token: t };
}

export function AddHost() {
  const nav = useNavigate();
  const { hosts, add } = useHosts();
  const [bootstrap] = useState(() => consumeBootstrapFragment());
  const [url, setUrl] = useState(bootstrap?.url ?? "");
  const [token, setToken] = useState(bootstrap?.token ?? "");
  const [label, setLabel] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    setPending(true);
    const host: Host = { url: url.replace(/\/$/, ""), token };
    const existing = hosts.find((h) => h.url === host.url && h.token === host.token);
    if (existing) {
      nav(`/h/${existing.hid}`, { replace: true });
      return;
    }
    try {
      const w = unwrap(await call(host, { op: "whoami" }), "whoami");
      const h = unwrap(await call(host, { op: "host_info" }), "host_info");
      const finalLabel = label.trim() || w.data.id || h.data.os || "host";
      const saved = add({ label: finalLabel, url: host.url, token: host.token });
      nav(`/h/${saved.hid}`, { replace: true });
    } catch (e) {
      setError(e instanceof ApiError ? `HTTP ${e.status}` : (e as Error).message);
    } finally {
      setPending(false);
    }
  }

  return (
    <Page title="Add host" right={<Button variant="ghost" onClick={() => nav(-1)}>Cancel</Button>}>
      <Card>
        <form onSubmit={onSubmit} className="flex flex-col gap-3">
          <Field label="URL" hint="e.g. http://192.168.1.10:8080">
            <input
              className="input"
              type="url"
              inputMode="url"
              autoComplete="off"
              autoCapitalize="off"
              autoCorrect="off"
              spellCheck={false}
              placeholder="http://10.0.0.5:8080"
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              required
            />
          </Field>
          <Field label="Token" hint="paste the admin token printed by nub at startup">
            <input
              className="input mono"
              type="password"
              autoComplete="off"
              autoCorrect="off"
              spellCheck={false}
              placeholder="bearer token"
              value={token}
              onChange={(e) => setToken(e.target.value)}
              required
            />
          </Field>
          <Field label="Label" hint="optional — what to call this host in the list">
            <input
              className="input"
              type="text"
              autoCapitalize="off"
              autoCorrect="off"
              placeholder="m73a"
              value={label}
              onChange={(e) => setLabel(e.target.value)}
            />
          </Field>
          {error && <div className="text-[var(--error)] text-sm px-1">{error}</div>}
          <Button type="submit" disabled={pending} autoFocus={!!bootstrap}>
            {pending ? "Connecting…" : "Connect & save"}
          </Button>
        </form>
      </Card>
    </Page>
  );
}
