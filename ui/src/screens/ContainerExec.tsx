import { useEffect, useRef, useState } from "react";
import { useParams, useSearchParams } from "react-router-dom";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import { bidiStream, type BidiStream, type Host } from "@/api/client";
import { useHosts } from "@/state/hosts";
import { useSession } from "@/state/session";
import { useContainerName } from "@/state/containerName";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { Page, type Crumb } from "@/components/Page";

const DEFAULT_CMD = "/bin/sh";

export function ContainerExec() {
  const { hid, cid } = useParams<{ hid: string; cid: string }>();
  const [params] = useSearchParams();
  const cmd = params.get("cmd") || DEFAULT_CMD;

  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };
  const session = useSession(host);

  const [error, setError] = useState<string | null>(null);
  const termRef = useRef<HTMLDivElement>(null);

  const containerName = useContainerName(host, cid);
  useExecTerminal(termRef, host, cid, cmd, !!session.session, setError);

  const sectionCrumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "containers");

  if (!saved) return <Page><p>Unknown host.</p></Page>;

  const crumbs: Crumb[] = [
    ...sectionCrumbs,
    { kind: "link", label: containerName, to: `/h/${hid}/c/${cid}` },
    { kind: "link", label: "exec" },
  ];

  const subnav = (
    <span className="text-[11px] text-[var(--text-tertiary)] truncate">
      <span className="mono">{cmd}</span> · {containerName}
    </span>
  );

  return (
    <Page crumbs={crumbs} subnav={subnav} fill>
      {error && <p className="px-5 pt-2 text-[var(--error)] text-xs">{error}</p>}
      <div ref={termRef} className="flex-1 min-h-0 bg-black px-1" />
    </Page>
  );
}

// ---- Hooks --------------------------------------------------------------

function useExecTerminal(
  ref: React.RefObject<HTMLDivElement | null>,
  host: Host | undefined,
  cid: string | undefined,
  cmd: string,
  ready: boolean,
  setError: (e: string | null) => void,
) {
  useEffect(() => {
    if (!host || !cid || !ready || !ref.current) return;
    const term = newTerminal();
    const fit = new FitAddon();
    term.loadAddon(fit);
    term.open(ref.current);
    fit.fit();
    term.focus();

    const stream = openExec(host, cid, cmd, term, setError);
    const onResize = () => fit.fit();
    window.addEventListener("resize", onResize);

    return () => {
      window.removeEventListener("resize", onResize);
      stream.close();
      term.dispose();
    };
  }, [host?.url, host?.token, cid, cmd, ready, ref, setError]);
}

// ---- Helpers ------------------------------------------------------------

function newTerminal(): Terminal {
  return new Terminal({
    fontFamily: '"JetBrains Mono", ui-monospace, monospace',
    fontSize: 13,
    cursorBlink: true,
    convertEol: true,
    theme: {
      background: "#000000",
      foreground: "#e4e4e7",
      cursor: "#fbbf24",
      selectionBackground: "#3f3f46",
    },
  });
}

function openExec(
  host: Host,
  cid: string,
  cmd: string,
  term: Terminal,
  setError: (e: string | null) => void,
): BidiStream {
  setError(null);
  const stream = bidiStream(
    host,
    { op: "exec", id: cid, cmd: parseCmd(cmd), tty: true },
    (chunk) => {
      if (chunk.type === "log") term.write(chunk.data);
    },
  );
  term.onData((data) => stream.send({ type: "stdin", data }));
  stream.done.catch((e: Error) => setError(e.message));
  return stream;
}

/** Split a command line on spaces. Naive — no shell quoting. For now the
 * default `/bin/sh` is the expected case; users wanting fancier exec can
 * pass a quoted ?cmd= via the URL once we wire that up. */
function parseCmd(s: string): string[] {
  return s.split(/\s+/).filter(Boolean);
}
