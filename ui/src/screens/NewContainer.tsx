import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { call, streamOp, unwrap, type Host } from "@/api/client";
import { useHosts } from "@/state/hosts";
import { useQuery } from "@/state/cache";
import type {
  ContainerDetail as ContainerDetailT,
  ImageSummary,
  NetworkSummary,
  PortPublish,
  RestartPolicySpec,
  VolumeMount,
} from "@/api/types";
import { Button } from "@/components/Button";
import { Collapsible } from "@/components/Collapsible";
import { Combobox } from "@/components/Combobox";
import { EditCell } from "@/components/EditCell";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { Page, type Crumb } from "@/components/Page";
import { PullProgress, reducePull, type PullState } from "@/components/PullProgress";
import { Row } from "@/components/Row";
import { Spinner } from "@/components/Spinner";
import { scrollFocusedIntoView } from "@/lib/scrollIntoViewOnFocus";

interface FormState {
  image: string;
  name: string;
  ports: PortPublish[];
  env: string[];
  volumes: VolumeMount[];
  network: string;
  cmd: string[];
  entrypoint: string[];
  workingDir: string;
  user: string;
  restart: RestartPolicySpec;
}

const EMPTY_FORM: FormState = {
  image: "",
  name: "",
  ports: [],
  env: [],
  volumes: [],
  network: "",
  cmd: [],
  entrypoint: [],
  workingDir: "",
  user: "",
  restart: { kind: "unless_stopped" },
};

const RESTART_OPTIONS: { value: RestartPolicySpec["kind"]; label: string }[] = [
  { value: "no", label: "no" },
  { value: "on_failure", label: "on failure" },
  { value: "always", label: "always" },
  { value: "unless_stopped", label: "unless stopped" },
];

