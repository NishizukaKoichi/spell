import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';

const API_BASE = process.env.NEXT_PUBLIC_API_BASE || 'http://localhost:8080';

export async function POST(request: NextRequest) {
  try {
    const response = await fetch(`${API_BASE}/dev/login`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
    });

    if (!response.ok) {
      return NextResponse.json(
        { error: 'Dev login failed' },
        { status: response.status }
      );
    }

    const data = await response.json();

    // Extract the session cookie from the backend response
    const setCookieHeader = response.headers.get('set-cookie');
    if (setCookieHeader) {
      // Parse the cookie to extract just the session token value
      const match = setCookieHeader.match(/spell_session=([^;]+)/);
      if (match) {
        const sessionToken = match[1];

        // Set the cookie on the response
        const cookieStore = await cookies();
        cookieStore.set('spell_session', sessionToken, {
          httpOnly: true,
          secure: process.env.NODE_ENV === 'production',
          sameSite: 'lax',
          maxAge: 60 * 60 * 24 * 30, // 30 days
          path: '/',
        });
      }
    }

    return NextResponse.json(data);
  } catch (error) {
    console.error('Dev login error:', error);
    return NextResponse.json(
      { error: 'Failed to connect to backend' },
      { status: 502 }
    );
  }
}
