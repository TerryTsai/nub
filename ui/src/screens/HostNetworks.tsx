import { useNavigate, useParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import type { NetworkSummary } from "@/api/types";
import { useHosts } from "@/state/hosts";
import { useSession } from "@/state/session";
import { useQuery } from "@/state/cache";
import { networkStatus } from "@/state/status";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { ListRow } from "@/components/ListRow";
import { Page } from "@/components/Page";

export function HostNetworks() {
  const { hid } = useParams<{ hid: string }>();
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };
  const session = useSession(host);

  const queryKey = host && session.session ? `${host.url}:list_networks` : null;
  const { data: networks, error } = useQuery<NetworkSummary[]>(queryKey, async () => {
    const r = unwrap(await call(host!, { op: "list_networks" }), "networks");
    return r.data;
  });

  const crumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "networks");

  if (!saved || !hid) return <Page><p>Unknown host.</p></Page>;

  return (
    <Page crumbs={crumbs}>
      {session.loading && <p className="text-xs text-[var(--text-tertiary)]">Connecting…</p>}
      {session.error && <p className="text-[var(--error)] text-xs">Couldn't connect: {session.error}</p>}
      {error && <p className="text-[var(--error)] text-xs">{error}</p>}
      {session.session && networks === null && !error && (
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
