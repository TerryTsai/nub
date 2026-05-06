import { useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import { useHosts } from "@/state/hosts";
import { invalidate } from "@/state/cache";
import { Button } from "@/components/Button";
import { Collapsible } from "@/components/Collapsible";
import { EditCell } from "@/components/EditCell";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { Page, type Crumb } from "@/components/Page";
import { Row } from "@/components/Row";
import { Spinner } from "@/components/Spinner";
import { scrollFocusedIntoView } from "@/lib/scrollIntoViewOnFocus";

interface KvEntry {
  key: string;
  value: string;
}

export function NewVolume() {
  const { hid } = useParams<{ hid: string }>();
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };

  const [name, setName] = useState("");
  const [driver, setDriver] = useState("");
  const [options, setOptions] = useState<KvEntry[]>([]);
  const [labels, setLabels] = useState<KvEntry[]>([]);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!host) return;
    const trimmed = name.trim();
    if (!trimmed) return;
    setPending(true);
    setError(null);
    try {
      const drv = driver.trim();
      unwrap(
        await call(host, {
          op: "create_volume",
          name: trimmed,
          driver: drv || undefined,
          labels: toMap(labels),
          options: toMap(options),
        }),
        "ok",
      );
      invalidate(`${host.url}:list_volumes`);
      nav(`/h/${hid}/volumes/${encodeURIComponent(trimmed)}`, { replace: true });
    } catch (e) {
      setError(`create volume: ${(e as Error).message}`);
    } finally {
      setPending(false);
    }
  }

  const sectionCrumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "volumes");

  if (!saved) return <Page><p>Unknown host.</p></Page>;

  const crumbs: Crumb[] = [...sectionCrumbs, { kind: "link", label: "new volume" }];

  return (
    <Page crumbs={crumbs}>
      {error && <p className="text-[var(--error)] text-xs">{error}</p>}

      <form onSubmit={onSubmit} className="contents" {...scrollFocusedIntoView()}>
        <Collapsible label="Volume" defaultOpen>
          <Row
            label="Name"
            right={
              <EditCell
                mono
                value={name}
                placeholder="new volume"
                onChange={(e) => setName(e.target.value)}
              />
            }
          />
        </Collapsible>

        <Collapsible label="spec" defaultOpen>
          <Row
            label="Driver"
            right={
              <EditCell
                mono
                value={driver}
                placeholder="local"
                onChange={(e) => setDriver(e.target.value)}
              />
            }
          />
        </Collapsible>

        <Collapsible label="options" count={options.length} defaultOpen>
          {options.map((entry, i) => (
            <KvRow
              key={i}
              entry={entry}
              onChange={(next) =>
                setOptions(options.map((x, j) => (j === i ? next : x)))
              }
              onRemove={() => setOptions(options.filter((_, j) => j !== i))}
            />
          ))}
          <AddBtn
            label="add option"
            onClick={() => setOptions([...options, { key: "", value: "" }])}
          />
        </Collapsible>

        <Collapsible label="labels" count={labels.length} defaultOpen>
          {labels.map((entry, i) => (
            <KvRow
              key={i}
              entry={entry}
              onChange={(next) =>
                setLabels(labels.map((x, j) => (j === i ? next : x)))
              }
              onRemove={() => setLabels(labels.filter((_, j) => j !== i))}
            />
          ))}
          <AddBtn
            label="add label"
            onClick={() => setLabels([...labels, { key: "", value: "" }])}
          />
        </Collapsible>

        <Collapsible label="create" defaultOpen>
          <div className="flex gap-2">
            <Button
              variant="ghost"
              onClick={() => nav(`/h/${hid}/volumes`)}
              className="flex-1"
            >
              Cancel
            </Button>
            <Button
              type="submit"
              disabled={pending || !name.trim()}
              className="flex-1"
            >
              {pending ? (<><Spinner /> Creating…</>) : "Create"}
            </Button>
          </div>
        </Collapsible>
      </form>
    </Page>
  );
}

function toMap(entries: KvEntry[]): Record<string, string> | undefined {
  const out: Record<string, string> = {};
  for (const { key, value } of entries) {
    const k = key.trim();
    if (k) out[k] = value;
  }
  return Object.keys(out).length ? out : undefined;
}

function KvRow({
  entry,
  onChange,
  onRemove,
}: {
  entry: KvEntry;
  onChange: (next: KvEntry) => void;
  onRemove: () => void;
}) {
  return (
    <div className="flex items-baseline gap-3">
      <div className="w-24 shrink-0">
        <EditCell
          mono
          value={entry.key}
          placeholder="key"
          onChange={(e) => onChange({ ...entry, key: e.target.value })}
        />
      </div>
      <EditCell
        mono
        className="flex-1"
        value={entry.value}
        placeholder="value"
        onChange={(e) => onChange({ ...entry, value: e.target.value })}
      />
      <RemoveBtn onClick={onRemove} />
    </div>
  );
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
