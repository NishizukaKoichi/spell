# Incident Response Playbook

## Quick Reference Card

### Emergency Contacts
- **On-Call Engineer**: TBD
- **Slack Channel**: `#spell-platform-incidents`
- **Status Page**: https://status.magicspell.io (TBD)

### Critical Endpoints
- Frontend: https://magicspell.io
- API Health: https://api.magicspell.io/healthz
- Metrics: https://api.magicspell.io/metrics

---

## 1. Rollback Procedures

### Frontend Rollback (Vercel)

**List recent deployments:**
```bash
vercel ls magicspell
```

**Rollback to specific deployment:**
```bash
# Via CLI
vercel rollback <deployment-url>

# Via Dashboard (recommended)
# 1. Go to https://vercel.com/magicspell/spell-caster/deployments
# 2. Find the last known good deployment
# 3. Click "..." → "Promote to Production"
```

**Verify rollback:**
```bash
curl -I https://magicspell.io
curl -sS https://magicspell.io | grep -i "version\|build"
```

**Estimated Time**: 30-60 seconds

---

### Backend Rollback (Fly.io)

**List releases:**
```bash
flyctl releases -a spell-platform
```

**Rollback to previous release:**
```bash
# Rollback one version
flyctl releases rollback -a spell-platform

# Rollback to specific version
flyctl releases rollback <version> -a spell-platform
```

**Verify rollback:**
```bash
curl -sS https://api.magicspell.io/healthz | jq
flyctl status -a spell-platform
```

**Estimated Time**: 2-3 minutes

---

## 2. Maintenance Mode

### Enable Maintenance Mode

**Set environment variable:**
```bash
# Frontend
vercel env add NEXT_PUBLIC_MAINTENANCE_MODE production
# Enter value: "true"

# Redeploy
vercel --prod
```

**Create maintenance page** (`app/maintenance/page.tsx`):
```typescript
export default function MaintenancePage() {
  return (
    <div className="flex min-h-screen items-center justify-center">
      <div className="text-center">
        <h1 className="text-4xl font-bold mb-4">
          ⚙️ Under Maintenance
        </h1>
        <p className="text-gray-600">
          We're performing scheduled maintenance.
          We'll be back shortly.
        </p>
        <p className="text-sm text-gray-400 mt-4">
          Status: https://status.magicspell.io
        </p>
      </div>
    </div>
  );
}
```

**Middleware to redirect** (`middleware.ts`):
```typescript
import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';

export function middleware(request: NextRequest) {
  const maintenanceMode = process.env.NEXT_PUBLIC_MAINTENANCE_MODE === 'true';

  if (maintenanceMode && !request.nextUrl.pathname.startsWith('/maintenance')) {
    return NextResponse.redirect(new URL('/maintenance', request.url));
  }

  return NextResponse.next();
}
```

### Disable Maintenance Mode
```bash
vercel env rm NEXT_PUBLIC_MAINTENANCE_MODE production
vercel --prod
```

**Estimated Time**: 1-2 minutes

---

## 3. Health Checks & Diagnostics

### Frontend Diagnostics

**Check deployment status:**
```bash
vercel ls magicspell --limit 5
```

**Stream logs:**
```bash
vercel logs magicspell --since=10m -f
```

**Check build logs:**
```bash
vercel inspect <deployment-url> --logs
```

### Backend Diagnostics

**Check machine status:**
```bash
flyctl status -a spell-platform
```

**Stream logs:**
```bash
flyctl logs -a spell-platform
```

**Check recent errors:**
```bash
flyctl logs -a spell-platform | grep -i error | tail -20
```

**SSH into machine (if needed):**
```bash
flyctl ssh console -a spell-platform
```

### Database Diagnostics

**Connect to Postgres:**
```bash
flyctl postgres connect -a spell-platform-db
```

**Check connections:**
```sql
SELECT count(*) FROM pg_stat_activity WHERE datname = 'spell_platform';
SELECT * FROM pg_stat_activity WHERE state = 'active';
```

**Check slow queries:**
```sql
SELECT query, calls, total_time, mean_time
FROM pg_stat_statements
ORDER BY mean_time DESC
LIMIT 10;
```

### Redis Diagnostics

**Connect to Redis:**
```bash
flyctl redis connect spell-platform-redis
```

**Check memory usage:**
```
INFO memory
```

**Check keyspace:**
```
INFO keyspace
DBSIZE
```

---

## 4. Common Incident Scenarios

### Scenario A: API is Down (5xx errors)

**1. Check machine status:**
```bash
flyctl status -a spell-platform
```

**2. Check logs:**
```bash
flyctl logs -a spell-platform | tail -50
```

**3. Restart machine (if needed):**
```bash
flyctl machine restart <machine-id> -a spell-platform
```

**4. If restart doesn't help, rollback:**
```bash
flyctl releases rollback -a spell-platform
```

**5. Verify recovery:**
```bash
curl -sS https://api.magicspell.io/healthz
```

---

### Scenario B: High Error Rate (4xx/5xx > 5%)

**1. Check recent deployments:**
```bash
vercel ls magicspell --limit 3
flyctl releases -a spell-platform --limit 3
```

**2. Identify error patterns:**
```bash
# Frontend
vercel logs magicspell --since=5m | grep -i error

# Backend
flyctl logs -a spell-platform | grep -E '(ERROR|WARN)' | tail -30
```

