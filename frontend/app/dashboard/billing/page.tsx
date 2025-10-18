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
  const { paymentMethod, isLoading: isLoadingPaymentMethod, mutate: mutatePaymentMethod } = usePaymentMethod();
  const [clientSecret, setClientSecret] = useState<string | null>(null);
  const [isLoadingSetupIntent, setIsLoadingSetupIntent] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showCardForm, setShowCardForm] = useState(false);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);

  // Development mode: Force show form for UI testing
  const isDev = process.env.NODE_ENV === 'development';
  const [devMode, setDevMode] = useState(false);

  const handleDeletePaymentMethod = async () => {
    setIsDeleting(true);
    setError(null);

    try {
      const response = await fetch('/api/billing/payment-method', {
        method: 'DELETE',
        credentials: 'include',
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.error || 'Failed to delete payment method');
      }

      // Refresh payment method data
      mutatePaymentMethod();
      setShowDeleteConfirm(false);
    } catch (err) {
      console.error('Delete error:', err);
      setError(err instanceof Error ? err.message : 'Failed to delete payment method');
    } finally {
      setIsDeleting(false);
    }
  };

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
        setError(err instanceof Error ? err.message : 'An error occurred');

        // Development mode: Allow testing UI without backend
        if (process.env.NODE_ENV === 'development') {
          console.warn('⚠️ Using mock client_secret for development. PaymentElement will not work.');
          setClientSecret('seti_mock_' + Math.random().toString(36).substr(2, 9));
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
              <div className="flex items-center gap-3 flex-1">
                <div className="w-12 h-8 bg-gradient-to-br from-blue-600 to-purple-600 rounded flex items-center justify-center text-white font-bold text-sm">
                  {paymentMethod.brand.charAt(0).toUpperCase() + paymentMethod.brand.slice(1) === 'Visa' ? 'VISA' : paymentMethod.brand.toUpperCase().slice(0, 4)}
                </div>
                <div>
                  <div className="font-medium">
                    {paymentMethod.brand.charAt(0).toUpperCase() + paymentMethod.brand.slice(1)} •••• {paymentMethod.last4}
                  </div>
                  <div className="text-sm text-muted-foreground">
                    Expires {paymentMethod.exp_month}/{paymentMethod.exp_year}
                  </div>
                </div>
              </div>
              <div className="flex gap-2">
                <button
                  onClick={() => setShowCardForm(true)}
                  className="px-4 py-2 bg-secondary text-secondary-foreground rounded-md hover:bg-secondary/80 transition-colors"
                >
                  Update
                </button>
                <button
                  onClick={() => setShowDeleteConfirm(true)}
                  className="px-4 py-2 bg-red-500/10 text-red-600 dark:text-red-400 rounded-md hover:bg-red-500/20 transition-colors"
                >
                  Delete
                </button>
              </div>
            </div>
            {error && (
              <p className="mt-2 text-sm text-destructive">{error}</p>
            )}

            {showDeleteConfirm && (
              <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
                <div className="bg-white dark:bg-gray-800 rounded-lg p-6 max-w-md w-full mx-4">
                  <h3 className="text-lg font-semibold mb-2">Delete Payment Method?</h3>
                  <p className="text-sm text-muted-foreground mb-6">
                    Are you sure you want to delete this payment method? You won&apos;t be able to use the Spell API until you add a new one.
                  </p>
                  <div className="flex gap-3 justify-end">
                    <button
                      onClick={() => setShowDeleteConfirm(false)}
                      disabled={isDeleting}
                      className="px-4 py-2 bg-secondary text-secondary-foreground rounded-md hover:bg-secondary/80 transition-colors disabled:opacity-50"
                    >
                      Cancel
                    </button>
                    <button
                      onClick={handleDeletePaymentMethod}
                      disabled={isDeleting}
                      className="px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-700 transition-colors disabled:opacity-50"
                    >
                      {isDeleting ? 'Deleting...' : 'Delete'}
                    </button>
                  </div>
                </div>
              </div>
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
        ) : error ? (
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
                <CardSetupForm
                  onCancel={() => {
                    setShowCardForm(false);
                    setClientSecret(null);
                  }}
                  onSuccess={() => {
                    // Close form and refresh payment method data
                    setShowCardForm(false);
                    setClientSecret(null);
                  }}
                />
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
