import { useState } from "react";
import * as Dialog from "@radix-ui/react-dialog";
import { Button } from "./Button";

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  description?: string;
  confirmLabel?: string;
  destructive?: boolean;
  onConfirm: () => void;
}

/** Centered confirm modal with the same design as the container Remove
 * dialog. Use in place of `window.confirm()` — iOS renders the native
 * one with an unstyled banner that doesn't fit the rest of the UI. */
export function ConfirmDialog({
  open,
  onOpenChange,
  title,
  description,
  confirmLabel = "Confirm",
  destructive,
  onConfirm,
}: Props) {
  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/60 z-40" />
        <Dialog.Content className="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 w-[min(92vw,360px)] bg-[var(--bg-base)] border border-[var(--border-subtle)] rounded-[var(--radius-lg)] p-5 flex flex-col gap-3 z-50">
          <Dialog.Title className="text-base font-semibold font-display">{title}</Dialog.Title>
          {description && (
            <Dialog.Description className="text-sm text-[var(--text-secondary)]">
              {description}
            </Dialog.Description>
          )}
          <div className="grid grid-cols-2 gap-2 mt-2">
            <Button variant="ghost" onClick={() => onOpenChange(false)}>Cancel</Button>
            <Button
              variant={destructive ? "destructive" : "primary"}
              onClick={() => {
                onOpenChange(false);
                onConfirm();
              }}
            >
              {confirmLabel}
            </Button>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

/** Tiny hook for the "open then act" pattern. Saves caller from juggling
 * two state slots. Returns a `prompt(action)` that opens the dialog with
 * the given on-confirm callback. */
export function useConfirm() {
  const [state, setState] = useState<{ onConfirm: () => void } | null>(null);
  return {
    open: state !== null,
    onOpenChange: (o: boolean) => { if (!o) setState(null); },
    onConfirm: () => state?.onConfirm(),
    prompt: (onConfirm: () => void) => setState({ onConfirm }),
  };
}
