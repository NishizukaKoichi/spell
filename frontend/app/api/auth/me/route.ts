import { NextResponse } from 'next/server';
import { cookies } from 'next/headers';

const API_BASE = process.env.NEXT_PUBLIC_API_BASE || 'https://magicspell.io';

export async function GET() {
  try {
    const cookieStore = await cookies();
    const sessionCookie = cookieStore.get('spell_session');

    if (!sessionCookie) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const res = await fetch(`${API_BASE}/auth/me`, {
      headers: {
        'Cookie': `spell_session=${sessionCookie.value}`,
      },
      cache: 'no-store',
    });

    if (res.status === 401 || res.status === 403) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: res.status });
    }

    if (!res.ok) {
      const body = await res.text();
      return NextResponse.json(
        { error: 'Failed to check authentication', detail: body.slice(0, 500) },
        { status: 502 }
      );
    }

    const data = await res.json();
    return NextResponse.json(data);
  } catch (error) {
    console.error('GET /api/auth/me error:', error);
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 });
  }
}
