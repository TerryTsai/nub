import { useRef, useState } from "react";
import { useParams } from "react-router-dom";
import { type Host } from "@/api/client";
import { useHosts } from "@/state/hosts";
import { useContainerName } from "@/state/containerName";
import { useResilientStream } from "@/state/resilientStream";
import { Button } from "@/components/Button";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { Page, type Crumb } from "@/components/Page";
import { StreamStatus } from "@/components/StatusLine";
import { TerminalView, type TerminalHandle } from "@/components/TerminalView";

const TAIL_LINES = 200;

// SGR codes — wrap stderr in red, reset after.
const RED = "\x1b[31m";
const RESET = "\x1b[0m";

export function ContainerLogs() {
  const { hid, cid } = useParams<{ hid: string; cid: string }>();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };

  const termRef = useRef<TerminalHandle>(null);
  const [follow, setFollow] = useState(true);

  const containerName = useContainerName(host, cid);
  const op = host && cid && follow
    ? ({ op: "stream_logs" as const, id: cid, follow: true, tail: TAIL_LINES })
    : null;
  const { state: connState, error } = useResilientStream(host, op, (chunk) => {
    if (chunk.type !== "log") return;
    const text = chunk.stderr ? `${RED}${chunk.data}${RESET}` : chunk.data;
    termRef.current?.write(text);
  });

  const sectionCrumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "containers");

  if (!saved) return <Page><p>Unknown host.</p></Page>;

  const crumbs: Crumb[] = [
    ...sectionCrumbs,
    { kind: "link", label: containerName, to: `/h/${hid}/c/${cid}` },
    { kind: "link", label: "logs" },
  ];

  const subnav = (
    <>
      <Button size="sm" variant={follow ? "primary" : "ghost"} onClick={() => setFollow((f) => !f)}>
        {follow ? "pause" : "follow"}
      </Button>
      <Button size="sm" variant="ghost" onClick={() => termRef.current?.copyAll()}>copy</Button>
      <Button size="sm" variant="ghost" onClick={() => termRef.current?.clear()}>clear</Button>
    </>
  );

  return (
    <Page crumbs={crumbs} subnav={subnav} fill>
      {error && <p className="px-5 pt-2 text-[var(--error)] text-xs">{error}</p>}
      <StreamStatus state={connState} />
      <TerminalView ref={termRef} />
    </Page>
  );
}
