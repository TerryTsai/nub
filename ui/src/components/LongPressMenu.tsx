import { useRef, useState, type ReactNode } from "react";
import * as DropdownMenu from "@radix-ui/react-dropdown-menu";

export interface LongPressItem {
  label: string;
  onSelect: () => void;
  destructive?: boolean;
  /** Disabled with optional reason; if a string, used as title for hover hint. */
  disabled?: boolean | string;
}

const HOLD_MS = 500;
const MOVE_THRESHOLD_PX = 10;

/** Wraps a child in pointer-events that distinguish a normal tap (calls
 * `onPress`) from a long-press (opens a Radix DropdownMenu of `items`).
 * Pointer movement above the threshold cancels both — that's a scroll.
 *
 * Used for "swipe-to-act"-style quick actions on mobile list rows. */
export function LongPressMenu({
  items,
  children,
  onPress,
}: {
  items: LongPressItem[];
  children: ReactNode;
  onPress?: () => void;
}) {
  const [open, setOpen] = useState(false);
  const timerRef = useRef<number | null>(null);
  const movedRef = useRef(false);
  const longFiredRef = useRef(false);
  const startPos = useRef({ x: 0, y: 0 });

  function clearTimer() {
    if (timerRef.current !== null) {
      window.clearTimeout(timerRef.current);
      timerRef.current = null;
    }
  }

  function onPointerDown(e: React.PointerEvent) {
    movedRef.current = false;
    longFiredRef.current = false;
    startPos.current = { x: e.clientX, y: e.clientY };
    timerRef.current = window.setTimeout(() => {
      timerRef.current = null;
      longFiredRef.current = true;
      navigator.vibrate?.(8);
      setOpen(true);
    }, HOLD_MS);
  }

  function onPointerMove(e: React.PointerEvent) {
    const dx = Math.abs(e.clientX - startPos.current.x);
    const dy = Math.abs(e.clientY - startPos.current.y);
    if (dx > MOVE_THRESHOLD_PX || dy > MOVE_THRESHOLD_PX) {
      movedRef.current = true;
      clearTimer();
    }
  }

  function onPointerUp() {
    clearTimer();
    if (!movedRef.current && !longFiredRef.current && onPress) {
      onPress();
    }
  }

  return (
    <DropdownMenu.Root open={open} onOpenChange={setOpen}>
      <DropdownMenu.Trigger asChild>
        <div
          onPointerDown={onPointerDown}
          onPointerMove={onPointerMove}
          onPointerUp={onPointerUp}
          onPointerCancel={clearTimer}
          style={{ WebkitTouchCallout: "none", WebkitUserSelect: "none" }}
        >
          {children}
        </div>
      </DropdownMenu.Trigger>
      <DropdownMenu.Portal>
        <DropdownMenu.Content
          className="min-w-[180px] bg-[var(--bg-base)] border border-[var(--border-subtle)] rounded-[var(--radius-md)] shadow-lg overflow-hidden z-50"
          sideOffset={4}
          collisionPadding={12}
          onCloseAutoFocus={(e) => e.preventDefault()}
        >
          {items.map((item, i) => {
            const isDisabled = !!item.disabled;
            const reason = typeof item.disabled === "string" ? item.disabled : undefined;
            return (
              <DropdownMenu.Item
                key={i}
                disabled={isDisabled}
                onSelect={(e) => {
                  e.preventDefault();
                  item.onSelect();
                }}
                title={reason}
                className={[
                  "px-3 py-2.5 text-sm cursor-pointer outline-none",
                  "data-[highlighted]:bg-[var(--bg-elevated)]",
                  "data-[disabled]:opacity-40 data-[disabled]:cursor-not-allowed",
                  item.destructive ? "text-[var(--error)]" : "",
                ].join(" ")}
              >
                {item.label}
              </DropdownMenu.Item>
            );
          })}
        </DropdownMenu.Content>
      </DropdownMenu.Portal>
    </DropdownMenu.Root>
  );
}
