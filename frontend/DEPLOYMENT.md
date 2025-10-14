# Spell Platform Frontend - Deployment Guide

This guide explains how to deploy the Caster Web UI (Next.js) to Vercel with Cloudflare Workers as a reverse proxy to the Fly.io backend.

## Architecture

The platform uses a **unified single-domain architecture**:

```
magicspell.io/api/*  → Fly.io (Rust backend)
magicspell.io/auth/* → Fly.io (OAuth endpoints)
magicspell.io/*      → Vercel (Next.js frontend)
```

This eliminates cross-domain cookie issues and simplifies CORS configuration.

## Prerequisites

- Cloudflare account (for reverse proxy and DNS)
- Vercel account (for frontend hosting)
- Domain `magicspell.io` registered and managed in Cloudflare
- Fly.io API already deployed at `spell-platform.fly.dev`
- GitHub OAuth App configured
- Stripe account set up

## 1. Vercel Setup

### 1.1 Link Project to Vercel

```bash
cd frontend
vercel link --project magicspell-caster
```

### 1.2 Add Domain

```bash
vercel domains add magicspell.io
```

Follow Vercel's instructions to verify domain ownership.

### 1.3 Configure DNS (via Cloudflare)

**Important**: The domain must be managed in Cloudflare (not Vercel) for the reverse proxy to work.

1. Update your domain registrar's NS records to Cloudflare's nameservers (provided in Cloudflare dashboard)
2. In Vercel dashboard, set the nameservers to Cloudflare's nameservers
3. The Cloudflare Worker will handle routing - no A/CNAME records needed in Vercel

### 1.4 Set Environment Variables

```bash
# Stripe (for payment processing)
vercel env add STRIPE_PUBLISHABLE_KEY
# → Enter: pk_live_...

vercel env add STRIPE_SECRET_KEY
# → Enter: sk_live_...

# No NEXT_PUBLIC_API_BASE needed - the frontend uses relative paths
# which are routed through the Cloudflare Worker reverse proxy
```

## 2. Cloudflare Workers Setup

### 2.1 Deploy the Reverse Proxy Worker

```bash
cd cloudflare-proxy
npm install
npx wrangler login
npm run deploy
```

### 2.2 Configure Workers Route in Cloudflare Dashboard

1. Go to Cloudflare Dashboard → Workers & Pages
2. Select your worker: `spell-platform-proxy`
3. Go to Settings → Triggers
4. Add a route: `magicspell.io/*`
5. Ensure the route is assigned to the correct zone

### 2.3 Verify Worker Configuration

Check that `wrangler.toml` has the correct origins:

```toml
[vars]
API_ORIGIN = "https://spell-platform.fly.dev"
FRONTEND_ORIGIN = "https://spell-caster-magicspell.vercel.app"
```

### 2.4 Test Routing

```bash
# Test API routing
curl -I https://magicspell.io/api/healthz

# Test frontend routing
curl -I https://magicspell.io/
```

See `cloudflare-proxy/README.md` for detailed configuration.

## 3. GitHub OAuth Configuration

Update your GitHub OAuth App settings:

- **Homepage URL**: `https://magicspell.io`
- **Authorization callback URL**: `https://magicspell.io/auth/github/callback`

Note: The callback URL is now on the unified domain and will be routed through the Cloudflare Worker to Fly.io.

## 4. Stripe Webhook Configuration

