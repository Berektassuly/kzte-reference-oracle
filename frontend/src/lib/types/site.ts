export type NavItem = {
  href: string;
  label: string;
};

export type FooterGroup = {
  title: string;
  items: Array<{
    label: string;
    href: string;
  }>;
};

export type OracleFeedDefinition = {
  id: string;
  symbol: string;
  asset: string;
  venue: string;
  description: string;
  accent: string;
  icon: "sol" | "btc" | "eth" | "jup" | "pyth" | "bonk" | "orca" | "hnt";
};

export type OracleFeedSnapshot = OracleFeedDefinition & {
  price: number;
  confidence: number;
  changePercent: number;
  publishTime: number;
  latencyMs: number;
  sparkline: number[];
};