**3. Check rate limiting:**
```bash
# Redis connections
flyctl redis connect spell-platform-redis
> INFO stats
> KEYS rate_limit:*
```

**4. If caused by bad deployment, rollback both:**
```bash
vercel rollback <last-good-deployment>
flyctl releases rollback -a spell-platform
```

---

### Scenario C: Database Connection Issues

**1. Check database status:**
```bash
flyctl postgres list
flyctl status -a spell-platform-db
```

**2. Check connection pool:**
```bash
flyctl postgres connect -a spell-platform-db
\l
SELECT count(*) FROM pg_stat_activity;
```

**3. Restart database (last resort):**
```bash
flyctl postgres restart -a spell-platform-db
```

**4. Update connection string if needed:**
```bash
flyctl secrets set DATABASE_URL=<new-url> -a spell-platform
```

---

### Scenario D: CORS Issues

**1. Verify CORS headers:**
```bash
curl -si -X OPTIONS https://api.magicspell.io/v1/ping \
  -H 'Origin: https://magicspell.io' \
  -H 'Access-Control-Request-Method: POST'
```

**2. Check allowed origins in code:**
```bash
# Check src/main.rs:20-23
grep -A 5 "allowed_origin" /Users/koichinishizuka/spell-platform/src/main.rs
```

**3. If origin missing, add and redeploy:**
```rust
.allowed_origin("https://new-origin.com")
```

---

### Scenario E: Certificate Expiration

**Check certificate status:**
```bash
flyctl certs show api.magicspell.io -a spell-platform
```

**If expired or expiring soon:**
```bash
# Remove old cert
flyctl certs delete api.magicspell.io -a spell-platform

# Re-issue
flyctl certs create api.magicspell.io -a spell-platform
```

**Wait for DNS propagation** (1-5 minutes), then verify:
```bash
curl -I https://api.magicspell.io/healthz
```

---

## 5. Performance Degradation

### High Response Time (p95 > 2s)

**1. Check machine resources:**
```bash
flyctl status -a spell-platform
flyctl machine list -a spell-platform
```

**2. Scale up if needed:**
```bash
# Add more machines
flyctl scale count 2 -a spell-platform

# Increase machine size
flyctl scale vm shared-cpu-2x -a spell-platform
```

**3. Check for slow queries:**
```sql
-- Connect to database
SELECT query, mean_time, calls
FROM pg_stat_statements
ORDER BY mean_time DESC
LIMIT 10;
```

**4. Clear Redis cache (if stale):**
```bash
flyctl redis connect spell-platform-redis
> FLUSHDB
```

---

### High Memory Usage (> 80%)

**1. Check memory:**
```bash
flyctl machine status <machine-id> -a spell-platform
```

**2. Restart machine:**
```bash
flyctl machine restart <machine-id> -a spell-platform
```

**3. If persistent, check for memory leaks in logs:**
```bash
flyctl logs -a spell-platform | grep -i "memory\|oom"
```

---

## 6. Communication Templates

### Incident Announcement (Slack)

```
🚨 **INCIDENT DETECTED**
Severity: [P0-Critical | P1-High | P2-Medium]
Component: [Frontend | API | Database | Redis]
Impact: [User-facing | Internal | Degraded performance]
Status: [Investigating | Identified | Monitoring | Resolved]

**Symptoms:**
- [Describe what users are experiencing]

**Actions Taken:**
- [List actions in progress]

**ETA to Resolution:** [Best estimate or "Unknown"]

Thread for updates 🧵
```

### Resolution Announcement

```
✅ **INCIDENT RESOLVED**
Duration: [Total incident time]
Root Cause: [Brief explanation]

**Actions Taken:**
- [Bullet list of fixes]

**Prevention:**
- [Measures to prevent recurrence]

**Postmortem:** [Link to doc or "Will be published within 48h"]
```

---

## 7. Escalation Matrix

| Severity | Response Time | Escalation |
|----------|---------------|------------|
| P0 (Critical) | Immediate | Page on-call + CTO |
| P1 (High) | 15 minutes | Page on-call |
| P2 (Medium) | 1 hour | Slack notification |
| P3 (Low) | Next business day | Create ticket |

**Severity Definitions:**
- **P0**: Complete outage, data loss, security breach
- **P1**: Major feature down, >10% error rate
- **P2**: Minor feature degraded, <10% error rate
- **P3**: Cosmetic issue, monitoring alert

---

## 8. Post-Incident Checklist

- [ ] Incident resolved and verified
- [ ] Monitoring confirms stable state (30 min)
- [ ] Slack announcement sent
- [ ] Postmortem scheduled (within 48h)
- [ ] Runbook updated with learnings
- [ ] Prevention tasks created in backlog

---

## 9. Quick Commands Reference

```bash
# Health checks
curl -sS https://api.magicspell.io/healthz | jq
curl -I https://magicspell.io

# Logs
vercel logs magicspell --since=10m -f
flyctl logs -a spell-platform

# Status
flyctl status -a spell-platform
vercel ls magicspell --limit 5

# Rollback
vercel rollback <deployment-url>
flyctl releases rollback -a spell-platform

# Restart
flyctl machine restart <machine-id> -a spell-platform

# Scale
flyctl scale count 2 -a spell-platform
```

---

**Last Updated:** 2025-10-13
**Version:** 1.0
**Owner:** Platform Team
