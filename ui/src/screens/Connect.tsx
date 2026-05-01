import { useState } from "react";
import { call, type Host, ApiError } from "@/api/client";
import type { HostInfo, WhoamiInfo } from "@/api/types";

interface ConnectedState {
  host: Host;
  whoami: WhoamiInfo;
  hostInfo: HostInfo;
}

export default function Connect() {
  const [url, setUrl] = useState("");
  const [token, setToken] = useState("");
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [connected, setConnected] = useState<ConnectedState | null>(null);

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    setPending(true);
    const host: Host = { url: url.replace(/\/$/, ""), token };
    try {
      const whoamiRes = await call(host, { op: "whoami" });
      if (whoamiRes.type !== "whoami") throw new Error("unexpected whoami response");
      const hostInfoRes = await call(host, { op: "host_info" });
      if (hostInfoRes.type !== "host_info") throw new Error("unexpected host_info response");
      setConnected({ host, whoami: whoamiRes.data, hostInfo: hostInfoRes.data });
    } catch (e) {
      setError(e instanceof ApiError ? `HTTP ${e.status}` : (e as Error).message);
    } finally {
      setPending(false);
    }
  }

  return (
    <div className="min-h-full grid place-items-center p-5">
      <div className="w-full max-w-md flex flex-col gap-4">
        <h1 className="text-3xl font-semibold tracking-tight px-1">nub</h1>

        {!connected && (
          <form onSubmit={onSubmit} className="glass rounded-[var(--radius-lg)] p-5 flex flex-col gap-3">
            <Field label="Host URL">
              <input
                className="input"
                type="url"
                inputMode="url"
                placeholder="http://127.0.0.1:8080"
                value={url}
                onChange={(e) => setUrl(e.target.value)}
                required
              />
            </Field>
            <Field label="Token">
              <input
                className="input mono"
                type="password"
                placeholder="paste admin token"
                value={token}
                onChange={(e) => setToken(e.target.value)}
                required
              />
            </Field>
            {error && <div className="text-[var(--error)] text-sm px-1">{error}</div>}
            <button type="submit" className="btn-primary" disabled={pending}>
              {pending ? "Connecting…" : "Connect"}
            </button>
          </form>
        )}

        {connected && <Connected state={connected} onReset={() => setConnected(null)} />}
      </div>

      <style>{`
        .input {
          background: var(--bg-elevated);
          border: 1px solid var(--border-subtle);
          border-radius: var(--radius-md);
          padding: 12px 14px;
          font: inherit;
          color: var(--text-primary);
          width: 100%;
          outline: none;
          transition: border-color 0.15s var(--ease-out);
        }
        .input:focus { border-color: var(--accent); }
        .btn-primary {
          background: var(--accent);
          color: var(--accent-fg, #fff);
          border: 0;
          border-radius: var(--radius-md);
          padding: 12px 16px;
          font: inherit;
          font-weight: 600;
          cursor: pointer;
          transition: transform 0.2s var(--ease-spring), opacity 0.15s var(--ease-out);
        }
        .btn-primary:disabled { opacity: 0.5; cursor: default; }
        .btn-primary:not(:disabled):active { transform: scale(0.97); }
        .btn-ghost {
          background: transparent;
          color: var(--text-secondary);
          border: 1px solid var(--border-subtle);
          border-radius: var(--radius-md);
          padding: 10px 14px;
          font: inherit;
          cursor: pointer;
        }
      `}</style>
    </div>
  );
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <label className="flex flex-col gap-1.5 px-0.5">
      <span className="text-xs font-medium uppercase tracking-wider text-[var(--text-tertiary)]">{label}</span>
      {children}
    </label>
  );
}

function Connected({ state, onReset }: { state: ConnectedState; onReset: () => void }) {
  const { whoami, hostInfo, host } = state;
  return (
    <div className="flex flex-col gap-3">
      <div className="glass rounded-[var(--radius-lg)] p-5 flex flex-col gap-3">
        <Row label="Connected to" value={host.url} mono />
        <Row label="As" value={whoami.id} mono />
        <Row
          label="Allowed"
          value={whoami.allowed.length === 0 ? "(none)" : whoami.allowed.join(", ")}
          mono
        />
      </div>
      <div className="glass rounded-[var(--radius-lg)] p-5 flex flex-col gap-3">
        <Row label="Engine" value={`${hostInfo.engine} ${hostInfo.version}`} />
        <Row label="OS" value={hostInfo.os} />
        <Row label="Kernel" value={hostInfo.kernel} mono />
        <Row label="CPUs" value={String(hostInfo.cpus)} />
        <Row label="Memory" value={formatBytes(hostInfo.mem_total)} />
        <Row
          label="Containers"
          value={`${hostInfo.containers_running} running / ${hostInfo.containers_total} total`}
        />
        <Row label="Images" value={String(hostInfo.images)} />
      </div>
      <button className="btn-ghost self-start" onClick={onReset}>Disconnect</button>
    </div>
  );
}

function Row({ label, value, mono }: { label: string; value: string; mono?: boolean }) {
  return (
    <div className="flex justify-between items-baseline gap-3">
      <span className="text-sm text-[var(--text-secondary)]">{label}</span>
      <span className={`text-sm text-right break-all ${mono ? "mono" : ""}`}>{value}</span>
    </div>
  );
}

function formatBytes(b: number): string {
  const u = ["B", "KiB", "MiB", "GiB", "TiB"];
  let i = 0;
  while (b >= 1024 && i < u.length - 1) { b /= 1024; i++; }
  return `${b.toFixed(b < 10 ? 1 : 0)} ${u[i]}`;
}
