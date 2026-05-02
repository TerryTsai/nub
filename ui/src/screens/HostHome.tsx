import { useEffect, useState } from "react";
import { Link, useNavigate, useParams, useSearchParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import { useHosts } from "@/state/hosts";
import { useSession } from "@/state/session";
import type { ContainerSummary } from "@/api/types";
import { Button } from "@/components/Button";
import { FAB } from "@/components/FAB";
import { Filters } from "@/components/Filters";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { ListRow } from "@/components/ListRow";
import { Page } from "@/components/Page";

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

  const session = useSession(host);
  const [containers, setContainers] = useState<ContainerSummary[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [params, setParams] = useSearchParams();
  const filter = asFilter(params.get("filter"));
  const setFilter = (v: ContainerFilter) => {
    if (v === "all") setParams({}, { replace: true });
    else setParams({ filter: v }, { replace: true });
  };

  async function refresh() {
    if (!host) return;
    setError(null);
    try {
      const r = unwrap(await call(host, { op: "list_containers", all: true }), "containers");
      setContainers(r.data);
    } catch (e) {
      setError((e as Error).message);
    }
  }

  useEffect(() => {
    if (host && session.session) refresh();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [hid, session.session]);

  const crumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "containers");

  if (!saved) {
    return (
      <Page>
        <p>Unknown host. <Link to="/" className="underline">Back to hosts</Link></p>
      </Page>
    );
  }

  const canCreate = session.session?.can("create_container") ?? false;

  const subnav = session.session && containers !== null ? (
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
    <Page
      crumbs={crumbs}
      subnav={subnav}
      fab={session.session && canCreate ? <FAB to={`/h/${hid}/run`} label="container" /> : undefined}
    >
      {session.loading && <p className="text-[var(--text-secondary)] text-sm">Connecting…</p>}

      {session.error && (
        <>
          <p className="text-[var(--error)] text-sm">Couldn't connect: {session.error}</p>
          <p className="text-xs text-[var(--text-tertiary)]">
            The token may have rotated (admin tokens regenerate on every nub restart).
          </p>
          <Link to="/add" className="self-start"><Button variant="ghost">Re-add host</Button></Link>
        </>
      )}

      {session.session && (
        <>
          {error && <p className="text-[var(--error)] text-xs">{error}</p>}
          {containers === null && !error && (
            <p className="text-xs text-[var(--text-tertiary)]">Loading…</p>
          )}
          {containers !== null && containers.length === 0 && (
            <p className="text-xs text-[var(--text-tertiary)]">No containers.</p>
          )}
          {containers !== null && containers.length > 0 && (
            <ContainerList
              containers={containers.filter((c) => matchesFilter(c.state, filter))}
              onPick={(id) => nav(`/h/${hid}/c/${id}`)}
            />
          )}
        </>
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
      {containers.map((c) => (
        <div key={c.id} className="px-1">
          <ListRow
            title={c.name || "(unnamed)"}
            subtitle={`${c.image} · ${c.status}`}
            status={c.state}
            onPress={() => onPick(c.id)}
          />
        </div>
      ))}
    </div>
  );
}
