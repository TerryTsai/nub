import type { ReactNode } from "react";
import { Link } from "react-router-dom";

/** Top-level page shell — fixed header with the nub wordmark, then the
 * current page title below. Mirrors the foundry app-header / breadcrumb
 * pattern at a smaller scale.
 */
export function Page({
  title,
  children,
  right,
}: {
  title: string;
  children: ReactNode;
  right?: ReactNode;
}) {
  return (
    <div className="min-h-full">
      <AppHeader />
      <main className="max-w-2xl mx-auto px-5 pt-3 pb-12 flex flex-col gap-3">
        <div className="flex items-baseline justify-between mb-1">
          <h1 className="text-xl font-semibold tracking-tight font-display">{title}</h1>
          {right}
        </div>
        {children}
      </main>
    </div>
  );
}

function AppHeader() {
  return (
    <header className="flex items-center h-11 px-5 gap-2 border-b border-[var(--border-subtle)] bg-[var(--bg-base)] sticky top-0 z-30">
      <Link to="/" className="flex items-center gap-2">
        <NubMark />
        <span className="text-sm font-semibold font-display">nub</span>
      </Link>
    </header>
  );
}

/** 16x16 diamond — visually echoes the foundry mark without copying it. */
function NubMark() {
  return (
    <svg width="16" height="16" viewBox="0 0 16 16" aria-hidden="true">
      <rect
        x="3.05"
        y="3.05"
        width="9.9"
        height="9.9"
        transform="rotate(45 8 8)"
        fill="none"
        stroke="var(--id-color)"
        strokeWidth="1.5"
      />
    </svg>
  );
}
