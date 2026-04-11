import { cn } from "@/lib/utils";

interface LogoProps {
  variant?: "light" | "dark";
  size?: "favicon" | "medium" | "full";
  className?: string;
}

export function Logo({ variant = "light", size = "full", className }: LogoProps) {
  const lineColor = variant === "dark" ? "#FFFFFF" : "#1B3A5C";
  const gold = "#D4A843";

  if (size === "favicon") {
    // 2 elements: outer ring + gold bindu
    return (
      <svg
        viewBox="0 0 100 100"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
        className={cn("shrink-0", className)}
        aria-label="Vedaksha logo"
      >
        <circle cx="50" cy="50" r="38" stroke={lineColor} strokeWidth="5" fill="none" />
        <circle cx="50" cy="50" r="12" fill={gold} />
      </svg>
    );
  }

  if (size === "medium") {
    // Simplified: bindu + inner ring + dashed outer ring + gold arc
    return (
      <svg
        viewBox="0 0 100 100"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
        className={cn("shrink-0", className)}
        aria-label="Vedaksha logo"
      >
        {/* Outer dashed ring */}
        <circle cx="50" cy="50" r="38" stroke={lineColor} strokeWidth="2" fill="none" strokeDasharray="6 4" />
        {/* Gold arc on lower half of outer ring */}
        <path d="M 17.2 68.5 A 38 38 0 0 0 82.8 68.5" stroke={gold} strokeWidth="2.5" fill="none" strokeLinecap="round" />
        {/* Inner solid ring */}
        <circle cx="50" cy="50" r="22" stroke={lineColor} strokeWidth="2" fill="none" />
        {/* Gold bindu */}
        <circle cx="50" cy="50" r="8" fill={gold} />
        <circle cx="50" cy="50" r="2.5" fill={lineColor} />
      </svg>
    );
  }

  // full — Orbital Astrolabe
  return (
    <svg
      viewBox="0 0 100 100"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      className={cn("shrink-0", className)}
      aria-label="Vedaksha logo"
    >
      {/* 6 radial lines from center */}
      {[0, 60, 120, 180, 240, 300].map((angle) => {
        const rad = (angle * Math.PI) / 180;
        const x1 = 50 + 10 * Math.cos(rad);
        const y1 = 50 - 10 * Math.sin(rad);
        const x2 = 50 + 44 * Math.cos(rad);
        const y2 = 50 - 44 * Math.sin(rad);
        return (
          <line
            key={angle}
            x1={x1}
            y1={y1}
            x2={x2}
            y2={y2}
            stroke={lineColor}
            strokeWidth="0.5"
            opacity="0.3"
          />
        );
      })}

      {/* Outer dashed orbit ring */}
      <circle
        cx="50" cy="50" r="38"
        stroke={lineColor}
        strokeWidth="1"
        fill="none"
        strokeDasharray="5 3.5"
      />

      {/* Gold arc on lower half of outer ring (~180 degrees) */}
      <path
        d="M 17.2 68.5 A 38 38 0 0 0 82.8 68.5"
        stroke={gold}
        strokeWidth="1.8"
        fill="none"
        strokeLinecap="round"
      />

      {/* Inner solid orbit ring */}
      <circle cx="50" cy="50" r="22" stroke={lineColor} strokeWidth="1.2" fill="none" />

      {/* Gold bindu (large, with dark center dot) */}
      <circle cx="50" cy="50" r="7" fill={gold} />
      <circle cx="50" cy="50" r="2" fill={lineColor} />

      {/* Gold planet marker on inner ring, top (~12 o'clock) */}
      <circle cx="50" cy="28" r="2" fill={gold} />
    </svg>
  );
}
