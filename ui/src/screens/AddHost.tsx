import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { ApiError, call, unwrap, type Host } from "@/api/client";
import { useHosts } from "@/state/hosts";
import { Button } from "@/components/Button";
import { Collapsible } from "@/components/Collapsible";
import { EditCell } from "@/components/EditCell";
import { Page } from "@/components/Page";
import { Row } from "@/components/Row";
import { Spinner } from "@/components/Spinner";
import { scrollFocusedIntoView } from "@/lib/scrollIntoViewOnFocus";

// Pure read of `#t=<token>` from the URL. Host URL is taken from
// `window.location.origin` since the user is loading the UI from the nub
// they want to add. Kept side-effect-free so React StrictMode's
// double-invoke of useState initializers is safe; the hash is stripped in
// a follow-up effect.
function readBootstrapFragment(): { url: string; token: string } | null {
  if (typeof window === "undefined") return null;
  const hash = window.location.hash.replace(/^#/, "");
  if (!hash) return null;
  const params = new URLSearchParams(hash);
  const t = params.get("t");
  if (!t) return null;
  return { url: window.location.origin, token: t };
}

export function AddHost() {
  const nav = useNavigate();
  const { hosts, add } = useHosts();
  const [bootstrap] = useState(() => readBootstrapFragment());
  const [url, setUrl] = useState(bootstrap?.url ?? "");
  const [token, setToken] = useState(bootstrap?.token ?? "");
  const [label, setLabel] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);

  // Strip the fragment once the bootstrap values are captured so a refresh
  // doesn't re-trigger and the token doesn't linger in the address bar.
  useEffect(() => {
    if (bootstrap) {
      history.replaceState(null, "", window.location.pathname + window.location.search);
    }
  }, [bootstrap]);

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
      const msg = e instanceof ApiError ? `HTTP ${e.status}` : (e as Error).message;
      setError(`add host: ${msg}`);
    } finally {
      setPending(false);
    }
  }

  return (
    <Page>
      {error && (
        <p className="text-[var(--error)] text-xs">{error}</p>
      )}

      <form onSubmit={onSubmit} className="contents" {...scrollFocusedIntoView()}>
        <Collapsible label="Host" defaultOpen>
          <Row
            label="Name"
            right={
              <EditCell
                mono
                value={label}
                placeholder="new host"
                onChange={(e) => setLabel(e.target.value)}
              />
            }
          />
          <Row
            label="URL"
            right={
              <EditCell
                mono
                type="url"
                inputMode="url"
                enterKeyHint="next"
                autoComplete="off"
                placeholder="http://10.0.0.5:8080"
                value={url}
                onChange={(e) => setUrl(e.target.value)}
                required
              />
            }
          />
          <Row
            label="Token"
            right={
              <EditCell
                mono
                type="password"
                enterKeyHint="done"
                autoComplete="current-password"
                placeholder="bearer token"
                value={token}
                onChange={(e) => setToken(e.target.value)}
                required
              />
            }
          />
        </Collapsible>

        <Collapsible label="connect" defaultOpen>
          <div className="flex gap-2">
            <Button variant="ghost" onClick={() => nav("/")} className="flex-1">
              Cancel
            </Button>
            <Button type="submit" disabled={pending} autoFocus={!!bootstrap} className="flex-1">
              {pending ? (<><Spinner /> Connecting…</>) : "Connect & save"}
            </Button>
          </div>
        </Collapsible>
      </form>
    </Page>
  );
}
