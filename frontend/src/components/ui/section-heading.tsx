import { cn } from "@/lib/utils/cn";

export function SectionHeading({
  eyebrow,
  title,
  description,
  className,
}: {
  eyebrow: string;
  title: string;
  description?: string;
  className?: string;
}) {
  return (
    <div className={cn("space-y-4", className)}>
      <p className="eyebrow">{eyebrow}</p>
      <div className="space-y-3">
        <h2 className="max-w-[14ch] text-4xl leading-[0.95] font-semibold tracking-[-0.045em] text-balance text-foreground sm:text-5xl">
          {title}
        </h2>
        {description ? (
          <p className="max-w-2xl text-[0.98rem] leading-8 text-subtle">{description}</p>
        ) : null}
      </div>
    </div>
  );
}
