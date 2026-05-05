import { useEffect, useRef, useState } from "react";
import { useParams } from "react-router-dom";
import { type Host } from "@/api/client";
import { useHosts } from "@/state/hosts";
import { useResilientStream } from "@/state/resilientStream";
import { Button } from "@/components/Button";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { Page, type Crumb } from "@/components/Page";

const TAIL_LINES = 200;
const MAX_LINES = 4000;

interface Line {
  stderr: boolean;
  data: string;
}

export function StackLogs() {
  const { hid, sname } = useParams<{ hid: string; sname: string }>();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };
  const name = sname ? decodeURIComponent(sname) : "";

  const [lines, setLines] = useState<Line[]>([]);
  const [follow, setFollow] = useState(true);

  const op = host && name && follow
    ? ({ op: "stream_stack_logs" as const, name, follow: true, tail: TAIL_LINES })
    : null;
  const { state: connState, error } = useResilientStream(host, op, (chunk) => {
    if (chunk.type !== "log") return;
    setLines((prev) => prev.concat({ stderr: chunk.stderr, data: chunk.data }).slice(-MAX_LINES));
  });
  const { paneRef, onScroll } = useAutoscroll(lines);

  const sectionCrumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "stacks");

  if (!saved) return <Page><p>Unknown host.</p></Page>;

  const crumbs: Crumb[] = [
    ...sectionCrumbs,
    { kind: "link", label: name, to: `/h/${hid}/stacks/${encodeURIComponent(name)}` },
    { kind: "link", label: "logs" },
  ];

  const subnav = (
    <>
      <span className="text-[11px] text-[var(--text-tertiary)] mr-auto truncate">
        {connState === "reconnecting" ? "reconnecting…" : name}
      </span>
      <Button size="sm" variant={follow ? "primary" : "ghost"} onClick={() => setFollow((f) => !f)}>
        {follow ? "pause" : "follow"}
      </Button>
      <Button size="sm" variant="ghost" onClick={() => copyAll(lines)}>copy</Button>
      <Button size="sm" variant="ghost" onClick={() => setLines([])}>clear</Button>
    </>
  );

  return (
    <Page crumbs={crumbs} subnav={subnav} fill>
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
