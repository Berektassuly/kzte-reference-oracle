import type { ReactNode } from "react";

import { cn } from "@/lib/utils/cn";

export function FeatureCard({
  icon,
  eyebrow,
  title,
  description,
  footer,
  className,
}: {
  icon: ReactNode;
  eyebrow?: string;
  title: string;
  description: string;
  footer?: ReactNode;
  className?: string;
}) {
  return (
    <article className={cn("surface-panel h-full p-6", className)}>
      <div className="flex h-full flex-col gap-5">
        <div className="flex h-11 w-11 items-center justify-center rounded-2xl border border-white/6 bg-white/4 text-purple">
          {icon}
        </div>
        <div className="space-y-3">
          {eyebrow ? <p className="eyebrow">{eyebrow}</p> : null}
          <h3 className="text-[1.28rem] font-semibold tracking-[-0.03em] text-foreground">{title}</h3>
          <p className="text-sm leading-7 text-subtle">{description}</p>
        </div>
        {footer ? <div className="mt-auto text-sm text-subtle">{footer}</div> : null}
      </div>
    </article>
  );
}
