"use client";
import React, { useEffect, useState } from "react";

type Spell = {
  id: string;
  name: string;
  author: string;
  version: string;
  description: string;
  verified: boolean;
};

export default function SpellsPage() {
  const [spells, setSpells] = useState<Spell[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const res = await fetch("/api/spells?limit=50", { cache: "no-store" });
        if (!res.ok) throw new Error(await res.text());
        const data: Spell[] = await res.json();
        if (!cancelled) setSpells(data);
      } catch {
        if (!cancelled) setError("呪文一覧の取得に失敗しました");
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  if (loading)
    return <div className="p-6">読み込み中…</div>;
  if (error) return <div className="p-6 text-red-600">{error}</div>;
  if (spells.length === 0)
    return <div className="p-6 opacity-70">公開中の呪文がありません。</div>;

  return (
    <div className="p-6">
      <h1 className="text-xl font-bold mb-4">Scroll</h1>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {spells.map((s) => (
          <div key={s.id} className="border p-4 rounded shadow-sm">
            <h2 className="text-lg font-semibold flex items-center gap-2">
              <span>{s.name}</span>
              {s.verified && <span aria-label="verified">✅</span>}
            </h2>
            <p className="text-sm opacity-70">
              by {s.author} (v{s.version})
            </p>
            <p className="mt-2 text-sm">{s.description}</p>
            <button
              className="mt-3 bg-blue-600 text-white px-3 py-1 rounded"
              onClick={() => {
                const u = new URL("/dashboard/cast", window.location.origin);
                u.searchParams.set("spellId", s.id);
                window.location.href = u.toString();
              }}
            >
              実行
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}
