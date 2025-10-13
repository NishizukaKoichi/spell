# CSP Hardening Guide

## Current CSP Policy

```
Content-Security-Policy:
  default-src 'self';
  script-src 'self' 'unsafe-inline' vercel-insights.com https://js.stripe.com;
  style-src 'self' 'unsafe-inline';
  img-src 'self' data: https://avatars.githubusercontent.com;
  connect-src 'self' https://api.magicspell.io https://api.stripe.com;
  frame-src https://js.stripe.com;
  frame-ancestors 'none';
  upgrade-insecure-requests;
```

## Phase 1: Report-Only Monitoring (24h)

### Enable Report-Only Mode

Temporarily change to `Content-Security-Policy-Report-Only` to collect violation reports without blocking:

**Update `next.config.ts`:**
```typescript
{
  key: 'Content-Security-Policy-Report-Only',  // Add "-Report-Only"
  value: [
    "default-src 'self'",
    "script-src 'self' 'unsafe-inline' vercel-insights.com https://js.stripe.com",
    "style-src 'self' 'unsafe-inline'",
    "img-src 'self' data: https://avatars.githubusercontent.com",
    "connect-src 'self' https://api.magicspell.io https://api.stripe.com",
    "frame-src https://js.stripe.com",
    "frame-ancestors 'none'",
    "upgrade-insecure-requests",
    "report-uri /api/csp-report",  // Add reporting endpoint
  ].join('; '),
}
```

### Create Report Endpoint

**Create `app/api/csp-report/route.ts`:**
```typescript
import { NextRequest, NextResponse } from 'next/server';

export async function POST(request: NextRequest) {
  try {
    const report = await request.json();

    // Log CSP violations
    console.error('CSP Violation:', JSON.stringify(report, null, 2));

    // Optional: Send to monitoring service
    // await fetch(process.env.MONITORING_WEBHOOK_URL, {
    //   method: 'POST',
    //   body: JSON.stringify(report),
    // });

    return NextResponse.json({ ok: true }, { status: 204 });
  } catch (error) {
    console.error('CSP report error:', error);
    return NextResponse.json({ error: 'Failed to process report' }, { status: 500 });
  }
}
```

### Monitor for 24 Hours

```bash
# Deploy with Report-Only
vercel --prod

# Watch for violations in logs
vercel logs magicspell --since=1h -f | grep "CSP Violation"
```

## Phase 2: Tighten Policy

After 24h of zero violations, remove unsafe directives:

### Remove `'unsafe-inline'` from `script-src`

**Add nonce-based CSP:**

1. Generate nonce in middleware:
```typescript
// middleware.ts
import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';
import crypto from 'crypto';

export function middleware(request: NextRequest) {
  const nonce = crypto.randomBytes(16).toString('base64');
  const cspHeader = `
    default-src 'self';
    script-src 'self' 'nonce-${nonce}' vercel-insights.com https://js.stripe.com;
    style-src 'self' 'unsafe-inline';
    img-src 'self' data: https://avatars.githubusercontent.com;
    connect-src 'self' https://api.magicspell.io https://api.stripe.com;
    frame-src https://js.stripe.com;
    frame-ancestors 'none';
    upgrade-insecure-requests;
  `.replace(/\s{2,}/g, ' ').trim();

  const response = NextResponse.next();
  response.headers.set('Content-Security-Policy', cspHeader);
  response.headers.set('x-nonce', nonce);

  return response;
}
```

2. Add nonce to scripts in layout:
```typescript
// app/layout.tsx
import { headers } from 'next/headers';

export default function RootLayout({ children }: { children: React.ReactNode }) {
  const nonce = headers().get('x-nonce');

  return (
    <html>
      <head>
        <script nonce={nonce} src="/analytics.js" />
      </head>
      <body>{children}</body>
    </html>
  );
}
```

### Add Stripe Network (When Using Stripe)

When enabling Stripe Checkout:
```typescript
"connect-src 'self' https://api.magicspell.io https://api.stripe.com https://m.stripe.network",
```

## Phase 3: Enforce and Monitor

### Switch to Enforcing Mode

Remove `-Report-Only` suffix:
```typescript
{
  key: 'Content-Security-Policy',  // Remove "-Report-Only"
  value: [...],
}
```

### Continuous Monitoring

Keep the `/api/csp-report` endpoint active for ongoing monitoring:

```bash
# Check for new violations weekly
vercel logs magicspell --since=7d | grep "CSP Violation" | wc -l
```

## Additional Hardening

### 1. Add Trusted Types (Advanced)

Prevent DOM XSS:
```typescript
"require-trusted-types-for 'script'",
"trusted-types default",
```

### 2. Subresource Integrity (SRI)

For external scripts, add integrity hashes:
```html
<script
  src="https://js.stripe.com/v3/"
  integrity="sha384-..."
  crossorigin="anonymous"
/>
```

### 3. Remove `data:` from `img-src` (If Possible)

Replace data URIs with actual image files:
```typescript
"img-src 'self' https://avatars.githubusercontent.com",
```

## Testing CSP

### Browser DevTools

1. Open DevTools → Console
2. Look for CSP violation warnings
3. Check Network tab for blocked requests

### Automated Testing

```bash
# Install CSP validator
npm install -D csp-validator

# Validate policy
npx csp-validator "default-src 'self'; script-src 'self'..."
```

### External Tools

- [CSP Evaluator](https://csp-evaluator.withgoogle.com/)
- [Mozilla Observatory](https://observatory.mozilla.org/)

## Rollback Plan

If CSP breaks functionality:

1. **Quick fix**: Add missing source to allowlist
2. **Emergency**: Switch to Report-Only mode
3. **Nuclear**: Remove CSP header entirely (last resort)

```bash
# Quick revert to previous deployment
vercel rollback
```

## References

- [MDN: Content Security Policy](https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP)
- [Google CSP Guide](https://web.dev/strict-csp/)
- [CSP Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Content_Security_Policy_Cheat_Sheet.html)
