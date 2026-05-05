import { forwardRef, type InputHTMLAttributes } from "react";

/** Inline-editable cell that lives in a Row's `right` slot (or anywhere a
 * static spec value would render). Looks like text by default; focus reveals
 * a solid accent underline. Use `mono` for IDs/paths/identifiers — same
 * amber treatment Row applies to mono read-only values, so view and edit
 * share one visual language.
 *
 * Font size is 16px so iOS Safari doesn't auto-zoom on focus — the same rule
 * the `.input` class enforces for boxed inputs. */
type Props = InputHTMLAttributes<HTMLInputElement> & { mono?: boolean };

export const EditCell = forwardRef<HTMLInputElement, Props>(function EditCell(
  { mono, className = "", ...rest },
  ref,
) {
  const tone = mono
    ? "mono text-[var(--id-color)]"
    : "text-[var(--text-primary)]";
  return (
    <input
      ref={ref}
      autoCapitalize="off"
      autoCorrect="off"
      spellCheck={false}
      className={`bg-transparent border-0 border-b border-dashed border-[var(--border-subtle)] hover:border-[var(--border-strong)] focus:border-[var(--accent)] focus:border-solid focus:outline-none w-full text-[16px] leading-snug py-0 transition-colors ${tone} placeholder:text-[var(--text-tertiary)] placeholder:italic ${className}`}
      {...rest}
    />
  );
});
