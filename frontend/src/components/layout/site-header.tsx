"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

import { NAV_ITEMS } from "@/lib/data/site";
import { cn } from "@/lib/utils/cn";
import { buttonVariants } from "@/components/ui/button";
import { Icon } from "@/components/ui/icon";

export function SiteHeader() {
  const pathname = usePathname();

  return (
    <header className="sticky top-0 z-40 px-4 pt-4 sm:px-6 lg:px-8">
      <div className="shell-container">
        <div className="surface-panel flex items-center gap-4 rounded-[1.75rem] px-3 py-3">
          <Link href="/" className="flex min-w-0 items-center gap-3 rounded-2xl px-2 py-1.5">
            <span className="flex h-11 w-11 items-center justify-center rounded-[1rem] border border-white/8 bg-white/5 text-purple shadow-[0_0_30px_rgba(183,126,255,0.15)]">
              <Icon name="brand" />
            </span>
            <span className="min-w-0">
              <span className="block text-[0.58rem] font-medium tracking-[0.44em] text-muted uppercase">
                Solana Oracle
              </span>
              <span className="block text-[1.1rem] leading-none font-semibold tracking-[-0.035em] text-foreground">
                SOL Oracle
              </span>
            </span>
          </Link>

          <nav className="mx-auto hidden items-center gap-2 md:flex">
            {NAV_ITEMS.map((item) => {
              const active = pathname === item.href;

              return (
                <Link
                  key={item.href}
                  href={item.href}
                  className={cn(
                    "rounded-2xl px-4 py-2.5 text-sm font-medium text-subtle transition hover:text-foreground",
                    active && "bg-white/8 text-foreground shadow-[inset_0_0_0_1px_rgba(255,255,255,0.05)]",
                  )}
                >
                  {item.label}
                </Link>
              );
            })}
          </nav>

          <Link
            href="/price-feeds"
            className={buttonVariants({
              variant: "primary",
              size: "default",
            })}
          >
            Connect Wallet
          </Link>
        </div>
      </div>
    </header>
  );
}
