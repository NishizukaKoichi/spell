# Phase 2 実装 引き継ぎプロンプト

## 現在の状態（2025-10-10）

### ✅ 完了済み

**Phase 1: API Keys + Rate Limiting**
- 本番稼働中（Fly.io）
- GitHub: https://github.com/NishizukaKoichi/spell-platform
- コミット: `0a54c66` (feat: Implement Phase 1)
- E2Eテスト全通過

**Phase 2: 準備完了**
- コミット: `11b5ea3` (wip: Add Phase 2 scaffolding)
- マイグレーション: `migrations/0004_billing.sql` 作成済み（未実行）
- モデル: `src/models/billing.rs` 実装済み
- 依存関係: `async-stripe`, `prometheus`, `hmac`, `sha2`, `hex` 追加済み

### 📍 現在地

```
/Users/koichinishizuka/spell-platform
```

### 🗄️ データベース接続情報

```bash
# PostgreSQL (Fly.io)
DATABASE_URL=postgres://spell_platform:3cTGKZw6xjtld6b@spell-platform-db.flycast:5432/spell_platform?sslmode=disable

# Redis (Upstash)
REDIS_URL=redis://default:***@fly-spell-platform-redis.upstash.io:6379

# ローカルプロキシ経由でアクセス可能
flyctl proxy 5432 -a spell-platform-db  # PostgreSQL
PGPASSWORD='3cTGKZw6xjtld6b' /opt/homebrew/opt/libpq/bin/psql -h localhost -U spell_platform -d spell_platform
```

### 🎯 本番環境

- URL: https://spell-platform.fly.dev
- アプリ名: `spell-platform`
- DB: `spell-platform-db`
- Redis: `spell-platform-redis`

---

## 次にやること（Phase 2 本実装）

### 必須タスク（優先順）

1. **Stripeサービス実装** (`src/services/stripe.rs`)
   - Checkout Session 作成
   - Webhook署名検証（HMAC-SHA256）
   - イベントハンドラ（`checkout.session.completed`, `customer.subscription.*`, `invoice.paid`）

2. **Billingルート** (`src/routes/billing.rs`)
   - `POST /v1/billing/checkout` - Stripe Checkout URL返却
   - `POST /webhooks/stripe` - Webhook受信・検証・処理

3. **Budgetsルート** (`src/routes/budgets.rs`)
   - `GET /v1/budgets` - 現在の予算設定取得
   - `POST /v1/budgets` - 予算作成
   - `PUT /v1/budgets` - 予算更新

4. **Cast予算チェック** (`src/routes/cast.rs` 更新)
   - `/v1/cast` 実行**前**にハードリミット検査
   - 超過時 → HTTP 402 `{"error":"budget_exceeded", ...}`
   - 実行後に `usage_counters` 更新 + `casts.cost_cents` 記録

5. **メトリクス** (`src/middleware/metrics.rs` + `/metrics`)
   - Prometheusフォーマット出力
   - カウンタ: `spell_cast_total`, `rate_limited_total`, `budget_block_total`, `stripe_webhook_total`
   - ヒストグラム: `spell_cast_duration_seconds`
   - ゲージ: `db_pool_in_use`, `redis_errors_total`

6. **ENV検証** (`src/main.rs`)
   - 起動時に `STRIPE_SECRET_KEY`, `STRIPE_WEBHOOK_SECRET`, `COST_PER_CAST_CENTS` 確認
   - 欠如時はbilling機能を無効化（ログ警告）

7. **デプロイ & マイグレーション**
   ```bash
   # Secrets設定（テスト用）
   flyctl secrets set \
     STRIPE_SECRET_KEY=sk_test_*** \
     STRIPE_WEBHOOK_SECRET=whsec_*** \
     COST_PER_CAST_CENTS=1 \
     BILLING_PLAN_DEFAULT=free

   # デプロイ
   flyctl deploy --ha=false

   # マイグレーション実行
   PGPASSWORD='3cTGKZw6xjtld6b' /opt/homebrew/opt/libpq/bin/psql \
     -h localhost -U spell_platform -d spell_platform \
     -f migrations/0004_billing.sql
   ```

8. **E2Eテスト** (`scripts/e2e_phase2.sh` 作成)
   - Checkout URL取得 → 手動決済 → Webhook受信確認
   - 低い `hard_limit_cents` 設定 → `/v1/cast` で 402 確認
   - `/metrics` カウンタ反映確認
   - 回帰: API Keys/Rate Limit が正常動作

---

## 実装ガイドライン

### 制約
- **既存APIの破壊禁止**: レスポンス形式変更NG、キー名変更NG
- **後方互換性**: `casts.cost_cents` は NULL許可（既存レコード対応）
- **優先順位**: ハード予算 > レート制限 > その他
- **フェイルセーフ**: Billing障害時も `/v1/cast` は動作継続

