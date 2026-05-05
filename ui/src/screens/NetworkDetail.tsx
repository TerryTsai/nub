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
import { KvLine } from "@/components/KvLine";
import { Heading } from "@/components/Heading";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { Page, type Crumb } from "@/components/Page";
import { Row } from "@/components/Row";
import { Section } from "@/components/Section";
import { Skeleton } from "@/components/Skeleton";
import { Spinner } from "@/components/Spinner";
import { StatusBadge } from "@/components/StatusBadge";
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

  return (
    <Page crumbs={crumbs}>
      <Heading
        category="Network"
        title={title}
        right={network && <StatusBadge status={networkStatus(network.in_use)} />}
      />

      {!network && (
        <Section>
          <Skeleton className="h-4 w-32" />
          <Skeleton className="h-3 w-full" />
          <Skeleton className="h-3 w-5/6" />
          <Skeleton className="h-3 w-4/6" />
          <Skeleton className="h-3 w-3/4" />
          <Skeleton className="h-3 w-2/3" />
        </Section>
      )}

      {network && (
        <>
          <Section>
            <Row label="ID" value={network.id} mono />
            <Row label="Name" value={network.name} mono />
            <Row label="Driver" value={network.driver} />
            {network.scope && <Row label="Scope" value={network.scope} />}
            <Row label="Internal" value={network.internal ? "yes" : "no"} />
            {detail?.ipam.map((c, i) => (
              <div key={i} className="contents">
                {c.subnet && <Row label="Subnet" value={c.subnet} mono />}
                {c.gateway && <Row label="Gateway" value={c.gateway} mono />}
              </div>
            ))}
            <Row label="Created" value={network.created} mono />
            {detail && (
              <Row
                label="Attached"
                value={`${detail.containers.length} container${detail.containers.length === 1 ? "" : "s"}`}
              />
            )}
          </Section>

          {detail && detail.containers.length > 0 && (
            <Collapsible label="attached" count={detail.containers.length}>
              {detail.containers.map((c) => (
                <Row key={c.id} label={c.name || c.id.slice(0, 12)} value={c.ipv4 || c.ipv6} mono />
              ))}
            </Collapsible>
          )}

          {detail && Object.keys(detail.options).length > 0 && (
            <Collapsible label="options" count={Object.keys(detail.options).length}>
              {Object.entries(detail.options).map(([k, v]) => (
                <KvLine key={k} k={k} v={v} copyAs={`${k}=${v}`} />
              ))}
            </Collapsible>
          )}

          {detail && Object.keys(detail.labels).length > 0 && (
            <Collapsible label="labels" count={Object.keys(detail.labels).length}>
              {Object.entries(detail.labels).map(([k, v]) => (
                <KvLine key={k} k={k} v={v} copyAs={`${k}=${v}`} />
              ))}
            </Collapsible>
          )}

          <Section label="danger">
            <Button
              variant="destructive"
              disabled={pending}
              onClick={() => setConfirmOpen(true)}
            >
              {pending ? <><Spinner /> Removing…</> : "Remove"}
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
