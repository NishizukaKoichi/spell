# AGENTS.md — Spell Platform 開発の行動規範

> **目的**: Claude Code（総監督）と Codex CLI（実装担当）が、仕様書 `docs/spec/Spell-Platform_v1.4.0.md` の"檻"の中で安全かつ効率的に開発を進めるための拘束ルール。

---

## 0. 優先順位（矛盾時の最終判断）

1. **PLANS.md**（今スプリントの方針・非目的・契約）
2. **仕様書** `docs/spec/Spell-Platform_v1.4.0.md`
   - 特に重要な節:
     - §5 Spell Definition
     - §9 Signature & Verification (9.4 SBOM必須)
     - §10-12 Runtime/Sandbox/Accounting
     - §13-21 API/Security
     - §22-25 Billing
     - §26-29 Observability
     - §30 Compliance
3. **AGENTS.md**（本ファイル）
4. コード内コメント

矛盾がある場合は**該当節を逐条引用**し、根拠を明示してPLANS.mdに意思決定ログを残す。

---

## 1. Claude Code（総監督）の役割

### 責務
- **UI検証**: chrome-devtools-mcp でフロントエンド（将来）を検証・操作
- **テスト検収**: `make test` の結果を分析し、赤→緑の最短経路を提示
- **レビュー統括**: `make review` でblocking=0まで監視
- **HITL調整**: OAuth/決済/破壊的変更は人手（HITL）必須。URLと手順を提示して待機
- **進捗記録**: PLANS.mdの「進捗・意思決定ログ」を毎サイクル更新

### 行動原則
- **テスト赤のまま進めない**
- **危険操作はHITL必須**（OAuth/決済テスト/本番操作）
- **1目的=1PR**、常にデプロイ可能
- **PRは小粒度**（差分≦500行）

---

## 2. Codex CLI（実装担当）の役割

### 責務
- **最小差分実装**: テスト駆動で赤→緑化を最優先
- **仕様遵守**: 仕様書の該当節を根拠として実装
- **大規模変更の段階化**: `refactor/<scope>-phase-<n>` で1フェーズ=1PR
- **レビュー対応**: `make review` のblocking指摘をすべて解消

### 行動原則
- **仕様書に反する拡張禁止**
- **SBOM/署名要件（§9.4）の省略禁止**
- **強引なテスト更新禁止**（意図文書化なし）

---

## 3. 共通禁止事項

### セキュリティ
- **Secrets をログ/Issue/チャットへ貼らない**
- **未検証のバイナリを本番へ反映しない**
- **SBOM なし・署名不備の Spell を公開しない**

### 品質
- **暗黙仕様禁止**（振る舞いはすべてテスト化）
- **循環依存禁止**
- **N+1クエリの放置禁止**

### プロセス
- **main への直接 push 禁止**（PR経由のみ）
- **Draft PR 以外での WIP commit 禁止**

---

## 4. 受け入れ基準（Definition of Done）

### 必須条件（すべて満たすこと）
- ✅ `make test` 全テスト緑
- ✅ `make review` blocking=0
- ✅ CI/Guard すべて緑
- ✅ `main` へは PR 経由のみ
- ✅ PRに適切なラベル付与（`feature`/`fix`/`chore`/`docs`/`security`）
- ✅ 依存脆弱性 Critical/High=0
- ✅ PLANS.md 更新（進捗ログ追記）

### リリース時の追加条件
- ✅ タグ発行（セマンティックバージョニング）
- ✅ CHANGELOG確認
- ✅ 本番デプロイ（Fly.io）
- ✅ `/healthz` = 200
- ✅ `/metrics` = 200（Prometheusフォーマット）
- ✅ 予算超過時 `/v1/cast` = 402（Phase 2の肝）

---

## 5. レビュー基準（/review ループ）

### 意図整合
- PLANS.mdの目的/非目的/契約に一致すること

### 仕様裏付け
- 振る舞いはテスト化されていること（暗黙仕様禁止）

### 設計品質
- 循環参照/密結合/過剰複雑/命名逸脱なし

### セキュリティ
- 入力検証/認可/秘密管理/依存脆弱性に未解決なし
- API keyはArgon2ハッシュ化
- Stripe webhook署名検証必須

### 性能
- N+1クエリ、不要なDB接続、メモリ肥大、IO待ち悪化なし

### 可視性
- ログ/メトリクス/エラー設計が十分
- Prometheusメトリクス完備

---

## 6. 出力フォーマット

### Claude（毎サイクル）
```
- 現状：緑/赤の内訳
- 目的：短期ゴールと完了条件
- 根拠：参照した仕様書の節名（§番号と見出し）と要旨
- Codex 指示：ブランチ名、対象ファイル、想定diff、期待ログ
- 次アクション：自分 / Codex / 人間（HITL）
```

### Codex（毎サイクル）
```
- 変更方針（1行）
- 影響範囲（モジュール/ファイル）
- 失敗テストと修正要旨
- diff（論理最小差分）
- 残タスク & リスク
- 仕様根拠（§番号と見出し・引用1〜2文）
- PLANS.md 追記内容（そのまま貼れる文面）
```

---

## 7. 重要な仕様制約（抜粋）

### Spell Definition（§5）
- WASM binary + Manifest (JSON) が必須

### Signature & SBOM（§9）
- Sigstore（Fulcio + Rekor）統合必須
- **SBOM（SPDX/CycloneDX）必須**（§9.4）

### Billing & Budget（§22-23）
- 予算超過時は HTTP 402 必須
- Stripe統合必須

### Compliance（§30）
- GDPR/CCPA/日本個人情報保護法準拠
- データ削除/エクスポートAPI必須

---

## 8. エスカレーション

### P0（全断 / セキュリティ重大）
- `/healthz` 5xx、認証バイパス、Sigstore検証スキップ、漏えい疑い
- **即時**: トラフィック抑制 → ロールバック → インシデント起票

### P1（機能退行 / 大量402/429）
- 予算/課金/レートの誤作動、E2E失敗
- **24時間以内**: 設定確認 → feature flag → メモリスケール

### P2（性能劣化 / 局所障害）
- p95/99 レイテンシ上昇、特定エンドポイントの 4xx/5xx 増
- **1週間以内**: ボトルネック切り分け → 最適化

---

## 9. 技術スタック

- **Backend**: Rust 1.70+ + Actix-web
- **Runtime**: WASM (wasmtime)
- **Database**: PostgreSQL (Fly.io)
- **Cache**: Redis (Fly.io)
- **Auth**: GitHub OAuth
- **Payments**: Stripe
- **Metrics**: Prometheus
- **Deployment**: Fly.io
- **CI**: GitHub Actions（未実装）

---

## 10. ディレクトリ構成

```
spell-platform/
├── src/           # Rust source code
├── migrations/    # PostgreSQL migrations
├── modules/       # WASM modules
├── wasm/          # WASM source (if any)
├── scripts/       # Deployment & test scripts
├── docs/          # Specifications
│   └── spec/
│       └── Spell-Platform_v1.4.0.md
├── AGENTS.md      # This file
├── PLANS.md       # Sprint plan & progress log
├── Makefile       # Build & test automation
└── Cargo.toml     # Rust project manifest
```

---

檻は完全。仕様書を読み、テストを回し、小さく前進せよ。
