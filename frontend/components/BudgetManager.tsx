'use client';

import { useState } from 'react';
import { useBudget } from '@/lib/budget';

const MIN_BUDGET = 10;
const MAX_BUDGET = 500;

export default function BudgetManager() {
  const { budget, isLoading, updateBudget } = useBudget();
  const [isEditing, setIsEditing] = useState(false);
  const [amount, setAmount] = useState<string>('');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  const currentLimit = budget?.hard_limit_cents
    ? budget.hard_limit_cents / 100
    : 50; // Default to $50

  const handleEdit = () => {
    setAmount(currentLimit.toString());
    setIsEditing(true);
    setError(null);
    setSuccess(false);
  };

  const handleCancel = () => {
    setIsEditing(false);
    setAmount('');
    setError(null);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setSuccess(false);

    const numAmount = parseFloat(amount);

    // Validation
    if (isNaN(numAmount)) {
      setError('Please enter a valid amount');
      return;
    }

    if (numAmount < MIN_BUDGET || numAmount > MAX_BUDGET) {
      setError(`Amount must be between $${MIN_BUDGET} and $${MAX_BUDGET}`);
      return;
    }

    setIsSubmitting(true);

    try {
      await updateBudget(Math.round(numAmount * 100), null);
      setSuccess(true);
      setIsEditing(false);
      setAmount('');

      // Clear success message after 3 seconds
      setTimeout(() => setSuccess(false), 3000);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update budget');
    } finally {
      setIsSubmitting(false);
    }
  };

  // Show loading only on initial load
  if (isLoading && !budget) {
    return (
      <div className="rounded-lg border border-border bg-card p-6">
        <h2 className="text-lg font-semibold mb-4">Spending Limit</h2>
        <div className="text-sm text-muted-foreground">Loading...</div>
      </div>
    );
  }

  return (
    <div className="rounded-lg border border-border bg-card p-6">
      <h2 className="text-lg font-semibold mb-4">Spending Limit</h2>

      {!isEditing ? (
        <>
          <p className="text-sm text-muted-foreground mb-4">
            Your monthly spending limit controls how much you can spend on API
            usage each month. You can adjust this limit between ${MIN_BUDGET}{' '}
            and ${MAX_BUDGET}.
          </p>

          {currentLimit < MIN_BUDGET && (
            <div className="mb-4 p-3 rounded-md bg-yellow-500/10 text-yellow-600 dark:text-yellow-400 text-sm">
              ⚠️ Warning: Your monthly spending limit is below the recommended minimum of ${MIN_BUDGET}. Consider increasing your limit to avoid service interruptions.
            </div>
          )}

          {success && (
            <div className="mb-4 p-3 rounded-md bg-green-500/10 text-green-600 dark:text-green-400 text-sm">
              Spending limit updated successfully!
            </div>
          )}

          <div className="flex items-center gap-4">
            <div className="flex-1">
              <div className="text-2xl font-bold">${currentLimit.toFixed(2)}</div>
              <div className="text-sm text-muted-foreground">
                Monthly spending limit
              </div>
            </div>
            <button
              onClick={handleEdit}
              className="px-4 py-2 bg-secondary text-secondary-foreground rounded-md hover:bg-secondary/80 transition-colors"
            >
              Change Limit
            </button>
          </div>
        </>
      ) : (
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label
              htmlFor="amount"
              className="block text-sm font-medium mb-2"
            >
              New Monthly Limit
            </label>
            <div className="relative">
              <span className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground">
                $
              </span>
              <input
                id="amount"
                type="number"
                step="1"
                min={MIN_BUDGET}
                max={MAX_BUDGET}
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                className="w-full pl-7 pr-4 py-2 bg-background border border-input rounded-md focus:outline-none focus:ring-2 focus:ring-ring"
                placeholder={`${MIN_BUDGET} - ${MAX_BUDGET}`}
                disabled={isSubmitting}
              />
            </div>
            <p className="mt-1 text-xs text-muted-foreground">
              Enter an amount between ${MIN_BUDGET} and ${MAX_BUDGET}
            </p>
          </div>

          {error && (
            <div className="p-3 rounded-md bg-destructive/10 text-destructive text-sm">
              {error}
            </div>
          )}

          <div className="flex gap-2">
            <button
              type="submit"
              disabled={isSubmitting}
              className="flex-1 px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 transition-colors disabled:opacity-50"
            >
              {isSubmitting ? 'Updating...' : 'Update Limit'}
            </button>
            <button
              type="button"
              onClick={handleCancel}
              disabled={isSubmitting}
              className="px-4 py-2 bg-secondary text-secondary-foreground rounded-md hover:bg-secondary/80 transition-colors disabled:opacity-50"
            >
              Cancel
            </button>
          </div>
        </form>
      )}
    </div>
  );
}
