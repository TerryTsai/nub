import { createContext, useCallback, useContext, useState, type ReactNode } from "react";
import * as Toast from "@radix-ui/react-toast";

export type ToastTone = "info" | "success" | "error";

interface ToastEntry {
  id: number;
  tone: ToastTone;
  message: string;
}

interface ToastApi {
  push: (message: string, tone?: ToastTone) => void;
}

const ToastContext = createContext<ToastApi>({ push: () => {} });

export function useToast(): ToastApi {
  return useContext(ToastContext);
}

/** Mounted once at app boot. Provides `useToast().push(message, tone)` to
 * any component below; renders the Radix viewport pinned bottom-center. */
export function Toaster({ children }: { children: ReactNode }) {
  const [items, setItems] = useState<ToastEntry[]>([]);

  const push = useCallback((message: string, tone: ToastTone = "info") => {
    setItems((prev) => [...prev, { id: Date.now() + Math.random(), tone, message }]);
  }, []);

  return (
    <ToastContext.Provider value={{ push }}>
      <Toast.Provider swipeDirection="down" duration={3500}>
        {children}
        {items.map((item) => (
          <Toast.Root
            key={item.id}
            className={`flex items-center gap-2 px-4 py-3 rounded-[var(--radius-md)] border bg-[var(--bg-base)] shadow-lg ${toneClass(item.tone)}`}
            onOpenChange={(open) => {
              if (!open) setItems((prev) => prev.filter((p) => p.id !== item.id));
            }}
          >
            <span className={`dot dot-${toneDot(item.tone)}`} aria-hidden="true" />
            <Toast.Description className="text-xs">{item.message}</Toast.Description>
          </Toast.Root>
        ))}
        <Toast.Viewport
          className="fixed left-1/2 -translate-x-1/2 z-50 flex flex-col gap-2 outline-none pointer-events-none [&>*]:pointer-events-auto"
          style={{ bottom: "calc(1rem + env(safe-area-inset-bottom))" }}
        />
      </Toast.Provider>
    </ToastContext.Provider>
  );
}

function toneClass(t: ToastTone): string {
  switch (t) {
    case "success": return "border-[var(--accent-border)]";
    case "error":   return "border-[var(--error-border)]";
    default:        return "border-[var(--border-subtle)]";
  }
}

function toneDot(t: ToastTone): "active" | "failed" | "idle" {
  if (t === "success") return "active";
  if (t === "error") return "failed";
  return "idle";
}
