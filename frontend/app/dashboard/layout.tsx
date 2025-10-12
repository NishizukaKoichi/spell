import Link from 'next/link';

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="min-h-screen bg-background">
      {/* Navigation */}
      <nav className="border-b border-border bg-card">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between h-16">
            <div className="flex items-center space-x-8">
              <Link href="/dashboard" className="text-xl font-bold">
                Spell Platform
              </Link>
              <div className="hidden md:flex space-x-4">
                <Link
                  href="/dashboard"
                  className="px-3 py-2 rounded-md text-sm font-medium hover:bg-accent transition-colors"
                >
                  Dashboard
                </Link>
                <Link
                  href="/dashboard/api-keys"
                  className="px-3 py-2 rounded-md text-sm font-medium hover:bg-accent transition-colors"
                >
                  API Keys
                </Link>
                <Link
                  href="/dashboard/billing"
                  className="px-3 py-2 rounded-md text-sm font-medium hover:bg-accent transition-colors"
                >
                  Billing
                </Link>
              </div>
            </div>
            <div className="flex items-center">
              <button className="px-3 py-2 rounded-md text-sm font-medium hover:bg-accent transition-colors">
                Sign Out
              </button>
            </div>
          </div>
        </div>
      </nav>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {children}
      </main>
    </div>
  );
}
