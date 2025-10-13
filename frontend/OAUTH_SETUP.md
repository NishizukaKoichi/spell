# OAuth & Billing Setup Guide

## GitHub OAuth Configuration

### 1. Create GitHub OAuth App

1. Go to https://github.com/settings/developers
2. Click "New OAuth App"
3. Fill in:
   - **Application name**: Spell
   - **Homepage URL**: `https://magicspell.io`
   - **Authorization callback URL**: `https://magicspell.io/api/auth/callback/github`
4. Click "Register application"
5. Save the **Client ID** and generate a **Client Secret**

### 2. Add Environment Variables

```bash
# Add GitHub OAuth credentials
echo "your_github_client_id" | vercel env add GITHUB_CLIENT_ID production
echo "your_github_client_secret" | vercel env add GITHUB_CLIENT_SECRET production

# Enable NextAuth trust host (required for production)
echo "1" | vercel env add NEXTAUTH_TRUST_HOST production

# Redeploy
vercel --prod
```

### 3. Add Preview Environment Support

For preview deployments, add the wildcard callback URL to GitHub:
- `https://*.vercel.app/api/auth/callback/github`

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
- `NEXT_PUBLIC_API_BASE` ✅ (already set)
- `NEXTAUTH_URL` ✅ (already set)
- `NEXTAUTH_SECRET` ✅ (already set)
- `NEXTAUTH_TRUST_HOST` ⏳ (add when enabling OAuth)
- `GITHUB_CLIENT_ID` ⏳ (add when ready)
- `GITHUB_CLIENT_SECRET` ⏳ (add when ready)
- `STRIPE_PUBLISHABLE_KEY` ⏳ (add when ready)

### Fly.io (Backend)
- `DATABASE_URL` ✅ (already set)
- `REDIS_URL` ✅ (already set)
- `STRIPE_SECRET_KEY` ⏳ (add when ready)
- `WEBHOOK_SIGNING_SECRET` ⏳ (add when ready)

## Security Notes

- Never commit secrets to git
- Rotate secrets regularly (every 90 days)
- Use different keys for development and production
- Monitor webhook signatures to prevent replay attacks
- Keep Stripe webhook signing secret secure

## Troubleshooting

### OAuth Redirect Mismatch
- Verify callback URL exactly matches in GitHub settings
- Check `NEXTAUTH_URL` is set correctly
- Ensure `NEXTAUTH_TRUST_HOST=1` is set

### Stripe Webhook Failures
- Check webhook signing secret is correct
- Verify endpoint is publicly accessible
- Check Stripe dashboard for webhook attempt logs
- Verify API is handling webhook POST requests

### CORS Issues with Stripe
- Ensure `https://m.stripe.network` is in CSP `connect-src`
- Check browser console for blocked requests
- Verify CORS headers on API responses
