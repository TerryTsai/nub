import { useState } from "react";
import { useParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import type { NetworkSummary } from "@/api/types";
import { useHosts } from "@/state/hosts";
import { useSession } from "@/state/session";
import { invalidate, useQuery } from "@/state/cache";
import { ConfirmDialog } from "@/components/ConfirmDialog";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { ListRow } from "@/components/ListRow";
import { Page } from "@/components/Page";

export function HostNetworks() {
  const { hid } = useParams<{ hid: string }>();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };
  const session = useSession(host);

  const [pending, setPending] = useState<NetworkSummary | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);

  const queryKey = host && session.session ? `${host.url}:list_networks` : null;
  const { data: networks, error: queryError, reload } = useQuery<NetworkSummary[]>(queryKey, async () => {
    const r = unwrap(await call(host!, { op: "list_networks" }), "networks");
    return r.data;
  });
  const error = actionError ?? queryError;

  async function removeNetwork(n: NetworkSummary) {
    if (!host) return;
    try {
      unwrap(await call(host, { op: "remove_network", id: n.id }), "ok");
      if (queryKey) invalidate(queryKey);
      reload();
    } catch (e) {
      setActionError((e as Error).message);
    }
  }

  const crumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "networks");

  if (!saved || !hid) return <Page><p>Unknown host.</p></Page>;

  return (
    <Page crumbs={crumbs}>
      {error && <p className="text-[var(--error)] text-xs">{error}</p>}
      {networks === null && !error && (
        <p className="text-xs text-[var(--text-tertiary)]">Loading…</p>
      )}
      {networks?.length === 0 && (
        <p className="text-xs text-[var(--text-tertiary)]">No networks.</p>
      )}
      {networks && networks.length > 0 && (
        <div className="flex flex-col -mx-1">
          {networks.map((n) => (
            <div key={n.id} className="px-1">
              <ListRow
                title={n.name}
                subtitle={`${n.driver}${n.scope ? ` · ${n.scope}` : ""}${n.internal ? " · internal" : ""}`}
                right={
                  <button
                    type="button"
                    onClick={() => setPending(n)}
                    aria-label="Remove network"
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
        title={pending ? `Remove network ${pending.name}?` : ""}
        confirmLabel="Remove"
        destructive
        onConfirm={() => { if (pending) removeNetwork(pending); setPending(null); }}
      />
    </Page>
  );
}
