"use client";

import { Logo } from "@/components/brand/Logo";
import { useEffect, useState } from "react";

export function Header() {
  const [dark, setDark] = useState(false);
  const [menuOpen, setMenuOpen] = useState(false);

  useEffect(() => {
    const isDark = document.documentElement.classList.contains("dark");
    setDark(isDark);
  }, []);

  function toggleDark() {
    const next = !dark;
    setDark(next);
    document.documentElement.classList.toggle("dark", next);
  }

  return (
    <header
      className="sticky top-0 z-50 backdrop-blur-sm bg-[var(--color-brand-bg)]/80 dark:bg-[var(--color-brand-bg)]/80 border-b border-[var(--color-brand-border)]"
    >
      <div className="h-14 flex items-center justify-between px-6">
        {/* Left: Logo + wordmark — links home */}
        <a href="/" className="flex items-center gap-3 no-underline outline-none border-none">
          <Logo
            size="medium"
            variant={dark ? "dark" : "light"}
            className="size-[34px] text-[var(--color-brand-primary)]"
          />
          <span className="text-xl font-bold tracking-wider">
            <span className="text-[var(--color-brand-text)]">Vedā</span><span className="text-[#D4A843]">kṣha</span>
          </span>
        </a>

        {/* Center: nav links (desktop) */}
        <nav className="hidden sm:flex items-center gap-6">
          <a
            href="/docs"
            className="text-base font-medium text-[var(--color-brand-text-secondary)] hover:text-[var(--color-brand-text)] transition-colors"
          >
            Docs
          </a>
          <a
            href="/ontology"
            className="text-base font-medium text-[var(--color-brand-text-secondary)] hover:text-[var(--color-brand-text)] transition-colors"
          >
            Ontology
          </a>
          <a
            href="/playground"
            className="text-base font-medium text-[#D4A843] hover:text-[#D4A843]/80 transition-colors"
          >
            Playground
          </a>
          <a
            href="/pricing"
            className="text-base font-medium text-[var(--color-brand-text-secondary)] hover:text-[var(--color-brand-text)] transition-colors"
          >
            Pricing
          </a>
          <a
            href="/about"
            className="text-base font-medium text-[var(--color-brand-text-secondary)] hover:text-[var(--color-brand-text)] transition-colors"
          >
            About
          </a>
        </nav>

        {/* Right: CTA + dark mode + mobile menu */}
        <div className="flex items-center gap-3">
          <a
            href="/docs"
            className="hidden sm:inline-flex items-center rounded-md bg-[var(--color-brand-text)] text-white px-4 py-2 text-sm font-semibold hover:opacity-90 transition-opacity"
          >
            Get Started
          </a>
          <button
            onClick={toggleDark}
            className="size-8 flex items-center justify-center rounded-md text-[var(--color-brand-text-secondary)] hover:text-[var(--color-brand-text)] hover:bg-[var(--color-brand-bg-subtle)] transition-colors"
            aria-label="Toggle dark mode"
          >
            {dark ? (
              <svg className="size-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z" />
              </svg>
            ) : (
              <svg className="size-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" />
              </svg>
            )}
          </button>
          {/* Hamburger (mobile only) */}
          <button
            onClick={() => setMenuOpen(!menuOpen)}
            className="sm:hidden size-8 flex items-center justify-center rounded-md text-[var(--color-brand-text-secondary)] hover:text-[var(--color-brand-text)] hover:bg-[var(--color-brand-bg-subtle)] transition-colors"
            aria-label="Toggle menu"
          >
            {menuOpen ? (
              <svg className="size-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
              </svg>
            ) : (
              <svg className="size-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M4 6h16M4 12h16M4 18h16" />
              </svg>
            )}
          </button>
        </div>
      </div>

      {/* Mobile menu */}
      {menuOpen ? (
        <nav className="sm:hidden border-t border-[var(--color-brand-border)] px-6 py-4 flex flex-col gap-3">
          <a
            href="/docs"
            className="text-base font-medium text-[var(--color-brand-text-secondary)] hover:text-[var(--color-brand-text)] transition-colors"
            onClick={() => setMenuOpen(false)}
          >
            Docs
          </a>
          <a
            href="/ontology"
            className="text-base font-medium text-[var(--color-brand-text-secondary)] hover:text-[var(--color-brand-text)] transition-colors"
            onClick={() => setMenuOpen(false)}
          >
            Ontology
          </a>
          <a
            href="/playground"
            className="text-base font-medium text-[#D4A843] hover:text-[#D4A843]/80 transition-colors"
            onClick={() => setMenuOpen(false)}
          >
            Playground
          </a>
          <a
            href="/pricing"
            className="text-base font-medium text-[var(--color-brand-text-secondary)] hover:text-[var(--color-brand-text)] transition-colors"
            onClick={() => setMenuOpen(false)}
          >
            Pricing
          </a>
          <a
            href="/about"
            className="text-base font-medium text-[var(--color-brand-text-secondary)] hover:text-[var(--color-brand-text)] transition-colors"
            onClick={() => setMenuOpen(false)}
          >
            About
          </a>
          <a
            href="/docs"
            className="inline-flex items-center justify-center rounded-md bg-[var(--color-brand-text)] text-white px-4 py-2 text-sm font-semibold hover:opacity-90 transition-opacity"
            onClick={() => setMenuOpen(false)}
          >
            Get Started
          </a>
        </nav>
      ) : null}
    </header>
  );
}
