import { useNavigate, useParams, useSearchParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import type { StackSummary } from "@/api/types";
import { useHosts } from "@/state/hosts";
import { useQuery } from "@/state/cache";
import { stackStatus } from "@/state/status";
import { FAB } from "@/components/FAB";
import { Filters } from "@/components/Filters";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { ListRow } from "@/components/ListRow";
import { Page } from "@/components/Page";
import { LoadingBar } from "@/components/LoadingBar";
import { relativeDate } from "@/lib/relativeDate";

type StackFilter = "all" | "active" | "idle" | "pending";
const STACK_FILTERS: StackFilter[] = ["all", "active", "idle", "pending"];

function asStackFilter(s: string | null): StackFilter {
  return STACK_FILTERS.includes(s as StackFilter) ? (s as StackFilter) : "all";
}

function matchesStackFilter(s: StackSummary, f: StackFilter): boolean {
  if (f === "all") return true;
  return s.status === f;
}

export function HostStacks() {
  const { hid } = useParams<{ hid: string }>();
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };

  const [params, setParams] = useSearchParams();
  const filter = asStackFilter(params.get("filter"));
  const setFilter = (v: StackFilter) => {
    if (v === "all") setParams({}, { replace: true });
    else setParams({ filter: v }, { replace: true });
  };

  const queryKey = host ? `${host.url}:list_stacks` : null;
  const { data: stacks, error } = useQuery<StackSummary[]>(queryKey, async () => {
    const r = unwrap(await call(host!, { op: "list_stacks" }), "stacks");
    return r.data;
  });

  const crumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "stacks");

  if (!saved || !hid) return <Page><p>Unknown host.</p></Page>;

  const countBy = (status: string) => stacks?.filter((s) => s.status === status).length ?? 0;
  const subnav = stacks !== null ? (
    <Filters
      attribute="status"
      value={filter}
      onChange={setFilter}
      options={[
        { value: "all", label: "All", count: stacks.length },
        { value: "active", label: "Active", count: countBy("active") },
        { value: "idle", label: "Idle", count: countBy("idle") },
        { value: "pending", label: "Pending", count: countBy("pending") },
      ]}
    />
  ) : undefined;

  const visible = stacks?.filter((s) => matchesStackFilter(s, filter)) ?? null;

  return (
    <Page crumbs={crumbs} subnav={subnav} fab={<FAB to={`/h/${hid}/stacks/new`} label="stack" />}>
      {error && <p className="text-[var(--error)] text-xs">{error}</p>}
      {stacks === null && !error && <LoadingBar />}
      {stacks?.length === 0 && (
        <p className="text-xs text-[var(--text-tertiary)]">No stacks.</p>
      )}
      {visible && visible.length === 0 && stacks && stacks.length > 0 && (
        <p className="text-xs text-[var(--text-tertiary)]">No matches.</p>
      )}
      {visible && visible.length > 0 && (
        <div className="flex flex-col">
          {visible.map((s) => (
            <ListRow
              key={s.name}
              title={s.name}
              mono
              subtitle={relativeDate(s.modified_at)}
              status={stackStatus(s.status)}
              onPress={() => nav(`/h/${hid}/stacks/${encodeURIComponent(s.name)}`)}
            />
          ))}
        </div>
      )}
    </Page>
  );
}
