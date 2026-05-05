import { useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import * as Dialog from "@radix-ui/react-dialog";
import { call, unwrap, type Host } from "@/api/client";
import { useHosts } from "@/state/hosts";
import { invalidate, peek, useQuery } from "@/state/cache";
import type { Action, ContainerDetail as ContainerDetailT, ContainerSummary } from "@/api/types";
import { containerStatus } from "@/state/status";
import { Button } from "@/components/Button";
import { KvLine } from "@/components/KvLine";
import { useToast } from "@/components/Toaster";
import { Heading } from "@/components/Heading";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
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

  const [pending, setPending] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const toast = useToast();

  const queryKey = host && cid ? `${host.url}:inspect:${cid}` : null;
  const { data: detail, error: queryError, reload } = useQuery<ContainerDetailT>(queryKey, async () => {
    const r = unwrap(await call(host!, { op: "get_container", id: cid! }), "container_detail");
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

  const sectionCrumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "containers");

  if (!saved) {
    return <Page><p>Unknown host.</p></Page>;
  }

  const seedName = host && cid
    ? peek<ContainerSummary[]>(`${host.url}:list_containers`)?.find((c) => c.id === cid)?.name
    : undefined;
  const displayName = detail?.name || seedName || cid?.slice(0, 12) || "?";
  const crumbs: Crumb[] = [
    ...sectionCrumbs,
    { kind: "link", label: displayName },
  ];

  const subnav = detail ? (
    <>
      <Button
        size="sm"
        variant="ghost"
        disabled={pending !== null || detail.running}
        onClick={() => act("start", { kind: "start" })}
      >
        {pending === "start" ? "…" : "Start"}
      </Button>
      <Button
        size="sm"
        variant="ghost"
        disabled={pending !== null || !detail.running}
        onClick={() => act("stop", { kind: "stop" })}
      >
        {pending === "stop" ? "…" : "Stop"}
      </Button>
      <Button
        size="sm"
        variant="ghost"
        disabled={pending !== null || !detail.running}
        onClick={() => act("restart", { kind: "restart" })}
      >
        {pending === "restart" ? "…" : "Restart"}
      </Button>
      <span className="w-px h-4 bg-[var(--border-subtle)] mx-1 shrink-0" />
      <Link to={`/h/${hid}/c/${cid}/logs`}>
        <Button size="sm" variant="ghost">Logs</Button>
      </Link>
      <Link to={`/h/${hid}/c/${cid}/stats`}>
        <Button size="sm" variant="ghost">Stats</Button>
      </Link>
      <Link to={`/h/${hid}/c/${cid}/exec`} aria-disabled={!detail.running}>
        <Button
          size="sm"
          variant="ghost"
          disabled={!detail.running}
          title={!detail.running ? "container is not running" : undefined}
        >
          Exec
        </Button>
      </Link>
      <Button size="sm" variant="ghost" onClick={() => nav(`/h/${hid}/c/${cid}/clone`)}>
        Clone
      </Button>
    </>
  ) : undefined;

  return (
    <Page crumbs={crumbs} subnav={subnav}>
      <Heading
        category="Container"
        title={displayName}
        right={detail && <StatusBadge status={containerStatus(detail.state, detail.exit_code, detail.health)} />}
      />

      {error && <p className="text-[var(--error)] text-xs">{error}</p>}

      {detail && (
        <>
          <Section>
            <Row label="Image" value={detail.image} mono />
            {detail.network_mode && <Row label="Network" value={detail.network_mode} mono />}
            {detail.restart_policy && <Row label="Restart" value={detail.restart_policy} />}
            {detail.entrypoint.length > 0 && <Row label="Entrypoint" value={detail.entrypoint.join(" ")} mono />}
            {detail.cmd.length > 0 && <Row label="Cmd" value={detail.cmd.join(" ")} mono />}
            {detail.working_dir && <Row label="Working dir" value={detail.working_dir} mono />}
            {detail.user && <Row label="User" value={detail.user} mono />}
            <Row label="Created" value={detail.created} mono />
            {detail.started_at && <Row label="Started" value={detail.started_at} mono />}
            {detail.finished_at && <Row label="Finished" value={detail.finished_at} mono />}
            {detail.exit_code !== 0 && <Row label="Exit code" value={String(detail.exit_code)} />}
            {detail.restart_count > 0 && <Row label="Restarts" value={String(detail.restart_count)} />}
            {detail.health && <Row label="Health" value={detail.health} />}
          </Section>

          {detail.env.length > 0 && (
            <Section label={`env (${detail.env.length})`}>
              {detail.env.map((e, i) => {
                const eq = e.indexOf("=");
                const k = eq >= 0 ? e.slice(0, eq) : e;
                const v = eq >= 0 ? e.slice(eq + 1) : "";
                return <KvLine key={i} k={k} v={v} copyAs={e} />;
              })}
            </Section>
          )}

          <Section label="danger">
            <RemoveButton
              pending={pending}
              running={detail.running}
              onConfirm={(force) => act("remove", { kind: "remove", force }, () => nav(`/h/${hid}`))}
            />
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
  pending,
  running,
  onConfirm,
}: {
  pending: string | null;
  running: boolean;
  onConfirm: (force: boolean) => void;
}) {
  const [open, setOpen] = useState(false);
  return (
    <>
      <Button
        variant="destructive"
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
