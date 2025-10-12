export default function DashboardPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Dashboard</h1>
        <p className="text-muted-foreground mt-2">
          Manage your API usage and billing
        </p>
      </div>

      {/* Usage Overview Card */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        <div className="rounded-lg border border-border bg-card p-6">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-medium text-muted-foreground">
              Monthly Spend
            </h3>
          </div>
          <div className="mt-2">
            <p className="text-2xl font-bold">$0.00</p>
            <p className="text-xs text-muted-foreground mt-1">
              of $50.00 limit
            </p>
          </div>
        </div>

        <div className="rounded-lg border border-border bg-card p-6">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-medium text-muted-foreground">
              API Calls
            </h3>
          </div>
          <div className="mt-2">
            <p className="text-2xl font-bold">0</p>
            <p className="text-xs text-muted-foreground mt-1">
              This month
            </p>
          </div>
        </div>

        <div className="rounded-lg border border-border bg-card p-6">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-medium text-muted-foreground">
              Remaining Budget
            </h3>
          </div>
          <div className="mt-2">
            <p className="text-2xl font-bold">$50.00</p>
            <p className="text-xs text-muted-foreground mt-1">
              100% available
            </p>
          </div>
        </div>
      </div>

      {/* Quick Actions */}
      <div className="rounded-lg border border-border bg-card p-6">
        <h2 className="text-lg font-semibold mb-4">Quick Actions</h2>
        <div className="grid gap-4 md:grid-cols-2">
          <a
            href="/dashboard/api-keys"
            className="flex items-center justify-between p-4 rounded-lg border border-border hover:bg-accent transition-colors"
          >
            <div>
              <h3 className="font-medium">Create API Key</h3>
              <p className="text-sm text-muted-foreground">
                Generate a new API key for your application
              </p>
            </div>
            <svg
              className="w-5 h-5 text-muted-foreground"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M9 5l7 7-7 7"
              />
            </svg>
          </a>

          <a
            href="/dashboard/billing"
            className="flex items-center justify-between p-4 rounded-lg border border-border hover:bg-accent transition-colors"
          >
            <div>
              <h3 className="font-medium">Manage Billing</h3>
              <p className="text-sm text-muted-foreground">
                Update payment method and spending limits
              </p>
            </div>
            <svg
              className="w-5 h-5 text-muted-foreground"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M9 5l7 7-7 7"
              />
            </svg>
          </a>
        </div>
      </div>
    </div>
  );
}
