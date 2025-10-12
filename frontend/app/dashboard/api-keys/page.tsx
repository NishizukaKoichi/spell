'use client';

import { useState } from 'react';
import { useApiKeys, CreateApiKeyResponse } from '@/lib/apiKeys';

export default function ApiKeysPage() {
  const { apiKeys, isLoading, createApiKey, deleteApiKey } = useApiKeys();
  const [isCreating, setIsCreating] = useState(false);
  const [keyName, setKeyName] = useState('');
  const [createdKey, setCreatedKey] = useState<CreateApiKeyResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [copiedId, setCopiedId] = useState<string | null>(null);

  const handleCreateKey = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    if (!keyName.trim()) {
      setError('Please enter a name for the API key');
      return;
    }

    setIsCreating(true);

    try {
      const newKey = await createApiKey(keyName.trim());
      setCreatedKey(newKey);
      setKeyName('');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create API key');
    } finally {
      setIsCreating(false);
    }
  };

  const handleDeleteKey = async (id: string, name: string) => {
    if (!confirm(`Are you sure you want to delete "${name}"?`)) {
      return;
    }

    try {
      await deleteApiKey(id);
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to delete API key');
    }
  };

  const handleCopyKey = async (text: string, id: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopiedId(id);
      setTimeout(() => setCopiedId(null), 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold tracking-tight">API Keys</h1>
        <p className="text-muted-foreground mt-2">
          Manage your API keys for accessing the Spell Platform API
        </p>
      </div>

      {/* Create New Key Form */}
      <div className="rounded-lg border border-border bg-card p-6">
        <h2 className="text-lg font-semibold mb-4">Create New API Key</h2>
        <form onSubmit={handleCreateKey} className="space-y-4">
          <div>
            <label htmlFor="keyName" className="block text-sm font-medium mb-2">
              Key Name
            </label>
            <input
              id="keyName"
              type="text"
              value={keyName}
              onChange={(e) => setKeyName(e.target.value)}
              placeholder="e.g., Production API Key"
              className="w-full px-4 py-2 bg-background border border-input rounded-md focus:outline-none focus:ring-2 focus:ring-ring"
              disabled={isCreating}
            />
            <p className="mt-1 text-xs text-muted-foreground">
              Give your API key a descriptive name to remember what it&apos;s used for
            </p>
          </div>

          {error && (
            <div className="p-3 rounded-md bg-destructive/10 text-destructive text-sm">
              {error}
            </div>
          )}

          <button
            type="submit"
            disabled={isCreating}
            className="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 transition-colors disabled:opacity-50"
          >
            {isCreating ? 'Creating...' : 'Create API Key'}
          </button>
        </form>
      </div>

      {/* Created Key Modal */}
      {createdKey && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-card border border-border rounded-lg p-6 max-w-lg w-full mx-4">
            <h3 className="text-lg font-semibold mb-2">API Key Created!</h3>
            <p className="text-sm text-muted-foreground mb-4">
              Make sure to copy your API key now. You won&apos;t be able to see it again!
            </p>

            <div className="relative">
              <div className="p-3 bg-secondary rounded font-mono text-sm break-all">
                {createdKey.api_key}
              </div>
              <button
                onClick={() => handleCopyKey(createdKey.api_key, createdKey.id)}
                className="absolute top-2 right-2 px-3 py-1 bg-primary text-primary-foreground text-xs rounded hover:bg-primary/90 transition-colors"
              >
                {copiedId === createdKey.id ? 'Copied!' : 'Copy'}
              </button>
            </div>

            <div className="mt-4 flex justify-end">
              <button
                onClick={() => setCreatedKey(null)}
                className="px-4 py-2 bg-secondary text-secondary-foreground rounded-md hover:bg-secondary/80 transition-colors"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      )}

      {/* API Keys List */}
      <div className="rounded-lg border border-border bg-card">
        <div className="p-6 border-b border-border">
          <h2 className="text-lg font-semibold">Your API Keys</h2>
        </div>

        {isLoading ? (
          <div className="p-6 text-center text-muted-foreground">
            Loading API keys...
          </div>
        ) : apiKeys.length === 0 ? (
          <div className="p-6 text-center text-muted-foreground">
            No API keys yet. Create one to get started!
          </div>
        ) : (
          <div className="divide-y divide-border">
            {apiKeys.map((key) => (
              <div
                key={key.id}
                className="p-4 flex items-center justify-between hover:bg-accent/50 transition-colors"
              >
                <div className="flex-1">
                  <div className="font-medium">{key.name}</div>
                  <div className="text-sm text-muted-foreground mt-1">
                    Created: {formatDate(key.created_at)}
                    {key.last_used_at && (
                      <span className="ml-4">
                        Last used: {formatDate(key.last_used_at)}
                      </span>
                    )}
                  </div>
                </div>
                <button
                  onClick={() => handleDeleteKey(key.id, key.name)}
                  className="ml-4 px-3 py-1 text-sm text-destructive hover:bg-destructive/10 rounded transition-colors"
                >
                  Delete
                </button>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
