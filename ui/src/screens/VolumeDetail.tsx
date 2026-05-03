import { useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import type { VolumeDetail as VolumeDetailT, VolumeSummary } from "@/api/types";
import { useHosts } from "@/state/hosts";
import { useSession } from "@/state/session";
import { invalidate, useQuery } from "@/state/cache";
import { volumeStatus } from "@/state/status";
import { Button } from "@/components/Button";
import { Collapsible } from "@/components/Collapsible";
import { ConfirmDialog } from "@/components/ConfirmDialog";
import { Heading } from "@/components/Heading";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { Page, type Crumb } from "@/components/Page";
import { Row } from "@/components/Row";
import { Section } from "@/components/Section";
import { StatusBadge } from "@/components/StatusBadge";

export function VolumeDetail() {
  const { hid, vname } = useParams<{ hid: string; vname: string }>();
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };
  const session = useSession(host);

  const [confirmOpen, setConfirmOpen] = useState(false);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const queryKey = host && session.session ? `${host.url}:list_volumes` : null;
  const { data: volumes } = useQuery<VolumeSummary[]>(queryKey, async () => {
    const r = unwrap(await call(host!, { op: "list_volumes" }), "volumes");
    return r.data;
  });
  const volume = volumes?.find((v) => v.name === vname);

  const inspectKey = host && session.session && vname ? `${host.url}:get_volume:${vname}` : null;
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
      setError((e as Error).message);
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
  const denyReason =
    session.session && !session.session.can("volumes:delete")
      ? "your token doesn't allow volumes:delete"
      : undefined;

  return (
    <Page crumbs={crumbs}>
      <Heading
        category="Volume"
        title={title}
        right={volume && <StatusBadge status={volumeStatus(volume.in_use)} />}
      />

      {!volume && session.session && (
        <p className="text-xs text-[var(--text-tertiary)]">Loading…</p>
      )}

      {volume && (
        <>
          <Section>
            <Row label="Name" value={volume.name} mono />
            <Row label="Driver" value={volume.driver} />
            <Row label="Mountpoint" value={volume.mountpoint} mono />
            <Row label="Created" value={volume.created_at} mono />
            {detail && detail.ref_count >= 0 && (
              <Row
                label="In use by"
                value={`${detail.ref_count} container${detail.ref_count === 1 ? "" : "s"}`}
              />
            )}
            {detail && detail.size >= 0 && <Row label="Size" value={formatBytes(detail.size)} />}
          </Section>

          {detail && (volume.scope || Object.keys(detail.options).length > 0 || Object.keys(detail.labels).length > 0) && (
            <Collapsible label="spec">
              {volume.scope && <Row label="Scope" value={volume.scope} />}
              {Object.keys(detail.options).length > 0 && (
                <Row
                  label="Options"
                  right={
                    <pre className="text-xs mono whitespace-pre-wrap break-all text-[var(--id-color)] leading-5">
                      {Object.entries(detail.options).map(([k, v]) => `${k}=${v}`).join("\n")}
                    </pre>
                  }
                />
              )}
              {Object.keys(detail.labels).length > 0 && (
                <Row
                  label="Labels"
                  right={
                    <pre className="text-xs mono whitespace-pre-wrap break-all text-[var(--id-color)] leading-5">
                      {Object.entries(detail.labels).map(([k, v]) => `${k}=${v}`).join("\n")}
                    </pre>
                  }
                />
              )}
            </Collapsible>
          )}

          <Section label="ops">
            <Button
              variant="destructive"
              disallowReason={denyReason}
              disabled={pending}
              onClick={() => setConfirmOpen(true)}
            >
              {pending ? "…" : "Remove"}
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
        </>
      )}
    </Page>
  );
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
