'use client';

import { useState } from 'react';
import { useStripe, useElements, PaymentElement } from '@stripe/react-stripe-js';
import { useRouter } from 'next/navigation';

interface CardSetupFormProps {
  onCancel?: () => void;
}

export default function CardSetupForm({ onCancel }: CardSetupFormProps) {
  const stripe = useStripe();
  const elements = useElements();
  const router = useRouter();
  const [isProcessing, setIsProcessing] = useState(false);
  const [error, setError] = useState<string | null>(null);

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

        // Success! Refresh the page after a short delay to let webhook process
        setTimeout(() => {
          router.refresh();
          window.location.reload();
        }, 2000);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'An error occurred');
      setIsProcessing(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <PaymentElement />

      {error && (
        <div className="p-3 rounded-md bg-destructive/10 text-destructive text-sm">
          {error}
        </div>
      )}

      <div className="flex gap-3">
        {onCancel && (
          <button
            type="button"
            onClick={onCancel}
            disabled={isProcessing}
            className="flex-1 px-4 py-2 bg-secondary text-secondary-foreground rounded-md hover:bg-secondary/80 transition-colors disabled:opacity-50"
          >
            Cancel
          </button>
        )}
        <button
          type="submit"
          disabled={!stripe || isProcessing}
          className={`${onCancel ? 'flex-1' : 'w-full'} px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 transition-colors disabled:opacity-50`}
        >
          {isProcessing ? 'Processing...' : 'Save Payment Method'}
        </button>
      </div>

      <p className="text-xs text-muted-foreground text-center">
        Your payment information is securely processed by Stripe. We never see
        or store your card details.
      </p>
    </form>
  );
}
