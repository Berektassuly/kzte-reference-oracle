import { getHermesStreamUrl } from "@/lib/oracle/live-feeds-server";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";

export async function GET() {
  const response = await fetch(getHermesStreamUrl(), {
    cache: "no-store",
    headers: {
      Accept: "text/event-stream",
    },
  });

  if (!response.ok || !response.body) {
    return new Response("Unable to open Pyth stream.", { status: 502 });
  }

  return new Response(response.body, {
    headers: {
      "Cache-Control": "no-cache, no-transform",
      Connection: "keep-alive",
      "Content-Type": "text/event-stream",
    },
  });
}
