import { useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import * as Dialog from "@radix-ui/react-dialog";
import { call, unwrap, type Host } from "@/api/client";
import { useHosts } from "@/state/hosts";
import { invalidate, peek, useQuery } from "@/state/cache";
import type { Action, ContainerDetail as ContainerDetailT, ContainerSummary, PortMapping } from "@/api/types";
import { containerStatus } from "@/state/status";
import { ActionMenu } from "@/components/ActionMenu";
import { Button } from "@/components/Button";
import { Collapsible } from "@/components/Collapsible";
import { KvLine } from "@/components/KvLine";
import { useToast } from "@/components/Toaster";
import { Heading } from "@/components/Heading";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { Page, type Crumb } from "@/components/Page";
import { Row } from "@/components/Row";
import { Section } from "@/components/Section";
import { Spinner } from "@/components/Spinner";
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
      setActionError(`${name}: ${msg}`);
      toast.pushOpError(name, e);
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

  const running = detail?.running ?? false;
  const actionsDisabled = !detail || pending !== null;
  const subnav = (
    <>
      <Button
        size="sm"
        variant="ghost"
        disabled={actionsDisabled || running}
        onClick={() => act("start", { kind: "start" })}
      >
        {pending === "start" ? <Spinner /> : "Start"}
      </Button>
      <Button
        size="sm"
        variant="ghost"
        disabled={actionsDisabled || !running}
        onClick={() => act("stop", { kind: "stop" })}
      >
        {pending === "stop" ? <Spinner /> : "Stop"}
      </Button>
      <Button
        size="sm"
        variant="ghost"
        disabled={actionsDisabled || !running}
        onClick={() => act("restart", { kind: "restart" })}
      >
        {pending === "restart" ? <Spinner /> : "Restart"}
      </Button>
      <ActionMenu
        items={[
          { label: "Logs", to: `/h/${hid}/c/${cid}/logs`, disabled: !detail },
          { label: "Stats", to: `/h/${hid}/c/${cid}/stats`, disabled: !detail },
          { label: "Exec", to: `/h/${hid}/c/${cid}/exec`, disabled: !detail || !running },
          { label: "Clone", to: `/h/${hid}/c/${cid}/clone`, disabled: !detail },
        ]}
      />
    </>
  );

  return (
    <Page crumbs={crumbs} subnav={subnav}>
      <Heading
        category="Container"
        title={displayName}
        right={<StatusBadge status={detail ? containerStatus(detail.state, detail.exit_code, detail.health) : null} />}
      />

      {error && <p className="text-[var(--error)] text-xs">{error}</p>}

      {/* meta: identity + timestamps */}
      <Section>
        <Row label="ID" value={detail?.id} mono />
        <Row label="Image" value={detail?.image} mono />
        <Row label="Created" value={detail?.created} mono />
      </Section>

      <Collapsible label="spec">
        <Row label="Entrypoint" value={detail?.entrypoint.join(" ")} mono />
        <Row label="Cmd" value={detail?.cmd.join(" ")} mono />
        <Row label="Working dir" value={detail?.working_dir} mono />
        <Row label="User" value={detail?.user} mono />
        <Row label="Network" value={detail?.network_mode} mono />
        <Row label="Restart" value={detail?.restart_policy} />
        <Row label="Privileged" value={detail ? (detail.privileged ? "yes" : "no") : undefined} />
        <Row label="Memory" value={memoryLabel(detail)} />
      </Collapsible>

      <Collapsible label="runtime">
        <Row label="Health" value={detail?.health} />
        <Row label="Exit code" value={exitCodeLabel(detail)} />
        <Row label="Restarts" value={detail ? String(detail.restart_count) : undefined} />
        <Row label="Started" value={detail?.started_at} mono />
        <Row label="Finished" value={detail?.finished_at} mono />
      </Collapsible>

      <Collapsible label="ports" count={detail?.ports.length}>
        {!detail || detail.ports.length === 0 ? (
          <p className="text-xs text-[var(--text-tertiary)]">—</p>
        ) : (
          detail.ports.map((p, i) => (
            <KvLine
              key={i}
              k={p.container_port}
              v={formatHostBinding(p)}
              copyAs={`${p.container_port} → ${formatHostBinding(p)}`}
            />
          ))
        )}
      </Collapsible>

      <Collapsible label="volumes" count={detail?.mounts.length}>
        {!detail || detail.mounts.length === 0 ? (
          <p className="text-xs text-[var(--text-tertiary)]">—</p>
        ) : (
          detail.mounts.map((m, i) => (
            <KvLine
              key={i}
              k={m.destination}
              v={`${m.source}${m.rw ? "" : " (ro)"}${m.kind === "tmpfs" ? " (tmpfs)" : ""}`}
              copyAs={`${m.source}:${m.destination}${m.rw ? "" : ":ro"}`}
            />
          ))
        )}
      </Collapsible>

      <Collapsible label="networks" count={detail ? Object.keys(detail.networks).length : undefined}>
        {!detail || Object.keys(detail.networks).length === 0 ? (
          <p className="text-xs text-[var(--text-tertiary)]">—</p>
        ) : (
          Object.entries(detail.networks).map(([name, ep]) => (
            <KvLine key={name} k={name} v={ep.ip_address || "—"} />
          ))
        )}
      </Collapsible>

      <Collapsible label="env" count={detail?.env.length}>
        {!detail || detail.env.length === 0 ? (
          <p className="text-xs text-[var(--text-tertiary)]">—</p>
        ) : (
          detail.env.map((e, i) => {
            const eq = e.indexOf("=");
            const k = eq >= 0 ? e.slice(0, eq) : e;
            const v = eq >= 0 ? e.slice(eq + 1) : "";
            return <KvLine key={i} k={k} v={v} copyAs={e} />;
          })
        )}
      </Collapsible>

      <Collapsible label="labels" count={detail ? Object.keys(detail.labels).length : undefined}>
        {!detail || Object.keys(detail.labels).length === 0 ? (
          <p className="text-xs text-[var(--text-tertiary)]">—</p>
        ) : (
          Object.entries(detail.labels).map(([k, v]) => (
            <KvLine key={k} k={k} v={v} copyAs={`${k}=${v}`} />
          ))
        )}
      </Collapsible>

      <Section label="danger">
        <RemoveButton
          pending={pending}
          running={detail?.running ?? false}
          disabled={!detail}
          onConfirm={(force) => act("remove", { kind: "remove", force }, () => nav(`/h/${hid}`))}
        />
      </Section>
    </Page>
  );
}