export function NewContainer() {
  const { hid, cid } = useParams<{ hid: string; cid?: string }>();
  const cloning = !!cid;
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };

  const [form, setForm] = useState<FormState>(EMPTY_FORM);
  const [pending, setPending] = useState<"create" | "create-start" | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [pull, setPull] = useState<PullState | null>(null);
  const [sourceName, setSourceName] = useState<string>("");

  const imagesKey = host ? `${host.url}:list_images` : null;
  const { data: images } = useQuery<ImageSummary[]>(imagesKey, async () => {
    const r = unwrap(await call(host!, { op: "list_images" }), "images");
    return r.data;
  });
  const localTags = imageTags(images);

  const networksKey = host ? `${host.url}:list_networks` : null;
  const { data: networks } = useQuery<NetworkSummary[]>(networksKey, async () => {
    const r = unwrap(await call(host!, { op: "list_networks" }), "networks");
    return r.data;
  });

  // Cloning: fetch the source container's inspect and pre-fill the form.
  useEffect(() => {
    if (!cloning || !host || !cid) return;
    let cancelled = false;
    (async () => {
      try {
        const r = unwrap(await call(host, { op: "get_container", id: cid }), "container_detail");
        if (cancelled) return;
        const d = r.data as ContainerDetailT;
        setSourceName(d.name);
        setForm({
          image: d.image,
          // Name cleared so the operator decides: same name (after stop+remove
          // of source) for downtime cutover, or a different name to run side
          // by side. Either way, leaving it blank lets the engine auto-name.
          name: "",
          ports: d.ports.map((p) => ({ container: p.container_port, host: p.host_port })),
          env: d.env.slice(),
          volumes: d.mounts
            .filter((m) => m.kind !== "tmpfs")
            .map((m) => ({ source: m.source, target: m.destination, read_only: !m.rw })),
          network: d.network_mode,
          cmd: d.cmd.slice(),
          entrypoint: d.entrypoint.slice(),
          workingDir: d.working_dir,
          user: d.user,
          restart: parseRestartPolicy(d.restart_policy),
        });
      } catch (e) {
        if (!cancelled) setError(`load source: ${(e as Error).message}`);
      }
    })();
    return () => { cancelled = true; };
  }, [cloning, host?.url, host?.token, cid]);

  async function submit(start: boolean) {
    if (!host) return;
    setPending(start ? "create-start" : "create");
    setError(null);
    const image = form.image.trim();
    try {
      // Pull only if the image isn't already local. CreateContainer rejects
      // non-local images — the API never auto-pulls.
      if (!localTags.has(image)) {
        setPull({ layers: {}, lastStatus: "starting pull…" });
        await streamOp(host, { op: "pull_image", reference: image }, (chunk) => {
          setPull((prev) => reducePull(prev ?? { layers: {}, lastStatus: "" }, chunk));
        });
      }
      // Pre-create named volumes — CreateContainer rejects unknown names.
      // /volumes/create is idempotent (existing names succeed), so we
      // don't need to query first.
      for (const v of form.volumes) {
        const src = v.source.trim();
        if (!src || src.startsWith("/") || src.startsWith("./") || src.startsWith("../")) continue;
        unwrap(await call(host, { op: "create_volume", name: src }), "ok");
      }
      const r = unwrap(
        await call(host, {
          op: "create_container",
          image,
          name: form.name.trim() || undefined,
          ports: form.ports.length ? form.ports : undefined,
          env: form.env.length ? form.env : undefined,
          volumes: form.volumes.length ? form.volumes : undefined,
          network: form.network.trim() || undefined,
          cmd: form.cmd.length ? form.cmd : undefined,
          entrypoint: form.entrypoint.length ? form.entrypoint : undefined,
          working_dir: form.workingDir.trim() || undefined,
          user: form.user.trim() || undefined,
          restart: form.restart,
        }),
        "container_created",
      );
      if (start) {
        unwrap(await call(host, { op: "start_container", id: r.data.id }), "ok");
      }
      nav(`/h/${hid}/c/${r.data.id}`, { replace: true });
    } catch (e) {
      setError(`create container: ${(e as Error).message}`);
      setPull(null);
    } finally {
      setPending(null);
    }
  }

  const sectionCrumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "containers");

  if (!saved) {
    return <Page><p>Unknown host.</p></Page>;
  }

  const crumbs: Crumb[] = [...sectionCrumbs, { kind: "link", label: "new container" }];
  const networkOptions = (networks ?? []).map((n) => ({ value: n.name }));
  const imageOptions = Array.from(localTags).sort().map((t) => ({ value: t }));

  return (
    <Page crumbs={crumbs}>
      {cloning && sourceName && (
        <p className="text-xs text-[var(--text-secondary)]">
          cloned from <span className="mono text-[var(--id-color)]">{sourceName}</span>
        </p>
      )}

      {error && <p className="text-[var(--error)] text-xs">{error}</p>}

      <form onSubmit={(e) => { e.preventDefault(); submit(true); }} className="contents" {...scrollFocusedIntoView()}>
        <Collapsible label="Container" defaultOpen>
          <Row
            label="Name"
            right={
              <EditCell
                mono
                value={form.name}
                placeholder="new container"
                onChange={(e) => setForm({ ...form, name: e.target.value })}
              />
            }
          />
          <Row
            label="Image"
            right={
              <Combobox
                cell
                mono
                freeText
                value={form.image}
                onChange={(v) => setForm({ ...form, image: v })}
                options={imageOptions}
              />
            }
          />
        </Collapsible>

        <Collapsible label="spec" defaultOpen>
          <Row
            label="Network"
            right={
              <Combobox
                cell
                mono
                freeText
                value={form.network}
                onChange={(v) => setForm({ ...form, network: v })}
                options={networkOptions}
              />
            }
          />
          <Row
            label="Restart"
            right={
              <Combobox
                cell
                dim={!cloning && form.restart.kind === EMPTY_FORM.restart.kind}
                value={form.restart.kind}
                onChange={(v) =>
                  setForm({ ...form, restart: { kind: v as RestartPolicySpec["kind"] } })
                }
                options={RESTART_OPTIONS}
              />
            }
          />
          <Row
            label="Entrypoint"
            right={
              <EditCell
                mono
                value={form.entrypoint.join(" ")}
                onChange={(e) =>
                  setForm({ ...form, entrypoint: splitTokens(e.target.value) })
                }
              />
            }
          />
          <Row
            label="Cmd"
            right={
              <EditCell
                mono
                value={form.cmd.join(" ")}
                onChange={(e) => setForm({ ...form, cmd: splitTokens(e.target.value) })}
              />
            }
          />
          <Row
            label="Working dir"
            right={
              <EditCell
                mono
                value={form.workingDir}
                onChange={(e) => setForm({ ...form, workingDir: e.target.value })}
              />
            }
          />
          <Row
            label="User"
            right={
              <EditCell
                mono
                value={form.user}
                onChange={(e) => setForm({ ...form, user: e.target.value })}
              />
            }
          />
        </Collapsible>

        <Collapsible label="ports" count={form.ports.length} defaultOpen>
          {form.ports.map((p, i) => (
            <PairRow
              key={i}
              left={p.container}
              right={p.host}
              inputModeRight="numeric"
              onChange={(left, right) => {
                const next = form.ports.slice();
                next[i] = { container: left, host: right };
                setForm({ ...form, ports: next });
              }}
              onRemove={() => setForm({ ...form, ports: form.ports.filter((_, j) => j !== i) })}
            />
          ))}
          <AddBtn
            label="add port"
            onClick={() =>
              setForm({ ...form, ports: [...form.ports, { container: "", host: "" }] })
            }
          />
        </Collapsible>

        <Collapsible label="volumes" count={form.volumes.length} defaultOpen>
          {form.volumes.map((m, i) => (
            <VolumeEntry
              key={i}
              mount={m}
              onChange={(patch) =>
                setForm({
                  ...form,
                  volumes: form.volumes.map((x, j) => (j === i ? { ...x, ...patch } : x)),
                })
              }
              onRemove={() =>
                setForm({ ...form, volumes: form.volumes.filter((_, j) => j !== i) })
              }
            />
          ))}
          <AddBtn
            label="add volume"
            onClick={() =>
              setForm({ ...form, volumes: [...form.volumes, { source: "", target: "" }] })
            }
          />
        </Collapsible>

        <Collapsible label="env" count={form.env.length} defaultOpen>
          {form.env.map((entry, i) => (
            <EnvRow
              key={i}
              entry={entry}
              onChange={(next) =>
                setForm({ ...form, env: form.env.map((x, j) => (j === i ? next : x)) })
              }
              onRemove={() => setForm({ ...form, env: form.env.filter((_, j) => j !== i) })}
            />
          ))}
          <AddBtn
            label="add variable"
            onClick={() => setForm({ ...form, env: [...form.env, ""] })}
          />
        </Collapsible>

        {pull && (
          <Collapsible label="pull progress" defaultOpen>
            <PullProgress pull={pull} />
          </Collapsible>
        )}

        <Collapsible label="create" defaultOpen>
          <div className="flex gap-2">
            <Button variant="ghost" onClick={() => nav(`/h/${hid}`)} className="flex-1">
              Cancel
            </Button>
            <SplitSubmit
              disabled={pending !== null || !form.image.trim()}
              pending={pending}
              onCreate={() => submit(false)}
              onCreateAndStart={() => submit(true)}
            />
          </div>
        </Collapsible>
      </form>
    </Page>
  );
}

