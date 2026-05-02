import { Link } from "react-router-dom";

export type HostTab = "containers" | "images" | "volumes" | "networks";

/** Tab strip for the four per-host resource views. Mirrors foundry's
 * Inbox/Status/... pill-tab pattern. The active tab is the route the user
 * is currently on. */
export function HostNav({ hid, active }: { hid: string; active: HostTab }) {
  const tabs: { key: HostTab; label: string; to: string }[] = [
    { key: "containers", label: "Containers", to: `/h/${hid}` },
    { key: "images", label: "Images", to: `/h/${hid}/images` },
    { key: "volumes", label: "Volumes", to: `/h/${hid}/volumes` },
    { key: "networks", label: "Networks", to: `/h/${hid}/networks` },
  ];
  return (
    <div className="flex gap-1 overflow-x-auto no-scrollbar -mx-5 px-5">
      {tabs.map((t) => {
        const cls =
          active === t.key
            ? "shrink-0 px-3 py-1.5 rounded-full text-xs border border-[var(--accent-border)] bg-[var(--accent-soft)] text-[var(--accent)]"
            : "shrink-0 px-3 py-1.5 rounded-full text-xs border border-transparent text-[var(--text-tertiary)] hover:text-[var(--text-secondary)]";
        return (
          <Link key={t.key} to={t.to} className={cls}>
            {t.label}
          </Link>
        );
      })}
    </div>
  );
}
