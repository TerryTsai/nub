import { useEffect, useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import { useHosts } from "@/state/hosts";
import { useSession } from "@/state/session";
import type { ContainerSummary } from "@/api/types";
import { Button } from "@/components/Button";
import { ListRow } from "@/components/ListRow";
import { Page } from "@/components/Page";
import { Section } from "@/components/Section";
import { RunSheet } from "./RunSheet";

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
  const [runOpen, setRunOpen] = useState(false);

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
      <Page title="?">
        <p>Unknown host. <Link to="/" className="underline">Back to hosts</Link></p>
      </Page>
    );
  }

  const back = (
    <Link to="/" aria-label="Back to hosts">
      <Button variant="ghost" className="text-sm">←</Button>
    </Link>
  );

  return (
    <Page title={saved.label} right={back}>
      {session.loading && <p className="text-[var(--text-secondary)] text-sm">Connecting…</p>}

      {session.error && (
        <Section label="Connection">
          <p className="text-[var(--error)] text-sm">Couldn't connect: {session.error}</p>
          <p className="text-xs text-[var(--text-tertiary)]">
            The token may have rotated (admin tokens regenerate on every nub restart).
          </p>
          <Link to="/add" className="self-start"><Button variant="ghost">Re-add host</Button></Link>
        </Section>
      )}

      {session.session && (
        <Section
          label="Containers"
          right={
            <div className="flex gap-2">
              <Button
                variant="primary"
                onClick={() => setRunOpen(true)}
                disallowReason={
                  session.session.can("create_container")
                    ? undefined
                    : "your token doesn't allow create_container"
                }
              >
                ▶ Run
              </Button>
              <Button variant="ghost" onClick={refresh} disabled={refreshing}>
                {refreshing ? "…" : "Refresh"}
              </Button>
            </div>
          }
        >
          {error && <p className="text-[var(--error)] text-xs">{error}</p>}
          {containers === null && !error && (
            <p className="text-xs text-[var(--text-tertiary)]">Loading…</p>
          )}
          {containers !== null && containers.length === 0 && (
            <p className="text-xs text-[var(--text-tertiary)]">No containers.</p>
          )}
          {containers !== null && containers.length > 0 && (
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
          )}
        </Section>
      )}

      {host && session.session && (
        <RunSheet
          host={host}
          open={runOpen}
          onOpenChange={setRunOpen}
          onCreated={(id) => nav(`/h/${hid}/c/${id}`)}
          disallowReason={
            session.session.can("create_container")
              ? undefined
              : "your token doesn't allow create_container"
          }
        />
      )}
    </Page>
  );
}
