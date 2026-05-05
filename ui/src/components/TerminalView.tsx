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
 * handle. */
export const TerminalView = forwardRef<TerminalHandle, Props>(function TerminalView(
  { onInput, cursorBlink = false },
  ref,
) {
  const containerRef = useRef<HTMLDivElement>(null);
  const termRef = useRef<Terminal | null>(null);

  useEffect(() => {
    if (!containerRef.current) return;
    const term = new Terminal({
      fontFamily: '"JetBrains Mono", ui-monospace, monospace',
      fontSize: 13,
      cursorBlink,
      convertEol: true,
      disableStdin: !onInput,
      theme: {
        background: "#000000",
        foreground: "#e4e4e7",
        cursor: "#fbbf24",
        selectionBackground: "#3f3f46",
      },
    });
    const fit = new FitAddon();
    term.loadAddon(fit);
    term.open(containerRef.current);
    fit.fit();
    if (onInput) {
      term.onData(onInput);
      term.focus();
    }
    termRef.current = term;
    // ResizeObserver also catches size changes when sibling status strips
    // appear/disappear above the terminal. window resize alone misses those.
    const ro = new ResizeObserver(() => fit.fit());
    ro.observe(containerRef.current);
    return () => {
      ro.disconnect();
      term.dispose();
      termRef.current = null;
    };
  }, [cursorBlink, onInput]);

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
