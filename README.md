# Spell Platform

A C2C (Creator-to-Consumer) platform for distributing and executing WASM-based workflows via API.

## Status

**Current Phase**: Phase 2 (Billing & Budgets) - In Progress

## Architecture

- **Backend**: Rust + Actix-web (Fly.io)
- **Frontend**: Next.js 14 (Vercel)
- **Reverse Proxy**: Cloudflare Workers (unified domain)
- **Runtime**: WASM (wasmtime)
- **Database**: PostgreSQL (Fly.io)
- **Cache**: Redis (Fly.io)
- **Auth**: GitHub OAuth
- **Payments**: Stripe
- **Metrics**: Prometheus

### Domain Architecture

The platform uses a **single-domain architecture** with Cloudflare Workers as a reverse proxy:

```
magicspell.io/api/*  → Fly.io (Rust backend)
magicspell.io/auth/* → Fly.io (OAuth endpoints)
magicspell.io/*      → Vercel (Next.js frontend)
```

This architecture eliminates cross-domain cookie issues and simplifies CORS configuration.

## Features

### Phase 1 (Completed)
- ✅ GitHub OAuth authentication
- ✅ Session-based auth with Bearer tokens
- ✅ API key management (create, list, delete)
- ✅ Rate limiting (Redis-based, 60 rpm authenticated / 10 rpm unauthenticated)
- ✅ WASM spell execution (`/v1/cast`)
- ✅ User tracking for all casts

### Phase 2 (Current)
- ✅ Billing integration (Stripe Checkout + Webhooks)
- ✅ Budget management (soft/hard limits, daily/monthly periods)
- ✅ Budget enforcement (HTTP 402 on hard limit)
- ✅ Usage tracking per cast
- ✅ Prometheus metrics (`/metrics`)
- ⏳ Migration application (script ready)
- ⏳ Production deployment

## API Endpoints

### Authentication
- `GET /auth/github` - Initiate GitHub OAuth
- `GET /auth/callback` - OAuth callback

### Health & Metrics
- `GET /healthz` - Health check
- `GET /metrics` - Prometheus metrics (no auth required)

### API Keys
- `POST /v1/keys` - Create API key (authenticated)
- `GET /v1/keys` - List API keys (authenticated)
- `DELETE /v1/keys/:prefix` - Delete API key (authenticated)

### Spells
- `POST /v1/cast` - Execute spell (authenticated, budget enforced)

### Billing
- `POST /v1/billing/checkout` - Create Stripe checkout session (authenticated)
- `POST /webhooks/stripe` - Stripe webhook (no auth, signature verified)

### Budgets
- `GET /v1/budgets` - Get budget (authenticated)
- `POST /v1/budgets` - Create/update budget (authenticated)
- `PUT /v1/budgets` - Update budget (authenticated)
- `DELETE /v1/budgets` - Delete budget (authenticated)
- `GET /v1/budgets/usage` - Get current usage (authenticated)

## Database Schema

### Phase 1 Tables
- `users` - User accounts (GitHub OAuth)
- `sessions` - Session tokens
- `api_keys` - API key hashes
- `casts` - Spell execution records

### Phase 2 Tables
- `billing_accounts` - Stripe customer & subscription info
- `usage_counters` - Usage aggregation per time window
- `budgets` - User-defined spending limits
- `casts.cost_cents` - Cost tracking per cast

## Environment Variables

### Required
- `DATABASE_URL` - PostgreSQL connection string
- `REDIS_URL` - Redis connection string
- `GITHUB_CLIENT_ID` - GitHub OAuth app client ID
- `GITHUB_CLIENT_SECRET` - GitHub OAuth app secret
- `GITHUB_REDIRECT_URI` - OAuth callback URL

### Optional (Phase 2)
- `STRIPE_SECRET_KEY` - Stripe API key (enables billing)
- `STRIPE_WEBHOOK_SECRET` - Stripe webhook signing secret
- `COST_PER_CAST_CENTS` - Cost per spell execution (default: 0)
- `WASM_MODULE_PATH` - Path to WASM modules (default: `./modules`)

