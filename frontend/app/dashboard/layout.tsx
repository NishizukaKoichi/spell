'use client';

import Link from 'next/link';
import { useAuth } from '@/lib/auth';
import { useRouter } from 'next/navigation';
import { useEffect } from 'react';

export default function DashboardLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const { user, isAuthenticated, isLoading, isError, logout } = useAuth();
  const router = useRouter();

  useEffect(() => {
    if (!isLoading && (isError || !isAuthenticated)) {
      router.push('/login');
    }
  }, [isLoading, isError, isAuthenticated, router]);

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-background">
        <div className="text-center">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary mx-auto"></div>
          <p className="mt-4 text-muted-foreground">Loading...</p>
        </div>
      </div>
    );
  }

  if (!isAuthenticated) {
    return null;
  }

  return (
    <div className="min-h-screen bg-background">
      {/* Navigation */}
      <nav className="border-b border-border bg-card">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between h-16">
            <div className="flex items-center space-x-8">
              <Link href="/dashboard" className="text-xl font-bold">
                Spell
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
            <div className="flex items-center gap-4">
              {user?.github_avatar_url && (
                <img
                  src={user.github_avatar_url}
                  alt={user.github_login}
                  className="w-8 h-8 rounded-full"
                />
              )}
              <span className="text-sm font-medium hidden md:inline">
                {user?.github_login}
              </span>
              <button
                onClick={logout}
                className="px-3 py-2 rounded-md text-sm font-medium hover:bg-accent transition-colors"
              >
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
