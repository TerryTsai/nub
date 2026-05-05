import { useNavigate, useParams, useSearchParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import type { ImageSummary } from "@/api/types";
import { useHosts } from "@/state/hosts";
import { useQuery } from "@/state/cache";
import { imageStatus } from "@/state/status";
import { FAB } from "@/components/FAB";
import { Filters } from "@/components/Filters";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { ListRow } from "@/components/ListRow";
import { Page } from "@/components/Page";
import { LoadingBar } from "@/components/LoadingBar";
import { relativeDate } from "@/lib/relativeDate";

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

  const [params, setParams] = useSearchParams();
  const filter = asImageFilter(params.get("filter"));
  const setFilter = (v: ImageFilter) => {
    if (v === "all") setParams({}, { replace: true });
    else setParams({ filter: v }, { replace: true });
  };

  const queryKey = host ? `${host.url}:list_images` : null;
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

  return (
    <Page crumbs={crumbs} subnav={subnav} fab={<FAB to={`/h/${hid}/images/new`} label="image" />}>
      {error && <p className="text-[var(--error)] text-xs">{error}</p>}
      {images === null && !error && <LoadingBar />}
      {images?.length === 0 && (
        <p className="text-xs text-[var(--text-tertiary)]">No images.</p>
      )}
      {visible && visible.length === 0 && images && images.length > 0 && (
        <p className="text-xs text-[var(--text-tertiary)]">No matches.</p>
      )}
      {visible && visible.length > 0 && (
        <div className="flex flex-col">
          {visible.map((img) => (
            <ListRow
              key={img.id}
              title={img.repo_tag}
              mono
              subtitle={relativeDate(img.created)}
              status={imageStatus(img.containers)}
              onPress={() => nav(`/h/${hid}/images/${img.id}`)}
            />
          ))}
        </div>
      )}
    </Page>
  );
}

