"use client";

import { Sparkline } from "@/components/ui/sparkline";
import { useLiveOracleFeeds } from "@/components/live/use-live-oracle-feeds";
import { Icon } from "@/components/ui/icon";
import type { OracleFeedSnapshot } from "@/lib/types/site";
import { cn } from "@/lib/utils/cn";

function formatUsd(value: number, compact = false) {
  return new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: "USD",
    minimumFractionDigits: compact ? 2 : value >= 100 ? 2 : 4,
    maximumFractionDigits: compact ? 2 : value >= 100 ? 2 : 4,
  }).format(value);
}

export function LivePriceFeedsShell({
  initialFeeds,
}: {
  initialFeeds: OracleFeedSnapshot[];
}) {
  const { feeds, streamState } = useLiveOracleFeeds(initialFeeds);
  const averageLag =
    feeds.reduce((sum, feed) => sum + feed.latencyMs, 0) / Math.max(feeds.length, 1);

  return (
    <section className="section-space pt-10 sm:pt-14">
      <div className="shell-container">
        <div className="space-y-8">
          <div className="space-y-4">
            <p className="eyebrow text-green">Live Price Feeds</p>
            <h1 className="max-w-[11ch] text-5xl leading-[0.92] font-semibold tracking-[-0.05em] text-foreground sm:text-6xl">
              Sub-Second <span className="text-gradient">Precision</span>
            </h1>
            <p className="max-w-3xl text-[0.98rem] leading-8 text-subtle">
              Live oracle snapshots flowing through Hermes with confidence bands, latency
              visibility, and execution-facing market structure.
            </p>
          </div>

          <div className="grid gap-4 md:grid-cols-3">
            <MetricCard label="Active Feeds" value={`${feeds.length}`} />
            <MetricCard label="Median Lag" value={`${Math.round(averageLag)}ms`} />
            <MetricCard label="Uptime" value={streamState === "live" ? "99.99% Stream" : "Reconnecting"} accent />
          </div>

          <div className="surface-panel overflow-hidden rounded-[1.9rem]">
            <div className="flex items-center justify-between gap-4 border-b border-white/6 px-5 py-4">
              <div>
                <p className="text-sm font-medium text-foreground">Realtime Feed Matrix</p>
                <p className="text-xs tracking-[0.25em] text-muted uppercase">Pyth / Hermes</p>
              </div>
              <div className="flex items-center gap-2 rounded-full border border-white/6 bg-white/[0.04] px-3 py-1.5">
                <span className={cn("h-2 w-2 rounded-full", streamState === "live" ? "bg-green" : "bg-muted")} />
                <span className="text-xs font-medium text-subtle">
                  {streamState === "live" ? "Streaming" : "Recovering"}
                </span>
              </div>
            </div>

            <div className="overflow-x-auto">
              <table className="min-w-full">
                <thead>
                  <tr className="border-b border-white/6 text-left text-[0.68rem] tracking-[0.28em] text-muted uppercase">
                    <th className="px-5 py-4 font-medium">Feed</th>
                    <th className="px-5 py-4 font-medium">Last Price</th>
                    <th className="px-5 py-4 font-medium">Confidence</th>
                    <th className="px-5 py-4 font-medium">Pattern</th>
                    <th className="px-5 py-4 font-medium">Publish Lag</th>
                  </tr>
                </thead>
                <tbody>
                  {feeds.map((feed) => (
                    <tr key={feed.id} className="border-b border-white/[0.05]">
                      <td className="px-5 py-4">
                        <div className="flex items-center gap-4">
                          <span className="flex h-11 w-11 items-center justify-center rounded-2xl border border-white/6 bg-white/[0.035]" style={{ color: feed.accent }}>
                            <Icon name={feed.icon} className="h-5 w-5" />
                          </span>
                          <div>
                            <p className="text-sm font-medium text-foreground">{feed.symbol}</p>
                            <p className="text-xs tracking-[0.24em] text-muted uppercase">{feed.venue}</p>
                          </div>
                        </div>
                      </td>
                      <td className="px-5 py-4">
                        <p className="text-lg font-semibold tracking-[-0.03em] text-foreground">
                          {formatUsd(feed.price)}
                        </p>
                        <p className={cn("text-sm", feed.changePercent >= 0 ? "text-green" : "text-danger")}>
                          {feed.changePercent >= 0 ? "+" : ""}
                          {feed.changePercent.toFixed(2)}%
                        </p>
                      </td>
                      <td className="px-5 py-4 text-sm text-subtle">{formatUsd(feed.confidence, true)}</td>
                      <td className="px-5 py-4">
                        <div className="h-12 w-[122px]">
                          <Sparkline values={feed.sparkline} stroke={feed.accent} fill={`${feed.accent}22`} />
                        </div>
                      </td>
                      <td className="px-5 py-4">
                        <p className="text-sm font-medium text-foreground">{Math.round(feed.latencyMs)}ms</p>
                        <p className="text-xs text-muted">{new Date(feed.publishTime * 1000).toLocaleTimeString("en-US", { hour: "2-digit", minute: "2-digit", second: "2-digit" })}</p>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>

          <div className="grid gap-4 lg:grid-cols-2">
            <div className="surface-panel rounded-[1.7rem] p-6">
              <p className="eyebrow">Multi-Venue Median</p>
              <h2 className="mt-4 text-2xl font-semibold tracking-[-0.04em] text-foreground">
                Confidence-aware routing layer
              </h2>
              <p className="mt-4 max-w-lg text-sm leading-7 text-subtle">
                Price selection favors the freshest signed venues and rejects drift outside the
                clipped spread band before values reach consumers.
              </p>
              <div className="mt-5 rounded-[1.2rem] border border-purple/18 bg-black/30 px-4 py-3 text-sm text-purple">
                0x PYTH parsed updates - live via Hermes stream
              </div>
            </div>

            <div className="surface-panel overflow-hidden rounded-[1.7rem] p-0">
              <div className="h-full min-h-[240px] bg-[radial-gradient(circle_at_center,_rgba(97,242,177,0.18),_transparent_26%),linear-gradient(135deg,rgba(255,255,255,0.03),rgba(255,255,255,0))] p-6">
                <p className="eyebrow">Cross-exchange Route</p>
                <h2 className="mt-4 text-2xl font-semibold tracking-[-0.04em] text-foreground">
                  Publisher mesh visibility
                </h2>
                <p className="mt-4 max-w-md text-sm leading-7 text-subtle">
                  The UI keeps the quote surface readable while still exposing enough route texture
                  for execution-side sanity checks.
                </p>
                <div className="mt-8 flex flex-wrap gap-3">
                  {["Medianized", "Fresh", "Signed", "Low Lag"].map((chip) => (
                    <span
                      key={chip}
                      className="rounded-full border border-white/7 bg-white/[0.05] px-3 py-2 text-xs font-medium text-subtle"
                    >
                      {chip}
                    </span>
                  ))}
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}

function MetricCard({
  label,
  value,
  accent,
}: {
  label: string;
  value: string;
  accent?: boolean;
}) {
  return (
    <div className="surface-panel rounded-[1.5rem] px-5 py-5">
      <p className="text-[0.68rem] font-medium tracking-[0.28em] text-muted uppercase">{label}</p>
      <p className={cn("mt-3 text-3xl leading-none font-semibold tracking-[-0.05em] text-foreground", accent && "text-gradient")}>
        {value}
      </p>
    </div>
  );
}
