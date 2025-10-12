# PLANS.md — Spell Platform 生きた設計書

> **目的**: Spell Platform を"道に迷わず"完成させるための行動計画・進捗ログ・意思決定の単一情報源。
> **読者**: AI（Claude/Codex）と人間（設計者/レビュア/運用担当）。

---

## 0. スコープ宣言（変更時はここを必ず更新）

### 今スプリントの目標

**Phase 2 完成 → Phase 3 準備 → v1.0 リリース**

1. **Phase 2 残タスク完了**
   - E2Eテスト実装（自動化）
   - CI/CD パイプライン構築（GitHub Actions）
   - ブランチ保護設定
   - Release Drafter 設定

2. **Phase 3 準備（仕様書§30準拠）**
   - GDPR/CCPA/個人情報保護法 対応エンドポイント実装
   - データ削除/エクスポートAPI
   - SBOM生成＆検証（§9.4必須）
   - Sigstore統合（Fulcio + Rekor）

3. **v1.0 品質基準達成**
   - すべてのAPI仕様（§13-21）完全実装
   - セキュリティ監査通過
   - 性能目標達成（p90 < 500ms）

### 非目的（触らない領域）

- ❌ 新しいフロントエンド（Phase 4以降）
- ❌ マルチリージョン展開（Phase 3後半以降）
- ❌ SOC 2認証（Phase 4以降）

### 成功指標（Definition of Done）

**必須条件（すべて満たすこと）**
- ✅ `make test` 全テスト緑
- ✅ `make review` blocking=0
- ✅ CI/Guard すべて緑
- ✅ `main` へは PR 経由のみ
- ✅ PRに適切なラベル付与（`feature`/`fix`/`chore`/`docs`/`security`）
- ✅ 依存脆弱性 Critical/High=0
- ✅ PLANS.md 更新（進捗ログ追記）
- ✅ `/healthz` = 200
- ✅ `/metrics` = 200（Prometheusフォーマット）
- ✅ 予算超過時 `/v1/cast` = 402（Phase 2の肝）

**リリース時の追加条件**
- ✅ タグ発行（セマンティックバージョニング）
- ✅ CHANGELOG確認
- ✅ 本番デプロイ（Fly.io）
- ✅ E2Eテスト全パス
- ✅ 監査ログ（Ledger）連番欠落なし

---

## 1. 仕様の骨子（契約・インタフェース・期待値）

### 外部契約（API仕様書§13-21より）

**認証（§14）**
- `GET /auth/github` - GitHub OAuth開始
- `GET /auth/callback` - OAuth callback
- Session token（Bearer）認証必須

**Health & Metrics（§26-29）**
- `GET /healthz` → 200 `{"status":"ok","version":"x.x.x"}`
- `GET /metrics` → 200（Prometheus形式、認証不要）

**API Keys（§19）**
- `POST /v1/keys` → API key作成（Argon2ハッシュ）
- `GET /v1/keys` → 一覧
- `DELETE /v1/keys/:prefix` → 削除

**Spells（§10-12）**
- `POST /v1/cast` → WASM実行（予算enforc前提）

**Billing（§22-25）**
- `POST /v1/billing/checkout` → Stripe Checkout session作成
- `POST /webhooks/stripe` → Stripe webhook（署名検証必須）

**Budgets（§23）**
- `GET /v1/budgets` → 予算取得
- `POST /v1/budgets` → 予算作成/更新
- `PUT /v1/budgets` → 予算更新
- `DELETE /v1/budgets` → 予算削除
- `GET /v1/budgets/usage` → 使用量取得

### 性能・可用性の最低ライン（§1.3より）

- 実行レイテンシ p90 < 500ms（MVP）
- API可用性 99.5%（MVP）
- 供給チェーン検証率 100%
- SBOM提出率 80%（MVP）

### セキュリティ契約（§18-21より）

- API keyはArgon2でハッシュ化
- Rate limiting: 60 rpm（認証済み）/ 10 rpm（未認証）
- Stripe webhook署名検証必須
- CORS/CSRF保護（TBD）

---

## 2. 作業分割（ToDo・フェーズ・PR設計）

### フェーズ分割（1フェーズ=1PR、常にデプロイ可能）

#### Phase 2 完成（今スプリント最優先）
1. ✅ Billing統合完了（デプロイ済み）
2. ⏳ E2Eテスト自動化（scripts/e2e_phase2.sh → Rust統合テスト）
3. ⏳ CI/CD構築（GitHub Actions）
4. ⏳ ブランチ保護 + Release Drafter

#### Phase 3 準拠性（次スプリント）
1. ⏳ GDPR/CCPA/日本法 対応エンドポイント
   - `DELETE /v1/users/me` - データ削除
   - `GET /v1/users/me/export` - データエクスポート
2. ⏳ SBOM生成・検証（§9.4必須）
   - cargo-sbom統合
   - SPDX/CycloneDX生成
