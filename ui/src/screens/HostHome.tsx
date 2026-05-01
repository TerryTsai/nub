import { useEffect, useState } from "react";
import { Link, useNavigate, useParams } from "react-router-dom";
import { call, type Host } from "@/api/client";
import { useHosts } from "@/state/hosts";
import { useSession } from "@/state/session";
import type { ContainerSummary } from "@/api/types";
import { Button } from "@/components/Button";
import { Card } from "@/components/Card";
import { Page } from "./Hosts";
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
      const r = await call(host, { op: "list_containers", all: true });
      if (r.type !== "containers") throw new Error("unexpected response");
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
        <Card>
          <p>Unknown host. <Link to="/" className="underline">Back to hosts</Link></p>
        </Card>
      </Page>
    );
  }

  const right = (
    <Link to="/" aria-label="Back to hosts">
      <Button variant="ghost" className="text-sm">←</Button>
    </Link>
  );

  return (
    <Page title={saved.label} right={right}>
      {session.loading && <Card><p className="text-[var(--text-secondary)]">Connecting…</p></Card>}

      {session.error && (
        <Card>
          <p className="text-[var(--error)]">Couldn't connect: {session.error}</p>
          <p className="text-sm text-[var(--text-secondary)]">
            The token may have rotated (admin tokens regenerate on every nub restart).
          </p>
          <Link to="/add"><Button variant="ghost">Re-add host</Button></Link>
        </Card>
      )}

      {session.session && (
        <Card>
          <div className="flex justify-between items-center">
            <h2 className="text-base font-semibold">Containers</h2>
            <div className="flex gap-2">
              <Button
                variant="primary"
                className="text-sm"
                onClick={() => setRunOpen(true)}
                disallowReason={
                  session.session.can("create_container")
                    ? undefined
                    : "your token doesn't allow create_container"
                }
              >
                ▶ Run
              </Button>
              <Button variant="ghost" className="text-sm" onClick={refresh} disabled={refreshing}>
                {refreshing ? "…" : "Refresh"}
              </Button>
            </div>
          </div>
          {error && <p className="text-[var(--error)] text-sm">{error}</p>}
          {containers === null && !error && (
            <p className="text-sm text-[var(--text-tertiary)]">Loading…</p>
          )}
          {containers !== null && containers.length === 0 && (
            <p className="text-sm text-[var(--text-tertiary)]">No containers.</p>
          )}
          {containers !== null && containers.length > 0 && (
            <ul className="flex flex-col gap-1 -mx-1">
              {containers.map((c) => (
                <li key={c.id}>
                  <button
                    type="button"
                    onClick={() => nav(`/h/${hid}/c/${c.id}`)}
                    className="w-full text-left p-2 rounded-[var(--radius-md)] hover:bg-[var(--border-subtle)] transition-colors flex flex-col gap-0.5"
                  >
                    <div className="flex items-center gap-2">
                      <span className={`dot dot-${c.state}`} aria-label={c.state} />
                      <span className="font-medium truncate flex-1">{c.name || "(unnamed)"}</span>
                      <span className="text-xs text-[var(--text-tertiary)] mono">{c.id}</span>
                    </div>
                    <div className="text-xs text-[var(--text-secondary)] truncate pl-4">
                      {c.image} · {c.status}
                    </div>
                  </button>
                </li>
              ))}
            </ul>
          )}
        </Card>
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
