export default function TermsOfService() {
  return (
    <div className="max-w-4xl mx-auto px-4 py-12">
      <h1 className="text-4xl font-bold mb-8">Terms of Service</h1>
      <p className="text-sm text-gray-600 mb-8">Last updated: October 13, 2025</p>

      <div className="prose prose-lg">
        <section className="mb-8">
          <h2 className="text-2xl font-semibold mb-4">1. Acceptance of Terms</h2>
          <p>
            By accessing and using Spell (&ldquo;the Service&rdquo;), you accept and agree to
            be bound by these Terms of Service. If you do not agree to these terms,
            please do not use the Service.
          </p>
        </section>

        <section className="mb-8">
          <h2 className="text-2xl font-semibold mb-4">2. Description of Service</h2>
          <p>
            Spell provides a platform for executing WebAssembly-based &ldquo;spells&rdquo;
            (computational tasks) via API. The Service includes:
          </p>
          <ul className="list-disc pl-6 mb-4">
            <li>API endpoints for spell execution</li>
            <li>Usage metrics and monitoring</li>
            <li>API key management</li>
            <li>Optional billing and subscription features</li>
          </ul>
        </section>

        <section className="mb-8">
          <h2 className="text-2xl font-semibold mb-4">3. Account Registration</h2>
          <p>
            To use the Service, you must:
          </p>
          <ul className="list-disc pl-6 mb-4">
            <li>Be at least 18 years old</li>
            <li>Provide accurate and complete information</li>
            <li>Maintain the security of your account credentials</li>
            <li>Notify us immediately of any unauthorized access</li>
          </ul>
        </section>

        <section className="mb-8">
          <h2 className="text-2xl font-semibold mb-4">4. Acceptable Use</h2>
          <p>You agree NOT to:</p>
          <ul className="list-disc pl-6 mb-4">
            <li>Violate any laws or regulations</li>
            <li>Infringe on intellectual property rights</li>
            <li>Attempt to gain unauthorized access to systems</li>
            <li>Distribute malware or harmful code</li>
            <li>Abuse rate limits or attempt to overwhelm the Service</li>
            <li>Use the Service for illegal or malicious purposes</li>
          </ul>
        </section>

        <section className="mb-8">
          <h2 className="text-2xl font-semibold mb-4">5. Rate Limits and Usage</h2>
          <p>
            The Service enforces rate limits:
          </p>
          <ul className="list-disc pl-6 mb-4">
            <li>Free tier: 10 requests/minute (IP-based)</li>
            <li>Authenticated users: 60 requests/minute (user-based)</li>
          </ul>
          <p>
            Excessive usage may result in temporary or permanent suspension.
          </p>
        </section>

        <section className="mb-8">
          <h2 className="text-2xl font-semibold mb-4">6. Payment and Billing</h2>
          <p>
            If you subscribe to a paid plan:
          </p>
          <ul className="list-disc pl-6 mb-4">
            <li>Fees are charged in advance on a recurring basis</li>
            <li>Payments are processed securely through Stripe</li>
            <li>Refunds are handled on a case-by-case basis</li>
            <li>Failure to pay may result in service suspension</li>
          </ul>
        </section>

        <section className="mb-8">
          <h2 className="text-2xl font-semibold mb-4">7. Intellectual Property</h2>
          <p>
            The Service and its original content, features, and functionality are
            owned by Spell and are protected by international copyright, trademark,
            and other intellectual property laws.
          </p>
        </section>

        <section className="mb-8">
          <h2 className="text-2xl font-semibold mb-4">8. Limitation of Liability</h2>
          <p>
            THE SERVICE IS PROVIDED &ldquo;AS IS&rdquo; WITHOUT WARRANTIES OF ANY KIND.
            WE SHALL NOT BE LIABLE FOR ANY INDIRECT, INCIDENTAL, SPECIAL,
            CONSEQUENTIAL, OR PUNITIVE DAMAGES.
          </p>
        </section>

        <section className="mb-8">
          <h2 className="text-2xl font-semibold mb-4">9. Service Availability</h2>
          <p>
            We strive for 99.9% uptime but do not guarantee uninterrupted service.
            We reserve the right to:
          </p>
          <ul className="list-disc pl-6 mb-4">
            <li>Modify or discontinue features with notice</li>
            <li>Perform scheduled maintenance</li>
            <li>Suspend accounts that violate these terms</li>
          </ul>
        </section>

        <section className="mb-8">
          <h2 className="text-2xl font-semibold mb-4">10. Termination</h2>
          <p>
            You may terminate your account at any time. We may suspend or terminate
            your account if you violate these terms, with or without notice.
          </p>
        </section>

        <section className="mb-8">
          <h2 className="text-2xl font-semibold mb-4">11. Changes to Terms</h2>
          <p>
            We reserve the right to modify these terms at any time. We will notify
            users of material changes via email or prominent notice on the Service.
          </p>
        </section>

        <section className="mb-8">
          <h2 className="text-2xl font-semibold mb-4">12. Governing Law</h2>
          <p>
            These terms shall be governed by the laws of [Your Jurisdiction],
            without regard to conflict of law provisions.
          </p>
        </section>

        <section className="mb-8">
          <h2 className="text-2xl font-semibold mb-4">13. Contact Us</h2>
          <p>
            If you have questions about these Terms of Service, please contact us at:
          </p>
          <p>
            <a href="mailto:legal@magicspell.io" className="text-blue-600 hover:underline">
              legal@magicspell.io
            </a>
          </p>
        </section>
      </div>
    </div>
  );
}
