import { NextResponse } from 'next/server';
import { cookies } from 'next/headers';

const API_BASE = process.env.NEXT_PUBLIC_API_BASE || 'https://api.magicspell.io';

export async function POST() {
  try {
    const cookieStore = await cookies();
    const sessionCookie = cookieStore.get('spell_session');

    // Development mode: Always use dev endpoint (bypasses auth)
    const isDev = process.env.NODE_ENV === 'development';
    const endpoint = isDev
      ? '/v1/billing/dev-setup-intent'
      : '/v1/billing/setup-intent';

    // In production, require session cookie
    if (!isDev && !sessionCookie) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const headers: HeadersInit = {};
    // Only send cookie in production (dev endpoint doesn't need it)
    if (!isDev && sessionCookie) {
      headers['Cookie'] = `spell_session=${sessionCookie.value}`;
    }

    const res = await fetch(`${API_BASE}${endpoint}`, {
      method: 'POST',
      headers,
    });

    // In dev mode, don't fail on 401 from backend (shouldn't happen with dev endpoint)
    if (!isDev && (res.status === 401 || res.status === 403)) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: res.status });
    }

    if (!res.ok) {
      const body = await res.text();
      console.error('Backend error response:', {
        status: res.status,
        statusText: res.statusText,
        body: body.slice(0, 500)
      });
      return NextResponse.json(
        { error: 'Failed to create setup intent', detail: body.slice(0, 500) },
        { status: 502 }
      );
    }

    const data = await res.json();
    return NextResponse.json(data);
  } catch (error) {
    console.error('POST /api/billing/setup-intent error:', error);
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 });
  }
}
