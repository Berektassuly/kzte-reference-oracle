import { NextResponse } from "next/server";

import { fetchLatestOracleFeeds } from "@/lib/oracle/live-feeds-server";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

export async function GET() {
  try {
    const feeds = await fetchLatestOracleFeeds();

    return NextResponse.json(
      {
        feeds,
        generatedAt: new Date().toISOString(),
      },
      {
        headers: {
          "Cache-Control": "no-store, max-age=0",
        },
      },
    );
  } catch (error) {
    return NextResponse.json(
      {
        error: "Failed to fetch live oracle feeds.",
        detail: error instanceof Error ? error.message : "Unknown error",
      },
      { status: 502 },
    );
  }
}
