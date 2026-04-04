import { cn } from "@/lib/utils/cn";

type IconName =
  | "brand"
  | "database"
  | "spark"
  | "flow"
  | "chart"
  | "shield"
  | "delivery"
  | "code"
  | "flash"
  | "grid"
  | "sol"
  | "btc"
  | "eth"
  | "jup"
  | "pyth"
  | "bonk"
  | "orca"
  | "hnt";

const iconClass = "h-5 w-5 stroke-current";

export function Icon({
  name,
  className,
}: {
  name: IconName;
  className?: string;
}) {
  const classes = cn(iconClass, className);

  switch (name) {
    case "brand":
      return (
        <svg viewBox="0 0 24 24" fill="none" className={classes}>
          <path d="M8 12.2a3.2 3.2 0 1 0 0-6.4 3.2 3.2 0 0 0 0 6.4Z" strokeWidth="1.6" />
          <path d="M16 18.2a3.2 3.2 0 1 0 0-6.4 3.2 3.2 0 0 0 0 6.4Z" strokeWidth="1.6" />
          <path d="m10.7 8.8 2.8 2.1" strokeWidth="1.6" strokeLinecap="round" />
          <path d="m10.8 15.2 2.8-2" strokeWidth="1.6" strokeLinecap="round" />
        </svg>
      );
    case "database":
      return (
        <svg viewBox="0 0 24 24" fill="none" className={classes}>
          <ellipse cx="12" cy="6.5" rx="5.75" ry="2.75" strokeWidth="1.6" />
          <path d="M6.25 6.5v10.5c0 1.52 2.57 2.75 5.75 2.75s5.75-1.23 5.75-2.75V6.5" strokeWidth="1.6" />
          <path d="M6.25 11.75c0 1.52 2.57 2.75 5.75 2.75s5.75-1.23 5.75-2.75" strokeWidth="1.6" />
        </svg>
      );
    case "spark":
      return (
        <svg viewBox="0 0 24 24" fill="none" className={classes}>
          <path d="M12 4v4m0 8v4M4 12h4m8 0h4m-2.8-5.2-2.8 2.8m-5.6 5.6-2.8 2.8m0-11.2 2.8 2.8m5.6 5.6 2.8 2.8" strokeWidth="1.6" strokeLinecap="round" />
        </svg>
      );
    case "flow":
      return (
        <svg viewBox="0 0 24 24" fill="none" className={classes}>
          <path d="M7 6.2h10l-4.4 5.2H17L7 17.8h5.6" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" />
        </svg>
      );
    case "chart":
      return (
        <svg viewBox="0 0 24 24" fill="none" className={classes}>
          <path d="M5 18V9m5 9V6m5 12v-7m4 7H3" strokeWidth="1.6" strokeLinecap="round" />
          <path d="m5 12.8 5-3.1 5 1.6 4-4.3" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" />
        </svg>
      );
    case "shield":
      return (
        <svg viewBox="0 0 24 24" fill="none" className={classes}>
          <path d="M12 3.8 18.4 6v5.4c0 4.1-2.4 7-6.4 8.8-4-1.8-6.4-4.7-6.4-8.8V6L12 3.8Z" strokeWidth="1.6" />
          <path d="m9.2 12.4 1.9 1.9 3.7-4.1" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" />
        </svg>
      );
    case "delivery":
      return (
        <svg viewBox="0 0 24 24" fill="none" className={classes}>
          <rect x="5" y="4.75" width="8.5" height="14.5" rx="2.2" strokeWidth="1.6" />
          <path d="M13.5 9H16l2.2 2.7V16a2.2 2.2 0 0 1-2.2 2.2h-2.5" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" />
          <circle cx="9" cy="18" r="1.35" strokeWidth="1.6" />
          <circle cx="16.1" cy="18" r="1.35" strokeWidth="1.6" />
        </svg>
      );
    case "code":
      return (
        <svg viewBox="0 0 24 24" fill="none" className={classes}>
          <path d="m8 8-4 4 4 4m8-8 4 4-4 4m-5-8-2 8" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" />
        </svg>
      );
    case "flash":
      return (
        <svg viewBox="0 0 24 24" fill="none" className={classes}>
          <path d="M13.2 3.8 6.8 12h4.3l-1 8.2L17.2 12H13l.2-8.2Z" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" />
        </svg>
      );
    case "grid":
      return (
        <svg viewBox="0 0 24 24" fill="none" className={classes}>
          <rect x="4.5" y="4.5" width="5.5" height="5.5" rx="1.2" strokeWidth="1.6" />
          <rect x="14" y="4.5" width="5.5" height="5.5" rx="1.2" strokeWidth="1.6" />
          <rect x="4.5" y="14" width="5.5" height="5.5" rx="1.2" strokeWidth="1.6" />
          <rect x="14" y="14" width="5.5" height="5.5" rx="1.2" strokeWidth="1.6" />
        </svg>
      );
    case "sol":
      return tokenIcon(classes, "#70e7b7");
    case "btc":
      return tokenCircle(classes, "#f59e0b", "B");
    case "eth":
      return tokenDiamond(classes, "#7a8cff");
    case "jup":
      return tokenCircle(classes, "#8d57ec", "J");
    case "pyth":
      return tokenCircle(classes, "#61d7ff", "P");
    case "bonk":
      return tokenCircle(classes, "#4db8ff", "B");
    case "orca":
      return tokenCircle(classes, "#57f0c5", "O");
    case "hnt":
      return tokenCircle(classes, "#89f18f", "H");
    default:
      return null;
  }
}

function tokenIcon(classes: string, fill: string) {
  return (
    <svg viewBox="0 0 24 24" fill="none" className={classes}>
      <path d="M6.5 7.5h10.8M5.8 11.9H16.6M6.5 16.3h10.8" stroke={fill} strokeWidth="2" strokeLinecap="round" />
    </svg>
  );
}

function tokenDiamond(classes: string, fill: string) {
  return (
    <svg viewBox="0 0 24 24" fill="none" className={classes}>
      <path d="m12 4 4.9 8L12 20l-4.9-8L12 4Z" fill={fill} fillOpacity="0.22" stroke={fill} strokeWidth="1.4" />
    </svg>
  );
}

function tokenCircle(classes: string, fill: string, label: string) {
  return (
    <svg viewBox="0 0 24 24" fill="none" className={classes}>
      <circle cx="12" cy="12" r="8.2" fill={fill} fillOpacity="0.18" stroke={fill} strokeWidth="1.4" />
      <text x="12" y="15" textAnchor="middle" fontSize="8.5" fill={fill} fontWeight="700" fontFamily="sans-serif">
        {label}
      </text>
    </svg>
  );
}
