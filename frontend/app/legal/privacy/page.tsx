export default function PrivacyPolicy() {
  return (
    <div className="max-w-4xl mx-auto px-4 py-12">
      <h1 className="text-4xl font-bold mb-8">Privacy Policy</h1>
      <p className="text-sm text-gray-600 mb-8">Last updated: October 13, 2025</p>

      <div className="prose prose-lg">
        <section className="mb-8">
          <h2 className="text-2xl font-semibold mb-4">1. Information We Collect</h2>
          <p>
            We collect information you provide directly to us, including:
          </p>
          <ul className="list-disc pl-6 mb-4">
            <li>Account information (email, GitHub username)</li>
            <li>Usage data and API requests</li>
            <li>Payment information (processed by Stripe)</li>
          </ul>
        </section>

        <section className="mb-8">
          <h2 className="text-2xl font-semibold mb-4">2. How We Use Your Information</h2>
          <p>We use the information we collect to:</p>
          <ul className="list-disc pl-6 mb-4">
            <li>Provide and maintain our service</li>
            <li>Process your transactions</li>
            <li>Send you technical notices and updates</li>
            <li>Respond to your requests and support needs</li>
          </ul>
        </section>

        <section className="mb-8">
          <h2 className="text-2xl font-semibold mb-4">3. Data Storage</h2>
          <p>
            Your data is stored securely on Fly.io infrastructure with encryption
            at rest and in transit. We retain your data as long as your account
            is active.
          </p>
        </section>

        <section className="mb-8">
          <h2 className="text-2xl font-semibold mb-4">4. Third-Party Services</h2>
          <p>We use the following third-party services:</p>
          <ul className="list-disc pl-6 mb-4">
            <li><strong>GitHub:</strong> Authentication</li>
            <li><strong>Stripe:</strong> Payment processing</li>
            <li><strong>Fly.io:</strong> Infrastructure hosting</li>
            <li><strong>Vercel:</strong> Frontend hosting</li>
          </ul>
        </section>

        <section className="mb-8">
          <h2 className="text-2xl font-semibold mb-4">5. Your Rights</h2>
          <p>You have the right to:</p>
          <ul className="list-disc pl-6 mb-4">
            <li>Access your personal data</li>
            <li>Request data deletion</li>
            <li>Export your data</li>
            <li>Opt-out of communications</li>
          </ul>
          <p>
            To exercise these rights, contact us at{' '}
            <a href="mailto:privacy@magicspell.io" className="text-blue-600 hover:underline">
              privacy@magicspell.io
            </a>
          </p>
        </section>

        <section className="mb-8">
          <h2 className="text-2xl font-semibold mb-4">6. Security</h2>
          <p>
            We implement appropriate technical and organizational measures to
            protect your personal information. However, no method of transmission
            over the internet is 100% secure.
          </p>
        </section>

        <section className="mb-8">
          <h2 className="text-2xl font-semibold mb-4">7. Changes to This Policy</h2>
          <p>
            We may update this privacy policy from time to time. We will notify
            you of any changes by posting the new policy on this page and updating
            the &ldquo;Last updated&rdquo; date.
          </p>
        </section>

        <section className="mb-8">
          <h2 className="text-2xl font-semibold mb-4">8. Contact Us</h2>
          <p>
            If you have any questions about this privacy policy, please contact us at:
          </p>
          <p>
            <a href="mailto:privacy@magicspell.io" className="text-blue-600 hover:underline">
              privacy@magicspell.io
            </a>
          </p>
        </section>
      </div>
    </div>
  );
}
