import { useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import type { NetworkSummary } from "@/api/types";
import { useHosts } from "@/state/hosts";
import { useSession } from "@/state/session";
import { invalidate, useQuery } from "@/state/cache";
import { networkStatus } from "@/state/status";
import { Button } from "@/components/Button";
import { ConfirmDialog } from "@/components/ConfirmDialog";
import { Heading } from "@/components/Heading";
import { useHostCrumb } from "@/components/HostCrumbs";
import { Page, type Crumb } from "@/components/Page";
import { Row } from "@/components/Row";
import { Section } from "@/components/Section";
import { StatusBadge } from "@/components/StatusBadge";

export function NetworkDetail() {
  const { hid, nid } = useParams<{ hid: string; nid: string }>();
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };
  const session = useSession(host);

  const [confirmOpen, setConfirmOpen] = useState(false);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const queryKey = host && session.session ? `${host.url}:list_networks` : null;
  const { data: networks } = useQuery<NetworkSummary[]>(queryKey, async () => {
    const r = unwrap(await call(host!, { op: "list_networks" }), "networks");
    return r.data;
  });
  const network = networks?.find((n) => n.id === nid);

  async function onRemove() {
    if (!host || !nid) return;
    setPending(true);
    setError(null);
    try {
      unwrap(await call(host, { op: "remove_network", id: nid }), "ok");
      if (queryKey) invalidate(queryKey);
      nav(`/h/${hid}/networks`, { replace: true });
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setPending(false);
    }
  }

  const hostCrumb = useHostCrumb(hid ?? "", saved?.label ?? "?");

  if (!saved) return <Page><p>Unknown host.</p></Page>;

  const title = network?.name ?? nid ?? "?";
  const crumbs: Crumb[] = [
    hostCrumb,
    { kind: "link", label: "networks", to: `/h/${hid}/networks` },
    { kind: "link", label: title },
  ];
  const denyReason =
    session.session && !session.session.can("remove_network")
      ? "your token doesn't allow remove_network"
      : undefined;

  return (
    <Page crumbs={crumbs}>
      <Heading
        category="Network"
        title={title}
        right={network && <StatusBadge status={networkStatus(network.in_use)} />}
      />

      {!network && session.session && (
        <p className="text-xs text-[var(--text-tertiary)]">Loading…</p>
      )}

      {network && (
        <>
          <Section>
            <div className="flex flex-col gap-2">
              <Row label="ID" value={network.id} mono />
              <Row label="Name" value={network.name} />
              <Row label="Driver" value={network.driver} />
              {network.scope && <Row label="Scope" value={network.scope} />}
              <Row label="Internal" value={network.internal ? "yes" : "no"} />
              <Row label="Created" value={network.created} mono />
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
            title={`Remove network ${network.name}?`}
            confirmLabel="Remove"
            destructive
            onConfirm={onRemove}
          />
        </>
      )}
    </Page>
  );
}
