/** Copy text to the clipboard with a fallback for non-secure contexts.
 *
 * `navigator.clipboard.writeText` only works in secure contexts (HTTPS or
 * localhost). The dev flow is `vite --host 0.0.0.0` over plain HTTP on the
 * LAN, where the modern API rejects silently. The hidden-textarea +
 * `document.execCommand("copy")` dance is deprecated but still supported
 * everywhere we care about, and works on insecure contexts. */
export async function copyText(text: string): Promise<boolean> {
  if (window.isSecureContext && navigator.clipboard?.writeText) {
    try {
      await navigator.clipboard.writeText(text);
      return true;
    } catch {
      // Permission denied or some other failure; fall through to legacy.
    }
  }
  const ta = document.createElement("textarea");
  ta.value = text;
  ta.style.position = "fixed";
  ta.style.opacity = "0";
  ta.style.pointerEvents = "none";
  document.body.appendChild(ta);
  ta.focus();
  ta.select();
  try {
    return document.execCommand("copy");
  } catch {
    return false;
  } finally {
    document.body.removeChild(ta);
  }
}
