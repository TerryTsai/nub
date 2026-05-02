import { useEffect, useLayoutEffect } from "react";
import { useLocation, useNavigationType } from "react-router-dom";

/** Save window scroll position per URL into sessionStorage; restore on
 * POP (back/forward), reset to 0 on PUSH/REPLACE. Runs in useLayoutEffect
 * so the restore happens before the browser paints — prevents the flicker
 * of "wrong scroll position" you'd otherwise see for one frame on back-nav.
 */
export function ScrollRestoration() {
  const { pathname, search } = useLocation();
  const navType = useNavigationType();
  const key = `nub:scroll:${pathname}${search}`;

  // Restore position synchronously before paint.
  useLayoutEffect(() => {
    if (navType === "POP") {
      const saved = sessionStorage.getItem(key);
      window.scrollTo(0, saved ? parseInt(saved, 10) : 0);
    } else {
      window.scrollTo(0, 0);
    }
  }, [key, navType]);

  // Track scroll changes for the next time we visit this URL.
  useEffect(() => {
    function save() {
      sessionStorage.setItem(key, String(window.scrollY));
    }
    window.addEventListener("scroll", save, { passive: true });
    return () => window.removeEventListener("scroll", save);
  }, [key]);

  return null;
}
