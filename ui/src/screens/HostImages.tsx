import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import type { ImageSummary } from "@/api/types";
import { useHosts } from "@/state/hosts";
import { useSession } from "@/state/session";
import { CountRefresh } from "@/components/CountRefresh";
import { HostNav } from "@/components/HostNav";
import { ListRow } from "@/components/ListRow";
import { Page } from "@/components/Page";

export function HostImages() {
  const { hid } = useParams<{ hid: string }>();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };
  const session = useSession(host);

  const [images, setImages] = useState<ImageSummary[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [refreshing, setRefreshing] = useState(false);

  async function refresh() {
    if (!host) return;
    setRefreshing(true);
    setError(null);
    try {
      const r = unwrap(await call(host, { op: "list_images" }), "images");
      setImages(r.data);
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setRefreshing(false);
    }
  }

  async function removeImage(image: ImageSummary) {
    if (!host) return;
    if (!confirm(`Remove ${image.repo_tag}?`)) return;
    try {
      unwrap(await call(host, { op: "remove_image", id: image.id, force: false }), "ok");
      await refresh();
    } catch (e) {
      setError((e as Error).message);
    }
  }

  useEffect(() => {
    if (host && session.session) refresh();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [hid, session.session]);

  if (!saved || !hid) return <Page><p>Unknown host.</p></Page>;

  const crumbs = [{ label: saved.label, to: `/h/${hid}` }];
  return (
    <Page crumbs={crumbs} nav={<HostNav hid={hid} active="images" />}>
      {error && <p className="text-[var(--error)] text-xs">{error}</p>}
      {images === null && !error && (
        <p className="text-xs text-[var(--text-tertiary)]">Loading…</p>
      )}
      {images?.length === 0 && (
        <p className="text-xs text-[var(--text-tertiary)]">No images.</p>
      )}
      {images && images.length > 0 && (
        <>
          <CountRefresh
            label={`${images.length} image${images.length !== 1 ? "s" : ""}`}
            onRefresh={refresh}
            refreshing={refreshing}
          />
          <div className="flex flex-col -mx-1">
            {images.map((img) => (
              <div key={img.id} className="px-1">
                <ListRow
                  title={img.repo_tag}
                  subtitle={`${img.id} · ${formatBytes(img.size)}${img.containers > 0 ? ` · in use by ${img.containers}` : ""}`}
                  right={
                    <button
                      type="button"
                      onClick={() => removeImage(img)}
                      aria-label="Remove image"
                      className="text-[11px] text-[var(--text-tertiary)] hover:text-[var(--error)] px-1 shrink-0"
                    >
                      remove
                    </button>
                  }
                />
              </div>
            ))}
          </div>
        </>
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
