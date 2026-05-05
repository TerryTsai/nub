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
import { Combobox } from "@/components/Combobox";
import { EditCell } from "@/components/EditCell";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { Page, type Crumb } from "@/components/Page";
import { PullProgress, reducePull, type PullState } from "@/components/PullProgress";
import { Row } from "@/components/Row";
import { Section } from "@/components/Section";

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
  start: boolean;
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
  start: true,
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
  const [pending, setPending] = useState(false);
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
          start: true,
        });
      } catch (e) {
        if (!cancelled) setError((e as Error).message);
      }
    })();
    return () => { cancelled = true; };
  }, [cloning, host?.url, host?.token, cid]);

  async function onSubmit(e: React.FormEvent) {
    if (!host) return;
    e.preventDefault();
    setPending(true);
    setError(null);
    const image = form.image.trim();
    try {
      // Pull only if the image isn't already local. Skips a confusing stall
      // on Podman's compat /containers/create, which doesn't auto-pull.
      if (!localTags.has(image)) {
        setPull({ layers: {}, lastStatus: "starting pull…" });
        await streamOp(host, { op: "pull_image", reference: image }, (chunk) => {
          setPull((prev) => reducePull(prev ?? { layers: {}, lastStatus: "" }, chunk));
        });
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
          start: form.start,
        }),
        "container_created",
      );
      nav(`/h/${hid}/c/${r.data.id}`, { replace: true });
    } catch (e) {
      setError((e as Error).message);
      setPull(null);
    } finally {
      setPending(false);
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
      <header className="flex flex-col gap-1">
        <span className="text-[10px] font-semibold uppercase tracking-wider text-[var(--text-tertiary)]">
          CONTAINER
        </span>
        <input
          className="bg-transparent border-0 border-b border-dashed border-transparent hover:border-[var(--border-strong)] focus:border-[var(--accent)] focus:border-solid focus:outline-none text-base font-semibold w-full transition-colors placeholder:text-[var(--text-tertiary)] placeholder:font-normal placeholder:italic"
          type="text"
          autoCapitalize="off"
          autoCorrect="off"
          spellCheck={false}
          placeholder="new container"
          value={form.name}
          onChange={(e) => setForm({ ...form, name: e.target.value })}
        />
      </header>

      {cloning && sourceName && (
        <p className="text-xs text-[var(--text-secondary)]">
          cloned from <span className="mono text-[var(--id-color)]">{sourceName}</span>
        </p>
      )}

      <form onSubmit={onSubmit} className="contents">
        <Section>
          <Row
            label="Image"
            right={
              <Combobox
                cell
                mono
                freeText
                freeTextHint="type or pick"
                placeholder="nginx:alpine"
                value={form.image}
                onChange={(v) => setForm({ ...form, image: v })}
                options={imageOptions}
              />
            }
          />
          <Row
            label="Network"
            right={
              <Combobox
                cell
                mono
                freeText
                freeTextHint="type or pick"
                placeholder="default"
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
                placeholder="/usr/bin/myprog"
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
                placeholder="--flag value"
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
                placeholder="/app"
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
                placeholder="1000 or 1000:1000"
                value={form.user}
                onChange={(e) => setForm({ ...form, user: e.target.value })}
              />
            }
          />
        </Section>

        <Section
          label="ports"
          right={
            <AddBtn
              label="add port"
              onClick={() =>
                setForm({ ...form, ports: [...form.ports, { container: "", host: "" }] })
              }
            />
          }
        >
          {form.ports.length === 0 ? (
            <Empty>no ports published</Empty>
          ) : (
            form.ports.map((p, i) => (
              <PairRow
                key={i}
                left={p.container}
                right={p.host}
                placeholderLeft="80/tcp"
                placeholderRight="8080"
                onChange={(left, right) => {
                  const next = form.ports.slice();
                  next[i] = { container: left, host: right };
                  setForm({ ...form, ports: next });
                }}
                onRemove={() => setForm({ ...form, ports: form.ports.filter((_, j) => j !== i) })}
              />
            ))
          )}
        </Section>

        <Section
          label="volumes"
          right={
            <AddBtn
              label="add volume"
              onClick={() =>
                setForm({ ...form, volumes: [...form.volumes, { source: "", target: "" }] })
              }
            />
          }
        >
          {form.volumes.length === 0 ? (
            <Empty>no volumes mounted</Empty>
          ) : (
            form.volumes.map((m, i) => (
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
            ))
          )}
        </Section>

        <Section
          label="environment"
          right={
            <AddBtn
              label="add variable"
              onClick={() => setForm({ ...form, env: [...form.env, ""] })}
            />
          }
        >
          {form.env.length === 0 ? (
            <Empty>no environment set</Empty>
          ) : (
            form.env.map((v, i) => (
              <SoloRow
                key={i}
                value={v}
                placeholder="KEY=value"
                onChange={(next) =>
                  setForm({ ...form, env: form.env.map((x, j) => (j === i ? next : x)) })
                }
                onRemove={() =>
                  setForm({ ...form, env: form.env.filter((_, j) => j !== i) })
                }
              />
            ))
          )}
        </Section>

        {pull && (
          <Section label="pull progress">
            <PullProgress pull={pull} />
          </Section>
        )}

        {error && <p className="text-[var(--error)] text-xs">{error}</p>}

        <Section label="create">
          <label className="flex items-center gap-2 cursor-pointer text-xs text-[var(--text-secondary)]">
            <input
              type="checkbox"
              checked={form.start}
              onChange={(e) => setForm({ ...form, start: e.target.checked })}
            />
            start after create
          </label>
          <div className="flex gap-2">
            <Button variant="ghost" onClick={() => nav(`/h/${hid}`)} className="flex-1">
              Cancel
            </Button>
            <Button
              type="submit"
              disabled={pending || !form.image.trim()}
              className="flex-1"
            >
              {pending ? "…" : "Create"}
            </Button>
          </div>
        </Section>
      </form>
    </Page>
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
      className="text-sm leading-none text-[var(--text-tertiary)] hover:text-[var(--accent)] transition-colors px-2"
    >
      +
    </button>
  );
}

