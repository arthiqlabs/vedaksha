export function Footer() {
  return (
    <footer className="border-t border-[var(--color-brand-border)] py-10 px-6">
      <div className="max-w-5xl mx-auto flex flex-col items-center gap-5">
        <span className="text-sm font-bold tracking-[0.25em] uppercase text-[var(--color-brand-text)]">
          VEDĀKṢHA
        </span>
        <p className="text-xs text-[var(--color-brand-text-secondary)] text-center">
          An ArthIQ Labs LLC product · BSL 1.1 License · <a href="https://vedaksha.net" className="hover:text-[var(--color-brand-text)] transition-colors">vedaksha.net</a> · <a href="mailto:info@arthiq.net" className="hover:text-[var(--color-brand-text)] transition-colors">info@arthiq.net</a>
        </p>
        <nav className="flex items-center gap-5 pt-2">
          {[
            { href: "/docs", label: "Docs" },
            { href: "/ai", label: "AI" },
            { href: "/pricing", label: "Pricing" },
            { href: "https://github.com/arthiqlabs/vedaksha", label: "GitHub" },
            { href: "/privacy", label: "Privacy" },
            { href: "/terms", label: "Terms" },
            { href: "/legal/bsl", label: "License" },
          ].map((link) => (
            <a
              key={link.label}
              href={link.href}
              className="text-xs text-[var(--color-brand-text-muted)] hover:text-[var(--color-brand-text-secondary)] transition-colors"
            >
              {link.label}
            </a>
          ))}
        </nav>
      </div>
    </footer>
  );
}
