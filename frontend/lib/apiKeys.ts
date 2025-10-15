import useSWR from 'swr';

export interface ApiKey {
  id: string;
  name: string;
  created_at: string;
  last_used_at: string | null;
}

export interface CreateApiKeyResponse {
  id: string;
  name: string;
  api_key: string;
}

const fetcher = (url: string) =>
  fetch(url, {
    credentials: 'include',
  }).then((res) => {
    if (!res.ok) {
      throw new Error('Failed to fetch API keys');
    }
    return res.json();
  });

export function useApiKeys() {
  const { data, error, isLoading, mutate } = useSWR<ApiKey[]>(
    `/api/keys`,
    fetcher
  );

  const createApiKey = async (name: string): Promise<CreateApiKeyResponse> => {
    const response = await fetch(`/api/keys`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      credentials: 'include',
      body: JSON.stringify({ name }),
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: 'Failed to create API key' }));
      throw new Error(error.error || 'Failed to create API key');
    }

    const newKey = await response.json();
    mutate();
    return newKey;
  };

  const deleteApiKey = async (id: string) => {
    const response = await fetch(`/api/keys/${id}`, {
      method: 'DELETE',
      credentials: 'include',
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: 'Failed to delete API key' }));
      throw new Error(error.error || 'Failed to delete API key');
    }

    mutate();
  };

  return {
    apiKeys: data ?? [],
    isLoading,
    isError: !!error,
    error,
    createApiKey,
    deleteApiKey,
    mutate,
  };
}
