# Spell - AI引き継ぎプロンプト

## プロジェクト概要

**プロダクト名**: Spell（旧称: Spell Platform）
**リポジトリ**: https://github.com/NishizukaKoichi/Spell
**本番環境**: https://magicspell.io
**アーキテクチャ**: WASM-based API platform for Creator-to-Consumer workflows

## 技術スタック

### バックエンド（Rust）
- **フレームワーク**: Actix-web
- **デプロイ先**: Fly.io (`spell-platform` app)
- **エンドポイント**: https://api.magicspell.io
- **バイナリ名**: `spell-api`
- **主要機能**:
  - GitHub OAuth認証
  - Stripe決済統合（webhook-based payment method saving）
  - WASM spell実行
  - API key管理
  - Budget/Usage tracking

### フロントエンド（Next.js）
- **バージョン**: Next.js 15.5.4 (Turbopack)
- **デプロイ先**: Vercel
- **エンドポイント**: https://magicspell.io
- **パッケージ名**: `frontend`

### インフラ
- **Database**: PostgreSQL (Fly.io)
- **Cache**: Redis (Fly.io)
- **Reverse Proxy**: Cloudflare Workers（単一ドメイン統合）
- **決済**: Stripe (Live Mode)
- **認証**: GitHub OAuth

## 現在の状態

### 最新の変更（2025-10-15）

#### ✅ 完了したこと
1. **Stripe webhook-based payment method saving実装**
   - `SetupIntent.succeeded` webhookハンドラー追加
   - 手動API呼び出しを削除、webhookで自動保存
   - Dev endpoint (`/v1/billing/dev-setup-intent`) 追加（ローカルテスト用）

2. **本番環境への切り替え**
   - Stripe Test Mode → Live Mode
   - API Keys configured via Fly.io secrets and Vercel environment variables

3. **プロダクト名変更**
   - `spell-platform` → `Spell`
   - GitHubリポジトリ: https://github.com/NishizukaKoichi/Spell
   - README更新済み

4. **CI/CD**
   - GitHub Actions: オールグリーン✅
   - cargo fmt修正済み
   - シークレットスキャン対応済み

### 主要ファイル

#### バックエンド
- `src/routes/billing.rs`: Stripe billing endpoints
  - `create_setup_intent()`: 本番用（認証必須）
  - `dev_create_setup_intent()`: 開発用（DEV_MODE_USER_ID使用）
- `src/services/stripe_service.rs`: Stripe統合
  - `handle_setup_intent_succeeded()`: Webhook handler
  - `handle_webhook_event()`: Webhookルーティング

#### フロントエンド
- `frontend/components/CardSetupForm.tsx`: Payment Element UI（webhook依存、手動API呼び出しなし）
- `frontend/app/api/billing/setup-intent/route.ts`: 環境別エンドポイント切り替え
- `frontend/.env.local`: ローカル環境変数（本番Stripe keys設定済み）

### 環境変数

#### ローカル開発（テスト済み）
```bash
# Backend
DATABASE_URL=postgres://spell_user:testpass123@localhost:5433/spell_platform
REDIS_URL=redis://localhost:6379
STRIPE_SECRET_KEY=sk_test_51NHGUiGYwUAALlaC...（テストキー）
STRIPE_WEBHOOK_SECRET=whsec_e9f9d95f4a3ac83c8661026fe8b4d4069cc8d3b82a5074fb81ad587c7afc54f1
DEV_MODE_USER_ID=2c5657b2-eb98-44c4-a243-eb6533b78133  # オプション：ローカルテスト用
SQLX_OFFLINE=true

# Frontend
NEXT_PUBLIC_STRIPE_PUBLISHABLE_KEY=pk_live_51NHGUiGYwUAALlaC...（本番キー）
NEXT_PUBLIC_API_BASE=http://localhost:8080
```

#### 本番環境（Fly.io + Vercel）
- Fly.io secrets: `flyctl secrets list -a spell-platform`
- Vercel env: `vercel env ls`

## 重要な仕様

### Stripe決済フロー
1. **ローカル開発**:
   - Stripe CLI webhook forwarding必須
   - テストカード: 4242 4242 4242 4242
   - Dev endpoint使用（認証バイパス）

2. **本番環境**:
   - Stripe webhook: https://api.magicspell.io/v1/webhooks/stripe
   - 実カードのみ（テストカード不可）
   - GitHub OAuth認証必須

### 認証フロー
- GitHub OAuth → セッションCookie (`spell_session`)
- Cloudflare Workers経由で単一ドメイン（CORS問題解消済み）

### ビルド・デプロイ
```bash
# Backend (Fly.io)
flyctl deploy -a spell-platform

# Frontend (Vercel)
git push origin main  # 自動デプロイ

# CI確認
gh run list --limit 5
```

## トラブルシューティング

### よくある問題

1. **Payment Element表示されない**
   - `NEXT_PUBLIC_STRIPE_PUBLISHABLE_KEY`確認
   - ブラウザコンソールでStripe.js読み込みエラー確認

2. **Webhook受信されない**
   - ローカル: `stripe listen --forward-to localhost:8080/v1/webhooks/stripe`
   - 本番: Stripeダッシュボードでwebhook設定確認

3. **401 Unauthorized**
   - ローカル: `DEV_MODE_USER_ID`設定されているか
   - 本番: GitHub OAuthログイン済みか

4. **CI失敗**
   - `cargo fmt --all`実行
   - `cargo clippy --fix --allow-dirty`実行

## 次のステップ（未実装）

### Phase 3候補
- [ ] Multi-region deployment（US, EU, APAC）
- [ ] GDPR/CCPA compliance
- [ ] Data export機能
- [ ] SBOM生成 & Sigstore統合
- [ ] SOC 2認証

## 連絡先・リソース

- **GitHub Issues**: https://github.com/NishizukaKoichi/Spell/issues
- **Documentation**: README.md, PLANS.md, frontend/OAUTH_SETUP.md
- **Stripe Dashboard**: https://dashboard.stripe.com/
- **Fly.io Dashboard**: https://fly.io/apps/spell-platform
- **Vercel Dashboard**: https://vercel.com/magicspell

## AI向け注意事項

1. **コード変更時**:
   - 必ず`cargo fmt`と`cargo clippy`を実行
   - `.env.local`のシークレットはコミット禁止
   - テストモードと本番モードを混同しない

2. **Stripe関連**:
   - Test/Live keysの切り替えに注意
   - Webhookシグネチャ検証は必須
   - Payment method保存は**webhookのみ**（手動API呼び出し禁止）

3. **デプロイ**:
   - mainブランチへのpushで自動デプロイ
   - CI失敗時は必ず修正してからマージ
   - Fly.ioアプリ名は`spell-platform`のまま（変更不要）

4. **命名規則**:
   - プロダクト名: `Spell`
   - リポジトリ: `Spell`
   - Rustバイナリ: `spell-api`
   - npmパッケージ: `frontend`
   - Fly.ioアプリ: `spell-platform`（変更しない）

---

**最終更新**: 2025-10-15
**作成者**: Claude Code
**メンテナー**: KOICHI NISHIZUKA
