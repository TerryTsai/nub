import { useParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import { useHosts } from "@/state/hosts";
import { useQuery } from "@/state/cache";
import type { HostInfo as HostInfoT } from "@/api/types";
import { Heading } from "@/components/Heading";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { Page } from "@/components/Page";
import { Row } from "@/components/Row";
import { Section } from "@/components/Section";
import { Skeleton } from "@/components/Skeleton";

export function HostInfoScreen() {
  const { hid } = useParams<{ hid: string }>();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };

  const queryKey = host ? `${host.url}:host_info` : null;
  const { data: info, error } = useQuery<HostInfoT>(queryKey, async () => {
    const r = unwrap(await call(host!, { op: "host_info" }), "host_info");
    return r.data;
  });

  const crumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "info");
  if (!saved) return <Page><p>Unknown host.</p></Page>;

  return (
    <Page crumbs={crumbs}>
      <Heading category="Host" title={saved.label} />

      {error && <p className="text-[var(--error)] text-xs">{error}</p>}
      {!info && !error && (
        <Section>
          <Skeleton className="h-4 w-32" />
          <Skeleton className="h-3 w-full" />
          <Skeleton className="h-3 w-5/6" />
          <Skeleton className="h-3 w-4/6" />
          <Skeleton className="h-3 w-3/4" />
          <Skeleton className="h-3 w-2/3" />
        </Section>
      )}

      {info && (
        <>
          <Section>
            <Row label="URL" value={saved.url} mono />
            <Row label="nub" value={info.nub} mono />
            <Row label="Engine" value={`${info.engine} ${info.version}`} mono />
            <Row label="OS" value={`${info.os}/${info.arch}`} mono />
            <Row label="Kernel" value={info.kernel} mono />
            <Row label="CPUs" value={String(info.cpus)} />
            <Row label="Memory" value={formatBytes(info.mem_total)} />
          </Section>

          <Section label="counts">
            <Row
              label="Containers"
              value={`${info.containers_running} running · ${info.containers_total} total`}
            />
            <Row label="Images" value={String(info.images)} />
          </Section>
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
