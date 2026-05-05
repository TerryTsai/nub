import { useNavigate, useParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import type { SecretSummary } from "@/api/types";
import { useHosts } from "@/state/hosts";
import { useQuery } from "@/state/cache";
import { FAB } from "@/components/FAB";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { ListRow } from "@/components/ListRow";
import { Page } from "@/components/Page";
import { SkeletonRows } from "@/components/Skeleton";
import { relativeDate } from "@/lib/relativeDate";

export function HostSecrets() {
  const { hid } = useParams<{ hid: string }>();
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };

  const queryKey = host ? `${host.url}:list_secrets` : null;
  const { data: secrets, error } = useQuery<SecretSummary[]>(queryKey, async () => {
    const r = unwrap(await call(host!, { op: "list_secrets" }), "secrets");
    return r.data;
  });

  const crumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "secrets");

  if (!saved || !hid) return <Page><p>Unknown host.</p></Page>;

  return (
    <Page crumbs={crumbs} fab={<FAB to={`/h/${hid}/secrets/new`} label="secret" />}>
      {error && <p className="text-[var(--error)] text-xs">{error}</p>}
      {secrets === null && !error && <SkeletonRows count={5} />}
      {secrets?.length === 0 && (
        <p className="text-xs text-[var(--text-tertiary)]">No secrets.</p>
      )}
      {secrets && secrets.length > 0 && (
        <div className="flex flex-col -mx-1">
          {secrets.map((s) => (
            <div key={s.name} className="px-1">
              <ListRow
                title={s.name}
                subtitle={relativeDate(s.modified_at)}
                onPress={() => nav(`/h/${hid}/secrets/${encodeURIComponent(s.name)}`)}
              />
            </div>
          ))}
        </div>
      )}
    </Page>
  );
}
