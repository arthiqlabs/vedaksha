export default function PrivacyPage() {
  return (
    <div className="max-w-3xl mx-auto px-6 py-20">
      <h1 className="text-2xl font-bold tracking-tight text-[var(--color-brand-text)] mb-2">
        Privacy Policy
      </h1>
      <p className="text-sm text-[var(--color-brand-text-muted)] mb-10">
        Effective date: April 11, 2026 &nbsp;&bull;&nbsp; Last updated: April 11, 2026
      </p>

      <div className="prose prose-sm text-[var(--color-brand-text-secondary)] space-y-8">

        {/* ── 1. Overview ───────────────────────────────────────────────── */}
        <section>
          <h2 className="text-lg font-semibold text-[var(--color-brand-text)] mb-3">
            1. Overview
          </h2>
          <p>
            Vedākṣha is an astronomical computation library published by{" "}
            <strong>ArthIQ Labs LLC</strong> ("we", "us", "our"). This Privacy
            Policy explains what information we collect through the{" "}
            <strong>vedaksha.net</strong> website and commercial licensing portal,
            and — equally importantly — what we do <em>not</em> collect or process.
          </p>
          <p className="mt-3">
            Vedākṣha the <em>computation engine</em> is designed to be
            PII-blind: it operates entirely on abstract mathematical inputs and
            never receives, stores, or transmits personal information of any kind.
          </p>
        </section>

        {/* ── 2. Data We Do NOT Collect ──────────────────────────────────── */}
        <section>
          <h2 className="text-lg font-semibold text-[var(--color-brand-text)] mb-3">
            2. Data We Do NOT Collect — PII-Blind by Design
          </h2>
          <p>
            The Vedākṣha computation library accepts <strong>only</strong>:
          </p>
          <ul className="list-disc pl-5 space-y-1 mt-2">
            <li>
              <strong>Julian Day numbers</strong> — an abstract floating-point
              count of days from an astronomical epoch, with no embedded calendar,
              name, or time-zone information.
            </li>
            <li>
              <strong>Geographic coordinates</strong> — decimal latitude and
              longitude, with no place name, city, country, or postal code.
            </li>
          </ul>
          <p className="mt-3">
            The computation layer <strong>never sees, processes, stores, or
            logs</strong>:
          </p>
          <ul className="list-disc pl-5 space-y-1 mt-2">
            <li>Names or any personal identifiers</li>
            <li>Birth dates or times as human-readable strings</li>
            <li>Place names or addresses</li>
            <li>Email addresses</li>
            <li>IP addresses or device identifiers</li>
            <li>Any data that could be linked to a natural person</li>
          </ul>
          <p className="mt-3">
            This is an architectural property enforced at the API boundary, not
            a policy commitment. Callers are responsible for converting personal
            data to Julian Day + coordinates <em>before</em> calling Vedākṣha.
          </p>
        </section>

        {/* ── 3. Data Classification in Output ──────────────────────────── */}
        <section>
          <h2 className="text-lg font-semibold text-[var(--color-brand-text)] mb-3">
            3. Data Classification in Output
          </h2>
          <p>
            Vedākṣha outputs include a <code>DataClassification</code> field with
            one of three values:
          </p>
          <ul className="list-disc pl-5 space-y-2 mt-2">
            <li>
              <strong>Anonymous</strong> — output contains only computed
              astronomical positions. No individual can be identified from this
              output alone.
            </li>
            <li>
              <strong>Pseudonymized</strong> — the caller has associated an
              opaque token (e.g., a UUID) with the output. Vedākṣha itself does not
              know whose token this is.
            </li>
            <li>
              <strong>Identified</strong> — the caller has chosen to annotate the
              output with identifying information outside of Vedākṣha's computation
              layer.
            </li>
          </ul>
          <p className="mt-3">
            The <code>DataClassification</code> value is set by the{" "}
            <strong>caller</strong>, not by Vedākṣha. Vedākṣha bears no
            responsibility for downstream handling of classified outputs.
          </p>
        </section>

        {/* ── 4. Data Collected by vedaksha.net ─────────────────────────── */}
        <section>
          <h2 className="text-lg font-semibold text-[var(--color-brand-text)] mb-3">
            4. Data Collected by vedaksha.net
          </h2>
          <p>
            The website and licensing portal collect limited data for the following
            purposes:
          </p>

          <h3 className="text-base font-semibold text-[var(--color-brand-text)] mt-5 mb-2">
            4.1 Analytics
          </h3>
          <p>
            We use privacy-respecting, <strong>cookie-free</strong> analytics
            (Plausible Analytics) to understand how visitors use the site.
            Collected data includes: page views, referrer URLs, browser type, and
            approximate country (derived from IP address, which is not stored).
            No IP addresses are retained. No cross-site tracking occurs.
          </p>

          <h3 className="text-base font-semibold text-[var(--color-brand-text)] mt-5 mb-2">
            4.2 Payment Processing (Stripe)
          </h3>
          <p>
            When purchasing a commercial license, payment is processed by{" "}
            <strong>Stripe, Inc.</strong> We do not store credit card numbers,
            bank account details, or other payment credentials. Stripe processes
            your payment data under its own privacy policy. We receive from Stripe
            only: your email address (for license delivery), and payment
            confirmation metadata (transaction ID, amount, date).
          </p>

          <h3 className="text-base font-semibold text-[var(--color-brand-text)] mt-5 mb-2">
            4.3 Infrastructure (Cloudflare &amp; Vercel)
          </h3>
          <p>
            vedaksha.net is served through <strong>Cloudflare</strong> (DNS, CDN,
            DDoS protection) and hosted on <strong>Vercel</strong>. These
            providers may process request metadata (IP address, headers) in
            transit. We do not instruct these providers to log or retain personal
            data beyond what is necessary for network security and delivery.
          </p>

          <h3 className="text-base font-semibold text-[var(--color-brand-text)] mt-5 mb-2">
            4.4 Contact Emails
          </h3>
          <p>
            If you contact us at <a href="mailto:info@arthiq.net" className="text-[var(--color-brand-link)]">info@arthiq.net</a>,
            we retain your email address and message content for the purpose of
            responding to your inquiry. We do not add you to any mailing list
            without explicit consent.
          </p>
        </section>

        {/* ── 5. Third-Party Services ────────────────────────────────────── */}
        <section>
          <h2 className="text-lg font-semibold text-[var(--color-brand-text)] mb-3">
            5. Third-Party Services
          </h2>
          <p>We rely on the following sub-processors. Each operates under its own privacy policy:</p>
          <ul className="list-disc pl-5 space-y-2 mt-2">
            <li>
              <strong>Stripe</strong> — payment processing.{" "}
              <a
                href="https://stripe.com/privacy"
                target="_blank"
                rel="noopener noreferrer"
                className="text-[var(--color-brand-link)]"
              >
                stripe.com/privacy
              </a>
            </li>
            <li>
              <strong>Vercel</strong> — hosting and serverless infrastructure.{" "}
              <a
                href="https://vercel.com/legal/privacy-policy"
                target="_blank"
                rel="noopener noreferrer"
                className="text-[var(--color-brand-link)]"
              >
                vercel.com/legal/privacy-policy
              </a>
            </li>
            <li>
              <strong>Cloudflare</strong> — DNS, CDN, and DDoS protection.{" "}
              <a
                href="https://www.cloudflare.com/privacypolicy/"
                target="_blank"
                rel="noopener noreferrer"
                className="text-[var(--color-brand-link)]"
              >
                cloudflare.com/privacypolicy
              </a>
            </li>
            <li>
              <strong>Plausible Analytics</strong> — cookie-free, privacy-first
              analytics.{" "}
              <a
                href="https://plausible.io/privacy"
                target="_blank"
                rel="noopener noreferrer"
                className="text-[var(--color-brand-link)]"
              >
                plausible.io/privacy
              </a>
            </li>
          </ul>
        </section>

        {/* ── 6. Legal Bases and Regulatory Rights ──────────────────────── */}
        <section>
          <h2 className="text-lg font-semibold text-[var(--color-brand-text)] mb-3">
            6. GDPR, DPDP, and CCPA
          </h2>

          <h3 className="text-base font-semibold text-[var(--color-brand-text)] mt-4 mb-2">
            6.1 Computation Layer
          </h3>
          <p>
            Because the Vedākṣha computation engine processes no personal data
            (see Section 2), data subject rights under the GDPR (EU), DPDP Act
            (India), CCPA (California), and similar laws do not apply to
            computation outputs. There is no personal data in those outputs for
            which a right of access, erasure, or portability could be exercised
            against ArthIQ Labs LLC.
          </p>

          <h3 className="text-base font-semibold text-[var(--color-brand-text)] mt-4 mb-2">
            6.2 License and Contact Data
          </h3>
          <p>
            For data held in connection with commercial licenses (email address,
            payment metadata) or direct correspondence, you may exercise your
            rights (access, rectification, erasure, portability, objection) by
            emailing{" "}
            <a href="mailto:info@arthiq.net" className="text-[var(--color-brand-link)]">
              info@arthiq.net
            </a>
            . We will respond within <strong>30 days</strong>.
          </p>

          <h3 className="text-base font-semibold text-[var(--color-brand-text)] mt-4 mb-2">
            6.3 Legal Basis (GDPR)
          </h3>
          <p>
            Where GDPR applies, we process personal data on the basis of:
          </p>
          <ul className="list-disc pl-5 space-y-1 mt-2">
            <li>
              <strong>Contract performance</strong> — processing your email and
              payment metadata to deliver your license.
            </li>
            <li>
              <strong>Legitimate interests</strong> — aggregate analytics to
              improve the site.
            </li>
            <li>
              <strong>Consent</strong> — where you contact us voluntarily.
            </li>
          </ul>

          <h3 className="text-base font-semibold text-[var(--color-brand-text)] mt-4 mb-2">
            6.4 CCPA (California)
          </h3>
          <p>
            We do not sell personal information. California residents may request
            disclosure of categories of personal information collected in the past
            12 months by emailing{" "}
            <a href="mailto:info@arthiq.net" className="text-[var(--color-brand-link)]">
              info@arthiq.net
            </a>
            .
          </p>
        </section>

        {/* ── 7. Data Retention ─────────────────────────────────────────── */}
        <section>
          <h2 className="text-lg font-semibold text-[var(--color-brand-text)] mb-3">
            7. Data Retention
          </h2>
          <ul className="list-disc pl-5 space-y-2">
            <li>
              <strong>License records</strong> (email, payment metadata): retained
              for the duration of the license plus 7 years for tax and accounting
              compliance.
            </li>
            <li>
              <strong>Contact emails</strong>: retained until the inquiry is
              resolved, then deleted within 12 months.
            </li>
            <li>
              <strong>Aggregate analytics</strong>: retained indefinitely (no
              personal data is stored).
            </li>
          </ul>
        </section>

        {/* ── 8. Children's Privacy ─────────────────────────────────────── */}
        <section>
          <h2 className="text-lg font-semibold text-[var(--color-brand-text)] mb-3">
            8. Children's Privacy
          </h2>
          <p>
            vedaksha.net and the Vedaksha commercial licensing portal are not
            directed at children under 13 years of age (or under 16 in the
            European Union). We do not knowingly collect personal information from
            children. If you believe a child has provided us personal information,
            please contact{" "}
            <a href="mailto:info@arthiq.net" className="text-[var(--color-brand-link)]">
              info@arthiq.net
            </a>{" "}
            and we will delete it promptly.
          </p>
        </section>

        {/* ── 9. International Transfers ────────────────────────────────── */}
        <section>
          <h2 className="text-lg font-semibold text-[var(--color-brand-text)] mb-3">
            9. International Data Transfers
          </h2>
          <p>
            ArthIQ Labs LLC is based in the United States. If you are located
            outside the US, please be aware that limited personal data (license
            email, contact correspondence) may be transferred to and processed in
            the US. We rely on Standard Contractual Clauses (SCCs) and other
            lawful transfer mechanisms where required by applicable law.
          </p>
        </section>

        {/* ── 10. Changes to This Policy ────────────────────────────────── */}
        <section>
          <h2 className="text-lg font-semibold text-[var(--color-brand-text)] mb-3">
            10. Changes to This Policy
          </h2>
          <p>
            We may update this Privacy Policy from time to time. Material changes
            will be announced on this page with an updated effective date. Your
            continued use of vedaksha.net after changes constitutes acceptance of
            the revised policy.
          </p>
        </section>

        {/* ── 11. Contact ───────────────────────────────────────────────── */}
        <section>
          <h2 className="text-lg font-semibold text-[var(--color-brand-text)] mb-3">
            11. Contact
          </h2>
          <p>
            Data protection and privacy inquiries:
          </p>
          <address className="not-italic mt-2 text-sm space-y-1">
            <p>ArthIQ Labs LLC</p>
            <p>
              <a href="mailto:info@arthiq.net" className="text-[var(--color-brand-link)]">
                info@arthiq.net
              </a>
            </p>
          </address>
        </section>

        <p className="text-xs text-[var(--color-brand-text-muted)] pt-4 border-t border-[var(--color-brand-border)]">
          &copy; 2026 ArthIQ Labs LLC. All rights reserved.
        </p>
      </div>
    </div>
  );
}
