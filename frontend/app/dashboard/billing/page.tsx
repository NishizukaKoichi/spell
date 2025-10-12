'use client';

import { useState } from 'react';
import { loadStripe } from '@stripe/stripe-js';
import { Elements } from '@stripe/react-stripe-js';
import CardSetupForm from '@/components/CardSetupForm';

const stripePromise = loadStripe(
  process.env.NEXT_PUBLIC_STRIPE_PUBLISHABLE_KEY || ''
);

export default function BillingPage() {
  const [clientSecret, setClientSecret] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleStartSetup = async () => {
    setIsLoading(true);
    setError(null);

    try {
      const API_URL =
        process.env.NEXT_PUBLIC_API_URL || 'https://spell-platform.fly.dev';

      const response = await fetch(`${API_URL}/setup-intent`, {
        method: 'POST',
        credentials: 'include',
      });

      if (!response.ok) {
        throw new Error('Failed to create setup intent');
      }

      const data = await response.json();
      setClientSecret(data.client_secret);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'An error occurred');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Billing</h1>
        <p className="text-muted-foreground mt-2">
          Manage your payment method and billing settings
        </p>
      </div>

      <div className="rounded-lg border border-border bg-card p-6">
        <h2 className="text-lg font-semibold mb-4">Payment Method</h2>

        {!clientSecret ? (
          <div>
            <p className="text-sm text-muted-foreground mb-4">
              Add a payment method to start using the Spell Platform API. You
              will be charged based on your usage.
            </p>
            <button
              onClick={handleStartSetup}
              disabled={isLoading}
              className="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 transition-colors disabled:opacity-50"
            >
              {isLoading ? 'Loading...' : 'Add Payment Method'}
            </button>
            {error && (
              <p className="mt-2 text-sm text-destructive">{error}</p>
            )}
          </div>
        ) : (
          <Elements stripe={stripePromise} options={{ clientSecret }}>
            <CardSetupForm />
          </Elements>
        )}
      </div>

      <div className="rounded-lg border border-border bg-card p-6">
        <h2 className="text-lg font-semibold mb-4">Spending Limit</h2>
        <p className="text-sm text-muted-foreground mb-4">
          Your initial spending limit is set to $50/month. You can adjust this
          limit after adding a payment method.
        </p>
        <div className="flex items-center gap-4">
          <div className="flex-1">
            <div className="text-2xl font-bold">$50.00</div>
            <div className="text-sm text-muted-foreground">
              Monthly spending limit
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
