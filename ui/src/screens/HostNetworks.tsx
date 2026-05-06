import { useNavigate, useParams, useSearchParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import type { NetworkSummary } from "@/api/types";
import { useHosts } from "@/state/hosts";
import { useQuery } from "@/state/cache";
import { networkStatus } from "@/state/status";
import { FAB } from "@/components/FAB";
import { Filters } from "@/components/Filters";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { ListRow } from "@/components/ListRow";
import { Page } from "@/components/Page";
import { LoadingBar } from "@/components/LoadingBar";
import { relativeDate } from "@/lib/relativeDate";

type NetworkFilter = "all" | "in-use" | "idle";
const NETWORK_FILTERS: NetworkFilter[] = ["all", "in-use", "idle"];

function asNetworkFilter(s: string | null): NetworkFilter {
  return NETWORK_FILTERS.includes(s as NetworkFilter) ? (s as NetworkFilter) : "all";
}

function matchesNetworkFilter(n: NetworkSummary, f: NetworkFilter): boolean {
  if (f === "all") return true;
  if (f === "in-use") return n.in_use;
  return !n.in_use;
}

export function HostNetworks() {
  const { hid } = useParams<{ hid: string }>();
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };

  const [params, setParams] = useSearchParams();
  const filter = asNetworkFilter(params.get("filter"));
  const setFilter = (v: NetworkFilter) => {
    if (v === "all") setParams({}, { replace: true });
    else setParams({ filter: v }, { replace: true });
  };

  const queryKey = host ? `${host.url}:list_networks` : null;
  const { data: networks, error } = useQuery<NetworkSummary[]>(queryKey, async () => {
    const r = unwrap(await call(host!, { op: "list_networks" }), "networks");
    return r.data;
  });

  const crumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "networks");

  if (!saved || !hid) return <Page><p>Unknown host.</p></Page>;

  const subnav = networks !== null ? (
    <Filters
      attribute="in use"
      value={filter}
      onChange={setFilter}
      options={[
        { value: "all", label: "All" },
        { value: "in-use", label: "In use" },
        { value: "idle", label: "Idle" },
      ]}
    />
  ) : undefined;

  const visible = networks?.filter((n) => matchesNetworkFilter(n, filter)) ?? null;

  return (
    <Page crumbs={crumbs} subnav={subnav} fab={<FAB to={`/h/${hid}/networks/new`} label="network" />}>
      {error && <p className="text-[var(--error)] text-xs">{error}</p>}
      {networks === null && !error && <LoadingBar />}
      {networks?.length === 0 && (
        <p className="text-xs text-[var(--text-tertiary)]">No networks.</p>
      )}
      {visible && visible.length === 0 && networks && networks.length > 0 && (
        <p className="text-xs text-[var(--text-tertiary)]">No matches.</p>
      )}
      {visible && visible.length > 0 && (
        <div className="flex flex-col">
          {visible.map((n) => (
            <ListRow
              key={n.id}
              title={n.name}
              mono
              subtitle={relativeDate(n.created)}
              status={networkStatus(n.in_use)}
              onPress={() => nav(`/h/${hid}/networks/${n.id}`)}
            />
          ))}
        </div>
      )}
    </Page>
  );
}
