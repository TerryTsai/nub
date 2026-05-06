import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import type { StackDetail } from "@/api/types";
import { useHosts } from "@/state/hosts";
import { useQuery, invalidate } from "@/state/cache";
import { containerStatus } from "@/state/status";
import { ActionMenu } from "@/components/ActionMenu";
import { Button } from "@/components/Button";
import { Collapsible } from "@/components/Collapsible";
import { ConfirmDialog } from "@/components/ConfirmDialog";
import { Heading } from "@/components/Heading";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { ListRow } from "@/components/ListRow";
import { Row } from "@/components/Row";
import { Page, type Crumb } from "@/components/Page";
import { Section } from "@/components/Section";
import { Spinner } from "@/components/Spinner";
import { useToast } from "@/components/Toaster";

type Pending = "redeploy" | "pull" | "save" | "delete" | null;

export function StackDetailScreen() {
  const { hid, sname } = useParams<{ hid: string; sname: string }>();
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };
  const toast = useToast();
  const name = sname ? decodeURIComponent(sname) : "";

  const queryKey = host && name ? `${host.url}:get_stack:${name}` : null;
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
      const msg = (e as Error).message;
      setActionError(`${kind}: ${msg}`);
      toast.pushOpError(kind, e);
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
      const msg = (e as Error).message;
      setActionError(`delete: ${msg}`);
      toast.pushOpError("delete", e);
      setPending(null);
    }
  }

  const dirty = yaml !== original;

  const actionsDisabled = !detail || pending !== null;
  const subnav = (
    <>
      <Button
        size="sm"
        variant="primary"
        onClick={onSave}
        disabled={actionsDisabled || !dirty}
      >
        {pending === "save" ? <Spinner /> : "Save"}
      </Button>
      <ActionMenu
        items={[
          { label: "Redeploy", onClick: onRedeploy, disabled: actionsDisabled },
          { label: "Pull", onClick: onPull, disabled: actionsDisabled },
          { label: "Logs", to: `/h/${hid}/stacks/${encodeURIComponent(name)}/logs`, disabled: !detail },
        ]}
      />
    </>
  );

  const droppedKeysCount = detail
    ? detail.unsupported.length + Object.keys(detail.service_unsupported).length
    : undefined;

  return (
    <Page crumbs={crumbs} subnav={subnav}>
      <Heading category="Stack" title={name} />

      {error && <p className="text-[var(--error)] text-xs">{error}</p>}
      {actionError && <p className="text-[var(--error)] text-xs">{actionError}</p>}

      <Section>
        <Row label="Network" value={detail?.network_name} mono />
        <Row label="Modified" value={detail?.modified_at} mono />
      </Section>

      <Collapsible label="containers" count={detail?.containers.length}>
        {!detail ? (
          <p className="text-xs text-[var(--text-tertiary)]">—</p>
        ) : detail.containers.length === 0 ? (
          <p className="text-xs text-[var(--text-tertiary)]">On disk, not deployed.</p>
        ) : (
          <div className="flex flex-col">
            {detail.containers.map((c) => (
              <ListRow
                key={c.id}
                title={c.name}
                mono
                subtitle={`${c.image} · ${c.status}`}
                status={containerStatus(c.state, c.exit_code, c.health)}
                onPress={() => nav(`/h/${hid}/c/${c.id}`)}
              />
            ))}
          </div>
        )}
      </Collapsible>

      <Collapsible label="compose">
        <textarea
          className="input-code"
          spellCheck={false}
          autoCapitalize="off"
          autoCorrect="off"
          rows={14}
          value={yaml}
          onChange={(e) => setYaml(e.target.value)}
          style={{ minHeight: "260px" }}
          disabled={!detail}
        />
      </Collapsible>

      <Collapsible label="dropped keys" count={droppedKeysCount}>
        {!detail || (detail.unsupported.length === 0 && Object.keys(detail.service_unsupported).length === 0) ? (
          <p className="text-xs text-[var(--text-tertiary)]">—</p>
        ) : (
          <>
            <p className="text-xs text-[var(--text-tertiary)]">
              Compose keys we recognized but don't translate. The deployed stack ignores them.
            </p>
            {detail.unsupported.length > 0 && (
              <p className="text-xs"><span className="text-[var(--text-tertiary)]">top-level:</span> {detail.unsupported.join(", ")}</p>
            )}
            {Object.entries(detail.service_unsupported).map(([svc, keys]) => (
              <p key={svc} className="text-xs"><span className="text-[var(--text-tertiary)]">{svc}:</span> {keys.join(", ")}</p>
            ))}
          </>
        )}
      </Collapsible>

      <Section label="danger">
        <Button
          variant="destructive"
          onClick={() => setConfirmDelete(true)}
          disabled={!detail || pending !== null}
        >
          {pending === "delete" ? <><Spinner /> Deleting…</> : "Delete"}
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
    </Page>
  );
}
