export default function TermsPage() {
  return (
    <div className="max-w-3xl mx-auto px-6 py-20">
      <h1 className="text-3xl font-bold tracking-tight text-[var(--color-brand-text)] mb-2">
        Terms of Use
      </h1>
      <p className="text-sm text-[var(--color-brand-text-muted)] mb-10">
        Effective date: April 11, 2026
      </p>

      <div className="prose prose-sm text-[var(--color-brand-text-secondary)] space-y-8">
        {/* ---------------------------------------------------------- */}
        {/* 1. Acceptance of Terms */}
        {/* ---------------------------------------------------------- */}
        <section>
          <h2 className="text-lg font-semibold text-[var(--color-brand-text)] mt-0">
            1. Acceptance of Terms
          </h2>
          <p>
            By accessing or using vedaksha.net, any Vedaksha software library
            (including Rust crates, Python packages, WebAssembly modules, and MCP
            server implementations), or any related services (collectively, the
            &ldquo;Service&rdquo;), you agree to be bound by these Terms of Use
            (&ldquo;Terms&rdquo;). If you do not agree to all of these Terms, you
            must not access or use the Service.
          </p>
          <p>
            You represent and warrant that you are at least eighteen (18) years of
            age and have the legal capacity to enter into these Terms. If you are
            using the Service on behalf of an organization, you represent that you
            have authority to bind that organization to these Terms.
          </p>
          <p>
            ArthIQ Labs LLC (&ldquo;ArthIQ,&rdquo; &ldquo;we,&rdquo;
            &ldquo;us,&rdquo; or &ldquo;our&rdquo;) reserves the right to
            modify these Terms at any time. Changes become effective upon posting
            to vedaksha.net. Your continued use of the Service after any
            modification constitutes acceptance of the revised Terms.
          </p>
        </section>

        {/* ---------------------------------------------------------- */}
        {/* 2. Description of Service */}
        {/* ---------------------------------------------------------- */}
        <section>
          <h2 className="text-lg font-semibold text-[var(--color-brand-text)]">
            2. Description of Service
          </h2>
          <p>
            Vedākṣha is an astronomical ephemeris and astrological computation
            library. It provides mathematical routines for computing planetary
            positions, house systems, divisional charts, dashas, and related
            astronomical and astrological data.
          </p>
          <p>The Service is made available in the following forms:</p>
          <ul className="list-disc pl-6 space-y-1">
            <li>Rust crates published to crates.io</li>
            <li>Python packages (via PyPI or source)</li>
            <li>WebAssembly (WASM) modules for browser and edge environments</li>
            <li>Model Context Protocol (MCP) server implementations</li>
            <li>The vedaksha.net website and any associated API endpoints</li>
          </ul>
        </section>

        {/* ---------------------------------------------------------- */}
        {/* 3. NOT Professional Advice */}
        {/* ---------------------------------------------------------- */}
        <section>
          <h2 className="text-lg font-semibold text-[var(--color-brand-text)]">
            3. NOT Professional Advice
          </h2>
          <div className="bg-[var(--color-brand-surface)] border border-[var(--color-brand-border)] rounded-lg p-5 space-y-4">
            <p className="font-bold uppercase">
              THE SERVICE IS NOT AND DOES NOT PROVIDE ANY FORM OF PROFESSIONAL
              ADVICE. BY USING THE SERVICE YOU ACKNOWLEDGE AND AGREE TO ALL OF
              THE FOLLOWING:
            </p>
            <ul className="list-disc pl-6 space-y-2">
              <li>
                <strong className="uppercase">
                  NOT FINANCIAL, INVESTMENT, OR TRADING ADVICE.
                </strong>{" "}
                The Service does not provide financial, investment, or trading
                advice of any kind. No output of the Service should be used as a
                basis for any financial decision, investment strategy, or trading
                activity.
              </li>
              <li>
                <strong className="uppercase">
                  NOT MEDICAL, HEALTH, OR PSYCHOLOGICAL ADVICE.
                </strong>{" "}
                The Service does not provide medical, health, or psychological
                advice. No output of the Service should be used as a substitute
                for professional medical diagnosis, treatment, or counseling.
              </li>
              <li>
                <strong className="uppercase">NOT LEGAL ADVICE.</strong> The
                Service does not provide legal advice. No output of the Service
                should be relied upon for legal decisions or as a substitute for
                consultation with a qualified attorney.
              </li>
              <li>
                <strong className="uppercase">
                  NOT LIFE GUIDANCE OR COUNSELING.
                </strong>{" "}
                The Service does not provide life guidance, personal counseling,
                or any form of advisory service for personal decision-making.
              </li>
              <li>
                <strong className="uppercase">
                  NOT CAREER OR RELATIONSHIP ADVICE.
                </strong>{" "}
                The Service does not provide career planning, employment
                guidance, relationship counseling, or any similar advisory
                services.
              </li>
              <li>
                <strong className="uppercase">
                  NOT RELIGIOUS INSTRUCTION.
                </strong>{" "}
                The Service does not provide religious instruction, spiritual
                guidance, or theological interpretation. The use of Vedic or
                astrological terminology is solely for the purpose of describing
                mathematical computation models.
              </li>
            </ul>
            <p className="font-bold uppercase">
              ASTROLOGICAL COMPUTATIONS PRODUCED BY THE SERVICE ARE MATHEMATICAL
              TRANSFORMATIONS OF ASTRONOMICAL DATA ONLY. ARTHIQ MAKES NO
              REPRESENTATIONS WHATSOEVER ABOUT THE VALIDITY, ACCURACY, OR
              APPLICABILITY OF ASTROLOGICAL INTERPRETATION.
            </p>
            <p>
              Users who build applications, services, or products using the
              Service bear sole and exclusive responsibility for how data is
              presented to their end users and for any claims, representations,
              or implications made based on the Service&rsquo;s output.
            </p>
          </div>
        </section>

        {/* ---------------------------------------------------------- */}
        {/* 4. Accuracy Disclaimer */}
        {/* ---------------------------------------------------------- */}
        <section>
          <h2 className="text-lg font-semibold text-[var(--color-brand-text)]">
            4. Accuracy Disclaimer
          </h2>
          <p>
            While Vedākṣha targets sub-arcsecond accuracy for planetary position
            calculations, ArthIQ does <strong>not</strong> warrant that
            computations are error-free, complete, or suitable for any particular
            purpose.
          </p>
          <p className="font-bold uppercase">
            THE SERVICE IS NOT CERTIFIED FOR AND MUST NOT BE USED FOR AEROSPACE,
            NAVIGATION, SAFETY-OF-LIFE, OR ANY OTHER APPLICATION WHERE
            INACCURATE COMPUTATION COULD RESULT IN BODILY HARM, LOSS OF LIFE, OR
            PROPERTY DAMAGE.
          </p>
          <p>
            Users requiring guaranteed accuracy for safety-critical or
            mission-critical applications must independently validate all outputs
            against authoritative sources.
          </p>
        </section>

        {/* ---------------------------------------------------------- */}
        {/* 5. Licensing & Commercial Use */}
        {/* ---------------------------------------------------------- */}
        <section>
          <h2 className="text-lg font-semibold text-[var(--color-brand-text)]">
            5. Licensing &amp; Commercial Use
          </h2>
          <p>
            Vedākṣha source code is licensed under the{" "}
            <strong>Business Source License 1.1 (BSL 1.1)</strong>. The following
            terms govern use:
          </p>
          <ul className="list-disc pl-6 space-y-2">
            <li>
              <strong>Non-commercial use</strong> is permitted free of charge
              under the BSL 1.1.
            </li>
            <li>
              <strong>Commercial use</strong> requires a one-time license fee of{" "}
              <strong>$500 USD per organization</strong>. &ldquo;Commercial
              use&rdquo; includes any use in connection with a product or service
              that generates revenue, whether directly or indirectly.
            </li>
            <li>
              Each version of Vedākṣha automatically converts to the{" "}
              <strong>Apache License 2.0</strong> five (5) years after its
              initial release date.
            </li>
          </ul>
          <p>
            The BSL 1.1 license text included with the source code is the
            authoritative license governing your use. In the event of any
            conflict between these Terms and the BSL 1.1 license text, the
            BSL 1.1 license text controls with respect to source code licensing.
          </p>
        </section>

        {/* ---------------------------------------------------------- */}
        {/* 6. Limitation of Liability */}
        {/* ---------------------------------------------------------- */}
        <section>
          <h2 className="text-lg font-semibold text-[var(--color-brand-text)]">
            6. Limitation of Liability
          </h2>
          <div className="bg-[var(--color-brand-surface)] border border-[var(--color-brand-border)] rounded-lg p-5">
            <p className="font-bold uppercase">
              TO THE MAXIMUM EXTENT PERMITTED BY APPLICABLE LAW, IN NO EVENT
              SHALL ARTHIQ LABS LLC, ITS OFFICERS, DIRECTORS, EMPLOYEES, AGENTS,
              OR AFFILIATES BE LIABLE FOR ANY INDIRECT, INCIDENTAL, SPECIAL,
              CONSEQUENTIAL, OR PUNITIVE DAMAGES, INCLUDING BUT NOT LIMITED TO
              DAMAGES FOR LOSS OF PROFITS, GOODWILL, USE, DATA, OR OTHER
              INTANGIBLE LOSSES, ARISING OUT OF OR IN CONNECTION WITH YOUR USE OF
              OR INABILITY TO USE THE SERVICE, REGARDLESS OF THE THEORY OF
              LIABILITY (CONTRACT, TORT, STRICT LIABILITY, OR OTHERWISE) AND EVEN
              IF ARTHIQ HAS BEEN ADVISED OF THE POSSIBILITY OF SUCH DAMAGES.
            </p>
            <p className="font-bold uppercase mt-4">
              ARTHIQ&rsquo;S TOTAL CUMULATIVE LIABILITY TO YOU FOR ALL CLAIMS
              ARISING OUT OF OR RELATED TO THESE TERMS OR THE SERVICE SHALL NOT
              EXCEED THE GREATER OF: (A) ONE HUNDRED UNITED STATES DOLLARS
              ($100.00 USD); OR (B) THE TOTAL FEES PAID BY YOU TO ARTHIQ IN THE
              TWELVE (12) MONTHS IMMEDIATELY PRECEDING THE EVENT GIVING RISE TO
              THE CLAIM.
            </p>
          </div>
        </section>

        {/* ---------------------------------------------------------- */}
        {/* 7. Indemnification */}
        {/* ---------------------------------------------------------- */}
        <section>
          <h2 className="text-lg font-semibold text-[var(--color-brand-text)]">
            7. Indemnification
          </h2>
          <p>
            You agree to indemnify, defend, and hold harmless ArthIQ Labs LLC,
            its officers, directors, employees, agents, and affiliates from and
            against any and all claims, liabilities, damages, losses, costs, and
            expenses (including reasonable attorneys&rsquo; fees) arising out of
            or related to:
          </p>
          <ul className="list-disc pl-6 space-y-2">
            <li>Your misuse of the Service;</li>
            <li>
              Your violation of these Terms or any applicable law or regulation;
            </li>
            <li>
              Any application, product, or service you build using the Service
              that causes harm to any third party;
            </li>
            <li>
              Any astrological, divinatory, predictive, or interpretive claims
              you or your applications make based on the Service&rsquo;s output;
            </li>
            <li>
              Any infringement or misappropriation of intellectual property
              rights arising from your use of the Service.
            </li>
          </ul>
        </section>

        {/* ---------------------------------------------------------- */}
        {/* 8. Governing Law & Jurisdiction */}
        {/* ---------------------------------------------------------- */}
        <section>
          <h2 className="text-lg font-semibold text-[var(--color-brand-text)]">
            8. Governing Law &amp; Jurisdiction
          </h2>
          <p>
            These Terms shall be governed by and construed in accordance with the
            laws of the State of Illinois, United States of America, without
            regard to its conflict-of-law principles.
          </p>
          <p>
            Any legal action or proceeding arising out of or related to these
            Terms or the Service shall be brought exclusively in the state or
            federal courts located in Lake County, Illinois. You hereby consent
            to the personal jurisdiction of such courts and waive any objection
            to venue therein.
          </p>
        </section>

        {/* ---------------------------------------------------------- */}
        {/* 9. Jury Trial Waiver */}
        {/* ---------------------------------------------------------- */}
        <section>
          <h2 className="text-lg font-semibold text-[var(--color-brand-text)]">
            9. Jury Trial Waiver
          </h2>
          <div className="bg-[var(--color-brand-surface)] border border-[var(--color-brand-border)] rounded-lg p-5">
            <p className="font-bold uppercase">
              EACH PARTY HEREBY IRREVOCABLY AND UNCONDITIONALLY WAIVES, TO THE
              FULLEST EXTENT PERMITTED BY APPLICABLE LAW, ANY AND ALL RIGHT TO
              TRIAL BY JURY IN ANY LEGAL ACTION OR PROCEEDING ARISING OUT OF OR
              RELATING TO THESE TERMS, THE SERVICE, OR THE TRANSACTIONS
              CONTEMPLATED HEREBY, WHETHER BASED IN CONTRACT, TORT, STRICT
              LIABILITY, OR ANY OTHER THEORY.
            </p>
            <p className="font-bold uppercase mt-4">
              EACH PARTY CERTIFIES AND ACKNOWLEDGES THAT (A) NO REPRESENTATIVE
              OF THE OTHER PARTY HAS REPRESENTED, EXPRESSLY OR OTHERWISE, THAT
              SUCH OTHER PARTY WOULD NOT SEEK TO ENFORCE THE FOREGOING WAIVER IN
              THE EVENT OF A LEGAL ACTION; (B) EACH PARTY UNDERSTANDS AND HAS
              CONSIDERED THE IMPLICATIONS OF THIS WAIVER; (C) EACH PARTY MAKES
              THIS WAIVER VOLUNTARILY; AND (D) EACH PARTY HAS BEEN INDUCED TO
              ENTER INTO THESE TERMS BY, AMONG OTHER THINGS, THE MUTUAL WAIVERS
              AND CERTIFICATIONS IN THIS SECTION.
            </p>
          </div>
        </section>

        {/* ---------------------------------------------------------- */}
        {/* 10. Intellectual Property */}
        {/* ---------------------------------------------------------- */}
        <section>
          <h2 className="text-lg font-semibold text-[var(--color-brand-text)]">
            10. Intellectual Property
          </h2>
          <p>
            The Vedākṣha name, logo, and all associated trademarks, service
            marks, and trade dress are the exclusive property of ArthIQ Labs LLC.
            Nothing in these Terms or the BSL 1.1 license grants you any right,
            title, or interest in ArthIQ&rsquo;s trademarks or branding.
          </p>
          <p>
            The BSL 1.1 license grants you a limited right to use the Vedākṣha
            source code in accordance with its terms. It does{" "}
            <strong>not</strong> transfer ownership of any intellectual property
            to you. All rights not expressly granted are reserved by ArthIQ.
          </p>
        </section>

        {/* ---------------------------------------------------------- */}
        {/* 11. Attribution Requirements */}
        {/* ---------------------------------------------------------- */}
        <section>
          <h2 className="text-lg font-semibold text-[var(--color-brand-text)]">
            11. Attribution Requirements
          </h2>
          <p className="font-bold">
            Attribution is mandatory for all use of the Service, whether
            commercial or non-commercial.
          </p>
          <p>
            All applications, products, services, and works that incorporate or
            rely on the Service must display the following attribution
            prominently:
          </p>
          <div className="bg-[var(--color-brand-surface)] border border-[var(--color-brand-border)] rounded-lg p-4 my-4 text-sm">
            Powered by Vedākṣha (vedaksha.net) &copy; ArthIQ Labs LLC
          </div>
          <p>
            <strong>&ldquo;Prominently&rdquo;</strong> means the attribution must
            be visible to end users without requiring navigation to sub-pages,
            secondary menus, or hidden sections. Acceptable placements include,
            but are not limited to:
          </p>
          <ul className="list-disc pl-6 space-y-1">
            <li>Application footer visible on all primary pages</li>
            <li>About page or screen</li>
            <li>
              CLI <code>--version</code> output
            </li>
            <li>API response headers</li>
            <li>README or primary documentation</li>
            <li>Splash screen or loading screen</li>
          </ul>
          <p>The attribution must:</p>
          <ul className="list-disc pl-6 space-y-1">
            <li>
              Include the phrase{" "}
              <strong>&ldquo;Powered by Vedākṣha&rdquo;</strong> verbatim.
            </li>
            <li>
              Include a hyperlink to{" "}
              <strong>
                <a
                  href="https://vedaksha.net"
                  className="text-[var(--color-brand-link)]"
                >
                  vedaksha.net
                </a>
              </strong>{" "}
              where technically feasible (e.g., web applications, documentation,
              README files).
            </li>
          </ul>
          <p className="font-bold">
            Failure to display the required attribution constitutes a material
            breach of these Terms and automatically terminates all rights granted
            hereunder, including any license to use the Service.
          </p>
        </section>

        {/* ---------------------------------------------------------- */}
        {/* 12. Termination */}
        {/* ---------------------------------------------------------- */}
        <section>
          <h2 className="text-lg font-semibold text-[var(--color-brand-text)]">
            12. Termination
          </h2>
          <p>
            ArthIQ may terminate or suspend your access to the Service
            immediately, without prior notice or liability, for any material
            breach of these Terms, including but not limited to failure to comply
            with attribution requirements (Section 11), unauthorized commercial
            use (Section 5), or misrepresentation of the Service&rsquo;s outputs
            (Section 3).
          </p>
          <p>
            Upon termination, your right to use the Service ceases immediately.
            The following sections survive termination: Sections 3 (Not
            Professional Advice), 4 (Accuracy Disclaimer), 6 (Limitation of
            Liability), 7 (Indemnification), 8 (Governing Law &amp;
            Jurisdiction), 9 (Jury Trial Waiver), 10 (Intellectual Property),
            and 13 (Severability &amp; Entire Agreement).
          </p>
        </section>

        {/* ---------------------------------------------------------- */}
        {/* 13. Severability & Entire Agreement */}
        {/* ---------------------------------------------------------- */}
        <section>
          <h2 className="text-lg font-semibold text-[var(--color-brand-text)]">
            13. Severability &amp; Entire Agreement
          </h2>
          <p>
            If any provision of these Terms is held to be invalid, illegal, or
            unenforceable by a court of competent jurisdiction, such invalidity,
            illegality, or unenforceability shall not affect any other provision
            of these Terms. The remaining provisions shall continue in full force
            and effect.
          </p>
          <p>
            These Terms, together with the BSL 1.1 license and any applicable
            commercial license agreement, constitute the entire agreement between
            you and ArthIQ with respect to the Service and supersede all prior or
            contemporaneous communications, proposals, and understandings,
            whether oral or written.
          </p>
          <p>
            No waiver of any provision of these Terms shall be deemed a further
            or continuing waiver of such provision or any other provision.
            ArthIQ&rsquo;s failure to enforce any right or provision of these
            Terms shall not constitute a waiver of such right or provision.
          </p>
        </section>

        {/* ---------------------------------------------------------- */}
        {/* 14. Contact */}
        {/* ---------------------------------------------------------- */}
        <section>
          <h2 className="text-lg font-semibold text-[var(--color-brand-text)]">
            14. Contact
          </h2>
          <p>
            For all inquiries regarding these Terms, licensing, or the Service,
            please contact:
          </p>
          <address className="not-italic mt-2">
            <strong>ArthIQ Labs LLC</strong>
            <br />
            Email:{" "}
            <a
              href="mailto:info@arthiq.net"
              className="text-[var(--color-brand-link)]"
            >
              info@arthiq.net
            </a>
          </address>
        </section>

        <hr className="border-[var(--color-brand-border)]" />
        <p className="text-xs text-[var(--color-brand-text-muted)]">
          &copy; 2026 ArthIQ Labs LLC. All rights reserved.
        </p>
      </div>
    </div>
  );
}
