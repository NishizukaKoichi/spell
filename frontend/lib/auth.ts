import useSWR from 'swr';

const API_URL = process.env.NEXT_PUBLIC_API_BASE || '';

export interface User {
  id: string;
  github_login: string;
  github_name: string | null;
  github_email: string | null;
  github_avatar_url: string | null;
}

interface AuthResponse {
  authenticated: boolean;
  user: User;
}

const fetcher = async (url: string): Promise<AuthResponse> => {
  const res = await fetch(url, {
    credentials: 'include', // Include cookies
  });

  if (!res.ok) {
    throw new Error('Not authenticated');
  }

  return res.json();
};

export function useAuth() {
  const { data, error, isLoading, mutate } = useSWR<AuthResponse>(
    `${API_URL}/auth/me`,
    fetcher,
    {
      revalidateOnFocus: false,
      shouldRetryOnError: false,
    }
  );

  const logout = async () => {
    try {
      await fetch(`${API_URL}/auth/logout`, {
        method: 'POST',
        credentials: 'include',
      });
      mutate(undefined, false); // Clear the cache
      window.location.href = '/login';
    } catch (error) {
      console.error('Logout failed:', error);
    }
  };

  return {
    user: data?.user,
    isAuthenticated: !!data?.authenticated,
    isLoading,
    isError: !!error,
    logout,
  };
}
