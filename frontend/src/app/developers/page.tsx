import Link from "next/link";

import { FeatureCard } from "@/components/ui/feature-card";
import { Icon } from "@/components/ui/icon";
import { buttonVariants } from "@/components/ui/button";

const codeSnippet = `import { HermesClient } from "@pythnetwork/hermes-client";

const hermes = new HermesClient("https://hermes.pyth.network");
const updates = await hermes.getLatestPriceUpdates([
  "0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d",
]);

const solUsd = updates.parsed[0];
publishQuote(solUsd.price.price, solUsd.price.expo);`;

export default function DevelopersPage() {
  return (
    <section className="section-space pt-10 sm:pt-14">
      <div className="shell-container">
        <div className="grid gap-8 lg:grid-cols-[0.78fr_1.22fr] lg:items-start">
          <div className="space-y-6 pt-6">
            <p className="eyebrow text-green">Developers Hub</p>
            <h1 className="max-w-[8ch] text-5xl leading-[0.9] font-semibold tracking-[-0.055em] text-foreground sm:text-6xl">
              Build with <span className="text-gradient">Instant Truth.</span>
            </h1>
            <p className="max-w-md text-[0.98rem] leading-8 text-subtle">
              A concise stack for publishers, indexers, and protocols that need low-latency Pyth
              price access without a noisy interface.
            </p>
            <div className="flex flex-col gap-3 sm:flex-row">
              <Link href="/price-feeds" className={buttonVariants({ variant: "primary", size: "hero" })}>
                Read Docs
              </Link>
              <Link href="/api/oracle/latest" className={buttonVariants({ variant: "secondary", size: "hero" })}>
                View API
              </Link>
            </div>
          </div>

          <div className="surface-panel overflow-hidden rounded-[1.9rem] p-5">
            <div className="flex items-center gap-2 pb-4">
              <span className="h-2.5 w-2.5 rounded-full bg-danger/80" />
              <span className="h-2.5 w-2.5 rounded-full bg-[#f5b055]/80" />
              <span className="h-2.5 w-2.5 rounded-full bg-green/80" />
            </div>
            <div className="rounded-[1.45rem] border border-white/6 bg-black/50 p-5">
              <pre className="overflow-x-auto font-mono text-[0.84rem] leading-7 text-subtle">
                <code>{codeSnippet}</code>
              </pre>
            </div>
          </div>
        </div>

        <div className="mt-16 space-y-5">
          <div>
            <p className="eyebrow">Developer Protocol</p>
            <h2 className="mt-4 text-[2rem] leading-none font-semibold tracking-[-0.045em] text-foreground">
              Components for realtime execution paths
            </h2>
          </div>

          <div className="grid gap-4 lg:grid-cols-3">
            <FeatureCard
              icon={<Icon name="flash" className="h-5 w-5" />}
              title="Fast Streams"
              description="Open Hermes-backed flows that sync directly into your market surfaces without extra ceremony."
              footer={<span className="text-purple">Live transport -&gt;</span>}
            />
            <FeatureCard
              icon={<Icon name="code" className="h-5 w-5 text-green" />}
              title="Typed Payloads"
              description="Keep feed parsing explicit so protocol state, confidence, and freshness stay inspectable in code."
              footer={<span className="text-green">Signed schema -&gt;</span>}
            />
            <FeatureCard
              icon={<Icon name="delivery" className="h-5 w-5 text-cyan" />}
              title="On-chain Routing"
              description="Bridge the feed layer into Solana programs, settlement surfaces, and venue-aware order logic."
              footer={<span className="text-cyan">Program-ready -&gt;</span>}
            />
          </div>
        </div>

        <div className="mt-16 grid gap-6 lg:grid-cols-[0.96fr_1.04fr]">
          <div className="surface-panel overflow-hidden rounded-[1.9rem] p-0">
            <div className="relative min-h-[270px] bg-[radial-gradient(circle_at_center,_rgba(255,255,255,0.12),_transparent_28%),linear-gradient(180deg,rgba(255,255,255,0.02),rgba(255,255,255,0))] p-6">
              <div className="absolute inset-0 opacity-35 [background-image:linear-gradient(rgba(255,255,255,0.12)_1px,transparent_1px),linear-gradient(90deg,rgba(255,255,255,0.12)_1px,transparent_1px)] [background-size:86px_86px] [mask-image:radial-gradient(circle_at_center,black,transparent_68%)]" />
              <div className="relative z-10 flex h-full flex-col justify-end">
                <p className="eyebrow">Signed Publisher Mesh</p>
                <p className="mt-4 max-w-[14ch] text-3xl leading-none font-semibold tracking-[-0.045em] text-foreground">
                  Route updates through the live oracle fabric.
                </p>
                <div className="mt-5 inline-flex rounded-full border border-white/7 bg-white/[0.05] px-4 py-2 text-sm text-green">
                  Pyth stream ready
                </div>
              </div>
            </div>
          </div>

          <div className="space-y-6">
            <p className="eyebrow text-purple">Engineered for high-frequency DeFi</p>
            <h2 className="max-w-[12ch] text-[2.6rem] leading-[0.95] font-semibold tracking-[-0.05em] text-foreground">
              Interface patterns that stay readable under load.
            </h2>
            <p className="max-w-2xl text-[0.98rem] leading-8 text-subtle">
              The code surface is intentionally minimal: faster scan paths, clearer state, and
              fewer ornamental wrappers between a protocol and its live oracle data.
            </p>
            <div className="space-y-4">
              {[
                "Hermes-backed fetch and stream routes sit inside App Router handlers.",
                "Live numbers hydrate immediately and continue updating through EventSource.",
                "Each page mirrors the Figma rhythm instead of default dashboard boilerplate.",
              ].map((bullet, index) => (
                <div key={bullet} className="flex gap-4">
                  <span className="mt-1.5 flex h-6 w-6 items-center justify-center rounded-full border border-white/8 bg-white/[0.05] text-xs font-medium text-foreground">
                    {index + 1}
                  </span>
                  <p className="text-sm leading-7 text-subtle">{bullet}</p>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
