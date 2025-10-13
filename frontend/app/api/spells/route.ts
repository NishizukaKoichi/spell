import { NextRequest, NextResponse } from "next/server";
import { z } from "zod";

const API_BASE = process.env.NEXT_PUBLIC_API_BASE || "https://api.magicspell.io";

const SpellsSchema = z.array(
  z.object({
    id: z.string(),
    name: z.string(),
    author: z.string(),
    version: z.string(),
    description: z.string().default(""),
    verified: z.boolean().default(false),
  })
);

export async function GET(req: NextRequest) {
  if (!API_BASE) {
    return NextResponse.json(
      { error: "API base not configured" },
      { status: 500 }
    );
  }

  // クエリ（任意: page/limit）
  const url = new URL(`${API_BASE}/v1/spells`);
  const page = req.nextUrl.searchParams.get("page");
  const limit = req.nextUrl.searchParams.get("limit");
  if (page) url.searchParams.set("page", page);
  if (limit) url.searchParams.set("limit", limit);

  // 認証中継（Cookie / Authorization）
  const headers: Record<string, string> = {
    Accept: "application/json",
  };
  const cookie = req.headers.get("cookie");
  if (cookie) headers["cookie"] = cookie;
  const auth = req.headers.get("authorization");
  if (auth) headers["authorization"] = auth;

  const res = await fetch(url.toString(), {
    method: "GET",
    headers,
    // Nodeランタイムで Cookie 手動中継（Edgeでも可だがまずは Node 推奨）
    cache: "no-store",
  });

  if (res.status === 401 || res.status === 403) {
    return NextResponse.json(
      { error: "unauthorized" },
      { status: res.status }
    );
  }
  if (!res.ok) {
    const body = await res.text();
    return NextResponse.json(
      { error: "upstream_error", detail: body.slice(0, 500) },
      { status: 502 }
    );
  }

  const data = await res.json();
  const parsed = SpellsSchema.safeParse(data);
  if (!parsed.success) {
    return NextResponse.json(
      { error: "invalid_shape", detail: parsed.error.flatten() },
      { status: 502 }
    );
  }

  return NextResponse.json(parsed.data, {
    status: 200,
    headers: { "Cache-Control": "no-store" },
  });
}
