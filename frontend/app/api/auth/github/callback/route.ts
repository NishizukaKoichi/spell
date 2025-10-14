import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';

const API_BASE = process.env.NEXT_PUBLIC_API_BASE || 'https://api.magicspell.io';

export async function GET(request: NextRequest) {
  try {
    const searchParams = request.nextUrl.searchParams;
    const code = searchParams.get('code');
    const state = searchParams.get('state');

    if (!code) {
      return NextResponse.redirect(new URL('/login?error=no_code', request.url));
    }

    // Forward to backend callback
    const backendUrl = new URL('/auth/github/callback', API_BASE);
    backendUrl.searchParams.set('code', code);
    if (state) {
      backendUrl.searchParams.set('state', state);
    }

    const response = await fetch(backendUrl.toString(), {
      redirect: 'manual', // Don't follow redirects automatically
    });

    // Extract session cookie from backend response
    const setCookieHeader = response.headers.get('set-cookie');
    if (setCookieHeader) {
      const match = setCookieHeader.match(/spell_session=([^;]+)/);
      if (match) {
        const sessionToken = match[1];
        const cookieStore = await cookies();

        // Set cookie in frontend domain
        cookieStore.set('spell_session', sessionToken, {
          httpOnly: true,
          secure: process.env.NODE_ENV === 'production',
          sameSite: 'lax',
          maxAge: 60 * 60 * 24 * 30, // 30 days
          path: '/',
          domain: process.env.NODE_ENV === 'production' ? '.magicspell.io' : undefined,
        });

        // Redirect to dashboard
        return NextResponse.redirect(new URL('/dashboard', request.url));
      }
    }

    // If no session cookie, something went wrong
    return NextResponse.redirect(
      new URL('/login?error=auth_failed', request.url)
    );
  } catch (error) {
    console.error('GitHub callback error:', error);
    return NextResponse.redirect(
      new URL('/login?error=server_error', request.url)
    );
  }
}
