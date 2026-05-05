import { Link, useNavigate, useParams, useSearchParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import { useHosts } from "@/state/hosts";
import { useQuery } from "@/state/cache";
import type { ContainerSummary } from "@/api/types";
import { containerStatus } from "@/state/status";
import { FAB } from "@/components/FAB";
import { Filters } from "@/components/Filters";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { ListRow } from "@/components/ListRow";
import { Page } from "@/components/Page";
import { SkeletonRows } from "@/components/Skeleton";

type ContainerFilter = "all" | "running" | "stopped";

const FILTER_VALUES: ContainerFilter[] = ["all", "running", "stopped"];

function asFilter(s: string | null): ContainerFilter {
  return FILTER_VALUES.includes(s as ContainerFilter) ? (s as ContainerFilter) : "all";
}

function matchesFilter(state: string, filter: ContainerFilter): boolean {
  if (filter === "all") return true;
  if (filter === "running") return state === "running";
  return state !== "running";
}

export function HostHome() {
  const { hid } = useParams<{ hid: string }>();
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };

  const [params, setParams] = useSearchParams();
  const filter = asFilter(params.get("filter"));
  const setFilter = (v: ContainerFilter) => {
    if (v === "all") setParams({}, { replace: true });
    else setParams({ filter: v }, { replace: true });
  };

  const queryKey = host ? `${host.url}:list_containers` : null;
  const { data: containers, error } = useQuery<ContainerSummary[]>(queryKey, async () => {
    const r = unwrap(await call(host!, { op: "list_containers", all: true }), "containers");
    return r.data;
  });

  const crumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "containers");

  if (!saved) {
    return (
      <Page>
        <p>Unknown host. <Link to="/" className="underline">Back to hosts</Link></p>
      </Page>
    );
  }

  const subnav = containers !== null ? (
    <Filters
      value={filter}
      onChange={setFilter}
      options={[
        { value: "all", label: "All", count: containers.length },
        { value: "running", label: "Running", count: countRunning(containers) },
        { value: "stopped", label: "Stopped", count: containers.length - countRunning(containers) },
      ]}
    />
  ) : undefined;

  return (
    <Page crumbs={crumbs} subnav={subnav} fab={<FAB to={`/h/${hid}/c/new`} label="container" />}>
      {error && <p className="text-[var(--error)] text-xs">{error}</p>}
      {containers === null && !error && <SkeletonRows count={5} />}
      {containers !== null && containers.length === 0 && (
        <p className="text-xs text-[var(--text-tertiary)]">No containers.</p>
      )}
      {containers !== null && containers.length > 0 && (
        <ContainerList
          containers={containers.filter((c) => matchesFilter(c.state, filter))}
          onPick={(id) => nav(`/h/${hid}/c/${id}`)}
        />
      )}
    </Page>
  );
}

function countRunning(cs: ContainerSummary[]): number {
  return cs.filter((c) => c.state === "running").length;
}

function ContainerList({
  containers,
  onPick,
}: {
  containers: ContainerSummary[];
  onPick: (id: string) => void;
}) {
  if (containers.length === 0) {
    return <p className="text-xs text-[var(--text-tertiary)]">No matches.</p>;
  }
  return (
    <div className="flex flex-col -mx-1">
      {containers.map((c) => {
        const stack = c.labels?.["nub.stack"];
        const subtitle = stack
          ? `stack:${stack} · ${c.image} · ${c.status}`
          : `${c.image} · ${c.status}`;
        return (
          <div key={c.id} className="px-1">
            <ListRow
              title={c.name || "(unnamed)"}
              subtitle={subtitle}
              status={containerStatus(c.state, c.exit_code, c.health)}
              onPress={() => onPick(c.id)}
            />
          </div>
        );
      })}
    </div>
  );
}
