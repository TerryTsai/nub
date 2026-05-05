import { useEffect, useRef, useState } from "react";
import { useParams, useSearchParams } from "react-router-dom";
import { bidiStream, type BidiStream, type Host } from "@/api/client";
import { useHosts } from "@/state/hosts";
import { useContainerName } from "@/state/containerName";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { Page, type Crumb } from "@/components/Page";
import { TerminalView, type TerminalHandle } from "@/components/TerminalView";

const DEFAULT_CMD = "/bin/sh";

export function ContainerExec() {
  const { hid, cid } = useParams<{ hid: string; cid: string }>();
  const [params] = useSearchParams();
  const cmd = params.get("cmd") || DEFAULT_CMD;

  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };

  const [error, setError] = useState<string | null>(null);
  const termRef = useRef<TerminalHandle>(null);
  const streamRef = useRef<BidiStream | null>(null);

  const containerName = useContainerName(host, cid);

  useEffect(() => {
    if (!host || !cid) return;
    setError(null);
    const stream = bidiStream(
      host,
      { op: "exec", id: cid, cmd: parseCmd(cmd), tty: true },
      (chunk) => {
        if (chunk.type === "log") termRef.current?.write(chunk.data);
      },
    );
    streamRef.current = stream;
    stream.done.catch((e: Error) => setError(e.message));
    return () => {
      stream.close();
      streamRef.current = null;
    };
  }, [host?.url, host?.token, cid, cmd]);

  const sectionCrumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "containers");

  if (!saved) return <Page><p>Unknown host.</p></Page>;

  const crumbs: Crumb[] = [
    ...sectionCrumbs,
    { kind: "link", label: containerName, to: `/h/${hid}/c/${cid}` },
    { kind: "link", label: "exec" },
  ];

  return (
    <Page crumbs={crumbs} fill>
      {error && <p className="px-5 pt-2 text-[var(--error)] text-xs">{error}</p>}
      <TerminalView
        ref={termRef}
        cursorBlink
        onInput={(data) => streamRef.current?.send({ type: "stdin", data })}
      />
    </Page>
  );
}

/** Split a command line on spaces. Naive — no shell quoting. For now the
 * default `/bin/sh` is the expected case; users wanting fancier exec can
 * pass a quoted ?cmd= via the URL once we wire that up. */
function parseCmd(s: string): string[] {
  return s.split(/\s+/).filter(Boolean);
}
