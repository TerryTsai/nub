import { useEffect, useRef, useState } from "react";
import { useParams, useSearchParams } from "react-router-dom";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import { bidiStream, call, unwrap, type BidiStream, type Host } from "@/api/client";
import { useHosts } from "@/state/hosts";
import { useSession } from "@/state/session";
import { Heading } from "@/components/Heading";
import { useHostCrumb } from "@/components/HostCrumbs";
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

  const [name, setName] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const termRef = useRef<HTMLDivElement>(null);

  useTitle(host, cid, setName);
  useExecTerminal(termRef, host, cid, cmd, !!session.session, setError);

  const hostCrumb = useHostCrumb(hid ?? "", saved?.label ?? "?");

  if (!saved) return <Page><p>Unknown host.</p></Page>;

  const containerName = name || cid?.slice(0, 12) || "?";
  const crumbs: Crumb[] = [
    hostCrumb,
    { kind: "link", label: containerName, to: `/h/${hid}/c/${cid}` },
    { kind: "link", label: "exec" },
  ];

  return (
    <Page crumbs={crumbs}>
      <Heading category={`Exec · ${cmd}`} title={containerName} />
      {error && <p className="text-[var(--error)] text-xs">{error}</p>}
      <div
        ref={termRef}
        className="border border-[var(--border-subtle)] rounded-[var(--radius-md)] p-2 bg-black"
        style={{ height: "70vh" }}
      />
    </Page>
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
        set(unwrap(r, "container_detail").data.name);
      })
      .catch(() => { /* keep id-as-title fallback */ });
    return () => { cancelled = true; };
  }, [host?.url, host?.token, cid, set]);
}

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
