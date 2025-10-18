'use client';

import { useState } from 'react';
import { useStripe, useElements, PaymentElement } from '@stripe/react-stripe-js';
import { useRouter } from 'next/navigation';

interface CardSetupFormProps {
  onCancel?: () => void;
  onSuccess?: () => void;
}

export default function CardSetupForm({ onCancel, onSuccess }: CardSetupFormProps) {
  const stripe = useStripe();
  const elements = useElements();
  const router = useRouter();
  const [isProcessing, setIsProcessing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!stripe || !elements) {
      return;
    }

    setIsProcessing(true);
    setError(null);

    try {
      // Confirm the SetupIntent
      const { error: stripeError, setupIntent } = await stripe.confirmSetup({
        elements,
        redirect: 'if_required',
      });

      if (stripeError) {
        setError(stripeError.message || 'An error occurred');
        setIsProcessing(false);
        return;
      }

      if (setupIntent && setupIntent.status === 'succeeded') {
        // In development mode, webhook will handle payment method save
        // In production, also rely on webhook
        console.log('✅ SetupIntent succeeded:', setupIntent.id);
        console.log('Payment method:', setupIntent.payment_method);

        // Show success message
        setSuccess(true);
        setIsProcessing(false);

        // Refresh after showing success message (give webhook time to process)
        setTimeout(() => {
          if (onSuccess) {
            onSuccess();
          }
          router.refresh();
        }, 3000);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'An error occurred');
      setIsProcessing(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <PaymentElement />

      {success && (
        <div className="p-4 rounded-md bg-green-50 dark:bg-green-950 border border-green-200 dark:border-green-800">
          <div className="flex items-center gap-2 text-green-700 dark:text-green-300">
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
            </svg>
            <span className="font-medium">Payment method saved successfully!</span>
          </div>
          <p className="text-sm text-green-600 dark:text-green-400 mt-1 ml-7">
            Redirecting...
          </p>
        </div>
      )}

      {error && (
        <div className="p-4 rounded-md bg-red-50 dark:bg-red-950 border border-red-200 dark:border-red-800">
          <div className="flex items-center gap-2 text-red-700 dark:text-red-300">
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
            <span className="font-medium">Failed to save payment method</span>
          </div>
          <p className="text-sm text-red-600 dark:text-red-400 mt-1 ml-7">
            {error}
          </p>
        </div>
      )}

      <div className="flex gap-3">
        {onCancel && (
          <button
            type="button"
            onClick={onCancel}
            disabled={isProcessing || success}
            className="flex-1 px-4 py-2 bg-secondary text-secondary-foreground rounded-md hover:bg-secondary/80 transition-colors disabled:opacity-50"
          >
            Cancel
          </button>
        )}
        <button
          type="submit"
          disabled={!stripe || isProcessing || success}
          className={`${onCancel ? 'flex-1' : 'w-full'} px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 transition-colors disabled:opacity-50`}
        >
          {isProcessing ? 'Processing...' : success ? 'Saved!' : 'Save Payment Method'}
        </button>
      </div>

      <p className="text-xs text-muted-foreground text-center">
        Your payment information is securely processed by Stripe. We never see
        or store your card details.
      </p>
    </form>
  );
}
