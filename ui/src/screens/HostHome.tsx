import { Link, useNavigate, useParams, useSearchParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import { useHosts } from "@/state/hosts";
import { useSession, type Session } from "@/state/session";
import { invalidate, useQuery } from "@/state/cache";
import type { Action, ContainerSummary } from "@/api/types";
import { containerStatus } from "@/state/status";
import { Button } from "@/components/Button";
import { FAB } from "@/components/FAB";
import { Filters } from "@/components/Filters";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { ListRow } from "@/components/ListRow";
import { LongPressMenu, type LongPressItem } from "@/components/LongPressMenu";
import { Page } from "@/components/Page";
import { useToast } from "@/components/Toaster";

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
  const [params, setParams] = useSearchParams();
  const filter = asFilter(params.get("filter"));
  const setFilter = (v: ContainerFilter) => {
    if (v === "all") setParams({}, { replace: true });
    else setParams({ filter: v }, { replace: true });
  };

  const queryKey = host && session.session ? `${host.url}:list_containers` : null;
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
              host={host!}
              session={session.session}
              hid={hid!}
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
  host,
  session,
  hid,
  containers,
  onPick,
}: {
  host: Host;
  session: Session;
  hid: string;
  containers: ContainerSummary[];
  onPick: (id: string) => void;
}) {
  const nav = useNavigate();
  const toast = useToast();

  async function act(c: ContainerSummary, action: Action, label: string) {
    try {
      unwrap(await call(host, { op: "container_action", id: c.id, action }), "ok");
      invalidate(`${host.url}:list_containers`);
      invalidate(`${host.url}:inspect:${c.id}`);
      toast.push(`${label} ${c.name || c.id.slice(0, 12)}`, "success");
    } catch (e) {
      toast.push((e as Error).message, "error");
    }
  }

  function quickItems(c: ContainerSummary): LongPressItem[] {
    const running = c.state === "running";
    const denyAction = !session.can("container_action") ? "your token doesn't allow container_action" : undefined;
    const denyLogs = !session.can("stream_logs") ? "your token doesn't allow stream_logs" : undefined;
    const denyExec = !session.can("exec") ? "your token doesn't allow exec" : undefined;
    return [
      {
        label: "Start",
        disabled: denyAction || running,
        onSelect: () => act(c, { kind: "start" }, "started"),
      },
      {
        label: "Stop",
        disabled: denyAction || !running,
        onSelect: () => act(c, { kind: "stop" }, "stopped"),
      },
      {
        label: "Restart",
        disabled: denyAction || !running,
        onSelect: () => act(c, { kind: "restart" }, "restarted"),
      },
      {
        label: "Logs",
        disabled: denyLogs,
        onSelect: () => nav(`/h/${hid}/c/${c.id}/logs`),
      },
      {
        label: "Exec",
        disabled: denyExec || (!running && "container is not running"),
        onSelect: () => nav(`/h/${hid}/c/${c.id}/exec`),
      },
    ];
  }

  if (containers.length === 0) {
    return <p className="text-xs text-[var(--text-tertiary)]">No matches.</p>;
  }
  return (
    <div className="flex flex-col -mx-1">
      {containers.map((c) => (
        <div key={c.id} className="px-1">
          <LongPressMenu items={quickItems(c)} onPress={() => onPick(c.id)}>
            <ListRow
              title={c.name || "(unnamed)"}
              subtitle={`${c.image} · ${c.status}`}
              status={containerStatus(c.state, c.exit_code, c.health)}
            />
          </LongPressMenu>
        </div>
      ))}
    </div>
  );
}
