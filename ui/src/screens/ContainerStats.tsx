import { useRef, useState } from "react";
import { useParams } from "react-router-dom";
import { type Host } from "@/api/client";
import { useHosts } from "@/state/hosts";
import { useSession } from "@/state/session";
import { useContainerName } from "@/state/containerName";
import { useResilientStream } from "@/state/resilientStream";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { Page, type Crumb } from "@/components/Page";
import { Sparkline } from "@/components/Sparkline";

const HISTORY = 60;

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

interface History {
  cpu: number[];
  mem: number[];
}

export function ContainerStats() {
  const { hid, cid } = useParams<{ hid: string; cid: string }>();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };
  const session = useSession(host);

  const [snap, setSnap] = useState<Snapshot | null>(null);
  const [rates, setRates] = useState<Rates>({ rx_per_s: 0, tx_per_s: 0 });
  const [history, setHistory] = useState<History>({ cpu: [], mem: [] });
  const lastRef = useRef<{ rx: number; tx: number; t: number } | null>(null);

  const containerName = useContainerName(host, cid);
  const op = host && cid && session.session ? ({ op: "stream_stats" as const, id: cid }) : null;
  const { state: connState, error } = useResilientStream(host, op, (chunk) => {
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
    const memPct = chunk.mem_limit > 0 ? (chunk.mem_used / chunk.mem_limit) * 100 : 0;
    setHistory((h) => ({
      cpu: append(h.cpu, chunk.cpu_pct),
      mem: append(h.mem, memPct),
    }));
  });

  const sectionCrumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "containers");

  if (!saved) return <Page><p>Unknown host.</p></Page>;

  const crumbs: Crumb[] = [
    ...sectionCrumbs,
    { kind: "link", label: containerName, to: `/h/${hid}/c/${cid}` },
    { kind: "link", label: "stats" },
  ];

  const subnav = (
    <span className="text-[11px] text-[var(--text-tertiary)] truncate">
      {connState === "reconnecting" ? "reconnecting…" : containerName}
    </span>
  );

  return (
    <Page crumbs={crumbs} subnav={subnav} fill>
      <div className="flex-1 min-h-0 overflow-auto px-5 py-4">
        {error && <p className="text-[var(--error)] text-xs">{error}</p>}
        {!snap && !error && <p className="text-xs text-[var(--text-tertiary)]">Connecting…</p>}
        {snap && <StatsView snap={snap} rates={rates} history={history} />}
      </div>
    </Page>
  );
}

function StatsView({ snap, rates, history }: { snap: Snapshot; rates: Rates; history: History }) {
  const memPct = snap.mem_limit > 0 ? (snap.mem_used / snap.mem_limit) * 100 : 0;
  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-col gap-1">
        <BigNumber label="CPU" value={`${snap.cpu_pct.toFixed(1)}%`} />
        <Sparkline values={history.cpu} max={100} capacity={HISTORY} />
      </div>
      <div className="flex flex-col gap-1">
        <Bar
          label="Memory"
          primary={`${formatBytes(snap.mem_used)} / ${formatBytes(snap.mem_limit)}`}
          secondary={`${memPct.toFixed(1)}%`}
          pct={memPct}
        />
        <Sparkline values={history.mem} max={100} capacity={HISTORY} />
      </div>
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

function append(buf: number[], v: number): number[] {
  const next = buf.length >= HISTORY ? buf.slice(1) : buf.slice();
  next.push(v);
  return next;
}
