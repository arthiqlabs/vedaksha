export default function BslPage() {
  return (
    <div className="max-w-3xl mx-auto px-6 py-20">
      <h1 className="text-2xl font-bold tracking-tight text-[var(--color-brand-text)] mb-2">
        Business Source License 1.1
      </h1>
      <p className="text-sm text-[var(--color-brand-text-muted)] mb-10">
        The license governing all Vedaksha software
      </p>

      <div className="space-y-8 text-[var(--color-brand-text-secondary)]">
        {/* Parameters */}
        <section className="border border-[var(--color-brand-border)] rounded-xl p-6 bg-[var(--color-brand-bg-subtle)]">
          <h2 className="text-lg font-semibold text-[var(--color-brand-text)] mb-4">
            License Parameters
          </h2>
          <dl className="space-y-3 text-sm">
            <div className="flex gap-2">
              <dt className="font-semibold text-[var(--color-brand-text)] w-40 shrink-0">Licensor:</dt>
              <dd>ArthIQ Labs LLC</dd>
            </div>
            <div className="flex gap-2">
              <dt className="font-semibold text-[var(--color-brand-text)] w-40 shrink-0">Licensed Work:</dt>
              <dd>Vedaksha (all crates in the vedaksha workspace). The Licensed Work is Copyright 2026 ArthIQ Labs LLC.</dd>
            </div>
            <div className="flex gap-2">
              <dt className="font-semibold text-[var(--color-brand-text)] w-40 shrink-0">Additional Use Grant:</dt>
              <dd>
                You may use the Licensed Work for any non-commercial purpose.
                Commercial use (any product or service generating revenue from
                external customers) requires a Commercial License. Commercial
                License Fee: USD $500 one-time per organization. Purchase at:{" "}
                <a href="/pricing" className="text-[var(--color-brand-link)] hover:underline">vedaksha.net/pricing</a>
              </dd>
            </div>
            <div className="flex gap-2">
              <dt className="font-semibold text-[var(--color-brand-text)] w-40 shrink-0">Change Date:</dt>
              <dd>Five (5) years from each version{"'"}s release date</dd>
            </div>
            <div className="flex gap-2">
              <dt className="font-semibold text-[var(--color-brand-text)] w-40 shrink-0">Change License:</dt>
              <dd>Apache License, Version 2.0</dd>
            </div>
          </dl>
          <p className="text-sm mt-4">
            For alternative licensing arrangements, contact{" "}
            <a href="mailto:info@arthiq.net" className="text-[var(--color-brand-link)] hover:underline">info@arthiq.net</a>.
          </p>
        </section>

        {/* What this means */}
        <section>
          <h2 className="text-lg font-semibold text-[var(--color-brand-text)] mb-3">
            What This Means
          </h2>
          <div className="space-y-3 text-sm">
            <p>
              <strong className="text-[var(--color-brand-text)]">Free for non-commercial use:</strong>{" "}
              Personal projects, academic research, education, internal tools,
              hobbyist use, evaluation, and non-commercial open source are all
              permitted at no cost.
            </p>
            <p>
              <strong className="text-[var(--color-brand-text)]">$500 for commercial use:</strong>{" "}
              Any product or service that generates revenue from external
              customers — including SaaS, mobile apps, web apps, APIs, and
              embedded systems — requires a one-time $500 commercial license
              per organization. This grants unlimited use across all products,
              perpetually for that version.
            </p>
            <p>
              <strong className="text-[var(--color-brand-text)]">Converts to Apache 2.0:</strong>{" "}
              Five years after each version{"'"}s release date, that version
              automatically becomes available under the Apache License 2.0 —
              a permissive open-source license with no commercial restrictions.
            </p>
          </div>
        </section>

        {/* Full license text */}
        <section>
          <h2 className="text-lg font-semibold text-[var(--color-brand-text)] mb-3">
            Full License Text
          </h2>
          <div className="border border-[var(--color-brand-border)] rounded-xl overflow-hidden">
            <div className="px-4 py-2.5 bg-[var(--color-brand-bg-subtle)] border-b border-[var(--color-brand-border)]">
              <span className="text-xs font-mono text-[var(--color-brand-text-muted)]">LICENSE</span>
            </div>
            <pre className="p-5 overflow-x-auto text-xs leading-relaxed font-mono text-[var(--color-brand-text-secondary)] bg-[var(--color-brand-bg-code)] whitespace-pre-wrap">
{`Business Source License 1.1

License text copyright (c) 2017 MariaDB Corporation Ab, All Rights Reserved.
"Business Source License" is a trademark of MariaDB Corporation Ab.

Terms

The Licensor hereby grants you the right to copy, modify, create derivative
works, redistribute, and make non-production use of the Licensed Work. The
Licensor may make an Additional Use Grant, above, permitting limited
production use.

Effective on the Change Date, or the fourth anniversary of the first
publicly available distribution of a specific version of the Licensed Work
under this License, whichever comes first, the Licensor hereby grants you
rights under the terms of the Change License, and the rights granted in
the paragraph above terminate.

If your use of the Licensed Work does not comply with the requirements
currently in effect as described in this License, you must purchase a
commercial license from the Licensor, its affiliated entities, or
authorized resellers, or you must refrain from using the Licensed Work.

All copies of the original and modified Licensed Work, and derivative
works of the Licensed Work, are subject to this License. This License
applies separately for each version of the Licensed Work and the Change
Date may vary for each version of the Licensed Work released by Licensor.

You must conspicuously display this License on each original or modified
copy of the Licensed Work. If you receive the Licensed Work in original
or modified form from a third party, the terms and conditions set forth
in this License apply to your use of that work.

Any use of the Licensed Work in violation of this License will
automatically terminate your rights under this License for the current
and all other versions of the Licensed Work.

This License does not grant you any right in any trademark or logo of
Licensor or its affiliates (provided that you may use a trademark or
logo of Licensor as expressly required by this License).

TO THE EXTENT PERMITTED BY APPLICABLE LAW, THE LICENSED WORK IS PROVIDED
ON AN "AS IS" BASIS. LICENSOR HEREBY DISCLAIMS ALL WARRANTIES AND
CONDITIONS, EXPRESS OR IMPLIED, INCLUDING (WITHOUT LIMITATION) WARRANTIES
OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, NON-INFRINGEMENT,
AND TITLE.`}
            </pre>
          </div>
        </section>

        <p className="text-xs text-[var(--color-brand-text-muted)]">
          Copyright 2026 ArthIQ Labs LLC. All rights reserved.
        </p>
      </div>
    </div>
  );
}
