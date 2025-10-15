import useSWR from 'swr';

interface Budget {
  user_id: string;
  period: string;
  soft_limit_cents: number | null;
  hard_limit_cents: number | null;
  notify_thresholds: number[];
  created_at: string;
  updated_at: string;
}

const fetcher = (url: string) =>
  fetch(url, {
    credentials: 'include',
  }).then((res) => {
    if (!res.ok) {
      if (res.status === 404) {
        return null;
      }
      throw new Error('Failed to fetch budget');
    }
    return res.json();
  });

export function useBudget() {
  const { data, error, isLoading, mutate } = useSWR<Budget | null>(
    '/api/budgets',
    fetcher
  );

  const updateBudget = async (
    hard_limit_cents: number | null,
    soft_limit_cents: number | null
  ) => {
    const response = await fetch('/api/budgets', {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
      },
      credentials: 'include',
      body: JSON.stringify({
        hard_limit_cents,
        soft_limit_cents,
        period: 'monthly',
      }),
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: 'Failed to update budget' }));
      throw new Error(error.error || 'Failed to update budget');
    }

    const updated = await response.json();
    mutate(updated, false);
    return updated;
  };

  return {
    budget: data,
    isLoading,
    isError: !!error,
    error,
    updateBudget,
    mutate,
  };
}
