import { LivePriceFeedsShell } from "@/components/live/live-price-feeds-shell";
import { fetchLatestOracleFeeds } from "@/lib/oracle/live-feeds-server";

export const dynamic = "force-dynamic";

export default async function PriceFeedsPage() {
  const initialFeeds = await fetchLatestOracleFeeds();

  return <LivePriceFeedsShell initialFeeds={initialFeeds} />;
}
