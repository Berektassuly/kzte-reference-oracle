"use client";

import Link from "next/link";
import { useDeferredValue, type ReactNode } from "react";

import { useLiveOracleFeeds } from "@/components/live/use-live-oracle-feeds";
import { buttonVariants } from "@/components/ui/button";
import { Icon } from "@/components/ui/icon";
import type { OracleFeedSnapshot } from "@/lib/types/site";
import { cn } from "@/lib/utils/cn";

function formatUsd(value: number) {
  return new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: "USD",
    minimumFractionDigits: value >= 100 ? 2 : 4,
    maximumFractionDigits: value >= 100 ? 2 : 4,
  }).format(value);
}

function formatCompact(value: number) {
  return new Intl.NumberFormat("en-US", {
    notation: "compact",
    maximumFractionDigits: 1,
  }).format(value);
}

export function LiveHomeSpotlight({
  initialFeeds,
}: {
  initialFeeds: OracleFeedSnapshot[];
}) {
  const { feeds, primaryFeed, streamState } = useLiveOracleFeeds(initialFeeds);
  const deferredPrimary = useDeferredValue(primaryFeed);

  const barValues =
    deferredPrimary?.sparkline.map((point, index) => {
      const min = Math.min(...deferredPrimary.sparkline);
      const max = Math.max(...deferredPrimary.sparkline);
      const normalized =
        max === min ? 62 : 32 + ((point - min) / (max - min)) * 58;

      return {
        key: `${deferredPrimary.id}-${index}`,
        height: `${normalized}%`,
      };
    }) ?? [];

  return (
    <section className="section-space pt-10 sm:pt-14">
      <div className="shell-container">
        <div className="surface-panel overflow-hidden rounded-[2.2rem] px-6 py-8 sm:px-10 sm:py-10 lg:px-14 lg:py-12">
          <div className="pointer-events-none absolute inset-x-0 top-0 h-56 bg-[radial-gradient(circle_at_top,_rgba(183,126,255,0.17),_transparent_62%)]" />
          <div className="relative text-center">
            <p className="eyebrow text-green">Verifiable Oracle</p>
            <h1 className="mx-auto mt-5 max-w-[10ch] text-5xl leading-[0.9] font-semibold tracking-[-0.05em] text-foreground sm:text-6xl lg:text-[4.7rem]">
              SOL Oracle
            </h1>
            <p className="mx-auto mt-5 max-w-2xl text-[0.98rem] leading-8 text-subtle sm:text-[1.02rem]">
              Real-time Solana price infrastructure for protocols, market makers, and
              low-latency execution paths powered by the Pyth Network oracle surface.
            </p>
            <div className="mt-8 flex flex-col items-center justify-center gap-3 sm:flex-row">
              <Link href="/price-feeds" className={buttonVariants({ variant: "primary", size: "hero" })}>
                Explore Feeds
              </Link>
              <Link href="/developers" className={buttonVariants({ variant: "secondary", size: "hero" })}>
                View Docs
              </Link>
            </div>
          </div>

          <div className="relative mx-auto mt-10 max-w-4xl rounded-[1.9rem] border border-white/6 bg-[#0b0d12] p-4 shadow-[0_28px_100px_rgba(3,4,8,0.56)] sm:p-5">
            <div className="grid gap-4 lg:grid-cols-[1.3fr_0.7fr]">
              <div className="rounded-[1.55rem] border border-white/5 bg-[linear-gradient(180deg,rgba(97,242,177,0.05),rgba(97,242,177,0.01))] p-5">
                <div className="flex items-start justify-between gap-4">
                  <div>
                    <p className="eyebrow">SOL / USD</p>
                    <p className="mt-3 text-4xl leading-none font-semibold tracking-[-0.05em] text-foreground sm:text-[3.1rem]">
                      {deferredPrimary ? formatUsd(deferredPrimary.price) : "--"}
                    </p>
                  </div>
                  <div className="rounded-2xl border border-purple/18 bg-purple/10 px-3 py-2 text-right">
                    <p className="text-[0.62rem] font-medium tracking-[0.34em] text-muted uppercase">
                      Network State
                    </p>
                    <p
                      className={cn(
                        "mt-2 text-sm font-medium",
                        streamState === "live" ? "text-green" : "text-subtle",
                      )}
                    >
                      {streamState === "live" ? "Live" : "Syncing"}
                    </p>
                  </div>
                </div>

                <div className="mt-8 flex h-[172px] items-end gap-2">
                  {barValues.map((bar) => (
                    <div
                      key={bar.key}
                      className="flex-1 rounded-t-xl bg-[linear-gradient(180deg,rgba(97,242,177,0.88),rgba(97,242,177,0.28))] shadow-[0_0_24px_rgba(97,242,177,0.12)]"
                      style={{ height: bar.height }}
                    />
                  ))}
                </div>
              </div>

              <div className="space-y-3 rounded-[1.55rem] border border-white/5 bg-white/[0.035] p-4">
                <StatRow label="Asset" value={deferredPrimary?.asset ?? "SOL"} accent="text-green" />
                <StatRow
                  label="Confidence"
                  value={deferredPrimary ? formatUsd(deferredPrimary.confidence) : "--"}
                />
                <StatRow
                  label="Latency"
                  value={deferredPrimary ? `${Math.round(deferredPrimary.latencyMs)}ms` : "--"}
                />
                <StatRow
                  label="Change"
                  value={
                    deferredPrimary
                      ? `${deferredPrimary.changePercent > 0 ? "+" : ""}${deferredPrimary.changePercent.toFixed(2)}%`
                      : "--"
                  }
                  accent={deferredPrimary && deferredPrimary.changePercent >= 0 ? "text-green" : "text-danger"}
                />
              </div>
            </div>
          </div>

          <div className="relative mt-6 grid gap-4 lg:grid-cols-3">
            <MiniInfoCard
              icon={<Icon name="flash" className="h-4 w-4" />}
              title="Fast Medianization"
              description="Publisher set converges to a stable quote with deterministic clipping and fresh-heartbeat enforcement."
            />
            <MiniInfoCard
              icon={<Icon name="shield" className="h-4 w-4" />}
              title="Signed Price Bus"
              description="Each update remains traceable to the oracle publisher mesh instead of an opaque aggregator."
            />
            <MiniInfoCard
              icon={<Icon name="delivery" className="h-4 w-4" />}
              title="Pyth Delegation"
              description="Live values stream from Hermes and remain ready for Solana-native routing and settlement surfaces."
            />
          </div>
        </div>

        <div className="mt-16">
          <div className="flex items-end justify-between gap-4">
            <div>
              <p className="eyebrow">Market Equalizer</p>
              <h2 className="mt-4 text-[2rem] leading-none font-semibold tracking-[-0.045em] text-foreground">
                Live benchmark matrix
              </h2>
              <p className="mt-4 max-w-2xl text-sm leading-7 text-subtle">
                A Pyth-powered snapshot of the feeds with the strongest routing relevance across
                the Solana stack.
              </p>
            </div>
            <p className="hidden text-sm text-purple sm:block">Source: Pyth Network</p>
          </div>

          <div className="mt-6 space-y-3">
            {feeds.slice(0, 3).map((feed) => (
              <div
                key={feed.id}
                className="surface-panel flex items-center justify-between gap-5 rounded-[1.4rem] px-5 py-4"
              >
                <div className="flex items-center gap-4">
                  <span className="flex h-10 w-10 items-center justify-center rounded-2xl border border-white/6 bg-white/[0.045]" style={{ color: feed.accent }}>
                    <Icon name={feed.icon} className="h-5 w-5" />
                  </span>
                  <div>
                    <p className="text-sm font-medium text-foreground">{feed.symbol}</p>
                    <p className="text-xs tracking-[0.25em] text-muted uppercase">{feed.venue}</p>
                  </div>
                </div>
                <div className="text-right">
                  <p className="text-lg font-semibold tracking-[-0.03em] text-foreground">
                    {formatUsd(feed.price)}
                  </p>
                  <p className={cn("text-sm", feed.changePercent >= 0 ? "text-green" : "text-danger")}>
                    {feed.changePercent >= 0 ? "+" : ""}
                    {feed.changePercent.toFixed(2)}%
                  </p>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}

function StatRow({
  label,
  value,
  accent,
}: {
  label: string;
  value: string;
  accent?: string;
}) {
  return (
    <div className="rounded-[1.15rem] border border-white/5 bg-black/30 px-4 py-3">
      <p className="text-[0.62rem] font-medium tracking-[0.3em] text-muted uppercase">{label}</p>
      <p className={cn("mt-2 text-[1.02rem] font-medium text-foreground", accent)}>{value}</p>
    </div>
  );
}

function MiniInfoCard({
  icon,
  title,
  description,
}: {
  icon: ReactNode;
  title: string;
  description: string;
}) {
  return (
    <div className="rounded-[1.5rem] border border-white/6 bg-white/[0.035] px-5 py-5 backdrop-blur-sm">
      <div className="flex h-10 w-10 items-center justify-center rounded-2xl border border-white/6 bg-white/[0.05] text-purple">
        {icon}
      </div>
      <h3 className="mt-4 text-lg font-semibold tracking-[-0.03em] text-foreground">{title}</h3>
      <p className="mt-3 text-sm leading-7 text-subtle">{description}</p>
    </div>
  );
}
