import { LiveHomeSpotlight } from "@/components/live/live-home-spotlight";
import { fetchLatestOracleFeeds } from "@/lib/oracle/live-feeds-server";

export const dynamic = "force-dynamic";

export default async function HomePage() {
  const initialFeeds = await fetchLatestOracleFeeds();

  return <LiveHomeSpotlight initialFeeds={initialFeeds} />;
}
