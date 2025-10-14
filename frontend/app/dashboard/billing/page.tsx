'use client';

import { useState } from 'react';
import { loadStripe } from '@stripe/stripe-js';
import { Elements } from '@stripe/react-stripe-js';
import CardSetupForm from '@/components/CardSetupForm';
import BudgetManager from '@/components/BudgetManager';
import { usePaymentMethod } from '@/lib/usage';

const stripePromise = loadStripe(
  process.env.NEXT_PUBLIC_STRIPE_PUBLISHABLE_KEY || ''
);

export default function BillingPage() {
  const { paymentMethod, isLoading: isLoadingPaymentMethod } = usePaymentMethod();
  const [clientSecret, setClientSecret] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleStartSetup = async () => {
    setIsLoading(true);
    setError(null);

    try {
      const response = await fetch('/api/billing/setup-intent', {
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

        {isLoadingPaymentMethod ? (
          <div className="text-sm text-muted-foreground">Loading payment method...</div>
        ) : paymentMethod && !clientSecret ? (
          <div>
            <div className="flex items-center gap-4 p-4 rounded-md bg-secondary/50">
              <div className="flex-1">
                <div className="font-medium">
                  {paymentMethod.brand.charAt(0).toUpperCase() + paymentMethod.brand.slice(1)} •••• {paymentMethod.last4}
                </div>
                <div className="text-sm text-muted-foreground">
                  Expires {paymentMethod.exp_month}/{paymentMethod.exp_year}
                </div>
              </div>
              <button
                onClick={handleStartSetup}
                disabled={isLoading}
                className="px-4 py-2 bg-secondary text-secondary-foreground rounded-md hover:bg-secondary/80 transition-colors disabled:opacity-50"
              >
                {isLoading ? 'Loading...' : 'Update Card'}
              </button>
            </div>
            {error && (
              <p className="mt-2 text-sm text-destructive">{error}</p>
            )}
          </div>
        ) : !clientSecret ? (
          <div>
            <p className="text-sm text-muted-foreground mb-4">
              Add a payment method to start using the Spell API. You
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

      <BudgetManager />
    </div>
  );
}
