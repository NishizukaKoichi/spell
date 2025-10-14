'use client';

import { useRouter } from 'next/navigation';
import { useState } from 'react';

export default function LoginPage() {
  const router = useRouter();
  const [isDevLogin, setIsDevLogin] = useState(false);

  const handleGitHubLogin = () => {
    // Redirect to backend GitHub OAuth endpoint
    window.location.href = `${process.env.NEXT_PUBLIC_API_BASE || ''}/auth/github`;
  };

  const handleDevLogin = async () => {
    setIsDevLogin(true);
    try {
      const response = await fetch('/api/dev/login', {
        method: 'POST',
        credentials: 'include',
      });

      if (response.ok) {
        router.push('/dashboard');
      } else {
        console.error('Dev login failed');
        alert('Dev login failed - make sure ENABLE_DEV_LOGIN=1 in backend .env');
      }
    } catch (error) {
      console.error('Dev login error:', error);
      alert('Dev login error - is backend running?');
    } finally {
      setIsDevLogin(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-background">
      <div className="w-full max-w-md space-y-8 p-8">
        <div className="text-center">
          <h1 className="text-4xl font-bold tracking-tight">Spell</h1>
          <p className="mt-2 text-muted-foreground">
            Cast spells with WASM-powered APIs
          </p>
        </div>

        <div className="mt-8 space-y-4">
          <button
            onClick={handleGitHubLogin}
            className="w-full flex items-center justify-center gap-3 px-4 py-3 border border-border rounded-lg bg-card hover:bg-accent transition-colors"
          >
            <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
              <path fillRule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" clipRule="evenodd" />
            </svg>
            <span className="font-medium">Sign in with GitHub</span>
          </button>

          <div className="relative">
            <div className="absolute inset-0 flex items-center">
              <div className="w-full border-t border-border"></div>
            </div>
            <div className="relative flex justify-center text-xs uppercase">
              <span className="bg-background px-2 text-muted-foreground">Dev Only</span>
            </div>
          </div>

          <button
            onClick={handleDevLogin}
            disabled={isDevLogin}
            className="w-full flex items-center justify-center gap-3 px-4 py-3 border border-amber-500/50 rounded-lg bg-amber-500/10 hover:bg-amber-500/20 transition-colors disabled:opacity-50"
          >
            <span className="font-medium text-amber-600 dark:text-amber-400">
              {isDevLogin ? 'Logging in...' : '⚠️ Dev Login (Local Only)'}
            </span>
          </button>
        </div>

        <p className="mt-4 text-xs text-center text-muted-foreground">
          By signing in, you agree to our terms and privacy policy
        </p>
      </div>
    </div>
  );
}
