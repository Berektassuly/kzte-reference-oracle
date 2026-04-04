import { cn } from "@/lib/utils/cn";

export function Sparkline({
  values,
  className,
  stroke = "#61f2b1",
  fill = "rgba(97, 242, 177, 0.14)",
}: {
  values: number[];
  className?: string;
  stroke?: string;
  fill?: string;
}) {
  const min = Math.min(...values);
  const max = Math.max(...values);
  const width = 160;
  const height = 68;
  const points = values.map((value, index) => {
    const x = (index / Math.max(values.length - 1, 1)) * width;
    const y =
      max === min ? height / 2 : height - ((value - min) / (max - min)) * (height - 8) - 4;
    return `${x},${y}`;
  });

  return (
    <svg viewBox={`0 0 ${width} ${height}`} className={cn("h-full w-full", className)} fill="none">
      <path d={`M0 ${height} L ${points.join(" L ")} L ${width} ${height} Z`} fill={fill} />
      <polyline points={points.join(" ")} stroke={stroke} strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}
