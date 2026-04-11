"use client";

import { useState } from "react";

export function InstallBar() {
  const [copied, setCopied] = useState(false);
  const command = "cargo add vedaksha";

  async function handleCopy() {
    await navigator.clipboard.writeText(command);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }

  return (
    <div className="w-full max-w-[480px] mx-auto">
      <div className="flex items-center justify-between gap-3 rounded-lg border border-[var(--color-brand-border)] bg-[var(--color-brand-bg-code)] px-4 py-3 font-mono text-sm">
        <code className="flex items-center gap-2 min-w-0">
          <span className="text-[var(--color-brand-text-muted)] select-none">$</span>
          <span className="text-[var(--color-brand-text)] truncate">{command}</span>
        </code>
        <button
          onClick={handleCopy}
          className="shrink-0 size-8 flex items-center justify-center rounded-md text-[var(--color-brand-text-muted)] hover:text-[var(--color-brand-text)] hover:bg-[var(--color-brand-border)]/50 transition-colors"
          aria-label="Copy install command"
        >
          {copied ? (
            <svg className="size-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
            </svg>
          ) : (
            <svg className="size-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
              <path strokeLinecap="round" strokeLinejoin="round" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
            </svg>
          )}
        </button>
      </div>
    </div>
  );
}
