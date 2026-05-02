import { useNavigate, useParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import type { DockerfileSummary } from "@/api/types";
import { useHosts } from "@/state/hosts";
import { useSession } from "@/state/session";
import { useQuery } from "@/state/cache";
import { FAB } from "@/components/FAB";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { ListRow } from "@/components/ListRow";
import { Page } from "@/components/Page";

export function HostDockerfiles() {
  const { hid } = useParams<{ hid: string }>();
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };
  const session = useSession(host);

  const queryKey = host && session.session ? `${host.url}:list_dockerfiles` : null;
  const { data: files, error } = useQuery<DockerfileSummary[]>(queryKey, async () => {
    const r = unwrap(await call(host!, { op: "list_dockerfiles" }), "dockerfiles");
    return r.data;
  });

  const crumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "dockerfiles");
  const canWrite = session.session?.can("write_dockerfile") ?? false;

  if (!saved || !hid) return <Page><p>Unknown host.</p></Page>;

  return (
    <Page
      crumbs={crumbs}
      fab={
        session.session && canWrite
          ? <FAB to={`/h/${hid}/dockerfiles/_new`} label="dockerfile" />
          : undefined
      }
    >
      {error && <p className="text-[var(--error)] text-xs">{error}</p>}
      {files === null && !error && (
        <p className="text-xs text-[var(--text-tertiary)]">Loading…</p>
      )}
      {files?.length === 0 && (
        <p className="text-xs text-[var(--text-tertiary)]">
          No dockerfiles yet. Tap + to add one.
        </p>
      )}
      {files && files.length > 0 && (
        <div className="flex flex-col -mx-1">
          {files.map((f) => (
            <div key={f.name} className="px-1">
              <ListRow
                title={f.name}
                mono
                subtitle={`${formatBytes(f.size)}${f.modified_at ? ` · ${f.modified_at}` : ""}`}
                onPress={() => nav(`/h/${hid}/dockerfiles/${encodeURIComponent(f.name)}`)}
              />
            </div>
          ))}
        </div>
      )}
    </Page>
  );
}

function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  const units = ["KB", "MB"];
  let v = n / 1024;
  for (const u of units) {
    if (v < 1024) return `${v.toFixed(v < 10 ? 1 : 0)} ${u}`;
    v /= 1024;
  }
  return `${v.toFixed(0)} GB`;
}
