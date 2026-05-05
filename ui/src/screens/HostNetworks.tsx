import { useNavigate, useParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import type { NetworkSummary } from "@/api/types";
import { useHosts } from "@/state/hosts";
import { useQuery } from "@/state/cache";
import { networkStatus } from "@/state/status";
import { FAB } from "@/components/FAB";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { ListRow } from "@/components/ListRow";
import { Page } from "@/components/Page";

export function HostNetworks() {
  const { hid } = useParams<{ hid: string }>();
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };

  const queryKey = host ? `${host.url}:list_networks` : null;
  const { data: networks, error } = useQuery<NetworkSummary[]>(queryKey, async () => {
    const r = unwrap(await call(host!, { op: "list_networks" }), "networks");
    return r.data;
  });

  const crumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "networks");

  if (!saved || !hid) return <Page><p>Unknown host.</p></Page>;

  return (
    <Page crumbs={crumbs} fab={<FAB to={`/h/${hid}/networks/new`} label="network" />}>
      {error && <p className="text-[var(--error)] text-xs">{error}</p>}
      {networks === null && !error && (
        <p className="text-xs text-[var(--text-tertiary)]">Loading networks…</p>
      )}
      {networks?.length === 0 && (
        <p className="text-xs text-[var(--text-tertiary)]">No networks.</p>
      )}
      {networks && networks.length > 0 && (
        <div className="flex flex-col -mx-1">
          {networks.map((n) => (
            <div key={n.id} className="px-1">
              <ListRow
                title={n.name}
                subtitle={`${n.driver}${n.scope ? ` · ${n.scope}` : ""}${n.internal ? " · internal" : ""}`}
                status={networkStatus(n.in_use)}
                onPress={() => nav(`/h/${hid}/networks/${n.id}`)}
              />
            </div>
          ))}
        </div>
      )}
    </Page>
  );
}
