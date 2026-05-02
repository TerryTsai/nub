import { useEffect, useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import { useHosts } from "@/state/hosts";
import { useSession } from "@/state/session";
import type { ContainerSummary } from "@/api/types";
import { Button } from "@/components/Button";
import { CountRefresh } from "@/components/CountRefresh";
import { FAB } from "@/components/FAB";
import { HostNav } from "@/components/HostNav";
import { ListRow } from "@/components/ListRow";
import { Page } from "@/components/Page";

export function HostHome() {
  const { hid } = useParams<{ hid: string }>();
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };

  const session = useSession(host);
  const [containers, setContainers] = useState<ContainerSummary[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [refreshing, setRefreshing] = useState(false);

  async function refresh() {
    if (!host) return;
    setRefreshing(true);
    setError(null);
    try {
      const r = unwrap(await call(host, { op: "list_containers", all: true }), "containers");
      setContainers(r.data);
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setRefreshing(false);
    }
  }

  useEffect(() => {
    if (host && session.session) refresh();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [hid, session.session]);

  if (!saved) {
    return (
      <Page>
        <p>Unknown host. <Link to="/" className="underline">Back to hosts</Link></p>
      </Page>
    );
  }

  const crumbs = [{ label: saved.label }];
  const canCreate = session.session?.can("create_container") ?? false;

  return (
    <Page
      crumbs={crumbs}
      nav={<HostNav hid={hid!} active="containers" />}
      fab={session.session && canCreate ? <FAB to={`/h/${hid}/run`} label="container" /> : undefined}
    >
      {session.loading && <p className="text-[var(--text-secondary)] text-sm">Connecting…</p>}

      {session.error && (
        <>
          <p className="text-[var(--error)] text-sm">Couldn't connect: {session.error}</p>
          <p className="text-xs text-[var(--text-tertiary)]">
            The token may have rotated (admin tokens regenerate on every nub restart).
          </p>
          <Link to="/add" className="self-start"><Button variant="ghost">Re-add host</Button></Link>
        </>
      )}

      {session.session && (
        <>
          {error && <p className="text-[var(--error)] text-xs">{error}</p>}
          {containers === null && !error && (
            <p className="text-xs text-[var(--text-tertiary)]">Loading…</p>
          )}
          {containers !== null && containers.length === 0 && (
            <p className="text-xs text-[var(--text-tertiary)]">No containers.</p>
          )}
          {containers !== null && containers.length > 0 && (
            <>
              <CountRefresh
                label={`${containers.length} container${containers.length !== 1 ? "s" : ""}`}
                onRefresh={refresh}
                refreshing={refreshing}
              />
              <div className="flex flex-col -mx-1">
                {containers.map((c) => (
                  <div key={c.id} className="px-1">
                    <ListRow
                      title={c.name || "(unnamed)"}
                      subtitle={`${c.image} · ${c.status}`}
                      status={c.state}
                      onPress={() => nav(`/h/${hid}/c/${c.id}`)}
                    />
                  </div>
                ))}
              </div>
            </>
          )}
        </>
      )}

    </Page>
  );
}

