import { Icon } from "@/components/ui/icon";
import { Sparkline } from "@/components/ui/sparkline";
import { fetchLatestOracleFeeds } from "@/lib/oracle/live-feeds-server";
import { cn } from "@/lib/utils/cn";

export const dynamic = "force-dynamic";

function formatUsd(value: number) {
  return new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: "USD",
    minimumFractionDigits: value >= 100 ? 2 : 4,
    maximumFractionDigits: value >= 100 ? 2 : 4,
  }).format(value);
}

const matrixIcons = [
  { name: "database" as const, tone: "text-white/80" },
  { name: "grid" as const, tone: "text-white/80" },
  { name: "spark" as const, tone: "text-white/80" },
  { name: "chart" as const, tone: "text-white/80" },
];

export default async function HowItWorksPage() {
  const feeds = await fetchLatestOracleFeeds();
  const venueRows = feeds.slice(0, 4);

  return (
    <section className="section-space pt-10 sm:pt-14">
      <div className="shell-container">
        <div className="grid gap-8 lg:grid-cols-[0.95fr_0.92fr] lg:gap-12">
          <div className="space-y-12">
            <div className="space-y-5 pt-6">
              <p className="eyebrow">How It Works</p>
              <h1 className="max-w-[10ch] text-5xl leading-[0.9] font-semibold tracking-[-0.055em] text-foreground sm:text-6xl lg:text-[4.4rem]">
                The Architecture of Truth.
              </h1>
              <p className="max-w-2xl text-[0.98rem] leading-8 text-subtle">
                The oracle pipeline is organized as a deterministic sequence: normalize signed
                venue inputs, aggregate across volatility, and publish a quote surface that
                downstream consumers can verify at account-read time.
              </p>
            </div>

            <div className="space-y-5">
              <div className="divider-line" />
              <div className="space-y-4">
                <p className="eyebrow text-purple">Input Layer</p>
                <h2 className="text-[2rem] leading-none font-semibold tracking-[-0.045em] text-foreground">
                  Data Sources.
                </h2>
                <p className="max-w-xl text-sm leading-7 text-subtle">
                  Signed venue packets arrive from spot books, route aggregators, and market-maker
                  channels before normalization into a common payload.
                </p>
              </div>

              <div className="flex flex-wrap gap-2">
                {["Jupiter Routes", "Phoenix Books", "Signed Venues"].map((chip) => (
                  <span
                    key={chip}
                    className="rounded-full border border-white/6 bg-white/[0.04] px-3 py-2 text-[0.7rem] font-medium tracking-[0.26em] text-subtle uppercase"
                  >
                    {chip}
                  </span>
                ))}
              </div>

              <div className="surface-panel rounded-[1.7rem] p-4 sm:p-5">
                <p className="eyebrow">Signed Venue Snapshots</p>
                <div className="mt-4 space-y-4">
                  {venueRows.map((feed) => (
                    <div key={feed.id} className="flex items-center justify-between gap-5 border-b border-white/6 pb-4 last:border-none last:pb-0">
                      <div>
                        <p className="text-sm font-medium text-foreground">{feed.symbol}</p>
                        <p className="text-[0.7rem] tracking-[0.24em] text-muted uppercase">{feed.venue}</p>
                      </div>
                      <div className="flex items-center gap-4">
                        <div className="hidden h-10 w-28 sm:block">
                          <Sparkline values={feed.sparkline} stroke={feed.accent} fill={`${feed.accent}22`} />
                        </div>
                        <p className={cn("text-sm font-medium", feed.changePercent >= 0 ? "text-green" : "text-danger")}>
                          {feed.changePercent >= 0 ? "+" : ""}
                          {feed.changePercent.toFixed(2)}%
                        </p>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>

            <div className="space-y-5">
              <div className="divider-line" />
              <div className="space-y-4">
                <p className="eyebrow text-purple">Delivery Layer</p>
                <h2 className="text-[2rem] leading-none font-semibold tracking-[-0.045em] text-foreground">
                  On-chain Delivery
                </h2>
                <p className="max-w-xl text-sm leading-7 text-subtle">
                  Validated quotes are committed to deterministic account layouts, making freshness
                  and provenance visible at the point of read.
                </p>
              </div>

              <div className="surface-panel rounded-[1.55rem] p-4">
                <div className="space-y-3 rounded-[1.2rem] border border-white/6 bg-black/35 p-4">
                  <DeliveryRow label="Publish Tx" value="sol_oracle::publish" accent="text-green" />
                  <DeliveryRow label="Feed Account" value="/oracle/sol-usd" />
                  <DeliveryRow label="Freshness" value="< 1 slot drift" accent="text-green" />
                </div>
              </div>
            </div>
          </div>

          <div className="space-y-12 pt-2 lg:pt-28">
            <div className="rounded-[1.8rem] border border-white/6 bg-black/50 p-4 shadow-[0_30px_80px_rgba(1,2,8,0.42)]">
              <div className="grid grid-cols-2 gap-3">
                {matrixIcons.map((item, index) => (
                  <div
                    key={`${item.name}-${index}`}
                    className="flex aspect-[1.48/1] items-center justify-center rounded-[1.2rem] border border-white/6 bg-white/[0.03]"
                  >
                    <Icon name={item.name} className={cn("h-7 w-7", item.tone)} />
                  </div>
                ))}
              </div>
            </div>

            <div className="space-y-5">
              <div className="divider-line" />
              <div className="space-y-4">
                <p className="eyebrow text-purple">Consensus Layer</p>
                <h2 className="text-[2rem] leading-none font-semibold tracking-[-0.045em] text-foreground">
                  Aggregation Engine
                </h2>
                <p className="max-w-xl text-sm leading-7 text-subtle">
                  Medianization, volatility-aware weighting, and heartbeat enforcement create a
                  stable quote surface before publication.
                </p>
              </div>

              <div className="space-y-3">
                {[
                  "Cross-venue medianization",
                  "Outlier clipping on spread drift",
                  "Heartbeat and signer quorum gates",
                ].map((bullet) => (
                  <div key={bullet} className="flex items-center gap-3 text-sm text-subtle">
                    <span className="h-2.5 w-2.5 rounded-full bg-green shadow-[0_0_12px_rgba(97,242,177,0.65)]" />
                    <span>{bullet}</span>
                  </div>
                ))}
              </div>
            </div>

            <div className="surface-panel overflow-hidden rounded-[1.8rem] p-0">
              <div className="relative min-h-[270px] bg-[radial-gradient(circle_at_center,_rgba(141,87,236,0.28),_transparent_22%),linear-gradient(180deg,rgba(255,255,255,0.02),rgba(255,255,255,0))]">
                <div className="absolute inset-0 opacity-35 [background-image:linear-gradient(rgba(255,255,255,0.14)_1px,transparent_1px),linear-gradient(90deg,rgba(255,255,255,0.14)_1px,transparent_1px)] [background-size:72px_72px] [mask-image:radial-gradient(circle_at_center,black,transparent_72%)]" />
                <div className="absolute left-1/2 top-1/2 h-[84px] w-[176px] -translate-x-1/2 -translate-y-1/2 rounded-[1.4rem] border border-purple/24 bg-[linear-gradient(180deg,rgba(141,87,236,0.24),rgba(19,15,34,0.88))] shadow-[0_0_80px_rgba(141,87,236,0.34)]" />
                <div className="absolute left-1/2 top-1/2 flex h-[84px] w-[176px] -translate-x-1/2 -translate-y-1/2 flex-col items-center justify-center rounded-[1.4rem]">
                  <p className="text-4xl leading-none font-semibold tracking-[-0.05em] text-[#d7b6ff]">
                    400ms
                  </p>
                  <p className="mt-3 text-[0.64rem] font-medium tracking-[0.32em] text-muted uppercase">
                    Median Propagation
                  </p>
                </div>
                {[
                  "12% 50%",
                  "26% 34%",
                  "31% 66%",
                  "70% 40%",
                  "78% 64%",
                  "88% 48%",
                ].map((position) => {
                  const [left, top] = position.split(" ");

                  return (
                    <span
                      key={position}
                      className="absolute h-2.5 w-2.5 rounded-full bg-white/75 shadow-[0_0_18px_rgba(255,255,255,0.22)]"
                      style={{ left, top }}
                    />
                  );
                })}
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}

function DeliveryRow({
  label,
  value,
  accent,
}: {
  label: string;
  value: string;
  accent?: string;
}) {
  return (
    <div className="flex items-center justify-between gap-4">
      <p className="text-[0.7rem] tracking-[0.26em] text-muted uppercase">{label}</p>
      <p className={cn("text-sm font-medium text-foreground", accent)}>{value}</p>
    </div>
  );
}
