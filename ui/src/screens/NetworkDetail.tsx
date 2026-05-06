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
import { EmptyRow } from "@/components/EmptyRow";
import { KvLine } from "@/components/KvLine";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { Page, type Crumb } from "@/components/Page";
import { Row } from "@/components/Row";
import { Spinner } from "@/components/Spinner";
import { useToast } from "@/components/Toaster";

export function NetworkDetail() {
  const { hid, nid } = useParams<{ hid: string; nid: string }>();
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };

  const [confirmOpen, setConfirmOpen] = useState(false);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const toast = useToast();

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
      const msg = (e as Error).message;
      setError(`remove: ${msg}`);
      toast.pushOpError("remove", e);
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

  const ipam0 = detail?.ipam[0];

  return (
    <Page crumbs={crumbs} subnav={undefined}>
      {error && <p className="text-[var(--error)] text-xs">{error}</p>}

      <Collapsible label="Network" defaultOpen>
        <Row label="Name" value={network?.name} mono />
        <Row label="Status" value={network ? networkStatus(network.in_use).label : undefined} />
        <Row label="ID" value={network?.id} mono />
        <Row label="Created" value={network?.created} mono />
      </Collapsible>

      <Collapsible label="spec">
        <Row label="Driver" value={network?.driver} />
        <Row label="Scope" value={network?.scope} />
        <Row label="Internal" value={network ? (network.internal ? "yes" : "no") : undefined} />
        <Row label="Subnet" value={ipam0?.subnet} mono />
        <Row label="Gateway" value={ipam0?.gateway} mono />
      </Collapsible>

      <Collapsible label="runtime">
        <Row label="Attached" value={attachedLabel(detail)} />
      </Collapsible>

      <Collapsible label="attached" count={detail?.containers.length}>
        {detail && detail.containers.length > 0
          ? detail.containers.map((c) => (
              <Row key={c.id} label={c.name || c.id.slice(0, 12)} value={c.ipv4 || c.ipv6} mono />
            ))
          : <EmptyRow />}
      </Collapsible>

      <Collapsible label="options" count={detail ? Object.keys(detail.options).length : undefined}>
        {detail && Object.keys(detail.options).length > 0
          ? Object.entries(detail.options).map(([k, v]) => (
              <KvLine key={k} k={k} v={v} copyAs={`${k}=${v}`} />
            ))
          : <EmptyRow />}
      </Collapsible>

      <Collapsible label="labels" count={detail ? Object.keys(detail.labels).length : undefined}>
        {detail && Object.keys(detail.labels).length > 0
          ? Object.entries(detail.labels).map(([k, v]) => (
              <KvLine key={k} k={k} v={v} copyAs={`${k}=${v}`} />
            ))
          : <EmptyRow />}
      </Collapsible>

      <Collapsible label="danger">
        <Button
          variant="destructive"
          disabled={!network || pending}
          onClick={() => setConfirmOpen(true)}
        >
          {pending ? <><Spinner /> Removing…</> : "Remove"}
        </Button>
      </Collapsible>

      {network && (
        <ConfirmDialog
          open={confirmOpen}
          onOpenChange={setConfirmOpen}
          title={`Remove network ${network.name}?`}
          confirmLabel="Remove"
          destructive
          onConfirm={onRemove}
        />
      )}
    </Page>
  );
}

function attachedLabel(detail: NetworkDetailT | null): string | undefined {
  if (!detail) return undefined;
  return `${detail.containers.length} container${detail.containers.length === 1 ? "" : "s"}`;
}
