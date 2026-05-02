import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import type { ImageSummary } from "@/api/types";
import { useHosts } from "@/state/hosts";
import { useSession } from "@/state/session";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
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

  async function refresh() {
    if (!host) return;
    setError(null);
    try {
      const r = unwrap(await call(host, { op: "list_images" }), "images");
      setImages(r.data);
    } catch (e) {
      setError((e as Error).message);
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

  const crumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "images");

  if (!saved || !hid) return <Page><p>Unknown host.</p></Page>;

  return (
    <Page crumbs={crumbs}>
      {error && <p className="text-[var(--error)] text-xs">{error}</p>}
      {images === null && !error && (
        <p className="text-xs text-[var(--text-tertiary)]">Loading…</p>
      )}
      {images?.length === 0 && (
        <p className="text-xs text-[var(--text-tertiary)]">No images.</p>
      )}
      {images && images.length > 0 && (
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
