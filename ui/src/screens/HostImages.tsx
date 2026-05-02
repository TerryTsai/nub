import { useNavigate, useParams, useSearchParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import type { ImageSummary } from "@/api/types";
import { useHosts } from "@/state/hosts";
import { useSession } from "@/state/session";
import { useQuery } from "@/state/cache";
import { imageStatus } from "@/state/status";
import { FAB } from "@/components/FAB";
import { Filters } from "@/components/Filters";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { ListRow } from "@/components/ListRow";
import { Page } from "@/components/Page";

type ImageFilter = "all" | "tagged" | "untagged";
const IMAGE_FILTERS: ImageFilter[] = ["all", "tagged", "untagged"];

function asImageFilter(s: string | null): ImageFilter {
  return IMAGE_FILTERS.includes(s as ImageFilter) ? (s as ImageFilter) : "all";
}

function isTagged(img: ImageSummary): boolean {
  return img.repo_tag !== "<none>" && !img.repo_tag.startsWith("<none>");
}

function matchesImageFilter(img: ImageSummary, f: ImageFilter): boolean {
  if (f === "all") return true;
  if (f === "tagged") return isTagged(img);
  return !isTagged(img);
}

export function HostImages() {
  const { hid } = useParams<{ hid: string }>();
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };
  const session = useSession(host);

  const [params, setParams] = useSearchParams();
  const filter = asImageFilter(params.get("filter"));
  const setFilter = (v: ImageFilter) => {
    if (v === "all") setParams({}, { replace: true });
    else setParams({ filter: v }, { replace: true });
  };

  const queryKey = host && session.session ? `${host.url}:list_images` : null;
  const { data: images, error } = useQuery<ImageSummary[]>(queryKey, async () => {
    const r = unwrap(await call(host!, { op: "list_images" }), "images");
    return r.data;
  });

  const crumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "images");

  if (!saved || !hid) return <Page><p>Unknown host.</p></Page>;

  const tagged = images?.filter(isTagged).length ?? 0;
  const subnav = images !== null ? (
    <Filters
      value={filter}
      onChange={setFilter}
      options={[
        { value: "all", label: "All", count: images.length },
        { value: "tagged", label: "Tagged", count: tagged },
        { value: "untagged", label: "Untagged", count: images.length - tagged },
      ]}
    />
  ) : undefined;

  const visible = images?.filter((i) => matchesImageFilter(i, filter)) ?? null;
  const canPull = session.session?.can("pull_image") ?? false;

  return (
    <Page
      crumbs={crumbs}
      subnav={subnav}
      fab={session.session && canPull ? <FAB to={`/h/${hid}/images/pull`} label="pull" /> : undefined}
    >
      {error && <p className="text-[var(--error)] text-xs">{error}</p>}
      {images === null && !error && (
        <p className="text-xs text-[var(--text-tertiary)]">Loading…</p>
      )}
      {images?.length === 0 && (
        <p className="text-xs text-[var(--text-tertiary)]">No images.</p>
      )}
      {visible && visible.length === 0 && images && images.length > 0 && (
        <p className="text-xs text-[var(--text-tertiary)]">No matches.</p>
      )}
      {visible && visible.length > 0 && (
        <div className="flex flex-col -mx-1">
          {visible.map((img) => (
            <div key={img.id} className="px-1">
              <ListRow
                title={img.repo_tag}
                subtitle={`${img.id} · ${formatBytes(img.size)}`}
                status={imageStatus(img.containers)}
                onPress={() => nav(`/h/${hid}/images/${img.id}`)}
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
  const units = ["KB", "MB", "GB", "TB"];
  let v = n / 1024;
  for (const u of units) {
    if (v < 1024) return `${v.toFixed(v < 10 ? 1 : 0)} ${u}`;
    v /= 1024;
  }
  return `${v.toFixed(0)} PB`;
}