function Empty({ children }: { children: React.ReactNode }) {
  return (
    <span className="text-[12px] italic text-[var(--text-tertiary)]">{children}</span>
  );
}

function PairRow({
  left,
  right,
  placeholderLeft,
  placeholderRight,
  onChange,
  onRemove,
}: {
  left: string;
  right: string;
  placeholderLeft: string;
  placeholderRight: string;
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
        onChange={(e) => onChange(e.target.value, right)}
      />
      <span className="text-[var(--text-tertiary)] text-xs">→</span>
      <EditCell
        mono
        className="flex-1"
        value={right}
        placeholder={placeholderRight}
        onChange={(e) => onChange(left, e.target.value)}
      />
      <RemoveBtn onClick={onRemove} />
    </div>
  );
}

function SoloRow({
  value,
  placeholder,
  onChange,
  onRemove,
}: {
  value: string;
  placeholder: string;
  onChange: (v: string) => void;
  onRemove: () => void;
}) {
  return (
    <div className="flex items-baseline gap-2">
      <EditCell
        mono
        className="flex-1"
        value={value}
        placeholder={placeholder}
        onChange={(e) => onChange(e.target.value)}
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
          placeholder="volume-name or /host/path"
          onChange={(e) => onChange({ source: e.target.value })}
        />
        <span className="text-[var(--text-tertiary)] text-xs">→</span>
        <EditCell
          mono
          className="flex-1"
          value={mount.target}
          placeholder="/container/path"
          onChange={(e) => onChange({ target: e.target.value })}
        />
        <RemoveBtn onClick={onRemove} />
      </div>
      <label className="flex items-center gap-2 cursor-pointer text-[11px] text-[var(--text-tertiary)]">
        <input
          type="checkbox"
          checked={!!mount.read_only}
          onChange={(e) => onChange({ read_only: e.target.checked })}
        />
        read only
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