1. Go to [Stripe Dashboard → Webhooks](https://dashboard.stripe.com/webhooks)
2. Add endpoint: `https://magicspell.io/v1/webhooks/stripe`
3. Select events:
   - `checkout.session.completed`
   - `customer.subscription.created`
   - `customer.subscription.updated`
   - `customer.subscription.deleted`
   - `invoice.payment_succeeded`
   - `invoice.payment_failed`
   - `payment_intent.succeeded`
   - `payment_intent.payment_failed`
4. Copy the webhook signing secret and set it in Fly.io:
   ```bash
   flyctl secrets set WEBHOOK_SIGNING_SECRET=whsec_... -a spell-platform
   ```

## 5. Deploy to Production

### 5.1 Preview Deployment

```bash
vercel --prod=false
```

Test the preview URL before production deployment.

### 5.2 Production Deployment

```bash
vercel --prod
```

Your site will be live at `https://magicspell.io`

## 6. DNS Hardening (Optional but Recommended)

### 6.1 Add CAA Records

```bash
vercel dns add magicspell.io @ CAA "0 issue \"letsencrypt.org\""
vercel dns add magicspell.io @ CAA "0 issuewild \"letsencrypt.org\""
```

### 6.2 Enable IPv6

Vercel automatically provides IPv6 (AAAA records) for your domain.

### 6.3 HSTS Preload (Post-Production)

After confirming stable production:

1. Ensure HSTS header includes `preload` directive
2. Submit to [hstspreload.org](https://hstspreload.org/)

## 7. Verification

### 7.1 Test Worker Routing to Backend

```bash
# Test health endpoint (should route to Fly.io)
curl -i https://magicspell.io/api/healthz

# Test metrics endpoint
curl https://magicspell.io/metrics
```

Expected: 200 OK responses from the Rust backend

### 7.2 Test Worker Routing to Frontend

```bash
# Test homepage (should route to Vercel)
curl -I https://magicspell.io/

# Test login page
curl -I https://magicspell.io/login
```

Expected: 200 OK responses from Next.js

### 7.3 Test Security Headers

```bash
curl -I https://magicspell.io
```

Verify presence of:
- `Strict-Transport-Security`
- `Content-Security-Policy`
- `X-Frame-Options`
- `Referrer-Policy`
- `Permissions-Policy`

### 7.4 Test Stripe Webhook

```bash
stripe trigger payment_intent.succeeded \
  --webhook-endpoint https://magicspell.io/v1/webhooks/stripe
```

Check Fly.io logs for successful processing:
```bash
flyctl logs -a spell-platform
```

The webhook request will be routed through the Cloudflare Worker to Fly.io.

## 8. Monitoring

### 8.1 Vercel Dashboard

- Monitor deployments at https://vercel.com/dashboard
- Check Analytics for traffic patterns
- Review Error tracking

### 8.2 Fly.io Dashboard

- Monitor API health at https://fly.io/dashboard
- Check machine status
- Review metrics and logs

## 9. Rollback Procedure

### 9.1 Vercel Rollback

```bash
vercel rollback
```

Or via dashboard: Deployments → Select previous deployment → Promote to Production

### 9.2 Fly.io Rollback

```bash
flyctl releases -a spell-platform
flyctl releases rollback -a spell-platform <version>
```

## 10. Troubleshooting

### Domain not resolving

- Check NS records point to Cloudflare (not Vercel)
- Verify DNS propagation: `dig NS magicspell.io`
- Ensure Cloudflare Worker route is active: `magicspell.io/*`
- Allow up to 48 hours for global DNS propagation

### Routing errors (404 on /api/* or /auth/*)

- Verify Worker is deployed: Check Cloudflare Dashboard → Workers & Pages
- Verify Worker route is configured: `magicspell.io/*` → `spell-platform-proxy`
- Check Worker logs in Cloudflare Dashboard
- Test Worker directly: `curl -I https://magicspell.io/api/healthz`

### Cookie/authentication issues

- Ensure backend Cookie settings are: `SameSite=Lax`, `Domain=.magicspell.io`
- Verify GitHub OAuth callback URL is: `https://magicspell.io/auth/github/callback`
- Check browser DevTools → Application → Cookies
- Clear Cloudflare cache if cookies seem stale

### Environment variables not working

- Verify they're set in Vercel dashboard under Project → Settings → Environment Variables
- Ensure production variables are assigned to "Production" environment
- Redeploy after changing environment variables

## 11. Security Best Practices

- ✅ Always use HTTPS (enforced by Vercel and Fly.io)
- ✅ Regularly rotate secrets (especially `NEXTAUTH_SECRET`)
- ✅ Monitor Stripe webhook logs for suspicious activity
- ✅ Keep dependencies updated (`npm audit` regularly)
- ✅ Review Vercel Security settings periodically
- ✅ Enable 2FA on Vercel and Fly.io accounts
- ✅ Restrict API keys to minimum required permissions
- ✅ Use separate keys for development and production

## 12. Post-Deployment Checklist

- [ ] NS records point to Cloudflare nameservers
- [ ] Cloudflare Worker is deployed and active
- [ ] Worker route `magicspell.io/*` is configured
- [ ] Frontend routes correctly to Vercel
- [ ] API routes (`/api/*`, `/auth/*`) correctly route to Fly.io
- [ ] GitHub OAuth flow works end-to-end
- [ ] Cookies are set with correct domain (`.magicspell.io`)
- [ ] Stripe checkout works
- [ ] Stripe webhooks are processed
- [ ] Security headers are present
- [ ] CSP doesn't block required resources
- [ ] SSL certificates are valid (Cloudflare manages this)
- [ ] Monitoring and alerts are configured
- [ ] Backups are enabled (database)
- [ ] Documentation is updated
