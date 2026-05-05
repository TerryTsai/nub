import { useParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import type { SecretSummary } from "@/api/types";
import { useHosts } from "@/state/hosts";
import { invalidate, useQuery } from "@/state/cache";
import { Collapsible } from "@/components/Collapsible";
import { FAB } from "@/components/FAB";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { ListRow } from "@/components/ListRow";
import { Page } from "@/components/Page";
import { useToast } from "@/components/Toaster";

export function HostSecrets() {
  const { hid } = useParams<{ hid: string }>();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };
  const toast = useToast();

  const queryKey = host ? `${host.url}:list_secrets` : null;
  const { data: secrets, error } = useQuery<SecretSummary[]>(queryKey, async () => {
    const r = unwrap(await call(host!, { op: "list_secrets" }), "secrets");
    return r.data;
  });

  const crumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "secrets");

  if (!saved || !hid) return <Page><p>Unknown host.</p></Page>;

  async function onDelete(name: string) {
    if (!host) return;
    if (!confirm(`Delete secret "${name}"?\nThis can't be undone.`)) return;
    try {
      unwrap(await call(host, { op: "delete_secret", name }), "ok");
      if (queryKey) invalidate(queryKey);
      toast.push(`removed ${name}`, "success");
    } catch (e) {
      toast.push((e as Error).message, "error");
    }
  }

  return (
    <Page crumbs={crumbs} fab={<FAB to={`/h/${hid}/secrets/new`} label="secret" />}>
      {error && <p className="text-[var(--error)] text-xs">{error}</p>}
      {secrets === null && !error && (
        <p className="text-xs text-[var(--text-tertiary)]">Loading secrets…</p>
      )}
      {secrets?.length === 0 && (
        <p className="text-xs text-[var(--text-tertiary)]">
          No secrets. Tap secret to add one, or run{" "}
          <code className="mono">nub secret put NAME</code> on the host.
        </p>
      )}
      {secrets && secrets.length > 0 && (
        <div className="flex flex-col -mx-1">
          {secrets.map((s) => (
            <div key={s.name} className="px-1">
              <ListRow
                title={s.name}
                subtitle={`${s.size}B · ${s.modified_at || "unknown date"}`}
                onPress={() => onDelete(s.name)}
              />
            </div>
          ))}
        </div>
      )}
      {secrets && secrets.length > 0 && (
        <Collapsible>
          <p className="text-xs text-[var(--text-tertiary)]">
            Values are write-only over the network. Read with{" "}
            <code className="mono">nub secret get NAME</code> on the host.
          </p>
        </Collapsible>
      )}
    </Page>
  );
}
