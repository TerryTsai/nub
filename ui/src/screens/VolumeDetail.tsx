import { useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import type { VolumeDetail as VolumeDetailT, VolumeSummary } from "@/api/types";
import { useHosts } from "@/state/hosts";
import { invalidate, useQuery } from "@/state/cache";
import { volumeStatus } from "@/state/status";
import { Button } from "@/components/Button";
import { Collapsible } from "@/components/Collapsible";
import { ConfirmDialog } from "@/components/ConfirmDialog";
import { KvLine } from "@/components/KvLine";
import { Heading } from "@/components/Heading";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { Page, type Crumb } from "@/components/Page";
import { Row } from "@/components/Row";
import { Section } from "@/components/Section";
import { Spinner } from "@/components/Spinner";
import { StatusBadge } from "@/components/StatusBadge";
import { useToast } from "@/components/Toaster";

export function VolumeDetail() {
  const { hid, vname } = useParams<{ hid: string; vname: string }>();
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };

  const [confirmOpen, setConfirmOpen] = useState(false);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const toast = useToast();

  const queryKey = host ? `${host.url}:list_volumes` : null;
  const { data: volumes } = useQuery<VolumeSummary[]>(queryKey, async () => {
    const r = unwrap(await call(host!, { op: "list_volumes" }), "volumes");
    return r.data;
  });
  const volume = volumes?.find((v) => v.name === vname);

  const inspectKey = host && vname ? `${host.url}:get_volume:${vname}` : null;
  const { data: detail } = useQuery<VolumeDetailT>(inspectKey, async () => {
    const r = unwrap(await call(host!, { op: "get_volume", name: vname! }), "volume_detail");
    return r.data;
  });

  async function onRemove(force: boolean) {
    if (!host || !vname) return;
    setPending(true);
    setError(null);
    try {
      unwrap(await call(host, { op: "delete_volume", name: vname, force }), "ok");
      if (queryKey) invalidate(queryKey);
      nav(`/h/${hid}/volumes`, { replace: true });
    } catch (e) {
      const msg = (e as Error).message;
      setError(`remove: ${msg}`);
      toast.pushOpError("remove", e);
    } finally {
      setPending(false);
    }
  }

  const sectionCrumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "volumes");

  if (!saved) return <Page><p>Unknown host.</p></Page>;

  const title = vname ?? "?";
  const shortTitle = title.length > 16 ? `${title.slice(0, 12)}…` : title;
  const crumbs: Crumb[] = [
    ...sectionCrumbs,
    { kind: "link", label: shortTitle },
  ];

  return (
    <Page crumbs={crumbs}>
      <Heading
        category="Volume"
        title={title}
        right={<StatusBadge status={volume ? volumeStatus(volume.in_use) : null} />}
      />

      {/* meta: identity + timestamps */}
      <Section>
        <Row label="Name" value={volume?.name} mono />
        <Row label="Created" value={volume?.created_at} mono />
      </Section>

      <Collapsible label="spec">
        <Row label="Driver" value={volume?.driver} />
        <Row label="Mountpoint" value={volume?.mountpoint} mono />
        <Row label="Scope" value={volume?.scope} />
        <Row label="Size" value={sizeLabel(detail)} />
      </Collapsible>

      <Collapsible label="runtime">
        <Row label="In use by" value={refCountLabel(detail)} />
      </Collapsible>

      <Collapsible label="options" count={detail ? Object.keys(detail.options).length : undefined}>
        {!detail || Object.keys(detail.options).length === 0 ? (
          <p className="text-xs text-[var(--text-tertiary)]">—</p>
        ) : (
          Object.entries(detail.options).map(([k, v]) => (
            <KvLine key={k} k={k} v={v} copyAs={`${k}=${v}`} />
          ))
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
        <Button
          variant="destructive"
          disabled={!volume || pending}
          onClick={() => setConfirmOpen(true)}
        >
          {pending ? <><Spinner /> Removing…</> : "Remove"}
        </Button>
        {error && <p className="text-[var(--error)] text-xs">{error}</p>}
      </Section>

      <ConfirmDialog
        open={confirmOpen}
        onOpenChange={setConfirmOpen}
        title={`Remove volume ${shortTitle}?`}
        description="Volume contents are deleted. This cannot be undone."
        confirmLabel="Remove"
        destructive
        onConfirm={() => onRemove(false)}
      />
    </Page>
  );
}

function sizeLabel(detail: VolumeDetailT | null): string | undefined {
  if (!detail) return undefined;
  if (detail.size < 0) return undefined;
  return formatBytes(detail.size);
}

function refCountLabel(detail: VolumeDetailT | null): string | undefined {
  if (!detail) return undefined;
  if (detail.ref_count < 0) return undefined;
  return `${detail.ref_count} container${detail.ref_count === 1 ? "" : "s"}`;
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
