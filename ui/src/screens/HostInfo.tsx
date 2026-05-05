import { useParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import { useHosts } from "@/state/hosts";
import { useQuery } from "@/state/cache";
import type { HostInfo as HostInfoT } from "@/api/types";
import { Collapsible } from "@/components/Collapsible";
import { Heading } from "@/components/Heading";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { Page } from "@/components/Page";
import { Row } from "@/components/Row";
import { Section } from "@/components/Section";

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

      <Section>
        <Row label="URL" value={saved.url} mono />
        <Row label="nub" value={info?.nub} mono />
        <Row label="Engine" value={info ? `${info.engine} ${info.version}` : undefined} mono />
        <Row label="OS" value={info ? `${info.os}/${info.arch}` : undefined} mono />
        <Row label="Kernel" value={info?.kernel} mono />
        <Row label="CPUs" value={info ? String(info.cpus) : undefined} />
        <Row label="Memory" value={info ? formatBytes(info.mem_total) : undefined} />
      </Section>

      <Collapsible label="counts">
        <Row
          label="Containers"
          value={info ? `${info.containers_running} running · ${info.containers_total} total` : undefined}
        />
        <Row label="Images" value={info ? String(info.images) : undefined} />
      </Collapsible>
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
