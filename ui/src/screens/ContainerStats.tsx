import { useEffect, useRef, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { call, streamOp, unwrap, type Host } from "@/api/client";
import { useHosts } from "@/state/hosts";
import { useSession } from "@/state/session";
import { Button } from "@/components/Button";
import { Page } from "@/components/Page";

interface Snapshot {
  cpu_pct: number;
  mem_used: number;
  mem_limit: number;
  net_rx: number;
  net_tx: number;
}

interface Rates {
  rx_per_s: number;
  tx_per_s: number;
}

export function ContainerStats() {
  const { hid, cid } = useParams<{ hid: string; cid: string }>();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };
  const session = useSession(host);

  const [snap, setSnap] = useState<Snapshot | null>(null);
  const [rates, setRates] = useState<Rates>({ rx_per_s: 0, tx_per_s: 0 });
  const [error, setError] = useState<string | null>(null);
  const [name, setName] = useState<string | null>(null);

  useTitle(host, cid, setName);
  useStatsStream(host, cid, !!session.session, setSnap, setRates, setError);

  if (!saved) return <Page title="?"><p>Unknown host.</p></Page>;

  return (
    <Page title={`Stats · ${name || cid?.slice(0, 12) || "?"}`} right={backLink(hid, cid)}>
      {error && <p className="text-[var(--error)] text-xs">{error}</p>}
      {!snap && !error && <p className="text-xs text-[var(--text-tertiary)]">Connecting…</p>}
      {snap && <StatsView snap={snap} rates={rates} />}
    </Page>
  );
}

function backLink(hid: string | undefined, cid: string | undefined) {
  return (
    <Link to={`/h/${hid}/c/${cid}`} aria-label="Back">
      <Button variant="ghost" className="text-sm">←</Button>
    </Link>
  );
}

function StatsView({ snap, rates }: { snap: Snapshot; rates: Rates }) {
  const memPct = snap.mem_limit > 0 ? (snap.mem_used / snap.mem_limit) * 100 : 0;
  return (
    <div className="flex flex-col gap-4">
      <BigNumber label="CPU" value={`${snap.cpu_pct.toFixed(1)}%`} />
      <Bar
        label="Memory"
        primary={`${formatBytes(snap.mem_used)} / ${formatBytes(snap.mem_limit)}`}
        secondary={`${memPct.toFixed(1)}%`}
        pct={memPct}
      />
      <div className="grid grid-cols-2 gap-2">
        <BigNumber label="Net rx/s" value={formatBytes(rates.rx_per_s)} />
        <BigNumber label="Net tx/s" value={formatBytes(rates.tx_per_s)} />
      </div>
      <div className="grid grid-cols-2 gap-2 text-xs text-[var(--text-tertiary)]">
        <span>total rx: {formatBytes(snap.net_rx)}</span>
        <span>total tx: {formatBytes(snap.net_tx)}</span>
      </div>
    </div>
  );
}

function BigNumber({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex flex-col gap-0.5">
      <span className="text-xs text-[var(--text-tertiary)] uppercase tracking-wide">{label}</span>
      <span className="text-2xl font-semibold mono">{value}</span>
    </div>
  );
}

function Bar({ label, primary, secondary, pct }: { label: string; primary: string; secondary: string; pct: number }) {
  const clamped = Math.max(0, Math.min(100, pct));
  return (
    <div className="flex flex-col gap-1">
      <div className="flex justify-between text-xs">
        <span className="text-[var(--text-tertiary)] uppercase tracking-wide">{label}</span>
        <span className="text-[var(--text-secondary)] mono">{secondary}</span>
      </div>
      <div className="h-2 rounded-full bg-[var(--border-subtle)] overflow-hidden">
        <div className="h-full bg-[var(--accent)]" style={{ width: `${clamped}%` }} />
      </div>
      <span className="text-sm mono">{primary}</span>
    </div>
  );
}

function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  const units = ["KB", "MB", "GB", "TB"];
  let v = n / 1024;
  for (const u of units) {
    if (v < 1024) return `${v.toFixed(v < 10 ? 1 : 0)} ${u}`;
    v /= 1024;
  }
  return `${v.toFixed(0)} PB`;
}

// ---- Hooks --------------------------------------------------------------

function useTitle(host: Host | undefined, cid: string | undefined, set: (n: string) => void) {
  useEffect(() => {
    if (!host || !cid) return;
    let cancelled = false;
    call(host, { op: "inspect_container", id: cid })
      .then((r) => {
        if (cancelled) return;
        set(unwrap(r, "container_detail").data.name);
      })
      .catch(() => { /* keep id-as-title fallback */ });
    return () => { cancelled = true; };
  }, [host?.url, host?.token, cid, set]);
}

function useStatsStream(
  host: Host | undefined,
  cid: string | undefined,
  ready: boolean,
  setSnap: (s: Snapshot) => void,
  setRates: (r: Rates) => void,
  setError: (e: string | null) => void,
) {
  const lastRef = useRef<{ rx: number; tx: number; t: number } | null>(null);

  useEffect(() => {
    if (!host || !cid || !ready) return;
    setError(null);
    lastRef.current = null;
    const controller = new AbortController();
    streamOp(
      host,
      { op: "stream_stats", id: cid },
      (chunk) => {
        if (chunk.type !== "stats") return;
        const now = performance.now();
        const prev = lastRef.current;
        if (prev) {
          const dt = (now - prev.t) / 1000;
          setRates({
            rx_per_s: dt > 0 ? Math.max(0, (chunk.net_rx - prev.rx) / dt) : 0,
            tx_per_s: dt > 0 ? Math.max(0, (chunk.net_tx - prev.tx) / dt) : 0,
          });
        }
        lastRef.current = { rx: chunk.net_rx, tx: chunk.net_tx, t: now };
        setSnap({
          cpu_pct: chunk.cpu_pct,
          mem_used: chunk.mem_used,
          mem_limit: chunk.mem_limit,
          net_rx: chunk.net_rx,
          net_tx: chunk.net_tx,
        });
      },
      controller.signal,
    ).catch((e) => setError((e as Error).message));
    return () => controller.abort();
  }, [host?.url, host?.token, cid, ready, setSnap, setRates, setError]);
}
