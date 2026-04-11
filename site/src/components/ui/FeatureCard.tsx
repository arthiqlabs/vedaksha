interface FeatureCardProps {
  number: string;
  title: string;
  description: string;
}

export function FeatureCard({ number, title, description }: FeatureCardProps) {
  const words = title.split(" ");
  const firstWord = words[0];
  const restWords = words.slice(1).join(" ");

  return (
    <div className="bg-[var(--color-brand-bg)] p-6 flex flex-col gap-2">
      <span className="text-xs font-medium uppercase tracking-wider text-[var(--color-brand-text-muted)]">
        {number}
      </span>
      <h3 className="text-base font-semibold uppercase tracking-wide">
        <span className="text-[var(--color-brand-text)]">{firstWord} </span>
        <span className="text-[#D4A843]">{restWords}</span>
      </h3>
      <p className="text-sm leading-relaxed text-[var(--color-brand-text-secondary)]">
        {description}
      </p>
    </div>
  );
}
