import type { ReactNode } from "react";

import { SiteFooter } from "@/components/layout/site-footer";
import { SiteHeader } from "@/components/layout/site-header";

export function SiteShell({ children }: { children: ReactNode }) {
  return (
    <div className="relative min-h-screen overflow-x-hidden">
      <div className="pointer-events-none fixed inset-x-0 top-0 h-[420px] bg-[radial-gradient(circle_at_top,_rgba(141,87,236,0.18),_transparent_62%)]" />
      <div className="pointer-events-none fixed inset-y-0 right-0 w-[360px] bg-[radial-gradient(circle_at_center,_rgba(97,242,177,0.14),_transparent_70%)] blur-2xl" />
      <SiteHeader />
      <main>{children}</main>
      <SiteFooter />
    </div>
  );
}
