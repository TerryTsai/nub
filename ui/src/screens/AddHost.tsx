import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { ApiError, call, type Host } from "@/api/client";
import { useHosts } from "@/state/hosts";
import { Button } from "@/components/Button";
import { Card } from "@/components/Card";
import { Field } from "@/components/Field";
import { Page } from "./Hosts";

export function AddHost() {
  const nav = useNavigate();
  const { add } = useHosts();
  const [url, setUrl] = useState("");
  const [token, setToken] = useState("");
  const [label, setLabel] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    setPending(true);
    const host: Host = { url: url.replace(/\/$/, ""), token };
    try {
      const w = await call(host, { op: "whoami" });
      if (w.type !== "whoami") throw new Error("unexpected response");
      const h = await call(host, { op: "host_info" });
      if (h.type !== "host_info") throw new Error("unexpected response");
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
          <Button type="submit" disabled={pending}>
            {pending ? "Connecting…" : "Connect & save"}
          </Button>
        </form>
      </Card>
    </Page>
  );
}