function memoryLabel(detail: ContainerDetailT | null): string | undefined {
  if (!detail) return undefined;
  return detail.memory_limit > 0 ? formatBytes(detail.memory_limit) : "no limit";
}

function exitCodeLabel(detail: ContainerDetailT | null): string | undefined {
  if (!detail) return undefined;
  // Running containers don't have a meaningful exit code yet — render as
  // the empty placeholder rather than a misleading "0".
  if (detail.running) return "";
  return String(detail.exit_code);
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

// "0.0.0.0:8080" → "8080"; "127.0.0.1:8080" → "127.0.0.1:8080";
// "" → "(not published)".
function formatHostBinding(p: PortMapping): string {
  if (!p.host_port) return "(not published)";
  if (!p.host_ip || p.host_ip === "0.0.0.0" || p.host_ip === "::") return p.host_port;
  return `${p.host_ip}:${p.host_port}`;
}

function RemoveButton({
  pending,
  running,
  disabled,
  onConfirm,
}: {
  pending: string | null;
  running: boolean;
  disabled?: boolean;
  onConfirm: (force: boolean) => void;
}) {
  const [open, setOpen] = useState(false);
  return (
    <>
      <Button
        variant="destructive"
        disabled={disabled || pending !== null}
        onClick={() => setOpen(true)}
      >
        {pending === "remove" ? <><Spinner /> Removing…</> : "Remove"}
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
