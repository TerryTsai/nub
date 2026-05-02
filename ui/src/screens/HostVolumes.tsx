import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import type { VolumeSummary } from "@/api/types";
import { useHosts } from "@/state/hosts";
import { useSession } from "@/state/session";
import { Button } from "@/components/Button";
import { HostNav } from "@/components/HostNav";
import { ListRow } from "@/components/ListRow";
import { Page } from "@/components/Page";
import { Section } from "@/components/Section";

export function HostVolumes() {
  const { hid } = useParams<{ hid: string }>();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };
  const session = useSession(host);

  const [volumes, setVolumes] = useState<VolumeSummary[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [refreshing, setRefreshing] = useState(false);

  async function refresh() {
    if (!host) return;
    setRefreshing(true);
    setError(null);
    try {
      const r = unwrap(await call(host, { op: "list_volumes" }), "volumes");
      setVolumes(r.data);
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setRefreshing(false);
    }
  }

  async function removeVolume(v: VolumeSummary) {
    if (!host) return;
    if (!confirm(`Remove volume ${v.name}?`)) return;
    try {
      unwrap(await call(host, { op: "remove_volume", name: v.name, force: false }), "ok");
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
    <Page crumbs={crumbs}>
      <HostNav hid={hid} active="volumes" />
      {error && <p className="text-[var(--error)] text-xs">{error}</p>}
      <Section
        label={`Volumes${volumes ? ` (${volumes.length})` : ""}`}
        right={
          <Button variant="ghost" onClick={refresh} disabled={refreshing}>
            {refreshing ? "…" : "Refresh"}
          </Button>
        }
      >
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
                      onClick={() => removeVolume(v)}
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
      </Section>
    </Page>
  );
}
