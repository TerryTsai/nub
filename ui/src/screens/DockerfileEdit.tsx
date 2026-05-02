import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import type { DockerfileContent } from "@/api/types";
import { useHosts } from "@/state/hosts";
import { useSession } from "@/state/session";
import { invalidate } from "@/state/cache";
import { Button } from "@/components/Button";
import { ConfirmDialog } from "@/components/ConfirmDialog";
import { Field } from "@/components/Field";
import { Heading } from "@/components/Heading";
import { useHostCrumb } from "@/components/HostCrumbs";
import { Page, type Crumb } from "@/components/Page";
import { Section } from "@/components/Section";

const NAME_RE = /^[A-Za-z0-9][A-Za-z0-9._-]*$/;
const NEW_SENTINEL = "_new";

export function DockerfileEdit() {
  const { hid, name: rawName } = useParams<{ hid: string; name: string }>();
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };
  const session = useSession(host);
  const isNew = rawName === NEW_SENTINEL;
  const name = isNew ? "" : decodeURIComponent(rawName ?? "");

  const [draftName, setDraftName] = useState(name);
  const [content, setContent] = useState("");
  const [original, setOriginal] = useState("");
  const [loading, setLoading] = useState(!isNew);
  const [pending, setPending] = useState<"save" | "delete" | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [confirmDelete, setConfirmDelete] = useState(false);

  useEffect(() => {
    if (isNew || !host || !session.session) return;
    let cancelled = false;
    (async () => {
      try {
        const r = unwrap(await call(host, { op: "read_dockerfile", name }), "dockerfile");
        if (cancelled) return;
        setContent((r.data as DockerfileContent).content);
        setOriginal((r.data as DockerfileContent).content);
      } catch (e) {
        if (!cancelled) setError((e as Error).message);
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => { cancelled = true; };
  }, [host?.url, host?.token, !!session.session, name, isNew]);

  async function onSave() {
    if (!host) return;
    const targetName = isNew ? draftName.trim() : name;
    if (!NAME_RE.test(targetName) || targetName.length > 128) {
      setError("name must start with a letter or digit and use only A–Z, 0–9, dot, underscore, dash");
      return;
    }
    setPending("save");
    setError(null);
    try {
      unwrap(
        await call(host, { op: "write_dockerfile", name: targetName, content }),
        "ok",
      );
      invalidate(`${host.url}:list_dockerfiles`);
      if (isNew) {
        nav(`/h/${hid}/dockerfiles/${encodeURIComponent(targetName)}`, { replace: true });
      } else {
        setOriginal(content);
      }
    } catch (e) {
      setError((e as Error).message);
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
      setError((e as Error).message);
      setPending(null);
    }
  }

  const hostCrumb = useHostCrumb(hid ?? "", saved?.label ?? "?");

  if (!saved) return <Page><p>Unknown host.</p></Page>;

  const title = isNew ? "New dockerfile" : name;
  const crumbs: Crumb[] = [
    hostCrumb,
    { kind: "link", label: "dockerfiles", to: `/h/${hid}/dockerfiles` },
    { kind: "link", label: title },
  ];
  const dirty = isNew ? draftName.trim() !== "" || content !== "" : content !== original;
  const canWrite = session.session?.can("write_dockerfile") ?? false;
  const canDelete = session.session?.can("delete_dockerfile") ?? false;
  const denySave = !canWrite ? "your token doesn't allow write_dockerfile" : undefined;
  const denyDel = !canDelete ? "your token doesn't allow delete_dockerfile" : undefined;

  return (
    <Page crumbs={crumbs}>
      <Heading category="Dockerfile" title={title} />

      {loading && <p className="text-xs text-[var(--text-tertiary)]">Loading…</p>}

      {!loading && (
        <>
          {isNew && (
            <Section label="Name">
              <Field label="Filename" hint="letters, digits, dot, underscore, dash">
                <input
                  className="input mono"
                  type="text"
                  autoCapitalize="off"
                  autoCorrect="off"
                  spellCheck={false}
                  placeholder="nginx.Dockerfile"
                  value={draftName}
                  onChange={(e) => setDraftName(e.target.value)}
                />
              </Field>
            </Section>
          )}

          <Section label="Content">
            <textarea
              className="input mono"
              spellCheck={false}
              autoCapitalize="off"
              autoCorrect="off"
              rows={18}
              placeholder={"FROM alpine:3.19\nRUN apk add --no-cache curl\n"}
              value={content}
              onChange={(e) => setContent(e.target.value)}
              style={{ minHeight: "320px", whiteSpace: "pre", overflowWrap: "normal", overflowX: "auto" }}
            />
          </Section>

          {error && <p className="text-[var(--error)] text-xs">{error}</p>}

          <Section label="Actions">
            <div className="flex gap-2">
              <Button
                variant="ghost"
                onClick={() => nav(`/h/${hid}/dockerfiles`)}
                className="flex-1"
              >
                Cancel
              </Button>
              <Button
                onClick={onSave}
                disabled={pending !== null || !dirty}
                disallowReason={denySave}
                className="flex-1"
              >
                {pending === "save" ? "…" : isNew ? "Create" : "Save"}
              </Button>
            </div>
            {!isNew && (
              <Button
                variant="destructive"
                onClick={() => setConfirmDelete(true)}
                disabled={pending !== null}
                disallowReason={denyDel}
                className="mt-2"
              >
                Delete
              </Button>
            )}
          </Section>

          <ConfirmDialog
            open={confirmDelete}
            onOpenChange={setConfirmDelete}
            title={`Delete ${name}?`}
            description="This removes the file from the dockerfiles directory."
            confirmLabel="Delete"
            destructive
            onConfirm={onDelete}
          />
        </>
      )}
    </Page>
  );
}
