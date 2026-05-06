import { useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import type { ImageDetail as ImageDetailT, ImageSummary } from "@/api/types";
import { useHosts } from "@/state/hosts";
import { invalidate, useQuery } from "@/state/cache";
import { imageStatus } from "@/state/status";
import { Button } from "@/components/Button";
import { Collapsible } from "@/components/Collapsible";
import { ConfirmDialog } from "@/components/ConfirmDialog";
import { CopyLine } from "@/components/CopyLine";
import { EmptyRow } from "@/components/EmptyRow";
import { KvLine } from "@/components/KvLine";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { Page, type Crumb } from "@/components/Page";
import { Row } from "@/components/Row";
import { Spinner } from "@/components/Spinner";
import { useToast } from "@/components/Toaster";

export function ImageDetail() {
  const { hid, iid } = useParams<{ hid: string; iid: string }>();
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };

  const [confirmOpen, setConfirmOpen] = useState(false);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const toast = useToast();

  const queryKey = host ? `${host.url}:list_images` : null;
  const { data: images } = useQuery<ImageSummary[]>(queryKey, async () => {
    const r = unwrap(await call(host!, { op: "list_images" }), "images");
    return r.data;
  });
  const image = images?.find((i) => i.id === iid);

  const inspectKey = host && iid ? `${host.url}:get_image:${iid}` : null;
  const { data: detail } = useQuery<ImageDetailT>(inspectKey, async () => {
    const r = unwrap(await call(host!, { op: "get_image", id: iid! }), "image_detail");
    return r.data;
  });

  async function onRemove() {
    if (!host || !iid) return;
    setPending(true);
    setError(null);
    try {
      unwrap(await call(host, { op: "delete_image", id: iid }), "ok");
      if (queryKey) invalidate(queryKey);
      nav(`/h/${hid}/images`, { replace: true });
    } catch (e) {
      const msg = (e as Error).message;
      setError(`remove: ${msg}`);
      toast.pushOpError("remove", e);
    } finally {
      setPending(false);
    }
  }

  const sectionCrumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "images");

  if (!saved) return <Page><p>Unknown host.</p></Page>;

  const title = image?.repo_tag && !image.repo_tag.startsWith("<none>") ? image.repo_tag : iid ?? "?";
  const crumbs: Crumb[] = [
    ...sectionCrumbs,
    { kind: "link", label: title },
  ];

  return (
    <Page crumbs={crumbs}>
      {error && <p className="text-[var(--error)] text-xs">{error}</p>}

      <Collapsible label="Image" defaultOpen>
        <Row label="Name" value={title} mono />
        <Row label="Status" value={image ? imageStatus(image.containers).label : undefined} />
        <Row label="ID" value={image?.id} mono />
        <Row label="Tag" value={image?.repo_tag} mono />
        <Row label="Created" value={image ? formatTimestamp(image.created) : undefined} mono />
      </Collapsible>

      <Collapsible label="spec">
        <Row label="Platform" value={platformLabel(detail)} />
        <Row label="Layers" value={detail ? String(detail.layers) : undefined} />
        <Row label="Size" value={image ? formatBytes(image.size) : undefined} />
        <Row label="Entrypoint" value={detail?.entrypoint.join(" ")} mono />
        <Row label="Cmd" value={detail?.cmd.join(" ")} mono />
        <Row label="Working dir" value={detail?.working_dir} mono />
        <Row label="User" value={detail?.user} mono />
        <Row label="Exposed" value={detail?.exposed_ports.join(", ")} mono />
      </Collapsible>

      <Collapsible label="runtime">
        <Row label="In use by" value={image ? `${image.containers} container${image.containers === 1 ? "" : "s"}` : undefined} />
      </Collapsible>

      <Collapsible label="digests" count={detail?.repo_digests.length}>
        {detail && detail.repo_digests.length > 0
          ? detail.repo_digests.map((d, i) => <CopyLine key={i} value={d} />)
          : <EmptyRow />}
      </Collapsible>

      <Collapsible label="env" count={detail?.env.length}>
        {detail && detail.env.length > 0
          ? detail.env.map((e, i) => {
              const eq = e.indexOf("=");
              const k = eq >= 0 ? e.slice(0, eq) : e;
              const v = eq >= 0 ? e.slice(eq + 1) : "";
              return <KvLine key={i} k={k} v={v} copyAs={e} />;
            })
          : <EmptyRow />}
      </Collapsible>

      <Collapsible label="labels" count={detail ? Object.keys(detail.labels).length : undefined}>
        {detail && Object.keys(detail.labels).length > 0
          ? Object.entries(detail.labels).map(([k, v]) => (
              <KvLine key={k} k={k} v={v} copyAs={`${k}=${v}`} />
            ))
          : <EmptyRow />}
      </Collapsible>

      <Collapsible label="danger">
        <Button
          variant="destructive"
          disabled={!image || pending}
          onClick={() => setConfirmOpen(true)}
        >
          {pending ? <><Spinner /> Removing…</> : "Remove"}
        </Button>
      </Collapsible>

      {image && (
        <ConfirmDialog
          open={confirmOpen}
          onOpenChange={setConfirmOpen}
          title={`Remove ${title}?`}
          description={image.containers > 0
            ? `In use by ${image.containers} container${image.containers === 1 ? "" : "s"}. Remove the dependent containers first.`
            : "This will delete the image."}
          confirmLabel="Remove"
          destructive
          onConfirm={onRemove}
        />
      )}
    </Page>
  );
}

function platformLabel(detail: ImageDetailT | null): string | undefined {
  if (!detail) return undefined;
  if (!detail.architecture) return undefined;
  return `${detail.os}/${detail.architecture}`;
}

function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  const units = ["KB", "MB", "GB", "TB"];
  let v = n / 1024;
  for (const u of units) {
    if (v < 1024) return `${v.toFixed(v < 10 ? 1 : 0)} ${u}`;
    v /= 1024;
  }
  return `${v.toFixed(0)} PB`;
}

function formatTimestamp(secs: number): string {
  return new Date(secs * 1000).toISOString().replace("T", " ").replace(/\.\d+Z$/, "Z");
}
