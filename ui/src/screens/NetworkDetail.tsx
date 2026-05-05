import { useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import type { NetworkDetail as NetworkDetailT, NetworkSummary } from "@/api/types";
import { useHosts } from "@/state/hosts";
import { invalidate, useQuery } from "@/state/cache";
import { networkStatus } from "@/state/status";
import { Button } from "@/components/Button";
import { Collapsible } from "@/components/Collapsible";
import { ConfirmDialog } from "@/components/ConfirmDialog";
import { Heading } from "@/components/Heading";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
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

  const [confirmOpen, setConfirmOpen] = useState(false);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const queryKey = host ? `${host.url}:list_networks` : null;
  const { data: networks } = useQuery<NetworkSummary[]>(queryKey, async () => {
    const r = unwrap(await call(host!, { op: "list_networks" }), "networks");
    return r.data;
  });
  const network = networks?.find((n) => n.id === nid);

  const inspectKey = host && nid ? `${host.url}:get_network:${nid}` : null;
  const { data: detail } = useQuery<NetworkDetailT>(inspectKey, async () => {
    const r = unwrap(await call(host!, { op: "get_network", id: nid! }), "network_detail");
    return r.data;
  });

  async function onRemove() {
    if (!host || !nid) return;
    setPending(true);
    setError(null);
    try {
      unwrap(await call(host, { op: "delete_network", id: nid }), "ok");
      if (queryKey) invalidate(queryKey);
      nav(`/h/${hid}/networks`, { replace: true });
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setPending(false);
    }
  }

  const sectionCrumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "networks");

  if (!saved) return <Page><p>Unknown host.</p></Page>;

  const title = network?.name ?? nid ?? "?";
  const crumbs: Crumb[] = [
    ...sectionCrumbs,
    { kind: "link", label: title },
  ];

  return (
    <Page crumbs={crumbs}>
      <Heading
        category="Network"
        title={title}
        right={network && <StatusBadge status={networkStatus(network.in_use)} />}
      />

      {!network && (
        <p className="text-xs text-[var(--text-tertiary)]">Loading…</p>
      )}

      {network && (
        <>
          <Section>
            <Row label="Name" value={network.name} />
            <Row label="Driver" value={network.driver} />
            <Row label="Internal" value={network.internal ? "yes" : "no"} />
            <Row label="Created" value={network.created} mono />
            {detail && (
              <Row
                label="Attached"
                value={`${detail.containers.length} container${detail.containers.length === 1 ? "" : "s"}`}
              />
            )}
          </Section>

          {detail && detail.containers.length > 0 && (
            <Section label="attached">
              {detail.containers.map((c) => (
                <Row key={c.id} label={c.name || c.id.slice(0, 12)} value={c.ipv4 || c.ipv6} mono />
              ))}
            </Section>
          )}

          {detail && (
            <Collapsible label="spec">
              <Row label="ID" value={network.id} mono />
              {network.scope && <Row label="Scope" value={network.scope} />}
              {detail.ipam.map((c, i) => (
                <div key={i} className="contents">
                  {c.subnet && <Row label="Subnet" value={c.subnet} mono />}
                  {c.gateway && <Row label="Gateway" value={c.gateway} mono />}
                </div>
              ))}
            </Collapsible>
          )}

          <Section label="ops">
            <Button
              variant="destructive"
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