3. ⏳ Sigstore統合（§9.2）
   - Fulcio署名
   - Rekor透明性ログ

### ToDo（粒度小・優先順位順）

- [ ] Makefileを作成（test/lint/build/deploy target）
- [ ] GitHub Actions CI/CD構築
  - [ ] test job（cargo test）
  - [ ] lint job（cargo clippy）
  - [ ] security job（cargo audit）
  - [ ] deploy job（Fly.io）
- [ ] E2Eテストを Rust統合テストに移植
- [ ] ブランチ保護設定（main）
- [ ] Release Drafter設定
- [ ] データ削除API実装（§30.3 GDPR準拠）
- [ ] データエクスポートAPI実装
- [ ] SBOM生成スクリプト（cargo-sbom）
- [ ] Sigstore署名スクリプト

---

## 3. リスク・ロールバック・ガードレール

### 主要リスク

1. **SBOM/Sigstore統合の複雑さ**
   - 外部ツール依存（cosign, rekor-cli）
   - CI/CDへの統合コスト

2. **GDPR/CCPA完全準拠の法的要件**
   - データ削除の完全性（Foreign key cascadeで漏れなし）
   - 72時間以内の通知義務

3. **性能劣化リスク**
   - Sigstore検証のレイテンシ追加
   - SBOM生成のビルド時間増加

### ロールバック手順

- 各フェーズPRに `revert` コマンドで戻せるよう差分を自立化
- Fly.io Image指名デプロイでロールバック
  ```bash
  flyctl deploy -a spell-platform --image <IMAGE_REF>
  ```

### ガードレール

- PR差分≦500行、コミットは論理最小単位
- すべての変更はテスト/レビューを**同じサイクル**で緑化
- Secrets をログ/Issue/チャットへ貼らない
- 本番デプロイ前に `/healthz` / `/metrics` 確認必須

---

## 4. 実行ループ（AIの手順）

1. **初回**：`make test` → 失敗テストを一覧化 → **赤→緑**の最短プランをここに書く
2. **レビュー**：`make review`→blockingを0にするまで修正→合格したら次フェーズへ
3. **ログ**：下の「進捗・意思決定ログ」に**毎サイクル**追記

---

## 5. 進捗・意思決定ログ（AIは必ず更新）

### 2025-10-12 13:00 - Phase 2 デプロイ完了（前任より引き継ぎ）

- **完了項目**：
  - ✅ Billing統合（Stripe Checkout + Webhook）
  - ✅ 予算管理（hard/soft limits）
  - ✅ 予算enforc（HTTP 402）
  - ✅ 使用量トラッキング
  - ✅ Prometheusメトリクス
  - ✅ 本番デプロイ（Image: deployment-01K7B0S2NHBQFAT261JA6BZMBY）

- **確認済みエンドポイント**：
  - `GET /healthz` = 200
  - `GET /metrics` = 200
  - `GET /v1/budgets` = 401（未認証時、正常）
  - `POST /v1/billing/checkout` = 401（未認証時、正常）

- **残タスク**：
  - ⏳ E2Eテスト実行（手動、scripts/e2e_phase2.sh）
  - ⏳ Stripe secrets設定（本格運用時）
  - ⏳ Monitoring設定（Prometheus/Grafana）

### 2025-10-12 14:30 - 檻フレームワーク適用開始（Claude × Codex 協調開発）

- **判断**：ai-cage-driven-devの檻フレームワークをspell-platformに適用
- **根拠**：運用引き継ぎプロンプトの要件（再現性・証跡・安全性）を満たすため
- **実装**：
  - ✅ AGENTS.md作成（行動規範・優先順位・DoD明文化）
  - ✅ PLANS.md作成（スコープ・仕様骨子・作業分割・進捗ログ）
  - ⏳ Makefile作成（次タスク）

- **次アクション**：
  - Makefile作成（test/lint/build/deploy）
  - CI/CD構築（GitHub Actions）
  - E2Eテスト自動化

（以降、毎サイクル追記）

---

## 6. 仕様書参照マップ（重要節の要旨）

### §5 Spell Definition
- WASM binary + Manifest (JSON) が必須

### §9.4 SBOM (Software Bill of Materials) - REQUIRED
- **すべてのSpellはSBOM必須**（SPDX or CycloneDX）
- CVE脆弱性スキャン必須

### §13-21 API Specification
- 全エンドポイント仕様（認証/Spells/Billing/Budgets）

### §22-25 Billing & Budget Management
- 予算超過時 HTTP 402 必須
- Stripe統合必須

### §26-29 Observability
- Ledger（監査ログ）連番欠落なし
- Prometheusメトリクス完備

### §30 Compliance
- GDPR/CCPA/日本個人情報保護法準拠
- データ削除/エクスポートAPI必須
- 72時間以内の通知義務（EU/日本）

---

檻は整った。仕様書を読み、小さく前進せよ。
