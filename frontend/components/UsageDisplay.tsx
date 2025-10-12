'use client';

import { useUsage, usePaymentMethod } from '@/lib/usage';

export default function UsageDisplay() {
  const { usage, isLoading: usageLoading } = useUsage();
  const { paymentMethod, isLoading: pmLoading } = usePaymentMethod();

  if (usageLoading || pmLoading) {
    return (
      <div className="rounded-lg border border-border bg-card p-6">
        <h2 className="text-lg font-semibold mb-4">Usage & Billing</h2>
        <div className="text-sm text-muted-foreground">Loading...</div>
      </div>
    );
  }

  const totalCost = usage?.total_cost_cents ?? 0;
  const hardLimit = usage?.hard_limit_cents ?? 5000; // Default $50
  const usagePercentage = hardLimit > 0 ? (totalCost / hardLimit) * 100 : 0;
  const remaining = Math.max(0, hardLimit - totalCost);

  return (
    <div className="rounded-lg border border-border bg-card p-6">
      <h2 className="text-lg font-semibold mb-4">Usage & Billing</h2>

      {/* Payment Method */}
      {paymentMethod && (
        <div className="mb-6 pb-6 border-b border-border">
          <h3 className="text-sm font-medium mb-2">Payment Method</h3>
          <div className="flex items-center gap-2">
            <div className="px-3 py-1 bg-secondary rounded text-sm">
              {paymentMethod.brand.toUpperCase()} ···· {paymentMethod.last4}
            </div>
            <div className="text-xs text-muted-foreground">
              Expires {paymentMethod.exp_month}/{paymentMethod.exp_year}
            </div>
          </div>
        </div>
      )}

      {/* Usage Stats */}
      <div className="space-y-4">
        <div>
          <div className="flex justify-between items-baseline mb-2">
            <h3 className="text-sm font-medium">This Month&apos;s Usage</h3>
            <div className="text-2xl font-bold">
              ${(totalCost / 100).toFixed(2)}
            </div>
          </div>

          {/* Progress Bar */}
          <div className="relative h-2 bg-secondary rounded-full overflow-hidden">
            <div
              className={`absolute inset-y-0 left-0 rounded-full transition-all ${
                usagePercentage > 90
                  ? 'bg-red-500'
                  : usagePercentage > 70
                    ? 'bg-yellow-500'
                    : 'bg-green-500'
              }`}
              style={{ width: `${Math.min(usagePercentage, 100)}%` }}
            />
          </div>

          <div className="flex justify-between items-center mt-2 text-sm text-muted-foreground">
            <span>{usagePercentage.toFixed(1)}% used</span>
            <span>Limit: ${(hardLimit / 100).toFixed(2)}</span>
          </div>
        </div>

        {/* Stats Grid */}
        <div className="grid grid-cols-2 gap-4 pt-4">
          <div>
            <div className="text-sm text-muted-foreground mb-1">
              API Calls
            </div>
            <div className="text-xl font-semibold">
              {usage?.total_calls?.toLocaleString() ?? 0}
            </div>
          </div>
          <div>
            <div className="text-sm text-muted-foreground mb-1">
              Remaining Budget
            </div>
            <div className="text-xl font-semibold">
              ${(remaining / 100).toFixed(2)}
            </div>
          </div>
        </div>
      </div>

      {usagePercentage > 90 && (
        <div className="mt-4 p-3 rounded-md bg-red-500/10 text-red-600 dark:text-red-400 text-sm">
          ⚠️ You&apos;re approaching your spending limit. Consider increasing your
          budget to avoid service interruptions.
        </div>
      )}
    </div>
  );
}
