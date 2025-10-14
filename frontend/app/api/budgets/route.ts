import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';

const API_BASE = process.env.NEXT_PUBLIC_API_BASE || 'https://magicspell.io';

export async function GET() {
  try {
    const cookieStore = await cookies();
    const sessionCookie = cookieStore.get('spell_session');

    if (!sessionCookie) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const res = await fetch(`${API_BASE}/budget`, {
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
        { error: 'upstream_error', detail: body.slice(0, 500) },
        { status: 502 }
      );
    }

    const data = await res.json();
    return NextResponse.json(data);
  } catch (error) {
    console.error('GET /api/budgets error:', error);
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 });
  }
}

export async function POST(req: NextRequest) {
  try {
    const cookieStore = await cookies();
    const sessionCookie = cookieStore.get('spell_session');

    if (!sessionCookie) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const body = await req.json();

    const res = await fetch(`${API_BASE}/budget`, {
      method: 'POST',
      headers: {
        'Cookie': `spell_session=${sessionCookie.value}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(body),
    });

    if (res.status === 401 || res.status === 403) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: res.status });
    }

    if (!res.ok) {
      const body = await res.text();
      return NextResponse.json(
        { error: 'upstream_error', detail: body.slice(0, 500) },
        { status: 502 }
      );
    }

    const data = await res.json();
    return NextResponse.json(data);
  } catch (error) {
    console.error('POST /api/budgets error:', error);
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 });
  }
}

export async function PUT(req: NextRequest) {
  try {
    const cookieStore = await cookies();
    const sessionCookie = cookieStore.get('spell_session');

    if (!sessionCookie) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const body = await req.json();

    const res = await fetch(`${API_BASE}/budget`, {
      method: 'PUT',
      headers: {
        'Cookie': `spell_session=${sessionCookie.value}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(body),
    });

    if (res.status === 401 || res.status === 403) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: res.status });
    }

    if (!res.ok) {
      const body = await res.text();
      return NextResponse.json(
        { error: 'upstream_error', detail: body.slice(0, 500) },
        { status: 502 }
      );
    }

    const data = await res.json();
    return NextResponse.json(data);
  } catch (error) {
    console.error('PUT /api/budgets error:', error);
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 });
  }
}

export async function DELETE() {
  try {
    const cookieStore = await cookies();
    const sessionCookie = cookieStore.get('spell_session');

    if (!sessionCookie) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const res = await fetch(`${API_BASE}/budget`, {
      method: 'DELETE',
      headers: {
        'Cookie': `spell_session=${sessionCookie.value}`,
      },
    });

    if (res.status === 401 || res.status === 403) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: res.status });
    }

    if (!res.ok) {
      const body = await res.text();
      return NextResponse.json(
        { error: 'upstream_error', detail: body.slice(0, 500) },
        { status: 502 }
      );
    }

    return new NextResponse(null, { status: 204 });
  } catch (error) {
    console.error('DELETE /api/budgets error:', error);
    return NextResponse.json({ error: 'Internal server error' }, { status: 500 });
  }
}
