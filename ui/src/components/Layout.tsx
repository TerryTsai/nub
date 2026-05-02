import { createContext, useContext, useLayoutEffect, useState, type ReactNode } from "react";
import { Outlet } from "react-router-dom";
import { AppHeader, type Crumb } from "./Page";

interface LayoutConfig {
  crumbs?: Crumb[];
  subnav?: ReactNode;
  fab?: ReactNode;
  fill?: boolean;
}

interface LayoutCtx {
  config: LayoutConfig;
  setConfig: (c: LayoutConfig) => void;
}

const LayoutContext = createContext<LayoutCtx>({
  config: {},
  setConfig: () => {},
});

export function useLayoutContext() {
  return useContext(LayoutContext);
}

/** Each page calls this with its layout config; the Layout shell reads
 * from context and renders the AppHeader / FAB / fill mode accordingly.
 *
 * useLayoutEffect commits before paint — there's one extra render but no
 * visible stale-config flash. */
export function usePageConfig(config: LayoutConfig) {
  const { setConfig } = useContext(LayoutContext);
  useLayoutEffect(() => {
    setConfig(config);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [config.crumbs, config.subnav, config.fab, config.fill]);
}

/** Mounted once at app boot. Stays mounted across all route changes —
 * only the <Outlet/> swaps. Fixes the "header flickers / page goes black
 * for a moment" jank that comes from unmounting the chrome on every nav.
 *
 * AppHeader is `position: fixed` (out of flow) so we reserve its height
 * here as padding-top: 44px alone, 84px when a subnav row is present
 * (44 header + 40 subnav). */
export function Layout() {
  const [config, setConfig] = useState<LayoutConfig>({});
  const { crumbs, subnav, fab, fill } = config;
  const padTop = subnav ? "pt-[84px]" : "pt-[44px]";
  return (
    <LayoutContext.Provider value={{ config, setConfig }}>
      <AppHeader crumbs={crumbs} subnav={subnav} />
      {fill ? (
        <div className={`h-full overflow-hidden ${padTop}`}>
          <main className="h-full flex flex-col">
            <Outlet />
          </main>
          {fab}
        </div>
      ) : (
        <div className={`min-h-full ${padTop}`}>
          <main className="max-w-2xl mx-auto px-5 pt-3 pb-24 flex flex-col gap-4">
            <Outlet />
          </main>
          {fab}
        </div>
      )}
    </LayoutContext.Provider>
  );
}
