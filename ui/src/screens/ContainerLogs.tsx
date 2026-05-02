import { useEffect, useRef, useState } from "react";
import { useParams } from "react-router-dom";
import { call, streamOp, unwrap, type Host } from "@/api/client";
import { useHosts } from "@/state/hosts";
import { useSession } from "@/state/session";
import { Button } from "@/components/Button";
import { useHostCrumb } from "@/components/HostCrumbs";
import { Page, type Crumb } from "@/components/Page";

const TAIL_LINES = 200;
const MAX_LINES = 4000;

interface Line {
  stderr: boolean;
  data: string;
}

export function ContainerLogs() {
  const { hid, cid } = useParams<{ hid: string; cid: string }>();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };
  const session = useSession(host);

  const [lines, setLines] = useState<Line[]>([]);
  const [follow, setFollow] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [name, setName] = useState<string | null>(null);

  useTitle(host, cid, setName);
  useStream(host, cid, follow, setLines, setError, !!session.session);
  const { paneRef, onScroll } = useAutoscroll(lines);

  const hostCrumb = useHostCrumb(hid ?? "", saved?.label ?? "?");

  if (!saved) return <Page><p>Unknown host.</p></Page>;

  const containerName = name || cid?.slice(0, 12) || "?";
  const crumbs: Crumb[] = [
    hostCrumb,
    { kind: "link", label: containerName, to: `/h/${hid}/c/${cid}` },
    { kind: "link", label: "logs" },
  ];

  return (
    <Page crumbs={crumbs} fill>
      <div className="flex items-center gap-2 px-5 py-2 border-b border-[var(--border-subtle)]">
        <span className="text-[11px] text-[var(--text-tertiary)] mr-auto truncate">
          {containerName}
        </span>
        <Button
          variant={follow ? "primary" : "ghost"}
          onClick={() => setFollow((f) => !f)}
        >
          {follow ? "pause" : "follow"}
        </Button>
        <Button variant="ghost" onClick={() => copyAll(lines)}>copy</Button>
        <Button variant="ghost" onClick={() => setLines([])}>clear</Button>
      </div>
      {error && <p className="px-5 pt-2 text-[var(--error)] text-xs">{error}</p>}
      <LogPane paneRef={paneRef} onScroll={onScroll} lines={lines} />
    </Page>
  );
}

async function copyAll(lines: Line[]) {
  await navigator.clipboard.writeText(lines.map((l) => l.data).join(""));
}

function LogPane({
  paneRef,
  onScroll,
  lines,
}: {
  paneRef: React.Ref<HTMLDivElement>;
  onScroll: React.UIEventHandler<HTMLDivElement>;
  lines: Line[];
}) {
  return (
    <div
      ref={paneRef}
      onScroll={onScroll}
      className="mono text-xs overflow-auto flex-1 min-h-0 px-3 py-2"
      style={{ whiteSpace: "pre-wrap" }}
    >
      {lines.length === 0 ? (
        <span className="text-[var(--text-tertiary)]">(no output yet)</span>
      ) : (
        lines.map((l, i) => (
          <span key={i} className={l.stderr ? "text-[var(--error)]" : ""}>
            {l.data}
          </span>
        ))
      )}
    </div>
  );
}

// ---- Hooks --------------------------------------------------------------

function useTitle(host: Host | undefined, cid: string | undefined, set: (n: string) => void) {
  useEffect(() => {
    if (!host || !cid) return;
    let cancelled = false;
    call(host, { op: "inspect_container", id: cid })
      .then((r) => {
        if (cancelled) return;
        const d = unwrap(r, "container_detail");
        set(d.data.name);
      })
      .catch(() => { /* keep id-as-title fallback */ });
    return () => { cancelled = true; };
  }, [host?.url, host?.token, cid, set]);
}

function useStream(
  host: Host | undefined,
  cid: string | undefined,
  follow: boolean,
  setLines: React.Dispatch<React.SetStateAction<Line[]>>,
  setError: (e: string | null) => void,
  ready: boolean,
) {
  useEffect(() => {
    if (!host || !cid || !follow || !ready) return;
    setError(null);
    const controller = new AbortController();
    streamOp(
      host,
      { op: "stream_logs", id: cid, follow: true, tail: TAIL_LINES },
      (chunk) => {
        if (chunk.type !== "log") return;
        setLines((prev) => prev.concat({ stderr: chunk.stderr, data: chunk.data }).slice(-MAX_LINES));
      },
      controller.signal,
    ).catch((e) => setError((e as Error).message));
    return () => controller.abort();
  }, [host?.url, host?.token, cid, follow, ready, setLines, setError]);
}

function useAutoscroll(lines: Line[]) {
  const paneRef = useRef<HTMLDivElement>(null);
  const atBottomRef = useRef(true);

  function onScroll(e: React.UIEvent<HTMLDivElement>) {
    const el = e.currentTarget;
    atBottomRef.current = el.scrollHeight - el.scrollTop - el.clientHeight < 24;
  }

  useEffect(() => {
    const el = paneRef.current;
    if (el && atBottomRef.current) el.scrollTop = el.scrollHeight;
  }, [lines]);

  return { paneRef, onScroll };
}
