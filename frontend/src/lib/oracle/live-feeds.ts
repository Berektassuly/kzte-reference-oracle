import type { OracleFeedDefinition, OracleFeedSnapshot } from "@/lib/types/site";

type HermesParsedPriceUpdate = {
  id: string;
  price: {
    price: string;
    conf: string;
    expo: number;
    publish_time: number;
  };
  ema_price?: {
    price: string;
    conf: string;
    expo: number;
    publish_time: number;
  };
  metadata?: {
    slot?: number;
    proof_available_time?: number;
    prev_publish_time?: number;
  };
};

type HermesLatestResponse = {
  parsed?: HermesParsedPriceUpdate[];
};

export const ORACLE_FEEDS: OracleFeedDefinition[] = [
  {
    id: "ef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d",
    symbol: "SOL / USD",
    asset: "SOL",
    venue: "Jupiter + Phoenix",
    description: "Primary Solana benchmark routed through Pyth publishers.",
    accent: "#61f2b1",
    icon: "sol",
  },
  {
    id: "e62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43",
    symbol: "BTC / USD",
    asset: "BTC",
    venue: "Coinbase + Binance",
    description: "Cross-venue macro benchmark used as reference ballast.",
    accent: "#f7b955",
    icon: "btc",
  },
  {
    id: "ff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace",
    symbol: "ETH / USD",
    asset: "ETH",
    venue: "Drift + Raydium",
    description: "High-liquidity Ethereum quote with low-latency confidence bands.",
    accent: "#7a8cff",
    icon: "eth",
  },
  {
    id: "0a0408d619e9380abad35060f9192039ed5042fa6f82301d0e48bb52be830996",
    symbol: "JUP / USD",
    asset: "JUP",
    venue: "Jupiter + MM desk",
    description: "Solana-native routing token feed aligned to signer quorum.",
    accent: "#8f6cff",
    icon: "jup",
  },
  {
    id: "0bbf28e9a841a1cc788f6a361b17ca072d0ea3098a1e5df1c3922d06719579ff",
    symbol: "PYTH / USD",
    asset: "PYTH",
    venue: "Pyth + centralized venues",
    description: "Protocol-native asset feed with oracle heartbeat visibility.",
    accent: "#72dfff",
    icon: "pyth",
  },
  {
    id: "72b021217ca3fe68922a19aaf990109cb9d84e9ad004b4d2025ad6f529314419",
    symbol: "BONK / USD",
    asset: "BONK",
    venue: "Meteora + Binance",
    description: "Memecoin reference feed with clipped outlier protection.",
    accent: "#73c7ff",
    icon: "bonk",
  },
  {
    id: "37505261e557e251290b8c8899453064e8d760ed5c65a779726f2490980da74c",
    symbol: "ORCA / USD",
    asset: "ORCA",
    venue: "Orca + market makers",
    description: "DEX-native quote surface tuned for Solana liquidity venues.",
    accent: "#57f0c5",
    icon: "orca",
  },
  {
    id: "649fdd7ec08e8e2a20f425729854e90293dcbe2376abc47197a14da6ff339756",
    symbol: "HNT / USD",
    asset: "HNT",
    venue: "Helium + broad venues",
    description: "Supplementary Solana-adjacent infrastructure benchmark.",
    accent: "#a1f38c",
    icon: "hnt",
  },
];

const FEED_MAP = new Map(
  ORACLE_FEEDS.map((feed) => [feed.id.toLowerCase(), feed] as const),
);

function scaleOracleNumber(value: string, exponent: number) {
  return Number(value) * 10 ** exponent;
}

function round(value: number, digits = 2) {
  return Number(value.toFixed(digits));
}

function buildSparkline(price: number, changePercent: number, latencyMs: number) {
  const base = Math.max(price * 0.009, 0.5);
  const drift = Math.max(Math.abs(changePercent) * 0.22, 0.12);
  const latencyFactor = Math.max(0.12, Math.min(latencyMs / 6000, 0.72));

  return Array.from({ length: 8 }, (_, index) => {
    const wave = Math.sin(index * 0.92) * base * 0.35;
    const slope = (index - 3.5) * drift * base * 0.08;
    return round(price - base + index * base * 0.32 + wave + slope + latencyFactor, 4);
  });
}

function mapUpdateToSnapshot(update: HermesParsedPriceUpdate): OracleFeedSnapshot | null {
  const definition = FEED_MAP.get(update.id.toLowerCase().replace(/^0x/, ""));

  if (!definition) {
    return null;
  }

  const price = scaleOracleNumber(update.price.price, update.price.expo);
  const confidence = scaleOracleNumber(update.price.conf, update.price.expo);
  const emaValue = update.ema_price
    ? scaleOracleNumber(update.ema_price.price, update.ema_price.expo)
    : price;
  const changePercent = emaValue === 0 ? 0 : ((price - emaValue) / emaValue) * 100;
  const latencyMs = Math.max(0, Date.now() - update.price.publish_time * 1000);

  return {
    ...definition,
    price,
    confidence,
    changePercent: round(changePercent, 2),
    publishTime: update.price.publish_time,
    latencyMs,
    sparkline: buildSparkline(price, changePercent, latencyMs),
  };
}

export function toHermesFeedId(feedId: string) {
  return feedId.startsWith("0x") ? feedId : `0x${feedId}`;
}

export function parseHermesLatestResponse(payload: HermesLatestResponse) {
  return (payload.parsed ?? [])
    .map((item) => mapUpdateToSnapshot(item))
    .filter((item): item is OracleFeedSnapshot => Boolean(item));
}

export function parseHermesStreamEvent(payload: string) {
  const data = JSON.parse(payload) as HermesLatestResponse;
  return parseHermesLatestResponse(data);
}

export function mergeOracleSnapshots(
  current: OracleFeedSnapshot[],
  incoming: OracleFeedSnapshot[],
) {
  const merged = new Map(current.map((feed) => [feed.id, feed] as const));

  for (const feed of incoming) {
    merged.set(feed.id, feed);
  }

  return ORACLE_FEEDS.map((feed) => merged.get(feed.id)).filter(
    (feed): feed is OracleFeedSnapshot => Boolean(feed),
  );
}

export function getOracleFeedIds() {
  return ORACLE_FEEDS.map((feed) => feed.id);
}