## Development

### Prerequisites
- Rust 1.70+
- PostgreSQL
- Redis

### Build
```bash
cargo build --release
```

### Run locally
```bash
# Set environment variables
export DATABASE_URL=postgresql://...
export REDIS_URL=redis://...
export GITHUB_CLIENT_ID=xxx
export GITHUB_CLIENT_SECRET=xxx
export GITHUB_REDIRECT_URI=http://localhost:8080/auth/github/callback

cargo run
```

Server starts on `http://0.0.0.0:8080`

## Deployment

### Fly.io Setup

```bash
# Login
flyctl auth login

# Deploy
flyctl deploy --remote-only

# Set secrets
flyctl secrets set \
  DATABASE_URL=postgresql://... \
  REDIS_URL=redis://... \
  GITHUB_CLIENT_ID=xxx \
  GITHUB_CLIENT_SECRET=xxx \
  GITHUB_REDIRECT_URI=https://magicspell.io/auth/github/callback \
  STRIPE_SECRET_KEY=sk_live_xxx \
  STRIPE_WEBHOOK_SECRET=whsec_xxx \
  COST_PER_CAST_CENTS=1
```

### Apply Migrations

```bash
# Option 1: Via script
./scripts/apply_migrations.sh

# Option 2: Manual via proxy
flyctl proxy 15432:5432 -a spell-platform-db &
psql -h localhost -p 15432 -U spell_platform -d spell_platform -f migrations/0004_billing.sql
```

### Cloudflare Workers Setup

The reverse proxy requires additional setup:

```bash
# 1. Deploy the Worker
cd cloudflare-proxy
npm install
npm run deploy

# 2. Configure DNS in Cloudflare Dashboard
# - Point magicspell.io to the Worker via a Workers Route
# - Set Proxy status to "Proxied" (orange cloud)
# - Update nameservers in your domain registrar to Cloudflare's nameservers

# 3. Verify routing
curl -I https://magicspell.io/api/healthz  # Should route to Fly.io
curl -I https://magicspell.io/            # Should route to Vercel
```

See `cloudflare-proxy/README.md` for detailed instructions.

### Frontend Deployment

The Next.js frontend is deployed on Vercel:

```bash
cd frontend
vercel --prod
```

**Important**: Set the domain nameservers to Cloudflare (not Vercel) to enable the reverse proxy.

## Testing

### Phase 2 E2E Tests

```bash
# Get session token first via OAuth
open https://magicspell.io/auth/github

# Export token
export TOKEN=<your_session_token>

# Run tests
./scripts/e2e_phase2.sh
```

Tests cover:
1. Health check
2. Metrics endpoint
3. Budget CRUD
4. Budget enforcement (402 on limit)
5. Usage tracking
6. Billing checkout (HITL)
7. Metrics validation

## Monitoring

### Prometheus Metrics

```bash
curl https://magicspell.io/metrics
```

Available metrics:
- `spell_cast_total` - Total spell executions
- `spell_cast_failed_total` - Failed executions
- `spell_cast_duration_seconds` - Execution latency histogram
- `spell_rate_limited_total` - Rate limit hits
- `spell_budget_blocked_total` - Budget limit blocks (402)
- `spell_stripe_webhook_total` - Stripe webhook events
- `spell_db_pool_in_use` - Database connection pool usage
- `spell_redis_errors_total` - Redis errors

## Roadmap

### Phase 3 (Future)
- [ ] Multi-region deployment (US, EU, APAC)
- [ ] GDPR/CCPA compliance endpoints
- [ ] Data export functionality
- [ ] SBOM generation & validation
- [ ] Sigstore integration
- [ ] SOC 2 certification

## Security

- API keys hashed with Argon2
- Rate limiting per user/IP
- Budget hard limits enforced pre-execution
- Stripe webhook signature verification
- CORS/CSRF protection (TBD)

## License

Proprietary

## Support

- GitHub Issues: https://github.com/NishizukaKoichi/spell-platform/issues
- Email: koichi@example.com