/** Submit pill split into two halves: "Create" (just create, leave stopped)
 * and "Create & start" (create and start). One rounded shape with a 1px
 * divider down the middle so the user reads it as one control with two
 * complementary actions. */
function SplitSubmit({
  disabled,
  pending,
  onCreate,
  onCreateAndStart,
}: {
  disabled: boolean;
  pending: "create" | "create-start" | null;
  onCreate: () => void;
  onCreateAndStart: () => void;
}) {
  const half =
    "flex-1 flex items-center justify-center gap-1 text-[13px] font-medium text-[var(--accent)] py-2 px-3 disabled:opacity-30 disabled:cursor-not-allowed active:opacity-75 transition-opacity";
  return (
    <div className="flex-1 flex rounded-full overflow-hidden border border-[var(--accent-border)] bg-[var(--accent-soft)]">
      <button
        type="button"
        onClick={onCreate}
        disabled={disabled}
        className={half}
      >
        {pending === "create" ? <><Spinner /> Creating…</> : "Create"}
      </button>
      <span className="w-px bg-[var(--accent-border)] shrink-0" aria-hidden="true" />
      <button
        type="submit"
        onClick={onCreateAndStart}
        disabled={disabled}
        className={half}
      >
        {pending === "create-start" ? <><Spinner /> Starting…</> : "Start"}
      </button>
    </div>
  );
}

function imageTags(images: ImageSummary[] | null): Set<string> {
  const out = new Set<string>();
  if (!images) return out;
  for (const i of images) {
    if (i.repo_tag && !i.repo_tag.startsWith("<none>")) out.add(i.repo_tag);
  }
  return out;
}

