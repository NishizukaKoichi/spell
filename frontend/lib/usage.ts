import useSWR from 'swr';

interface Usage {
  user_id: string;
  period: string;
  total_calls: number;
  total_cost_cents: number;
  hard_limit_cents: number | null;
  soft_limit_cents: number | null;
}

interface PaymentMethod {
  brand: string;
  last4: string;
  exp_month: number;
  exp_year: number;
}

const fetcher = (url: string) =>
  fetch(url, {
    credentials: 'include',
  }).then((res) => {
    if (!res.ok) {
      if (res.status === 404) {
        return null;
      }
      throw new Error('Failed to fetch data');
    }
    return res.json();
  });

export function useUsage() {
  const { data, error, isLoading, mutate } = useSWR<Usage | null>(
    '/api/usage',
    fetcher,
    {
      onError: () => {
        // Silently handle 404 errors (usage endpoint not yet implemented)
      },
    }
  );

  return {
    usage: data,
    isLoading,
    isError: !!error,
    error,
    mutate,
  };
}

export function usePaymentMethod() {
  const { data, error, isLoading, mutate } = useSWR<PaymentMethod | null>(
    '/api/payment-method',
    fetcher
  );

  return {
    paymentMethod: data,
    isLoading,
    isError: !!error,
    error,
    mutate,
  };
}
