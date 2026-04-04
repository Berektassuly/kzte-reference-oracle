import { cn } from "@/lib/utils/cn";

type ButtonVariant = "primary" | "secondary" | "ghost";
type ButtonSize = "default" | "compact" | "hero";

export function buttonVariants({
  variant = "primary",
  size = "default",
  fullWidth = false,
}: {
  variant?: ButtonVariant;
  size?: ButtonSize;
  fullWidth?: boolean;
}) {
  return cn(
    "inline-flex items-center justify-center rounded-2xl border text-sm font-medium transition duration-200 ease-out",
    "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-purple/60",
    fullWidth && "w-full",
    size === "compact" && "h-11 px-4 text-xs",
    size === "default" && "h-12 px-5",
    size === "hero" && "h-[3.35rem] px-6 text-[0.92rem]",
    variant === "primary" &&
      "border-purple/40 bg-purple px-5 text-[#14081f] shadow-[0_0_40px_rgba(183,126,255,0.28)] hover:bg-[#c996ff]",
    variant === "secondary" &&
      "border-white/6 bg-white/6 text-foreground hover:border-white/12 hover:bg-white/10",
    variant === "ghost" &&
      "border-transparent bg-transparent text-subtle hover:border-white/8 hover:bg-white/6 hover:text-foreground",
  );
}
