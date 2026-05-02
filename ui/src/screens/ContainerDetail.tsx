import { useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import * as Dialog from "@radix-ui/react-dialog";
import { call, unwrap, type Host } from "@/api/client";
import { useHosts } from "@/state/hosts";
import { useSession } from "@/state/session";
import { invalidate, peek, useQuery } from "@/state/cache";
import type { Action, ContainerDetail as ContainerDetailT, ContainerSummary } from "@/api/types";
import { containerStatus } from "@/state/status";
import { Button } from "@/components/Button";
import { useToast } from "@/components/Toaster";
import { Heading } from "@/components/Heading";
import { useHostCrumb } from "@/components/HostCrumbs";
import { Page, type Crumb } from "@/components/Page";
import { Row } from "@/components/Row";
import { Section } from "@/components/Section";
import { StatusBadge } from "@/components/StatusBadge";

export function ContainerDetail() {
  const { hid, cid } = useParams<{ hid: string; cid: string }>();
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };
  const session = useSession(host);

  const [pending, setPending] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const toast = useToast();

  const queryKey = host && session.session && cid ? `${host.url}:inspect:${cid}` : null;
  const { data: detail, error: queryError, reload } = useQuery<ContainerDetailT>(queryKey, async () => {
    const r = unwrap(await call(host!, { op: "inspect_container", id: cid! }), "container_detail");
    return r.data;
  });
  const error = actionError ?? queryError;

  async function act(name: string, action: Action, after?: () => void) {
    if (!host || !cid) return;
    setPending(name);
    setActionError(null);
    try {
      unwrap(await call(host, { op: "container_action", id: cid, action }), "ok");
      invalidate(`${host.url}:list_containers`);
      toast.push(`${pastTense(name)} ${detail?.name || cid.slice(0, 12)}`, "success");
      if (after) after();
      else reload();
    } catch (e) {
      const msg = (e as Error).message;
      setActionError(msg);
      toast.push(msg, "error");
    } finally {
      setPending(null);
    }
  }

  const hostCrumb = useHostCrumb(hid ?? "", saved?.label ?? "?");

  if (!saved) {
    return <Page><p>Unknown host.</p></Page>;
  }

  const seedName = host && cid
    ? peek<ContainerSummary[]>(`${host.url}:list_containers`)?.find((c) => c.id === cid)?.name
    : undefined;
  const displayName = detail?.name || seedName || cid?.slice(0, 12) || "?";
  const crumbs: Crumb[] = [
    hostCrumb,
    { kind: "link", label: displayName },
  ];
  const can = (op: string) => session.session?.can(op) ?? false;
  const denyReason = (op: string) =>
    session.session && !can(op) ? `your token doesn't allow ${op}` : undefined;

  return (
    <Page crumbs={crumbs}>
      <Heading
        category="Container"
        title={displayName}
        right={detail && <StatusBadge status={containerStatus(detail.state, detail.exit_code, detail.health)} />}
      />

      {session.loading && <p className="text-[var(--text-secondary)] text-sm">Connecting…</p>}
      {session.error && <p className="text-[var(--error)] text-sm">{session.error}</p>}

      {detail && (
        <>
          <Section>
            <div className="flex flex-col gap-2">
              <Row label="Image" value={detail.image} mono />
              <Row label="Created" value={detail.created} mono />
              {detail.started_at && <Row label="Started" value={detail.started_at} mono />}
              {detail.finished_at && <Row label="Finished" value={detail.finished_at} mono />}
              {detail.exit_code !== 0 && <Row label="Exit code" value={String(detail.exit_code)} />}
              {detail.restart_count > 0 && <Row label="Restarts" value={String(detail.restart_count)} />}
              {detail.health && <Row label="Health" value={detail.health} />}
              {detail.network_mode && <Row label="Network" value={detail.network_mode} />}
              {detail.restart_policy && <Row label="Restart policy" value={detail.restart_policy} />}
            </div>
          </Section>

          {detail.cmd.length > 0 && (
            <Section label="Process">
              <div className="flex flex-col gap-2">
                <Row label="Cmd" value={detail.cmd.join(" ")} mono />
                {detail.entrypoint.length > 0 && <Row label="Entrypoint" value={detail.entrypoint.join(" ")} mono />}
                {detail.working_dir && <Row label="Working dir" value={detail.working_dir} mono />}
                {detail.user && <Row label="User" value={detail.user} mono />}
              </div>
            </Section>
          )}

          {detail.env.length > 0 && (
            <Section label="Environment">
              <pre className="text-xs mono whitespace-pre-wrap break-all text-[var(--text-secondary)]">
                {detail.env.join("\n")}
              </pre>
            </Section>
          )}

          <Section label="View">
            <div className="grid grid-cols-3 gap-2">
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
              <Link to={`/h/${hid}/c/${cid}/exec`}>
                <Button
                  variant="ghost"
                  className="w-full"
                  disallowReason={
                    !detail.running ? "container is not running" : denyReason("exec")
                  }
                >
                  Exec →
                </Button>
              </Link>
            </div>
          </Section>

          <Section label="Actions">
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
            {error && <p className="text-[var(--error)] text-xs">{error}</p>}
          </Section>
        </>
      )}
    </Page>
  );
}

function pastTense(name: string): string {
  switch (name) {
    case "start":   return "started";
    case "stop":    return "stopped";
    case "restart": return "restarted";
    case "remove":  return "removed";
    case "kill":    return "killed";
    default:        return name;
  }
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
          <Dialog.Overlay className="fixed inset-0 bg-black/60" />
          <Dialog.Content className="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 w-[min(92vw,360px)] bg-[var(--bg-base)] border border-[var(--border-subtle)] rounded-[var(--radius-lg)] p-5 flex flex-col gap-3 z-50">
            <Dialog.Title className="text-base font-semibold font-display">Remove container?</Dialog.Title>
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