function parseRestartPolicy(s: string): RestartPolicySpec {
  switch (s) {
    case "always":          return { kind: "always" };
    case "on-failure":      return { kind: "on_failure" };
    case "unless-stopped":  return { kind: "unless_stopped" };
    case "no":
    case "":
    default:                return { kind: "no" };
  }
}

// Split a free-form whitespace string into tokens. Round-trips with
// `tokens.join(" ")` for the common case; we accept that paths/args
// containing literal spaces lose that boundary — operators with that
// shape can keep editing via a different surface (compose stack).
function splitTokens(s: string): string[] {
  return s.split(/\s+/).filter(Boolean);
}

function AddBtn({ onClick, label }: { onClick: () => void; label: string }) {
  return (
    <button
      type="button"
      onClick={onClick}
      aria-label={label}
      className="text-sm text-[var(--text-tertiary)] hover:text-[var(--accent)] transition-colors self-start px-2 py-1 leading-none"
    >
      +
    </button>
  );
}

function PairRow({
  left,
  right,
  placeholderLeft,
  placeholderRight,
  inputModeLeft,
  inputModeRight,
  onChange,
  onRemove,
}: {
  left: string;
  right: string;
  placeholderLeft?: string;
  placeholderRight?: string;
  inputModeLeft?: React.HTMLAttributes<HTMLInputElement>["inputMode"];
  inputModeRight?: React.HTMLAttributes<HTMLInputElement>["inputMode"];
  onChange: (left: string, right: string) => void;
  onRemove: () => void;
}) {
  return (
    <div className="flex items-baseline gap-2">
      <EditCell
        mono
        className="flex-1"
        value={left}
        placeholder={placeholderLeft}
        inputMode={inputModeLeft}
        onChange={(e) => onChange(e.target.value, right)}
      />
      <span className="text-[var(--text-tertiary)] text-xs">→</span>
      <EditCell
        mono
        className="flex-1"
        value={right}
        placeholder={placeholderRight}
        inputMode={inputModeRight}
        onChange={(e) => onChange(left, e.target.value)}
      />
      <RemoveBtn onClick={onRemove} />
    </div>
  );
}

// Env row matches the KvLine layout from detail pages: 96px key column +
// flex-1 value column. The key column is mono and editable; the value
// column is mono and editable. Form state stores each entry as
// `KEY=VALUE`; we split on `=` for display and rejoin on edit.
function EnvRow({
  entry,
  onChange,
  onRemove,
}: {
  entry: string;
  onChange: (next: string) => void;
  onRemove: () => void;
}) {
  const eq = entry.indexOf("=");
  const k = eq >= 0 ? entry.slice(0, eq) : entry;
  const v = eq >= 0 ? entry.slice(eq + 1) : "";
  return (
    <div className="flex items-baseline gap-3">
      <div className="w-24 shrink-0">
        <EditCell
          mono
          value={k}
          onChange={(e) => onChange(`${e.target.value}=${v}`)}
        />
      </div>
      <EditCell
        mono
        className="flex-1"
        value={v}
        onChange={(e) => onChange(`${k}=${e.target.value}`)}
      />
      <RemoveBtn onClick={onRemove} />
    </div>
  );
}

function VolumeEntry({
  mount,
  onChange,
  onRemove,
}: {
  mount: VolumeMount;
  onChange: (patch: Partial<VolumeMount>) => void;
  onRemove: () => void;
}) {
  return (
    <div className="flex flex-col gap-1">
      <div className="flex items-baseline gap-2">
        <EditCell
          mono
          className="flex-1"
          value={mount.source}
          onChange={(e) => onChange({ source: e.target.value })}
        />
        <span className="text-[var(--text-tertiary)] text-xs">→</span>
        <EditCell
          mono
          className="flex-1"
          value={mount.target}
          onChange={(e) => onChange({ target: e.target.value })}
        />
        <RemoveBtn onClick={onRemove} />
      </div>
      <label className="flex items-center gap-2 cursor-pointer text-xs text-[var(--text-tertiary)]">
        <input
          type="checkbox"
          checked={!!mount.read_only}
          onChange={(e) => onChange({ read_only: e.target.checked })}
        />
        ro
      </label>
    </div>
  );
}

function RemoveBtn({ onClick }: { onClick: () => void }) {
  return (
    <button
      type="button"
      onClick={onClick}
      aria-label="remove"
      className="text-[var(--text-tertiary)] hover:text-[var(--error)] transition-colors text-sm leading-none px-1 shrink-0"
    >
      ×
    </button>
  );
}
