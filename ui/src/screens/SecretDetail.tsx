import { useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import type { SecretSummary } from "@/api/types";
import { useHosts } from "@/state/hosts";
import { invalidate, useQuery } from "@/state/cache";
import { Button } from "@/components/Button";
import { Collapsible } from "@/components/Collapsible";
import { ConfirmDialog } from "@/components/ConfirmDialog";
import { Heading } from "@/components/Heading";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { Page, type Crumb } from "@/components/Page";
import { Row } from "@/components/Row";
import { Section } from "@/components/Section";
import { Spinner } from "@/components/Spinner";
import { useToast } from "@/components/Toaster";

export function SecretDetail() {
  const { hid, name } = useParams<{ hid: string; name: string }>();
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };
  const toast = useToast();

  const [confirmOpen, setConfirmOpen] = useState(false);
  const [pending, setPending] = useState(false);

  const queryKey = host ? `${host.url}:list_secrets` : null;
  const { data: secrets } = useQuery<SecretSummary[]>(queryKey, async () => {
    const r = unwrap(await call(host!, { op: "list_secrets" }), "secrets");
    return r.data;
  });

  const sectionCrumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "secrets");

  if (!saved || !hid || !name) return <Page><p>Unknown host.</p></Page>;

  const secret = secrets?.find((s) => s.name === name);

  const crumbs: Crumb[] = [
    ...sectionCrumbs,
    { kind: "link", label: name },
  ];

  async function onRemove() {
    if (!host || !name) return;
    setPending(true);
    try {
      unwrap(await call(host, { op: "delete_secret", name }), "ok");
      if (queryKey) invalidate(queryKey);
      toast.push(`removed ${name}`, "success");
      nav(`/h/${hid}/secrets`, { replace: true });
    } catch (e) {
      toast.pushOpError("remove", e);
    } finally {
      setPending(false);
    }
  }

  if (secrets && !secret) return <Page crumbs={crumbs}><p>Unknown secret.</p></Page>;

  return (
    <Page crumbs={crumbs}>
      <Heading category="Secret" title={name} />

      {!secret && (
        <p className="text-xs text-[var(--text-tertiary)]">Loading…</p>
      )}

      {secret && (
        <>
          <Section>
            <Row label="Size" value={`${secret.size}B`} />
            <Row label="Modified" value={secret.modified_at || "unknown"} mono />
          </Section>

          <Collapsible>
            <p className="text-xs text-[var(--text-tertiary)]">
              Encrypted at rest with the host's age key. Values are never read back over the network — use{" "}
              <code className="mono">nub secret get NAME</code> on the host.
            </p>
          </Collapsible>

          <Section label="danger">
            <Button
              variant="destructive"
              disabled={pending}
              onClick={() => setConfirmOpen(true)}
            >
              {pending ? (<><Spinner /> Removing…</>) : "Remove"}
            </Button>
          </Section>

          <ConfirmDialog
            open={confirmOpen}
            onOpenChange={setConfirmOpen}
            title="Remove secret?"
            description="This can't be undone."
            confirmLabel="Remove"
            destructive
            onConfirm={onRemove}
          />
        </>
      )}
    </Page>
  );
}
