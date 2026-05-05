import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import type { DockerfileContent } from "@/api/types";
import { useHosts } from "@/state/hosts";
import { invalidate } from "@/state/cache";
import { Button } from "@/components/Button";
import { ConfirmDialog } from "@/components/ConfirmDialog";
import { Heading } from "@/components/Heading";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { Page, type Crumb } from "@/components/Page";
import { Row } from "@/components/Row";
import { Section } from "@/components/Section";
import { Spinner } from "@/components/Spinner";
import { scrollFocusedIntoView } from "@/lib/scrollIntoViewOnFocus";

const NEW_SENTINEL = "_new";

export function DockerfileEdit() {
  const { hid, name: rawName } = useParams<{ hid: string; name: string }>();
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };
  const isNew = rawName === NEW_SENTINEL;
  const name = isNew ? "" : decodeURIComponent(rawName ?? "");

  const [draftName, setDraftName] = useState(name);
  const [content, setContent] = useState("");
  const [original, setOriginal] = useState("");
  const [meta, setMeta] = useState<{ size: number; modified_at: string } | null>(null);
  const [loading, setLoading] = useState(!isNew);
  const [pending, setPending] = useState<"save" | "delete" | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [confirmDelete, setConfirmDelete] = useState(false);

  useEffect(() => {
    if (isNew || !host) return;
    let cancelled = false;
    (async () => {
      try {
        const r = unwrap(await call(host, { op: "get_dockerfile", name }), "dockerfile");
        if (cancelled) return;
        const d = r.data as DockerfileContent;
        setContent(d.content);
        setOriginal(d.content);
        setMeta({ size: d.size, modified_at: d.modified_at });
      } catch (e) {
        if (!cancelled) setError(`load dockerfile: ${(e as Error).message}`);
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => { cancelled = true; };
  }, [host?.url, host?.token, name, isNew]);

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!host) return;
    const targetName = isNew ? draftName.trim() : name;
    setPending("save");
    setError(null);
    try {
      unwrap(
        await call(host, { op: "put_dockerfile", name: targetName, content }),
        "ok",
      );
      invalidate(`${host.url}:list_dockerfiles`);
      if (isNew) {
        nav(`/h/${hid}/dockerfiles/${encodeURIComponent(targetName)}`, { replace: true });
      } else {
        setOriginal(content);
      }
    } catch (e) {
      setError(`save dockerfile: ${(e as Error).message}`);
    } finally {
      setPending(null);
    }
  }

  async function onDelete() {
    if (!host || isNew) return;
    setPending("delete");
    setError(null);
    try {
      unwrap(await call(host, { op: "delete_dockerfile", name }), "ok");
      invalidate(`${host.url}:list_dockerfiles`);
      nav(`/h/${hid}/dockerfiles`, { replace: true });
    } catch (e) {
      setError(`delete dockerfile: ${(e as Error).message}`);
      setPending(null);
    }
  }

  const sectionCrumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "dockerfiles");

  if (!saved) return <Page><p>Unknown host.</p></Page>;

  const crumbLabel = isNew ? "new dockerfile" : name;
  const crumbs: Crumb[] = [
    ...sectionCrumbs,
    { kind: "link", label: crumbLabel },
  ];
  const dirty = isNew ? draftName.trim() !== "" || content !== "" : content !== original;

  return (
    <Page crumbs={crumbs}>
      {isNew ? (
        <Heading
          category="Dockerfile"
          editable={{
            value: draftName,
            onChange: setDraftName,
            placeholder: "new.Dockerfile",
          }}
        />
      ) : (
        <Heading category="Dockerfile" title={name} />
      )}

      {error && <p className="text-[var(--error)] text-xs">{error}</p>}

      {!isNew && (
        <Section>
          <Row label="Size" value={meta ? `${meta.size}B` : undefined} />
          <Row label="Modified" value={meta?.modified_at} mono />
        </Section>
      )}

      {!loading && (
        <form onSubmit={onSubmit} className="contents" {...scrollFocusedIntoView()}>
          <Section label="content">
            <textarea
              className="input-code"
              spellCheck={false}
              autoCapitalize="off"
              autoCorrect="off"
              rows={18}
              placeholder={"FROM alpine:3.19\nRUN apk add --no-cache curl\n"}
              value={content}
              onChange={(e) => setContent(e.target.value)}
              style={{ minHeight: "320px" }}
            />
          </Section>

          <Section label={isNew ? "create" : "save"}>
            <div className="flex gap-2">
              <Button
                variant="ghost"
                onClick={() => nav(`/h/${hid}/dockerfiles`)}
                className="flex-1"
              >
                Cancel
              </Button>
              <Button
                type="submit"
                disabled={pending !== null || !dirty}
                className="flex-1"
              >
                {pending === "save" ? (
                  <><Spinner /> {isNew ? "Creating…" : "Saving…"}</>
                ) : isNew ? "Create" : "Save"}
              </Button>
            </div>
          </Section>

          {!isNew && (
            <Section label="danger">
              <Button
                variant="destructive"
                onClick={() => setConfirmDelete(true)}
                disabled={pending !== null}
              >
                Delete
              </Button>
            </Section>
          )}

          <ConfirmDialog
            open={confirmDelete}
            onOpenChange={setConfirmDelete}
            title={`Delete ${name}?`}
            description="This removes the file from the dockerfiles directory."
            confirmLabel="Delete"
            destructive
            onConfirm={onDelete}
          />
        </form>
      )}
    </Page>
  );
}
