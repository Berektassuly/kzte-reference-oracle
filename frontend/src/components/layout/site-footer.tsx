import Link from "next/link";

import { FOOTER_GROUPS } from "@/lib/data/site";

export function SiteFooter() {
  return (
    <footer className="px-4 pb-8 pt-18 sm:px-6 lg:px-8">
      <div className="shell-container">
        <div className="divider-line mb-10" />
        <div className="grid gap-10 lg:grid-cols-[1.4fr_repeat(3,minmax(0,0.6fr))]">
          <div className="space-y-4">
            <p className="eyebrow">SOL Oracle</p>
            <h2 className="max-w-sm text-2xl leading-tight font-semibold tracking-[-0.045em] text-foreground">
              Low-latency truth for Solana&apos;s hardest market paths.
            </h2>
            <p className="max-w-md text-sm leading-7 text-subtle">
              Built around live Pyth Network feeds, deterministic aggregation, and a cleaner
              trading-surface UI.
            </p>
          </div>

          {FOOTER_GROUPS.map((group) => (
            <div key={group.title} className="space-y-4">
              <p className="eyebrow">{group.title}</p>
              <div className="space-y-3">
                {group.items.map((item) => (
                  <Link
                    key={item.label}
                    href={item.href}
                    className="block text-sm text-subtle transition hover:text-foreground"
                  >
                    {item.label}
                  </Link>
                ))}
              </div>
            </div>
          ))}
        </div>
      </div>
    </footer>
  );
}
