/**
 * Spell Platform - Cloudflare Reverse Proxy
 *
 * Routes:
 * - /api/* → Fly.io (Rust API)
 * - /* → Vercel (Next.js Frontend)
 */

interface Env {
  API_ORIGIN: string;
  FRONTEND_ORIGIN: string;
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);

    // Route API requests to Fly.io
    if (url.pathname.startsWith('/api/') || url.pathname === '/api') {
      return proxyToAPI(request, env.API_ORIGIN);
    }

    // Route auth endpoints to Fly.io
    if (url.pathname.startsWith('/auth/')) {
      return proxyToAPI(request, env.API_ORIGIN);
    }

    // Route all other requests to Vercel
    return proxyToFrontend(request, env.FRONTEND_ORIGIN);
  },
};

async function proxyToAPI(request: Request, apiOrigin: string): Promise<Response> {
  const url = new URL(request.url);
  const apiUrl = new URL(url.pathname + url.search, apiOrigin);

  // Create new request to API
  const apiRequest = new Request(apiUrl.toString(), {
    method: request.method,
    headers: request.headers,
    body: request.body,
    redirect: 'manual',
  });

  // Forward to API
  const response = await fetch(apiRequest);

  // Clone response to modify headers
  const newResponse = new Response(response.body, response);

  // Ensure CORS headers are set correctly
  newResponse.headers.set('Access-Control-Allow-Origin', 'https://magicspell.io');
  newResponse.headers.set('Access-Control-Allow-Credentials', 'true');

  // Ensure cookies use correct domain
  const setCookie = newResponse.headers.get('Set-Cookie');
  if (setCookie) {
    // Remove any existing Domain attribute and let it default to magicspell.io
    const cookieWithoutDomain = setCookie.replace(/;\s*Domain=[^;]+/gi, '');
    newResponse.headers.set('Set-Cookie', cookieWithoutDomain);
  }

  return newResponse;
}

async function proxyToFrontend(request: Request, frontendOrigin: string): Promise<Response> {
  const url = new URL(request.url);
  const frontendUrl = new URL(url.pathname + url.search, frontendOrigin);

  // Create new request to frontend
  const frontendRequest = new Request(frontendUrl.toString(), {
    method: request.method,
    headers: request.headers,
    body: request.body,
    redirect: 'manual',
  });

  // Forward to frontend
  const response = await fetch(frontendRequest);

  // Clone response
  const newResponse = new Response(response.body, response);

  // Add security headers
  newResponse.headers.set('X-Frame-Options', 'DENY');
  newResponse.headers.set('X-Content-Type-Options', 'nosniff');
  newResponse.headers.set('Referrer-Policy', 'strict-origin-when-cross-origin');

  return newResponse;
}
