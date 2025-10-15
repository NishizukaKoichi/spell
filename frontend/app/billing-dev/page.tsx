'use client';

import { useState, useEffect } from 'react';
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
  const [isLoadingSetupIntent, setIsLoadingSetupIntent] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showCardForm, setShowCardForm] = useState(false);

  // Development mode: Force show form for UI testing
  const isDev = process.env.NODE_ENV === 'development';
  const [devMode, setDevMode] = useState(false);

  // Auto-fetch setup intent when user wants to add/update card
  useEffect(() => {
    if (!showCardForm) return;

    const fetchSetupIntent = async () => {
      setIsLoadingSetupIntent(true);
      setError(null);

      try {
        const response = await fetch('/api/billing/setup-intent', {
          method: 'POST',
          credentials: 'include',
        });

        if (!response.ok) {
          const errorData = await response.json().catch(() => ({}));
          throw new Error(errorData.error || 'Failed to create setup intent');
        }

        const data = await response.json();
        if (!data.client_secret) {
          throw new Error('No client_secret returned from server');
        }
        setClientSecret(data.client_secret);
      } catch (err) {
        console.error('Setup intent error:', err);

        // Development mode: Allow testing UI without backend
        if (process.env.NODE_ENV === 'development') {
          console.warn('⚠️ Using mock client_secret for development. PaymentElement will not work.');
          setClientSecret('seti_mock_' + Math.random().toString(36).substr(2, 9));
          // Don't set error in dev mode
        } else {
          setError(err instanceof Error ? err.message : 'An error occurred');
        }
      } finally {
        setIsLoadingSetupIntent(false);
      }
    };

    fetchSetupIntent();
  }, [showCardForm]);

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Billing</h1>
        <p className="text-muted-foreground mt-2">
          Manage your payment method and billing settings
        </p>
        {isDev && (
          <button
            onClick={() => {
              setDevMode(!devMode);
              if (!devMode) {
                setShowCardForm(true);
                setClientSecret('seti_mock_dev_ui_test');
              } else {
                setShowCardForm(false);
                setClientSecret(null);
              }
            }}
            className="mt-2 px-3 py-1 text-xs bg-yellow-500 text-black rounded"
          >
            {devMode ? '🔧 Dev Mode ON' : '🔧 Toggle Dev Mode'}
          </button>
        )}
      </div>

      <div className="rounded-lg border border-border bg-card p-6">
        <h2 className="text-lg font-semibold mb-4">Payment Method</h2>

        {isLoadingPaymentMethod && paymentMethod === undefined && !devMode ? (
          <div className="text-sm text-muted-foreground">Loading payment method...</div>
        ) : paymentMethod && !showCardForm ? (
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
                onClick={() => setShowCardForm(true)}
                className="px-4 py-2 bg-secondary text-secondary-foreground rounded-md hover:bg-secondary/80 transition-colors"
              >
                Update Card
              </button>
            </div>
            {error && (
              <p className="mt-2 text-sm text-destructive">{error}</p>
            )}
          </div>
        ) : !showCardForm ? (
          <div>
            <p className="text-sm text-muted-foreground mb-4">
              Add a payment method to start using the Spell API. You
              will be charged based on your usage.
            </p>
            <button
              onClick={() => setShowCardForm(true)}
              className="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 transition-colors"
            >
              Add Payment Method
            </button>
            {error && (
              <p className="mt-2 text-sm text-destructive">{error}</p>
            )}
          </div>
        ) : isLoadingSetupIntent ? (
          <div className="text-sm text-muted-foreground">
            Preparing payment form...
          </div>
        ) : error && process.env.NODE_ENV !== 'development' ? (
          <div>
            <p className="text-sm text-destructive mb-4">{error}</p>
            <button
              onClick={() => {
                setShowCardForm(false);
                setError(null);
              }}
              className="px-4 py-2 bg-secondary text-secondary-foreground rounded-md hover:bg-secondary/80 transition-colors"
            >
              Cancel
            </button>
          </div>
        ) : clientSecret ? (
          <div>
            {process.env.NODE_ENV === 'development' && (
              <div className="mb-4 p-2 bg-yellow-100 text-yellow-900 text-xs rounded">
                <div>Dev Mode: Using mock client_secret: {clientSecret.substring(0, 20)}...</div>
                <div>Stripe Key: {process.env.NEXT_PUBLIC_STRIPE_PUBLISHABLE_KEY ? 'Set' : 'Missing'}</div>
              </div>
            )}
            {stripePromise ? (
              <Elements
                stripe={stripePromise}
                options={{ clientSecret }}
                key={clientSecret}
              >
                <CardSetupForm onCancel={() => {
                  setShowCardForm(false);
                  setClientSecret(null);
                }} />
              </Elements>
            ) : (
              <div className="text-sm text-destructive">Stripe not loaded - check NEXT_PUBLIC_STRIPE_PUBLISHABLE_KEY</div>
            )}
          </div>
        ) : (
          <div className="text-sm text-muted-foreground">
            {process.env.NODE_ENV === 'development' && (
              <div>Debug: isLoading={String(isLoadingSetupIntent)}, error={String(!!error)}, clientSecret={String(!!clientSecret)}</div>
            )}
            No form to display
          </div>
        )}
      </div>

      <BudgetManager />
    </div>
  );
}
