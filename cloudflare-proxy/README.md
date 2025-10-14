# Spell Platform - Cloudflare Reverse Proxy

統合リバースプロキシで単一ドメイン (magicspell.io) からAPIとフロントエンドを配信します。

## アーキテクチャ

```
magicspell.io/api/*  → Fly.io (Rust API)
magicspell.io/auth/* → Fly.io (OAuth)
magicspell.io/*      → Vercel (Next.js)
```

## メリット

- Cookie/セッションのドメイン跨ぎ問題を解決
- CORS設定の簡素化
- TLS証明書の一元管理
- Cloudflareの DDoS/WAF 保護

## デプロイ

### 1. 依存関係のインストール

```bash
cd cloudflare-proxy
npm install
```

### 2. Wrangler でログイン

```bash
npx wrangler login
```

### 3. デプロイ

```bash
npm run deploy
```

### 4. DNS設定

Cloudflareダッシュボードで以下を設定：

1. `magicspell.io` の Aレコードを削除（Workers Routeを使用）
2. Workers Routeを設定:
   - Route: `magicspell.io/*`
   - Worker: `spell-platform-proxy`

## 開発

ローカルでWorkerをテスト:

```bash
npm run dev
```

## 環境変数

`wrangler.toml` で設定:

- `API_ORIGIN`: Fly.io APIのURL（例: https://spell-platform.fly.dev）
- `FRONTEND_ORIGIN`: VercelのURL（例: https://spell-caster-magicspell.vercel.app）
