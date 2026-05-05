/** focus handler that scrolls the focused element above the soft keyboard.
 *
 * Mobile Safari and Chrome don't reliably scroll fields into view when the
 * keyboard opens — long forms can have the active field land *behind* the
 * keyboard. Spread the result onto a form (or any input) to get the safe
 * behavior:
 *
 *     <form {...scrollFocusedIntoView()}>
 *
 * The 200ms timeout lets the keyboard finish opening before we measure;
 * scrolling earlier picks the wrong layout. */
export function scrollFocusedIntoView() {
  return {
    onFocus: (e: React.FocusEvent<HTMLElement>) => {
      const target = e.target as HTMLElement;
      if (!target || typeof target.scrollIntoView !== "function") return;
      window.setTimeout(() => {
        target.scrollIntoView({ behavior: "smooth", block: "center" });
      }, 200);
    },
  };
}
