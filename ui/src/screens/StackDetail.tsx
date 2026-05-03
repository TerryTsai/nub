import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import type { StackDetail } from "@/api/types";
import { useHosts } from "@/state/hosts";
import { useSession } from "@/state/session";
import { useQuery, invalidate } from "@/state/cache";
import { containerStatus } from "@/state/status";
import { Button } from "@/components/Button";
import { ConfirmDialog } from "@/components/ConfirmDialog";
import { Heading } from "@/components/Heading";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { ListRow } from "@/components/ListRow";
import { Page, type Crumb } from "@/components/Page";
import { Section } from "@/components/Section";
import { useToast } from "@/components/Toaster";

type Pending = "redeploy" | "pull" | "save" | "delete" | null;

export function StackDetailScreen() {
  const { hid, sname } = useParams<{ hid: string; sname: string }>();
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };
  const session = useSession(host);
  const toast = useToast();
  const name = sname ? decodeURIComponent(sname) : "";

  const queryKey = host && session.session && name ? `${host.url}:get_stack:${name}` : null;
  const { data: detail, error, reload } = useQuery<StackDetail>(queryKey, async () => {
    const r = unwrap(await call(host!, { op: "get_stack", name }), "stack_detail");
    return r.data;
  });

  const [yaml, setYaml] = useState("");
  const [original, setOriginal] = useState("");
  const [pending, setPending] = useState<Pending>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const [confirmDelete, setConfirmDelete] = useState(false);

  useEffect(() => {
    if (detail) {
      setYaml(detail.yaml);
      setOriginal(detail.yaml);
    }
  }, [detail?.yaml, detail?.modified_at]);

  const sectionCrumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "stacks");
  if (!saved) return <Page><p>Unknown host.</p></Page>;

  const crumbs: Crumb[] = [...sectionCrumbs, { kind: "link", label: name || "?" }];

  async function act(kind: Exclude<Pending, null>, run: () => Promise<unknown>, success: string) {
    if (!host) return;
    setPending(kind);
    setActionError(null);
    try {
      await run();
      invalidate(`${host.url}:get_stack:${name}`);
      invalidate(`${host.url}:list_stacks`);
      invalidate(`${host.url}:list_containers`);
      toast.push(success, "success");
      reload();
    } catch (e) {
      setActionError((e as Error).message);
    } finally {
      setPending(null);
    }
  }

  async function onRedeploy() {
    await act("redeploy", async () => {
      unwrap(await call(host!, { op: "redeploy_stack", name }), "stack_created");
    }, `Redeployed ${name}`);
  }

  async function onPull() {
    await act("pull", async () => {
      unwrap(await call(host!, { op: "pull_stack", name }), "stack_created");
    }, `Pulled and redeployed ${name}`);
  }

  async function onSave() {
    await act("save", async () => {
      unwrap(await call(host!, { op: "update_stack", name, yaml }), "stack_created");
    }, `Updated ${name}`);
  }

  async function onDelete() {
    if (!host) return;
    setPending("delete");
    setActionError(null);
    try {
      unwrap(await call(host, { op: "delete_stack", name }), "ok");
      invalidate(`${host.url}:list_stacks`);
      invalidate(`${host.url}:list_containers`);
      toast.push(`Deleted ${name}`, "success");
      nav(`/h/${hid}/stacks`, { replace: true });
    } catch (e) {
      setActionError((e as Error).message);
      setPending(null);
    }
  }

  const dirty = yaml !== original;
  const canRedeploy = session.session?.can("stacks:redeploy") ?? false;
  const canPull = session.session?.can("stacks:pull") ?? false;
  const canUpdate = session.session?.can("stacks:update") ?? false;
  const canDelete = session.session?.can("stacks:delete") ?? false;
  const denyRedeploy = !canRedeploy ? "your token doesn't allow stacks:redeploy" : undefined;
  const denyPull = !canPull ? "your token doesn't allow stacks:pull" : undefined;
  const denyUpdate = !canUpdate ? "your token doesn't allow stacks:update" : undefined;
  const denyDelete = !canDelete ? "your token doesn't allow stacks:delete" : undefined;

  return (
    <Page crumbs={crumbs}>
      <Heading category="Stack" title={name} />

      {error && <p className="text-[var(--error)] text-xs">{error}</p>}
      {!detail && !error && <p className="text-xs text-[var(--text-tertiary)]">Loading…</p>}

      {detail && (
        <>
          <Section label="Containers">
            {detail.containers.length === 0 ? (
              <p className="text-xs text-[var(--text-tertiary)]">No containers — stack is on disk but not deployed.</p>
            ) : (
              <div className="flex flex-col -mx-1">
                {detail.containers.map((c) => (
                  <div key={c.id} className="px-1">
                    <ListRow
                      title={c.name}
                      subtitle={`${c.image} · ${c.status}`}
                      status={containerStatus(c.state, c.exit_code, c.health)}
                      onPress={() => nav(`/h/${hid}/c/${c.id}`)}
                    />
                  </div>
                ))}
              </div>
            )}
          </Section>

          {(detail.unsupported.length > 0 || Object.keys(detail.service_unsupported).length > 0) && (
            <Section label="Dropped">
              <p className="text-xs text-[var(--text-tertiary)] mb-1">
                Compose keys we recognized but don't translate. The deployed stack ignores them.
              </p>
              {detail.unsupported.length > 0 && (
                <p className="text-xs"><span className="text-[var(--text-tertiary)]">top-level:</span> {detail.unsupported.join(", ")}</p>
              )}
              {Object.entries(detail.service_unsupported).map(([svc, keys]) => (
                <p key={svc} className="text-xs"><span className="text-[var(--text-tertiary)]">{svc}:</span> {keys.join(", ")}</p>
              ))}
            </Section>
          )}

          <Section label="Compose YAML">
            <textarea
              className="input mono"
              spellCheck={false}
              autoCapitalize="off"
              autoCorrect="off"
              rows={14}
              value={yaml}
              onChange={(e) => setYaml(e.target.value)}
              style={{ minHeight: "260px", whiteSpace: "pre", overflowWrap: "normal", overflowX: "auto" }}
            />
          </Section>

          {actionError && <p className="text-[var(--error)] text-xs">{actionError}</p>}

          <Section label="Actions">
            <div className="flex gap-2">
              <Button onClick={onSave} disabled={pending !== null || !dirty} disallowReason={denyUpdate} className="flex-1">
                {pending === "save" ? "…" : "Save & redeploy"}
              </Button>
              <Button variant="ghost" onClick={() => nav(`/h/${hid}/stacks/${encodeURIComponent(name)}/logs`)} className="flex-1">
                Logs
              </Button>
            </div>
            <div className="flex gap-2 mt-2">
              <Button variant="ghost" onClick={onRedeploy} disabled={pending !== null} disallowReason={denyRedeploy} className="flex-1">
                {pending === "redeploy" ? "…" : "Redeploy"}
              </Button>
              <Button variant="ghost" onClick={onPull} disabled={pending !== null} disallowReason={denyPull} className="flex-1">
                {pending === "pull" ? "…" : "Pull & redeploy"}
              </Button>
            </div>
            <Button
              variant="destructive"
              onClick={() => setConfirmDelete(true)}
              disabled={pending !== null}
              disallowReason={denyDelete}
              className="mt-2"
            >
              Delete
            </Button>
          </Section>

          <ConfirmDialog
            open={confirmDelete}
            onOpenChange={setConfirmDelete}
            title={`Delete stack ${name}?`}
            description="Stops and removes all containers, drops the stack network, and removes the manifest. Named volumes are preserved."
            confirmLabel="Delete"
            destructive
            onConfirm={onDelete}
          />
        </>
      )}
    </Page>
  );
}
