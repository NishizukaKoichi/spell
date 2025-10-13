# OAuth & Billing Setup Guide

## GitHub OAuth Configuration

### 1. Create GitHub OAuth App

1. Go to https://github.com/settings/developers
2. Click "New OAuth App"
3. Fill in:
   - **Application name**: Spell
   - **Homepage URL**: `https://magicspell.io`
   - **Authorization callback URL**: `https://api.magicspell.io/auth/callback`

   ⚠️ **Important**: The callback URL must point to the **backend API** (api.magicspell.io), not the frontend.
   This is because the Rust backend handles OAuth authentication directly, not NextAuth.

4. Click "Register application"
5. Save the **Client ID** and generate a **Client Secret**
6. Add these secrets to the backend via Fly.io:
   ```bash
   flyctl secrets set GITHUB_CLIENT_ID=your_client_id -a spell-platform
   flyctl secrets set GITHUB_CLIENT_SECRET=your_client_secret -a spell-platform
   ```

### 2. Verify Backend Configuration

Check that the backend has the correct environment variables:

```bash
# Check current secrets on Fly.io
flyctl secrets list -a spell-platform

# You should see:
# - GITHUB_CLIENT_ID
# - GITHUB_CLIENT_SECRET
# - SESSION_SECRET (for cookie signing)
# - DATABASE_URL
# - REDIS_URL
```

### 3. Test OAuth Flow

1. Visit https://magicspell.io/login
2. Click "Sign in with GitHub"
3. You should be redirected to:
   - `https://api.magicspell.io/auth/github` (backend initiates OAuth)
   - GitHub authorization page
   - `https://api.magicspell.io/auth/callback` (backend receives callback)
   - `https://magicspell.io/dashboard` (frontend after successful auth)

⚠️ **Common Issues:**
- If stuck at "You are being redirected to the authorized application", check that the GitHub callback URL is **exactly** `https://api.magicspell.io/auth/callback`
- Verify backend logs: `flyctl logs -a spell-platform`

## Stripe Integration

### 1. Get Stripe Keys

1. Go to https://dashboard.stripe.com/apikeys
2. Copy your **Publishable key** (starts with `pk_live_`)
3. Create and copy a **Secret key** (starts with `sk_live_`)

### 2. Add Environment Variables

```bash
# Frontend (Publishable Key)
echo "pk_live_your_publishable_key" | vercel env add STRIPE_PUBLISHABLE_KEY production

# Backend (Secret Key via Fly.io)
flyctl secrets set STRIPE_SECRET_KEY=sk_live_your_secret_key -a spell-platform
```

### 3. Configure Webhook

1. Go to https://dashboard.stripe.com/webhooks
2. Click "Add endpoint"
3. Endpoint URL: `https://api.magicspell.io/v1/webhooks/stripe`
4. Select events:
   - `checkout.session.completed`
   - `customer.subscription.created`
   - `customer.subscription.updated`
   - `customer.subscription.deleted`
   - `invoice.payment_succeeded`
   - `invoice.payment_failed`
   - `payment_intent.succeeded`
   - `payment_intent.payment_failed`
5. Copy the **Signing secret** (starts with `whsec_`)
6. Add to Fly.io:
   ```bash
   flyctl secrets set WEBHOOK_SIGNING_SECRET=whsec_your_signing_secret -a spell-platform
   ```

### 4. Update CSP for Stripe

If using Stripe Checkout, update `connect-src` in `next.config.ts`:

```typescript
"connect-src 'self' https://api.magicspell.io https://api.stripe.com https://m.stripe.network",
```

## Verification

### Test GitHub OAuth

1. Navigate to https://magicspell.io/login
2. Click "Sign in with GitHub"
3. Authorize the application
4. Verify redirect back to dashboard

### Test Stripe Webhook

```bash
# Install Stripe CLI
brew install stripe/stripe-brew/stripe

# Login
stripe login

# Test webhook
stripe trigger payment_intent.succeeded \
  --webhook-endpoint https://api.magicspell.io/v1/webhooks/stripe
```

Check Fly.io logs:
```bash
flyctl logs -a spell-platform | grep webhook
```

## Environment Variables Summary

### Vercel (Frontend)
- `NEXT_PUBLIC_API_BASE` ✅ (set to `https://api.magicspell.io`)
- `NEXT_PUBLIC_STRIPE_PUBLISHABLE_KEY` ⏳ (add when ready)

⚠️ **Note**: `NEXTAUTH_*` variables are NOT needed because this project doesn't use NextAuth.
OAuth is handled by the Rust backend API.

### Fly.io (Backend) - spell-platform
- `DATABASE_URL` ✅ (already set)
- `REDIS_URL` ✅ (already set)
- `SESSION_SECRET` ✅ (already set)
- `GITHUB_CLIENT_ID` ⏳ (add for OAuth)
- `GITHUB_CLIENT_SECRET` ⏳ (add for OAuth)
- `STRIPE_SECRET_KEY` ⏳ (add when ready)
- `STRIPE_WEBHOOK_SECRET` ⏳ (add when ready)

## Security Notes

- Never commit secrets to git
- Rotate secrets regularly (every 90 days)
- Use different keys for development and production
- Monitor webhook signatures to prevent replay attacks
- Keep Stripe webhook signing secret secure

## Troubleshooting

### OAuth Redirect Mismatch / Stuck at "You are being redirected"
- **Most Common**: GitHub callback URL is wrong. It MUST be `https://api.magicspell.io/auth/callback` (backend, not frontend)
- Check backend logs: `flyctl logs -a spell-platform | grep auth`
- Verify `GITHUB_CLIENT_ID` and `GITHUB_CLIENT_SECRET` are set in Fly.io
- Test backend endpoint directly: `curl -I https://api.magicspell.io/auth/github` (should return 302 redirect to GitHub)

### Stripe Webhook Failures
- Check webhook signing secret is correct
- Verify endpoint is publicly accessible
- Check Stripe dashboard for webhook attempt logs
- Verify API is handling webhook POST requests

### CORS Issues with Stripe
- Ensure `https://m.stripe.network` is in CSP `connect-src`
- Check browser console for blocked requests
- Verify CORS headers on API responses
