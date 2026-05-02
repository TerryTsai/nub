import { useEffect, useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import * as Dialog from "@radix-ui/react-dialog";
import { call, unwrap, type Host } from "@/api/client";
import { useHosts } from "@/state/hosts";
import { useSession } from "@/state/session";
import type { Action, ContainerDetail as ContainerDetailT } from "@/api/types";
import { Button } from "@/components/Button";
import { Card, Row } from "@/components/Card";
import { Page } from "./Hosts";

export function ContainerDetail() {
  const { hid, cid } = useParams<{ hid: string; cid: string }>();
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };
  const session = useSession(host);

  const [detail, setDetail] = useState<ContainerDetailT | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState<string | null>(null);

  async function refresh() {
    if (!host || !cid) return;
    try {
      const r = unwrap(await call(host, { op: "inspect_container", id: cid }), "container_detail");
      setDetail(r.data);
      setError(null);
    } catch (e) {
      setError((e as Error).message);
    }
  }

  useEffect(() => {
    if (host && session.session) refresh();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [hid, cid, session.session]);

  async function act(name: string, action: Action, after?: () => void) {
    if (!host || !cid) return;
    setPending(name);
    try {
      unwrap(await call(host, { op: "container_action", id: cid, action }), "ok");
      if (after) after();
      else await refresh();
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setPending(null);
    }
  }

  const back = (
    <Link to={`/h/${hid}`} aria-label="Back">
      <Button variant="ghost" className="text-sm">←</Button>
    </Link>
  );

  if (!saved) {
    return <Page title="?" right={back}><Card><p>Unknown host.</p></Card></Page>;
  }

  const can = (op: string) => session.session?.can(op) ?? false;
  const denyReason = (op: string) =>
    session.session && !can(op) ? `your token doesn't allow ${op}` : undefined;

  return (
    <Page title={detail?.name || cid?.slice(0, 12) || "?"} right={back}>
      {session.loading && <Card><p className="text-[var(--text-secondary)]">Connecting…</p></Card>}
      {session.error && <Card><p className="text-[var(--error)]">{session.error}</p></Card>}

      {detail && (
        <>
          <Card>
            <div className="flex items-center gap-2">
              <span className={`dot dot-${detail.state}`} aria-label={detail.state} />
              <span className="text-sm">{detail.state}{detail.exit_code !== 0 ? ` (exit ${detail.exit_code})` : ""}</span>
            </div>
            <Row label="Image" value={detail.image} mono />
            <Row label="Created" value={detail.created} mono />
            {detail.started_at && <Row label="Started" value={detail.started_at} mono />}
            {detail.finished_at && <Row label="Finished" value={detail.finished_at} mono />}
            {detail.restart_count > 0 && <Row label="Restarts" value={String(detail.restart_count)} />}
            {detail.network_mode && <Row label="Network" value={detail.network_mode} />}
            {detail.restart_policy && <Row label="Restart policy" value={detail.restart_policy} />}
          </Card>

          <Card>
            <div className="grid grid-cols-2 gap-2">
              <Button
                variant="primary"
                disallowReason={denyReason("container_action")}
                disabled={pending !== null || detail.running}
                onClick={() => act("start", { kind: "start" })}
              >
                {pending === "start" ? "…" : "Start"}
              </Button>
              <Button
                variant="ghost"
                disallowReason={denyReason("container_action")}
                disabled={pending !== null || !detail.running}
                onClick={() => act("stop", { kind: "stop" })}
              >
                {pending === "stop" ? "…" : "Stop"}
              </Button>
              <Button
                variant="ghost"
                disallowReason={denyReason("container_action")}
                disabled={pending !== null || !detail.running}
                onClick={() => act("restart", { kind: "restart" })}
              >
                {pending === "restart" ? "…" : "Restart"}
              </Button>
              <RemoveButton
                disallow={denyReason("container_action")}
                pending={pending}
                running={detail.running}
                onConfirm={(force) => act("remove", { kind: "remove", force }, () => nav(`/h/${hid}`))}
              />
            </div>
            {error && <p className="text-[var(--error)] text-sm">{error}</p>}
          </Card>

          <Card>
            <div className="grid grid-cols-2 gap-2">
              <Link to={`/h/${hid}/c/${cid}/logs`}>
                <Button
                  variant="ghost"
                  className="w-full"
                  disallowReason={denyReason("stream_logs")}
                >
                  Logs →
                </Button>
              </Link>
              <Link to={`/h/${hid}/c/${cid}/stats`}>
                <Button
                  variant="ghost"
                  className="w-full"
                  disallowReason={denyReason("stream_stats")}
                >
                  Stats →
                </Button>
              </Link>
            </div>
          </Card>

          {detail.cmd.length > 0 && (
            <Card>
              <Row label="Cmd" value={detail.cmd.join(" ")} mono />
              {detail.entrypoint.length > 0 && <Row label="Entrypoint" value={detail.entrypoint.join(" ")} mono />}
              {detail.working_dir && <Row label="Working dir" value={detail.working_dir} mono />}
              {detail.user && <Row label="User" value={detail.user} mono />}
            </Card>
          )}

          {detail.env.length > 0 && (
            <Card>
              <h3 className="text-sm font-semibold">Environment</h3>
              <pre className="text-xs mono whitespace-pre-wrap break-all text-[var(--text-secondary)]">
                {detail.env.join("\n")}
              </pre>
            </Card>
          )}
        </>
      )}
    </Page>
  );
}

function RemoveButton({
  disallow,
  pending,
  running,
  onConfirm,
}: {
  disallow: string | undefined;
  pending: string | null;
  running: boolean;
  onConfirm: (force: boolean) => void;
}) {
  const [open, setOpen] = useState(false);
  return (
    <>
      <Button
        variant="destructive"
        disallowReason={disallow}
        disabled={pending !== null}
        onClick={() => setOpen(true)}
      >
        {pending === "remove" ? "…" : "Remove"}
      </Button>
      <Dialog.Root open={open} onOpenChange={setOpen}>
        <Dialog.Portal>
          <Dialog.Overlay className="fixed inset-0 bg-black/50 backdrop-blur-sm" />
          <Dialog.Content className="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 w-[min(92vw,360px)] glass-strong rounded-[var(--radius-sheet)] p-5 flex flex-col gap-3 z-50">
            <Dialog.Title className="text-lg font-semibold">Remove container?</Dialog.Title>
            <Dialog.Description className="text-sm text-[var(--text-secondary)]">
              {running
                ? "This container is running. Use force to stop and remove it."
                : "This will delete the container."}
            </Dialog.Description>
            <div className="grid grid-cols-2 gap-2 mt-2">
              <Button variant="ghost" onClick={() => setOpen(false)}>Cancel</Button>
              <Button
                variant="destructive"
                onClick={() => {
                  setOpen(false);
                  onConfirm(running);
                }}
              >
                {running ? "Force remove" : "Remove"}
              </Button>
            </div>
          </Dialog.Content>
        </Dialog.Portal>
      </Dialog.Root>
    </>
  );
}
