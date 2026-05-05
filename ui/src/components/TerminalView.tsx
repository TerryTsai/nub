import { forwardRef, useEffect, useImperativeHandle, useRef } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";

export interface TerminalHandle {
  write: (data: string) => void;
  clear: () => void;
  copyAll: () => Promise<void>;
  focus: () => void;
}

interface Props {
  /** When provided, keystrokes are forwarded here and the terminal accepts
   * input. Without it the terminal renders read-only — used for logs. */
  onInput?: (data: string) => void;
  cursorBlink?: boolean;
}

/** Shared xterm host for Exec (interactive) and Logs (read-only). One
 * theme/font definition lives here; callers write via the imperative
 * handle.
 *
 * xterm.js has no built-in touch scroll, so a vertical swipe on mobile
 * does nothing by default. When the terminal is read-only (no `onInput`),
 * we wire a touch handler that converts pans into `scrollLines()` calls
 * so users can read scrollback. Interactive terminals are left alone —
 * touch there is reserved for xterm's own selection behavior. */
export const TerminalView = forwardRef<TerminalHandle, Props>(function TerminalView(
  { onInput, cursorBlink = false },
  ref,
) {
  const containerRef = useRef<HTMLDivElement>(null);
  const termRef = useRef<Terminal | null>(null);
  // Stable ref so changing `onInput` between renders doesn't tear down
  // the xterm instance (and lose scrollback). The effect only re-mounts
  // the terminal when read-only/interactive switches.
  const onInputRef = useRef(onInput);
  onInputRef.current = onInput;
  const interactive = !!onInput;

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const term = new Terminal({
      fontFamily: '"JetBrains Mono", ui-monospace, monospace',
      fontSize: 13,
      cursorBlink,
      convertEol: true,
      disableStdin: !interactive,
      theme: {
        background: "#000000",
        foreground: "#e4e4e7",
        cursor: "#fbbf24",
        selectionBackground: "#3f3f46",
      },
    });
    const fit = new FitAddon();
    term.loadAddon(fit);
    term.open(el);
    fit.fit();
    if (interactive) {
      term.onData((data) => onInputRef.current?.(data));
      term.focus();
    }
    termRef.current = term;
    // ResizeObserver also catches size changes when sibling status strips
    // appear/disappear above the terminal. window resize alone misses those.
    const ro = new ResizeObserver(() => fit.fit());
    ro.observe(el);

    // Touch-to-scroll for read-only terminals. Interactive terminals keep
    // xterm's defaults so selection-on-touch still works.
    let lastY: number | null = null;
    let active = false;
    function onTouchStart(e: TouchEvent) {
      if (e.touches.length !== 1) {
        lastY = null;
        active = false;
        return;
      }
      lastY = e.touches[0].clientY;
      active = false;
    }
    function onTouchMove(e: TouchEvent) {
      const t = termRef.current;
      const host = containerRef.current;
      if (lastY === null || !t || !host) return;
      const y = e.touches[0].clientY;
      const dy = lastY - y;
      // Don't intercept tiny jitters — leaves room for tap/long-press.
      if (!active && Math.abs(dy) < 6) return;
      active = true;
      const lineHeight = host.clientHeight / Math.max(t.rows, 1);
      const lines = Math.round(dy / lineHeight);
      if (lines !== 0) {
        t.scrollLines(lines);
        lastY = y;
      }
      e.preventDefault();
    }
    function onTouchEnd() {
      lastY = null;
      active = false;
    }

    if (!interactive) {
      el.addEventListener("touchstart", onTouchStart, { passive: true });
      el.addEventListener("touchmove", onTouchMove, { passive: false });
      el.addEventListener("touchend", onTouchEnd, { passive: true });
      el.addEventListener("touchcancel", onTouchEnd, { passive: true });
    }

    return () => {
      ro.disconnect();
      if (!interactive) {
        el.removeEventListener("touchstart", onTouchStart);
        el.removeEventListener("touchmove", onTouchMove);
        el.removeEventListener("touchend", onTouchEnd);
        el.removeEventListener("touchcancel", onTouchEnd);
      }
      term.dispose();
      termRef.current = null;
    };
  }, [cursorBlink, interactive]);

  useImperativeHandle(ref, () => ({
    write: (data) => termRef.current?.write(data),
    clear: () => termRef.current?.clear(),
    copyAll: async () => {
      const term = termRef.current;
      if (!term) return;
      term.selectAll();
      const text = term.getSelection();
      term.clearSelection();
      if (text) await navigator.clipboard.writeText(text);
    },
    focus: () => termRef.current?.focus(),
  }), []);

  return <div ref={containerRef} className="flex-1 min-h-0 bg-black px-1" />;
});
