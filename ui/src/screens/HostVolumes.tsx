import { useNavigate, useParams, useSearchParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import type { VolumeSummary } from "@/api/types";
import { useHosts } from "@/state/hosts";
import { useQuery } from "@/state/cache";
import { volumeStatus } from "@/state/status";
import { Filters } from "@/components/Filters";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { ListRow } from "@/components/ListRow";
import { Page } from "@/components/Page";
import { LoadingBar } from "@/components/LoadingBar";
import { relativeDate } from "@/lib/relativeDate";

type VolumeFilter = "all" | "in-use" | "idle";
const VOLUME_FILTERS: VolumeFilter[] = ["all", "in-use", "idle"];

function asVolumeFilter(s: string | null): VolumeFilter {
  return VOLUME_FILTERS.includes(s as VolumeFilter) ? (s as VolumeFilter) : "all";
}

function matchesVolumeFilter(v: VolumeSummary, f: VolumeFilter): boolean {
  if (f === "all") return true;
  if (f === "in-use") return v.in_use;
  return !v.in_use;
}

export function HostVolumes() {
  const { hid } = useParams<{ hid: string }>();
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };

  const [params, setParams] = useSearchParams();
  const filter = asVolumeFilter(params.get("filter"));
  const setFilter = (v: VolumeFilter) => {
    if (v === "all") setParams({}, { replace: true });
    else setParams({ filter: v }, { replace: true });
  };

  const queryKey = host ? `${host.url}:list_volumes` : null;
  const { data: volumes, error } = useQuery<VolumeSummary[]>(queryKey, async () => {
    const r = unwrap(await call(host!, { op: "list_volumes" }), "volumes");
    return r.data;
  });

  const crumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "volumes");

  if (!saved || !hid) return <Page><p>Unknown host.</p></Page>;

  const inUse = volumes?.filter((v) => v.in_use).length ?? 0;
  const subnav = volumes !== null ? (
    <Filters
      attribute="in use"
      value={filter}
      onChange={setFilter}
      options={[
        { value: "all", label: "All", count: volumes.length },
        { value: "in-use", label: "In use", count: inUse },
        { value: "idle", label: "Idle", count: volumes.length - inUse },
      ]}
    />
  ) : undefined;

  const visible = volumes?.filter((v) => matchesVolumeFilter(v, filter)) ?? null;

  return (
    <Page crumbs={crumbs} subnav={subnav}>
      {error && <p className="text-[var(--error)] text-xs">{error}</p>}
      {volumes === null && !error && <LoadingBar />}
      {volumes?.length === 0 && (
        <p className="text-xs text-[var(--text-tertiary)]">No volumes.</p>
      )}
      {visible && visible.length === 0 && volumes && volumes.length > 0 && (
        <p className="text-xs text-[var(--text-tertiary)]">No matches.</p>
      )}
      {visible && visible.length > 0 && (
        <div className="flex flex-col">
          {visible.map((v) => (
            <ListRow
              key={v.name}
              title={v.name}
              mono
              subtitle={relativeDate(v.created_at)}
              status={volumeStatus(v.in_use)}
              onPress={() => nav(`/h/${hid}/volumes/${encodeURIComponent(v.name)}`)}
            />
          ))}
        </div>
      )}
    </Page>
  );
}
