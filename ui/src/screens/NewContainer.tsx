import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { call, streamOp, unwrap, type Host } from "@/api/client";
import { useHosts } from "@/state/hosts";
import { useSession } from "@/state/session";
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
import { Field } from "@/components/Field";
import { Heading } from "@/components/Heading";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { Page, type Crumb } from "@/components/Page";
import { PullProgress, reducePull, type PullState } from "@/components/PullProgress";
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

export function NewContainer() {
  const { hid, cid } = useParams<{ hid: string; cid?: string }>();
  const cloning = !!cid;
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };
  const session = useSession(host);

  const [form, setForm] = useState<FormState>(EMPTY_FORM);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [pull, setPull] = useState<PullState | null>(null);
  const [sourceName, setSourceName] = useState<string>("");

  const imagesKey = host && session.session ? `${host.url}:list_images` : null;
  const { data: images } = useQuery<ImageSummary[]>(imagesKey, async () => {
    const r = unwrap(await call(host!, { op: "list_images" }), "images");
    return r.data;
  });
  const localTags = imageTags(images);

  const networksKey = host && session.session ? `${host.url}:list_networks` : null;
  const { data: networks } = useQuery<NetworkSummary[]>(networksKey, async () => {
    const r = unwrap(await call(host!, { op: "list_networks" }), "networks");
    return r.data;
  });

  // Cloning: fetch the source container's inspect and pre-fill the form.
  useEffect(() => {
    if (!cloning || !host || !session.session || !cid) return;
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
  }, [cloning, host?.url, host?.token, !!session.session, cid]);

  const denyReason =
    session.session && !session.session.can("containers:create")
      ? "your token doesn't allow containers:create"
      : undefined;

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

  return (
    <Page crumbs={crumbs}>
      <Heading category="Container" title="New container" />

      {cloning && sourceName && (
        <p className="text-xs text-[var(--text-secondary)]">
          cloned from <span className="mono text-[var(--id-color)]">{sourceName}</span> — fields
          are pre-filled; edit before creating
        </p>
      )}

      {denyReason && <p className="text-[var(--warn)] text-xs">{denyReason}</p>}

      <form onSubmit={onSubmit} className="contents">
        <Section label="Container">
          <Field
            label="Image"
            hint={
              localTags.size > 0
                ? "pick a pulled image, or type a reference (will pull on create)"
                : "e.g. nginx:alpine, postgres:16, ghcr.io/..."
            }
          >
            <Combobox
              value={form.image}
              onChange={(v) => setForm({ ...form, image: v })}
              placeholder="image:tag"
              freeText
              freeTextHint="type or pick"
              mono
              options={Array.from(localTags).sort().map((t) => ({ value: t }))}
            />
          </Field>
          <Field label="Name" hint="optional — engine auto-names if blank">
            <input
              className="input mono"
              type="text"
              autoCapitalize="off"
              autoCorrect="off"
              spellCheck={false}
              placeholder="my-app"
              value={form.name}
              onChange={(e) => setForm({ ...form, name: e.target.value })}
            />
          </Field>
        </Section>

        <Section label="Ports">
          <PairList
            pairs={form.ports.map((p) => [p.container, p.host])}
            placeholder={["80/tcp", "8080"]}
            addLabel="+ Add port"
            onChange={(rows) =>
              setForm({ ...form, ports: rows.map(([container, host]) => ({ container, host })) })
            }
          />
        </Section>

        <Section label="Volumes">
          <VolumeList
            mounts={form.volumes}
            onChange={(volumes) => setForm({ ...form, volumes })}
          />
        </Section>

        <Section label="Environment">
          <StringList
            values={form.env}
            placeholder="KEY=value"
            addLabel="+ Add variable"
            onChange={(env) => setForm({ ...form, env })}
          />
        </Section>

        <Section label="Network">
          <Field label="Mode" hint="leave blank for default; nub rejects host/container modes">
            <Combobox
              value={form.network}
              onChange={(v) => setForm({ ...form, network: v })}
              placeholder="default"
              freeText
              freeTextHint="type or pick"
              mono
              options={networkOptions}
            />
          </Field>
        </Section>

        <Section label="Process">
          <Field label="Entrypoint" hint="overrides the image's ENTRYPOINT; one token per row">
            <StringList
              values={form.entrypoint}
              placeholder="/usr/bin/myprog"
              addLabel="+ Add token"
              onChange={(entrypoint) => setForm({ ...form, entrypoint })}
            />
          </Field>
          <Field label="Cmd" hint="overrides the image's CMD; one token per row">
            <StringList
              values={form.cmd}
              placeholder="--flag"
              addLabel="+ Add token"
              onChange={(cmd) => setForm({ ...form, cmd })}
            />
          </Field>
          <Field label="Working dir" hint="optional">
            <input
              className="input mono"
              type="text"
              autoCapitalize="off"
              autoCorrect="off"
              spellCheck={false}
              placeholder="/app"
              value={form.workingDir}
              onChange={(e) => setForm({ ...form, workingDir: e.target.value })}
            />
          </Field>
          <Field label="User" hint="UID, name, or UID:GID — optional">
            <input
              className="input mono"
              type="text"
              autoCapitalize="off"
              autoCorrect="off"
              spellCheck={false}
              placeholder="1000:1000"
              value={form.user}
              onChange={(e) => setForm({ ...form, user: e.target.value })}
            />
          </Field>
        </Section>

        <Section label="Restart policy">
          <RestartPicker value={form.restart} onChange={(restart) => setForm({ ...form, restart })} />
          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={form.start}
              onChange={(e) => setForm({ ...form, start: e.target.checked })}
            />
            <span className="text-xs">Start after create</span>
          </label>
        </Section>

        {pull && (
          <Section label="Pull progress">
            <PullProgress pull={pull} />
          </Section>
        )}

        {error && <p className="text-[var(--error)] text-xs">{error}</p>}

        <Section label="Actions">
          <div className="flex gap-2">
            <Button variant="ghost" onClick={() => nav(`/h/${hid}`)} className="flex-1">
              Cancel
            </Button>
            <Button
              type="submit"
              disabled={pending || !form.image.trim()}
              disallowReason={denyReason}
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

function PairList({
  pairs,
  placeholder,
  addLabel,
  onChange,
}: {
  pairs: [string, string][];
  placeholder: [string, string];
  addLabel: string;
  onChange: (rows: [string, string][]) => void;
}) {
  function set(i: number, j: 0 | 1, v: string) {
    const next = pairs.map((p) => [...p] as [string, string]);
    next[i][j] = v;
    onChange(next);
  }
  return (
    <div className="flex flex-col gap-2">
      {pairs.map((p, i) => (
        <div key={i} className="flex gap-2">
          <input
            className="input mono flex-1"
            value={p[0]}
            placeholder={placeholder[0]}
            onChange={(e) => set(i, 0, e.target.value)}
          />
          <span className="self-center text-[var(--text-tertiary)]">→</span>
          <input
            className="input mono flex-1"
            value={p[1]}
            placeholder={placeholder[1]}
            onChange={(e) => set(i, 1, e.target.value)}
          />
          <Button variant="ghost" onClick={() => onChange(pairs.filter((_, j) => j !== i))}>
            ×
          </Button>
        </div>
      ))}
      <Button variant="ghost" onClick={() => onChange([...pairs, ["", ""]])} className="self-start">
        {addLabel}
      </Button>
    </div>
  );
}

function VolumeList({
  mounts,
  onChange,
}: {
  mounts: VolumeMount[];
  onChange: (m: VolumeMount[]) => void;
}) {
  function update(i: number, patch: Partial<VolumeMount>) {
    onChange(mounts.map((x, j) => (j === i ? { ...x, ...patch } : x)));
  }
  return (
    <div className="flex flex-col gap-2">
      {mounts.map((m, i) => (
        <div key={i} className="flex flex-col gap-1.5 border border-[var(--border-subtle)] rounded-[var(--radius-md)] p-2">
          <div className="flex gap-2">
            <input
              className="input mono flex-1"
              value={m.source}
              placeholder="volume-name or /host/path"
              onChange={(e) => update(i, { source: e.target.value })}
            />
            <span className="self-center text-[var(--text-tertiary)]">→</span>
            <input
              className="input mono flex-1"
              value={m.target}
              placeholder="/container/path"
              onChange={(e) => update(i, { target: e.target.value })}
            />
            <Button variant="ghost" onClick={() => onChange(mounts.filter((_, j) => j !== i))}>
              ×
            </Button>
          </div>
          <label className="flex items-center gap-2 cursor-pointer pl-1">
            <input
              type="checkbox"
              checked={!!m.read_only}
              onChange={(e) => update(i, { read_only: e.target.checked })}
            />
            <span className="text-[11px] text-[var(--text-tertiary)]">read only</span>
          </label>
        </div>
      ))}
      <Button
        variant="ghost"
        onClick={() => onChange([...mounts, { source: "", target: "" }])}
        className="self-start"
      >
        + Add volume
      </Button>
    </div>
  );
}

function StringList({
  values,
  placeholder,
  addLabel,
  onChange,
}: {
  values: string[];
  placeholder: string;
  addLabel: string;
  onChange: (vs: string[]) => void;
}) {
  return (
    <div className="flex flex-col gap-2">
      {values.map((v, i) => (
        <div key={i} className="flex gap-2">
          <input
            className="input mono flex-1"
            value={v}
            placeholder={placeholder}
            onChange={(e) => onChange(values.map((x, j) => (j === i ? e.target.value : x)))}
          />
          <Button variant="ghost" onClick={() => onChange(values.filter((_, j) => j !== i))}>
            ×
          </Button>
        </div>
      ))}
      <Button variant="ghost" onClick={() => onChange([...values, ""])} className="self-start">
        {addLabel}
      </Button>
    </div>
  );
}

function RestartPicker({
  value,
  onChange,
}: {
  value: RestartPolicySpec;
  onChange: (v: RestartPolicySpec) => void;
}) {
  const opts: { kind: RestartPolicySpec["kind"]; label: string }[] = [
    { kind: "no", label: "No" },
    { kind: "on_failure", label: "On failure" },
    { kind: "always", label: "Always" },
    { kind: "unless_stopped", label: "Unless stopped" },
  ];
  return (
    <div className="grid grid-cols-2 gap-2">
      {opts.map((o) => {
        const active = value.kind === o.kind;
        return (
          <button
            key={o.kind}
            type="button"
            onClick={() => onChange({ kind: o.kind } as RestartPolicySpec)}
            className={`btn ${active ? "btn-primary" : "btn-ghost"}`}
          >
            {o.label}
          </button>
        );
      })}
    </div>
  );
}
