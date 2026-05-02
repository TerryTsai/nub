import { useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import type { VolumeSummary } from "@/api/types";
import { useHosts } from "@/state/hosts";
import { useSession } from "@/state/session";
import { invalidate, useQuery } from "@/state/cache";
import { volumeStatus } from "@/state/status";
import { Button } from "@/components/Button";
import { ConfirmDialog } from "@/components/ConfirmDialog";
import { Heading } from "@/components/Heading";
import { useHostCrumb } from "@/components/HostCrumbs";
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

  async function onRemove(force: boolean) {
    if (!host || !vname) return;
    setPending(true);
    setError(null);
    try {
      unwrap(await call(host, { op: "remove_volume", name: vname, force }), "ok");
      if (queryKey) invalidate(queryKey);
      nav(`/h/${hid}/volumes`, { replace: true });
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setPending(false);
    }
  }

  const hostCrumb = useHostCrumb(hid ?? "", saved?.label ?? "?");

  if (!saved) return <Page><p>Unknown host.</p></Page>;

  const title = vname ?? "?";
  const shortTitle = title.length > 16 ? `${title.slice(0, 12)}…` : title;
  const crumbs: Crumb[] = [
    hostCrumb,
    { kind: "link", label: "volumes", to: `/h/${hid}/volumes` },
    { kind: "link", label: shortTitle },
  ];
  const denyReason =
    session.session && !session.session.can("remove_volume")
      ? "your token doesn't allow remove_volume"
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
            <div className="flex flex-col gap-2">
              <Row label="Name" value={volume.name} mono />
              <Row label="Driver" value={volume.driver} />
              {volume.scope && <Row label="Scope" value={volume.scope} />}
              <Row label="Mountpoint" value={volume.mountpoint} mono />
              <Row label="Created" value={volume.created_at} mono />
            </div>
          </Section>

          <Section label="Actions">
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
