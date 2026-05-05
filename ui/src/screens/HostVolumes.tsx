import { useNavigate, useParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import type { VolumeSummary } from "@/api/types";
import { useHosts } from "@/state/hosts";
import { useQuery } from "@/state/cache";
import { volumeStatus } from "@/state/status";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { ListRow } from "@/components/ListRow";
import { Page } from "@/components/Page";
import { SkeletonRows } from "@/components/Skeleton";
import { relativeDate } from "@/lib/relativeDate";

export function HostVolumes() {
  const { hid } = useParams<{ hid: string }>();
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };

  const queryKey = host ? `${host.url}:list_volumes` : null;
  const { data: volumes, error } = useQuery<VolumeSummary[]>(queryKey, async () => {
    const r = unwrap(await call(host!, { op: "list_volumes" }), "volumes");
    return r.data;
  });

  const crumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "volumes");

  if (!saved || !hid) return <Page><p>Unknown host.</p></Page>;

  return (
    <Page crumbs={crumbs}>
      {error && <p className="text-[var(--error)] text-xs">{error}</p>}
      {volumes === null && !error && <SkeletonRows count={5} />}
      {volumes?.length === 0 && (
        <p className="text-xs text-[var(--text-tertiary)]">No volumes.</p>
      )}
      {volumes && volumes.length > 0 && (
        <div className="flex flex-col -mx-1">
          {volumes.map((v) => (
            <div key={v.name} className="px-1">
              <ListRow
                title={v.name}
                mono
                subtitle={relativeDate(v.created_at)}
                status={volumeStatus(v.in_use)}
                onPress={() => nav(`/h/${hid}/volumes/${encodeURIComponent(v.name)}`)}
              />
            </div>
          ))}
        </div>
      )}
    </Page>
  );
}
