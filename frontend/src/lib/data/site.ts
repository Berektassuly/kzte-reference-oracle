import type { FooterGroup, NavItem } from "@/lib/types/site";

export const NAV_ITEMS: NavItem[] = [
  { href: "/", label: "Home" },
  { href: "/how-it-works", label: "How It Works" },
  { href: "/developers", label: "Developers Hub" },
  { href: "/price-feeds", label: "Price Feeds" },
];

export const FOOTER_GROUPS: FooterGroup[] = [
  {
    title: "Platform",
    items: [
      { label: "Feeds", href: "/price-feeds" },
      { label: "How It Works", href: "/how-it-works" },
    ],
  },
  {
    title: "Developers",
    items: [
      { label: "Hub", href: "/developers" },
      { label: "Realtime API", href: "/api/oracle/latest" },
    ],
  },
  {
    title: "Network",
    items: [
      { label: "Pyth", href: "https://pyth.network/" },
      { label: "Hermes", href: "https://docs.pyth.network/price-feeds/core/api-instances-and-providers/hermes" },
    ],
  },
];
