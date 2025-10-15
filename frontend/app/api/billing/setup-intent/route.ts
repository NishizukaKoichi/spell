import { NextResponse } from 'next/server';
import { cookies } from 'next/headers';

const API_BASE = process.env.NEXT_PUBLIC_API_BASE || 'https://api.magicspell.io';

export async function POST() {
  try {
    const cookieStore = await cookies();
    const sessionCookie = cookieStore.get('spell_session');

    if (!sessionCookie) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const res = await fetch(`${API_BASE}/v1/billing/setup-intent`, {
      method: 'POST',
      headers: {
        'Cookie': `spell_session=${sessionCookie.value}`,
      },
    });

    if (res.status === 401 || res.status === 403) {
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
