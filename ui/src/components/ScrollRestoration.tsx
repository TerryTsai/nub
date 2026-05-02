import { useEffect, useLayoutEffect } from "react";
import { useLocation, useNavigationType } from "react-router-dom";

/** Save window.scrollY per URL into sessionStorage; restore on POP
 * (back/forward), reset to 0 on PUSH/REPLACE.
 *
 * Two subtleties:
 *  - We disable the browser's native scrollRestoration so it doesn't fight
 *    us. (Default on most browsers is "auto"; we want "manual".)
 *  - On POP we retry the scrollTo over several animation frames. The new
 *    page often mounts with empty placeholder content, then grows tall
 *    after a fetch lands; if we only restore once, we cap at the short
 *    height. Retrying for ~half a second covers the late-arriving data.
 */
export function ScrollRestoration() {
  const { pathname, search } = useLocation();
  const navType = useNavigationType();
  const key = `nub:scroll:${pathname}${search}`;

  // Manual mode once, at mount. Browser otherwise restores scroll on POP
  // before our restore runs, causing a one-frame jump.
  useEffect(() => {
    if ("scrollRestoration" in window.history) {
      window.history.scrollRestoration = "manual";
    }
  }, []);

  // Restore (or reset) before paint.
  useLayoutEffect(() => {
    if (navType !== "POP") {
      window.scrollTo(0, 0);
      return;
    }
    const saved = sessionStorage.getItem(key);
    const target = saved ? parseInt(saved, 10) : 0;
    let cancelled = false;
    let attempts = 0;
    function tryScroll() {
      if (cancelled) return;
      window.scrollTo(0, target);
      const reached = Math.abs(window.scrollY - target) <= 1;
      attempts += 1;
      if (!reached && attempts < 30) {
        requestAnimationFrame(tryScroll);
      }
    }
    tryScroll();
    return () => { cancelled = true; };
  }, [key, navType]);

  // Save scroll state for the next time we visit this URL.
  useEffect(() => {
    function save() {
      sessionStorage.setItem(key, String(window.scrollY));
    }
    window.addEventListener("scroll", save, { passive: true });
    return () => window.removeEventListener("scroll", save);
  }, [key]);

  return null;
}
