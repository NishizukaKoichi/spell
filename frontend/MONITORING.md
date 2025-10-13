# Monitoring & Error Tracking Setup

## 1. Vercel Analytics & Speed Insights

### Enable via Dashboard
1. Go to https://vercel.com/magicspell/spell-caster/settings/analytics
2. Enable **Web Analytics** (free tier)
3. Enable **Speed Insights** (free tier)

### Verify Installation
Check `_app.tsx` or `layout.tsx` includes:
```typescript
import { Analytics } from '@vercel/analytics/react';
import { SpeedInsights } from '@vercel/speed-insights/next';
```

## 2. Sentry Integration

### Frontend Setup

Install dependencies:
```bash
npm install --save @sentry/nextjs
```

Create `sentry.client.config.ts`:
```typescript
import * as Sentry from "@sentry/nextjs";

Sentry.init({
  dsn: process.env.NEXT_PUBLIC_SENTRY_DSN,
  environment: process.env.VERCEL_ENV || 'development',
  tracesSampleRate: 1.0,
  replaysSessionSampleRate: 0.1,
  replaysOnErrorSampleRate: 1.0,
});
```

Add environment variable:
```bash
vercel env add NEXT_PUBLIC_SENTRY_DSN production
# Enter: https://xxx@xxx.ingest.sentry.io/xxx
```

### Backend Setup (Rust/Actix)

Add to `Cargo.toml`:
```toml
sentry = "0.32"
sentry-actix = "0.32"
```

Update `src/main.rs`:
```rust
let _guard = sentry::init((
    env::var("SENTRY_DSN").unwrap_or_default(),
    sentry::ClientOptions {
        release: sentry::release_name!(),
        environment: Some(env::var("FLY_REGION").unwrap_or("development".into()).into()),
        traces_sample_rate: 1.0,
        ..Default::default()
    },
));

HttpServer::new(move || {
    App::new()
        .wrap(sentry_actix::Sentry::new())
        // ... rest of config
})
```

Add Fly.io secret:
```bash
flyctl secrets set SENTRY_DSN=https://xxx@xxx.ingest.sentry.io/xxx -a spell-platform
```

## 3. Uptime Monitoring

### Recommended Services
- **UptimeRobot** (free tier: 50 monitors, 5-min interval)
- **BetterUptime** (free tier: 10 monitors, 3-min interval)
- **Pingdom** (paid)

### Monitor Endpoints
1. **Frontend**: https://magicspell.io
   - Expected: HTTP 200 or 307
   - Alert threshold: 3 consecutive failures

2. **API Health**: https://api.magicspell.io/healthz
   - Expected: HTTP 200 + `{"status":"ok"}`
   - Alert threshold: 3 consecutive failures
   - Check interval: 1 minute

3. **API CORS**: https://api.magicspell.io/v1/ping (OPTIONS)
   - Expected: `access-control-allow-origin: https://magicspell.io`
   - Alert threshold: 3 consecutive failures

### UptimeRobot Setup Commands (via API)

```bash
# Frontend monitor
curl -X POST https://api.uptimerobot.com/v2/newMonitor \
  -d "api_key=YOUR_API_KEY" \
  -d "friendly_name=MagicSpell Frontend" \
  -d "url=https://magicspell.io" \
  -d "type=1" \
  -d "interval=300"

# API health monitor
curl -X POST https://api.uptimerobot.com/v2/newMonitor \
  -d "api_key=YOUR_API_KEY" \
  -d "friendly_name=MagicSpell API Health" \
  -d "url=https://api.magicspell.io/healthz" \
  -d "type=1" \
  -d "interval=60"
```

## 4. Fly.io Metrics

Enable Prometheus metrics in `fly.toml`:
```toml
[metrics]
port = 9091
path = "/metrics"
```

View metrics:
```bash
flyctl dashboard spell-platform
```

## 5. Log Aggregation

### Vercel Logs
```bash
# Stream production logs
vercel logs magicspell --since=1h -f

# Search for errors
vercel logs magicspell --since=1d | grep -i error
```

### Fly.io Logs
```bash
# Stream logs
flyctl logs -a spell-platform

# Search for errors
flyctl logs -a spell-platform | grep -i error
```

### Structured Logging (Backend)

Ensure all logs are JSON format:
```rust
env_logger::Builder::from_default_env()
    .format_timestamp_millis()
    .format(|buf, record| {
        writeln!(buf, "{{\"level\":\"{}\",\"msg\":\"{}\",\"target\":\"{}\"}}",
            record.level(), record.args(), record.target())
    })
    .init();
```

## 6. Alerts Configuration

### Critical Alerts (PagerDuty/Slack)
- API downtime > 1 minute
- Error rate > 5% (1 minute window)
- Response time p95 > 2s

### Warning Alerts (Email/Slack)
- Error rate > 1% (5 minute window)
- Response time p95 > 1s
- Memory usage > 80%

### Slack Webhook Setup
```bash
# Add Slack webhook URL
flyctl secrets set SLACK_WEBHOOK_URL=https://hooks.slack.com/services/xxx -a spell-platform
vercel env add SLACK_WEBHOOK_URL production
```

## 7. Performance Monitoring

### Lighthouse CI

Install:
```bash
npm install -D @lhci/cli
```

Create `.lighthouserc.js`:
```javascript
module.exports = {
  ci: {
    collect: {
      url: ['https://magicspell.io'],
      numberOfRuns: 3,
    },
    assert: {
      preset: 'lighthouse:recommended',
      assertions: {
        'categories:performance': ['error', {minScore: 0.8}],
        'categories:accessibility': ['error', {minScore: 0.9}],
        'categories:best-practices': ['error', {minScore: 0.9}],
        'categories:seo': ['error', {minScore: 0.9}],
      },
    },
  },
};
```

Run:
```bash
npx @lhci/cli autorun --upload.target=temporary-public-storage
```

## 8. Current Status

✅ **Baseline Metrics (2025-10-13)**
- Frontend: HTTP 307 (healthy redirect)
- API Health: 200 OK, TTFB: 0.79s
- CORS: Working correctly

### Next Steps
1. Enable Vercel Analytics in dashboard
2. Set up Sentry accounts (Frontend + Backend)
3. Configure UptimeRobot monitors
4. Add Lighthouse CI to GitHub Actions
