import useSWR from 'swr';

const API_URL = process.env.NEXT_PUBLIC_API_BASE || '';

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
    `${API_URL}/usage`,
    fetcher,
    {
      refreshInterval: 30000, // Refresh every 30 seconds
      revalidateOnFocus: true,
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
    `${API_URL}/payment-method`,
    fetcher,
    {
      revalidateOnFocus: false,
    }
  );

  return {
    paymentMethod: data,
    isLoading,
    isError: !!error,
    error,
    mutate,
  };
}
