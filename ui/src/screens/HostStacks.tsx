import { useNavigate, useParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import type { StackSummary } from "@/api/types";
import { useHosts } from "@/state/hosts";
import { useSession } from "@/state/session";
import { useQuery } from "@/state/cache";
import { stackStatus } from "@/state/status";
import { FAB } from "@/components/FAB";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { ListRow } from "@/components/ListRow";
import { Page } from "@/components/Page";

export function HostStacks() {
  const { hid } = useParams<{ hid: string }>();
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };
  const session = useSession(host);

  const queryKey = host && session.session ? `${host.url}:list_stacks` : null;
  const { data: stacks, error } = useQuery<StackSummary[]>(queryKey, async () => {
    const r = unwrap(await call(host!, { op: "list_stacks" }), "stacks");
    return r.data;
  });

  const crumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "stacks");

  if (!saved || !hid) return <Page><p>Unknown host.</p></Page>;

  const canCreate = session.session?.can("stacks:create") ?? false;

  return (
    <Page
      crumbs={crumbs}
      fab={session.session && canCreate ? <FAB to={`/h/${hid}/stacks/new`} label="stack" /> : undefined}
    >
      {session.loading && <p className="text-xs text-[var(--text-tertiary)]">Connecting…</p>}
      {session.error && <p className="text-[var(--error)] text-xs">Couldn't connect: {session.error}</p>}
      {error && <p className="text-[var(--error)] text-xs">{error}</p>}
      {session.session && stacks === null && !error && (
        <p className="text-xs text-[var(--text-tertiary)]">Loading stacks…</p>
      )}
      {stacks?.length === 0 && (
        <p className="text-xs text-[var(--text-tertiary)]">No stacks.</p>
      )}
      {stacks && stacks.length > 0 && (
        <div className="flex flex-col -mx-1">
          {stacks.map((s) => (
            <div key={s.name} className="px-1">
              <ListRow
                title={s.name}
                subtitle={`${s.container_count} container${s.container_count === 1 ? "" : "s"}`}
                status={stackStatus(s.status)}
                onPress={() => nav(`/h/${hid}/stacks/${encodeURIComponent(s.name)}`)}
              />
            </div>
          ))}
        </div>
      )}
    </Page>
  );
}
