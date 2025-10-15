'use client';

import { SWRConfig } from 'swr';

export function Providers({ children }: { children: React.ReactNode }) {
  return (
    <SWRConfig
      value={{
        // Disable all automatic revalidation
        refreshInterval: 0,
        revalidateOnFocus: false,
        revalidateOnMount: false,
        revalidateOnReconnect: false,
        revalidateIfStale: false,
        // Only revalidate on explicit mutate() calls
        dedupingInterval: 0,
      }}
    >
      {children}
    </SWRConfig>
  );
}
