const features = [
  "All 9 workspace crates",
  "Commercial use in any product or service",
  "Unlimited users and deployments",
  "Perpetual license for the purchased version",
  "Priority email support",
  "Brand kit with official badges",
];

const freeFeatures = [
  "Personal and hobbyist projects",
  "Academic and research use",
  "Internal tools (no external customers)",
  "Evaluation and prototyping",
  "Non-commercial open source",
  "Education and learning",
];

export default function PricingPage() {
  return (
    <div className="max-w-4xl mx-auto px-6 py-20">
      <div className="text-center mb-12">
        <h1 className="text-2xl font-bold tracking-tight text-[var(--color-brand-text)] mb-2">
          Pricing
        </h1>
        <p className="text-[var(--color-brand-text-secondary)]">
          Simple, one-time licensing. No subscriptions. No per-seat fees.
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 max-w-3xl mx-auto">
        {/* Free tier */}
        <div className="border border-[var(--color-brand-border)] rounded-xl p-8">
          <div className="mb-6">
            <p className="text-sm font-semibold uppercase tracking-wider text-[var(--color-brand-text-muted)] mb-2">
              Non-Commercial
            </p>
            <p className="text-3xl font-bold text-[var(--color-brand-text)]">
              Free
            </p>
            <p className="text-sm text-[var(--color-brand-text-muted)] mt-1">
              Forever
            </p>
          </div>
          <ul className="space-y-2.5">
            {freeFeatures.map((f) => (
              <li
                key={f}
                className="flex items-center gap-2 text-sm text-[var(--color-brand-text-secondary)]"
              >
                <svg
                  className="size-4 text-green-600 shrink-0"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                  strokeWidth={2}
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    d="M5 13l4 4L19 7"
                  />
                </svg>
                {f}
              </li>
            ))}
          </ul>
        </div>

        {/* Commercial tier */}
        <div className="border-2 border-[var(--color-brand-primary)] rounded-xl p-8 relative">
          <div className="absolute -top-3 left-6 bg-[var(--color-brand-primary)] text-white text-xs font-semibold px-3 py-1 rounded-full">
            Commercial
          </div>
          <div className="mb-6">
            <p className="text-sm font-semibold uppercase tracking-wider text-[var(--color-brand-text-muted)] mb-2">
              Commercial License
            </p>
            <p className="text-3xl font-bold text-[var(--color-brand-text)]">
              $500
            </p>
            <p className="text-sm text-[var(--color-brand-text-muted)] mt-1">
              One-time per organization
            </p>
          </div>
          <ul className="space-y-2.5 mb-8">
            {features.map((f) => (
              <li
                key={f}
                className="flex items-center gap-2 text-sm text-[var(--color-brand-text-secondary)]"
              >
                <svg
                  className="size-4 text-green-600 shrink-0"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                  strokeWidth={2}
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    d="M5 13l4 4L19 7"
                  />
                </svg>
                {f}
              </li>
            ))}
          </ul>
          <a
            href="https://buy.stripe.com/8x2dRb0yl7YN1USc1D0x200"
            target="_blank"
            rel="noopener noreferrer"
            className="block w-full text-center bg-[var(--color-brand-text)] text-white py-2.5 rounded-lg text-sm font-semibold hover:opacity-90 transition-opacity no-underline"
          >
            Purchase License
          </a>
          <p className="text-xs text-[var(--color-brand-text-muted)] mt-3 text-center">
            Secure checkout via Stripe. Invoice available at{" "}
            <a href="mailto:info@arthiq.net" className="text-[var(--color-brand-link)] hover:underline">info@arthiq.net</a>
          </p>
        </div>
      </div>

      <div className="text-center mt-10">
        <p className="text-sm text-[var(--color-brand-text-muted)]">
          Licensed under BSL 1.1. Each version converts to Apache 2.0 after 5
          years.
        </p>
        <p className="text-sm text-[var(--color-brand-text-muted)] mt-1">
          Contact{" "}
          <a
            href="mailto:info@arthiq.net"
            className="text-[var(--color-brand-link)] hover:underline"
          >
            info@arthiq.net
          </a>{" "}
          if you have any questions on licensing.
        </p>
      </div>
    </div>
  );
}