### コード規約
- エラーハンドリング: `anyhow::Error` → `actix_web::Error` 変換
- トランザクション: `sqlx::Transaction` で書き込み整合性保証
- ログレベル: `log::info!` (正常), `log::error!` (失敗), `log::debug!` (詳細)

### Webhook署名検証（重要）
```rust
// HMAC-SHA256 検証
use hmac::{Hmac, Mac};
use sha2::Sha256;

fn verify_stripe_signature(
    payload: &[u8],
    signature: &str,
    secret: &str,
) -> Result<(), anyhow::Error> {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())?;
    mac.update(payload);
    let expected = hex::encode(mac.finalize().into_bytes());

    if signature != expected {
        anyhow::bail!("Invalid signature");
    }
    Ok(())
}
```

---

## 既存コード参照

### 認証パターン
```rust
// src/routes/keys.rs:29-34 参照
let user_id = {
    let ext = req.extensions();
    ext.get::<User>()
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("User not authenticated"))?
        .id
};
```

### DB操作パターン
```rust
// src/routes/cast.rs:42-53 参照
sqlx::query(
    r#"
    INSERT INTO table (col1, col2) VALUES ($1, $2)
    "#,
)
.bind(&value1)
.bind(&value2)
.execute(&state.db)
.await?;
```

### エラーレスポンス
```rust
// HTTP 402 例
Err(actix_web::error::ErrorPaymentRequired(
    serde_json::to_string(&BudgetExceededError::new(
        "monthly".to_string(),
        1000,
        1012,
    ))?
))
```

---

## トラブルシューティング

### コンパイルエラー
- `RefCell already borrowed` → スコープで即座にドロップ（Phase 1 の `rate_limit.rs:76` 参照）
- 型不一致 → `EitherBody` 使用（`rate_limit.rs:31,56` 参照）

### Stripe Webhook テスト
```bash
# Stripe CLI でローカルテスト
stripe listen --forward-to localhost:8080/webhooks/stripe
stripe trigger checkout.session.completed
```

### マイグレーションロールバック
```sql
-- 緊急時のみ
DROP TABLE IF EXISTS budgets CASCADE;
DROP TABLE IF EXISTS usage_counters CASCADE;
DROP TABLE IF EXISTS billing_accounts CASCADE;
ALTER TABLE casts DROP COLUMN IF EXISTS cost_cents;
```

---

## 受け入れ基準

- [ ] Checkout URL が正常に返却される
- [ ] Webhook 署名検証が動作する（不正署名で 400）
- [ ] 決済完了後に `billing_accounts.status='active'` に更新
- [ ] 予算超過で `/v1/cast` が **必ず** 402 を返す
- [ ] `/metrics` に 429, 402, Webhook処理結果が反映される
- [ ] API Keys/Rate Limit の回帰テスト通過
- [ ] README.md にエラーコード・使い方を追記

---

## 次回セッション開始プロンプト

以下をそのままClaude Codeに貼り付けてください：

```
Phase 2（課金・予算・メトリクス）の実装を再開します。

プロジェクト: /Users/koichinishizuka/spell-platform
GitHub: https://github.com/NishizukaKoichi/spell-platform

Phase 1完了済み（本番稼働中）:
- API Keys（作成/一覧/削除）✅
- レート制限（Redis、60req/分）✅
- 認証（APIキー + セッション）✅

Phase 2準備完了:
- マイグレーション: migrations/0004_billing.sql ✅
- モデル: src/models/billing.rs ✅
- 依存関係: async-stripe, prometheus ✅

次のタスク:
1. src/services/stripe.rs 実装（Checkout + Webhook検証）
2. src/routes/billing.rs 実装（/v1/billing/checkout, /webhooks/stripe）
3. src/routes/budgets.rs 実装（CRUD）
4. src/routes/cast.rs に予算チェック追加（402エラー）
5. src/middleware/metrics.rs + /metrics 実装
6. ENV検証（STRIPE_SECRET_KEY等）
7. デプロイ + マイグレーション実行
8. E2Eテスト（Checkout→Webhook→402→metrics）

制約:
- 既存API破壊禁止
- 優先順位: ハード予算 > レート制限
- Billing障害時も /v1/cast は動作継続

詳細は HANDOFF_PHASE2.md 参照。
順番にタスクを実行し、完了ごとにコミットしてください。
```

---

## 参考リンク

- Stripe API Docs: https://stripe.com/docs/api
- Stripe Webhooks: https://stripe.com/docs/webhooks
- async-stripe Crate: https://docs.rs/async-stripe
- Prometheus Text Format: https://prometheus.io/docs/instrumenting/exposition_formats/

---

生成日時: 2025-10-10
前回コミット: 11b5ea3
次回作業者: 新規セッションのClaude Code
