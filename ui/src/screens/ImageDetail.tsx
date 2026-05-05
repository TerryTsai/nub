import { useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import type { ImageDetail as ImageDetailT, ImageSummary } from "@/api/types";
import { useHosts } from "@/state/hosts";
import { invalidate, useQuery } from "@/state/cache";
import { imageStatus } from "@/state/status";
import { Button } from "@/components/Button";
import { ConfirmDialog } from "@/components/ConfirmDialog";
import { Heading } from "@/components/Heading";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { Page, type Crumb } from "@/components/Page";
import { Row } from "@/components/Row";
import { Section } from "@/components/Section";
import { StatusBadge } from "@/components/StatusBadge";

export function ImageDetail() {
  const { hid, iid } = useParams<{ hid: string; iid: string }>();
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };

  const [confirmOpen, setConfirmOpen] = useState(false);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);

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

  async function onRemove(force: boolean) {
    if (!host || !iid) return;
    setPending(true);
    setError(null);
    try {
      unwrap(await call(host, { op: "delete_image", id: iid, force }), "ok");
      if (queryKey) invalidate(queryKey);
      nav(`/h/${hid}/images`, { replace: true });
    } catch (e) {
      setError((e as Error).message);
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
      <Heading
        category="Image"
        title={title}
        right={image && <StatusBadge status={imageStatus(image.containers)} />}
      />

      {!image && (
        <p className="text-xs text-[var(--text-tertiary)]">Loading…</p>
      )}

      {image && (
        <>
          <Section>
            <Row label="ID" value={image.id} mono />
            <Row label="Tag" value={image.repo_tag} mono />
            <Row label="Size" value={formatBytes(image.size)} />
            {detail?.architecture && <Row label="Platform" value={`${detail.os}/${detail.architecture}`} />}
            {detail && <Row label="Layers" value={String(detail.layers)} />}
            {detail && detail.entrypoint.length > 0 && (
              <Row label="Entrypoint" value={detail.entrypoint.join(" ")} mono />
            )}
            {detail && detail.cmd.length > 0 && <Row label="Cmd" value={detail.cmd.join(" ")} mono />}
            {detail?.working_dir && <Row label="Working dir" value={detail.working_dir} mono />}
            {detail?.user && <Row label="User" value={detail.user} mono />}
            {detail && detail.exposed_ports.length > 0 && (
              <Row label="Exposed" value={detail.exposed_ports.join(", ")} mono />
            )}
            <Row label="Created" value={formatTimestamp(image.created)} />
            <Row label="In use by" value={`${image.containers} container${image.containers === 1 ? "" : "s"}`} />
            {detail && detail.env.length > 0 && (
              <Row
                label={`Env (${detail.env.length})`}
                right={
                  <pre className="text-xs mono whitespace-pre-wrap break-all text-[var(--id-color)] leading-5">
                    {detail.env.join("\n")}
                  </pre>
                }
              />
            )}
          </Section>

          <Section label="danger">
            <Button
              variant="destructive"
              disabled={pending}
              onClick={() => setConfirmOpen(true)}
            >
              {pending ? "…" : "Remove"}
            </Button>
            {error && <p className="text-[var(--error)] text-xs">{error}</p>}
          </Section>

          <ConfirmDialog
            open={confirmOpen}
            onOpenChange={setConfirmOpen}
            title={`Remove ${title}?`}
            description={image.containers > 0
              ? `In use by ${image.containers} container${image.containers === 1 ? "" : "s"}. Remove with force?`
              : "This will delete the image."}
            confirmLabel={image.containers > 0 ? "Force remove" : "Remove"}
            destructive
            onConfirm={() => onRemove(image.containers > 0)}
          />
        </>
      )}
    </Page>
  );
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
