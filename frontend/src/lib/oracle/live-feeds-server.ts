import { ORACLE_FEEDS, parseHermesLatestResponse, toHermesFeedId } from "@/lib/oracle/live-feeds";

const HERMES_BASE_URL = "https://hermes.pyth.network";

export async function fetchLatestOracleFeeds() {
  const searchParams = new URLSearchParams();

  for (const feed of ORACLE_FEEDS) {
    searchParams.append("ids[]", toHermesFeedId(feed.id));
  }

  const response = await fetch(
    `${HERMES_BASE_URL}/v2/updates/price/latest?${searchParams.toString()}`,
    {
      cache: "no-store",
      headers: {
        Accept: "application/json",
      },
    },
  );

  if (!response.ok) {
    throw new Error(`Hermes latest request failed with status ${response.status}`);
  }

  const payload = await response.json();

  return parseHermesLatestResponse(payload);
}

export function getHermesStreamUrl() {
  const searchParams = new URLSearchParams();

  for (const feed of ORACLE_FEEDS) {
    searchParams.append("ids[]", toHermesFeedId(feed.id));
  }

  return `${HERMES_BASE_URL}/v2/updates/price/stream?${searchParams.toString()}`;
}
