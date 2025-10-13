'use client';

import { useEffect, useState } from 'react';

type ApiKeyMasked = {
  id: string;
  name: string;
  prefix: string;
  last4: string;
  status: 'active' | 'revoked';
  created_at: string;
  expires_at?: string | null;
};

export default function ApiKeysPage() {
  const [keys, setKeys] = useState<ApiKeyMasked[]>([]);
  const [loading, setLoading] = useState(false);
  const [newKeyName, setNewKeyName] = useState('');
  const [issuedKey, setIssuedKey] = useState<string | null>(null); // ★ 一度だけ表示

  const refresh = async () => {
    setLoading(true);
    const res = await fetch('/api/keys', { cache: 'no-store' });
    if (res.ok) setKeys(await res.json());
    setLoading(false);
  };

  useEffect(() => { refresh(); }, []);

  const createKey = async () => {
    if (!newKeyName.trim()) return;
    setLoading(true);
    const res = await fetch('/api/keys', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name: newKeyName.trim(), ttl_days: 365 }),
    });
    setLoading(false);
    if (!res.ok) return alert('APIキーの作成に失敗しました');
    const data = await res.json();
    setIssuedKey(data.api_key); // ★ このタイミングでしか生キーを見せない
    setNewKeyName('');
    await refresh();
  };

  const revokeKey = async (id: string) => {
    if (!confirm('このAPIキーを失効します。よろしいですか？')) return;
    const res = await fetch(`/api/keys/${id}`, { method: 'DELETE' });
    if (!res.ok) return alert('失効に失敗しました');
    await refresh();
  };

  return (
    <main className="mx-auto max-w-3xl p-6 space-y-6">
      <header className="space-y-2">
        <h1 className="text-2xl font-semibold">API Keys</h1>
        <p className="text-sm text-gray-500">発行時に表示される鍵は**一度だけ**。必ず安全な場所に保存すること。</p>
      </header>

      <section className="p-4 rounded-2xl border">
        <div className="flex gap-2">
          <input
            className="flex-1 border rounded-xl px-3 py-2"
            placeholder="例: primary"
            value={newKeyName}
            onChange={(e) => setNewKeyName(e.target.value)}
          />
          <button
            onClick={createKey}
            disabled={loading}
            className="rounded-2xl px-4 py-2 border hover:bg-gray-50"
          >
            新規発行
          </button>
        </div>

        {issuedKey && (
          <div className="mt-4 p-4 rounded-xl border bg-yellow-50">
            <div className="font-medium mb-1">このAPIキーは一度しか表示されません：</div>
            <code className="break-all">{issuedKey}</code>
            <div className="mt-2 text-sm text-gray-600">
              例）<code>Authorization: Bearer {issuedKey.slice(0, 12)}…</code>
            </div>
            <div className="mt-3">
              <button
                onClick={() => { navigator.clipboard.writeText(issuedKey); }}
                className="rounded-xl px-3 py-1.5 border hover:bg-gray-100"
              >
                コピー
              </button>
              <button
                onClick={() => setIssuedKey(null)}
                className="ml-2 rounded-xl px-3 py-1.5 border hover:bg-gray-100"
              >
                閉じる
              </button>
            </div>
          </div>
        )}
      </section>

      <section className="p-4 rounded-2xl border">
        <div className="flex items-center justify-between mb-3">
          <h2 className="font-medium">既存のキー</h2>
          <button onClick={refresh} className="text-sm underline">再読込</button>
        </div>
        {loading ? (
          <div className="text-sm text-gray-500">読み込み中…</div>
        ) : keys.length === 0 ? (
          <div className="text-sm text-gray-500">まだキーがありません。</div>
        ) : (
          <ul className="space-y-2">
            {keys.map(k => (
              <li key={k.id} className="flex items-center justify-between border rounded-xl px-3 py-2">
                <div className="text-sm">
                  <div className="font-mono">
                    {k.prefix}_…{k.last4} <span className="ml-2 text-gray-500">({k.name})</span>
                  </div>
                  <div className="text-gray-500 text-xs">
                    {k.status}・発行 {new Date(k.created_at).toLocaleString()}
                    {k.expires_at ? `・期限 ${new Date(k.expires_at).toLocaleDateString()}` : ''}
                  </div>
                </div>
                <button onClick={() => revokeKey(k.id)} className="text-sm text-red-600 underline">失効</button>
              </li>
            ))}
          </ul>
        )}
      </section>
    </main>
  );
}
