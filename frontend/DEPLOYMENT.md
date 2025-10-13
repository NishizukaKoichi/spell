# Spell Platform Frontend - Deployment Guide

This guide explains how to deploy the Caster Web UI (Next.js) to Vercel with the API on Fly.io.

## Prerequisites

- Vercel account
- Domain `magicspell.io` registered
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

### 1.3 Configure DNS (via Vercel)

1. Update your domain registrar's NS records to Vercel's nameservers (provided in Vercel dashboard)
2. Vercel will automatically configure:
   - A/AAAA records for apex (`magicspell.io`)
   - CNAME for www (will redirect to apex via `vercel.json`)

### 1.4 Set Environment Variables

```bash
# Production API endpoint
vercel env add NEXT_PUBLIC_API_BASE
# → Enter: https://api.magicspell.io

# GitHub OAuth
vercel env add GITHUB_CLIENT_ID
# → Enter your production GitHub OAuth Client ID

vercel env add GITHUB_CLIENT_SECRET
# → Enter your production GitHub OAuth Client Secret

# Stripe
vercel env add STRIPE_PUBLISHABLE_KEY
# → Enter: pk_live_...

vercel env add STRIPE_SECRET_KEY
# → Enter: sk_live_...

# NextAuth (if using)
vercel env add NEXTAUTH_URL
# → Enter: https://magicspell.io

vercel env add NEXTAUTH_SECRET
# → Enter: Generated via `openssl rand -base64 32`
```

## 2. Fly.io API Configuration

### 2.1 Create Certificate for API Subdomain

```bash
flyctl certs create api.magicspell.io -a spell-platform
```

### 2.2 Get Fly.io App Hostname

```bash
flyctl status -a spell-platform
```

Note the hostname (e.g., `spell-platform.fly.dev`)

### 2.3 Add DNS CNAME via Vercel

```bash
vercel dns add magicspell.io api CNAME spell-platform.fly.dev
```

### 2.4 Verify Certificate

```bash
flyctl certs show api.magicspell.io -a spell-platform
```

Wait for status to show "Ready"

## 3. GitHub OAuth Configuration

Update your GitHub OAuth App settings:

- **Homepage URL**: `https://magicspell.io`
- **Authorization callback URL**: `https://magicspell.io/api/auth/callback/github`

## 4. Stripe Webhook Configuration

1. Go to [Stripe Dashboard → Webhooks](https://dashboard.stripe.com/webhooks)
2. Add endpoint: `https://api.magicspell.io/v1/webhooks/stripe`
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

### 7.1 Test CORS Preflight

```bash
curl -i https://api.magicspell.io/v1/health \
  -H "Origin: https://magicspell.io" \
  -H "Access-Control-Request-Method: GET"
```

Expected: 200 OK with CORS headers

### 7.2 Test Actual Request

```bash
curl -i https://api.magicspell.io/v1/health \
  -H "Origin: https://magicspell.io"
```

Expected: CORS headers including exposed rate limit headers

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

### 7.4 Test Stripe Webhook (Locally)

```bash
stripe trigger payment_intent.succeeded \
  --webhook-endpoint https://api.magicspell.io/v1/webhooks/stripe
```

Check Fly.io logs for successful processing:
```bash
flyctl logs -a spell-platform
```

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

- Check NS records point to Vercel
- Verify DNS propagation: `dig magicspell.io`
- Allow up to 48 hours for global DNS propagation

### CORS errors in browser

- Verify API CORS configuration includes `https://magicspell.io`
- Check browser console for specific error messages
- Ensure cookies/credentials are properly configured

### Certificate errors

- Run `flyctl certs check api.magicspell.io -a spell-platform`
- Verify CNAME record exists and points to Fly.io hostname
- Wait for certificate validation (can take a few minutes)

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

- [ ] Domain resolves to Vercel
- [ ] API subdomain resolves to Fly.io
- [ ] GitHub OAuth flow works end-to-end
- [ ] Stripe checkout works
- [ ] Stripe webhooks are processed
- [ ] CORS preflight succeeds
- [ ] Security headers are present
- [ ] CSP doesn't block required resources
- [ ] Rate limiting works and headers are exposed
- [ ] SSL certificates are valid
- [ ] Monitoring and alerts are configured
- [ ] Backups are enabled (database)
- [ ] Documentation is updated
