import { useState } from "react";
import { useParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import type { VolumeSummary } from "@/api/types";
import { useHosts } from "@/state/hosts";
import { useSession } from "@/state/session";
import { invalidate, useQuery } from "@/state/cache";
import { ConfirmDialog } from "@/components/ConfirmDialog";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { ListRow } from "@/components/ListRow";
import { Page } from "@/components/Page";

export function HostVolumes() {
  const { hid } = useParams<{ hid: string }>();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };
  const session = useSession(host);

  const [pending, setPending] = useState<VolumeSummary | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);

  const queryKey = host && session.session ? `${host.url}:list_volumes` : null;
  const { data: volumes, error: queryError, reload } = useQuery<VolumeSummary[]>(queryKey, async () => {
    const r = unwrap(await call(host!, { op: "list_volumes" }), "volumes");
    return r.data;
  });
  const error = actionError ?? queryError;

  async function removeVolume(v: VolumeSummary) {
    if (!host) return;
    try {
      unwrap(await call(host, { op: "remove_volume", name: v.name, force: false }), "ok");
      if (queryKey) invalidate(queryKey);
      reload();
    } catch (e) {
      setActionError((e as Error).message);
    }
  }

  const crumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "volumes");

  if (!saved || !hid) return <Page><p>Unknown host.</p></Page>;

  return (
    <Page crumbs={crumbs}>
      {error && <p className="text-[var(--error)] text-xs">{error}</p>}
      {volumes === null && !error && (
        <p className="text-xs text-[var(--text-tertiary)]">Loading…</p>
      )}
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
                subtitle={`${v.driver}${v.scope ? ` · ${v.scope}` : ""} · ${v.mountpoint}`}
                right={
                  <button
                    type="button"
                    onClick={() => setPending(v)}
                    aria-label="Remove volume"
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
      <ConfirmDialog
        open={pending !== null}
        onOpenChange={(o) => { if (!o) setPending(null); }}
        title={pending ? `Remove volume ${pending.name.slice(0, 12)}?` : ""}
        description="Volume contents are deleted. This cannot be undone."
        confirmLabel="Remove"
        destructive
        onConfirm={() => { if (pending) removeVolume(pending); setPending(null); }}
      />
    </Page>
  );
}
