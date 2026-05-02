import { useHosts, type SavedHost } from "@/state/hosts";
import { MenuCrumb, type MenuItem } from "@/components/MenuCrumb";
import type { Crumb } from "@/components/Page";

export type Section = "containers" | "images" | "volumes" | "networks" | "dockerfiles";

const SECTIONS: { key: Section; label: string; subpath: string }[] = [
  { key: "containers", label: "containers", subpath: "" },
  { key: "images", label: "images", subpath: "/images" },
  { key: "volumes", label: "volumes", subpath: "/volumes" },
  { key: "networks", label: "networks", subpath: "/networks" },
  { key: "dockerfiles", label: "dockerfiles", subpath: "/dockerfiles" },
];

/** Two-segment breadcrumb for top-level host pages: workspace dropdown
 * (all saved hosts + add) and section dropdown (containers/images/...). */
export function useHostSectionCrumbs(hid: string, hostLabel: string, section: Section): Crumb[] {
  const { hosts } = useHosts();
  return [hostMenu(hid, hostLabel, hosts), sectionMenu(hid, section)];
}

/** Host dropdown only — used by container detail / logs / stats / run pages
 * where the breadcrumb continues with the specific resource. */
export function useHostCrumb(hid: string, hostLabel: string): Crumb {
  const { hosts } = useHosts();
  return hostMenu(hid, hostLabel, hosts);
}

function hostMenu(hid: string, hostLabel: string, hosts: SavedHost[]): Crumb {
  const items: MenuItem[] = [
    ...hosts.map((h) => ({
      label: h.label,
      to: `/h/${h.hid}`,
      current: h.hid === hid,
    })),
    { label: "add host", to: "/add", add: true },
  ];
  return { kind: "menu", node: <MenuCrumb label={hostLabel} items={items} /> };
}

function sectionMenu(hid: string, section: Section): Crumb {
  const items: MenuItem[] = SECTIONS.map((s) => ({
    label: s.label,
    to: `/h/${hid}${s.subpath}`,
    current: s.key === section,
  }));
  const current = SECTIONS.find((s) => s.key === section)!;
  return { kind: "menu", node: <MenuCrumb label={current.label} items={items} /> };
}
