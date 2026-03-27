import type { Config } from "tailwindcss";

const config: Config = {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        obd: {
          bg: "rgb(var(--obd-bg) / <alpha-value>)",
          surface: "rgb(var(--obd-surface) / <alpha-value>)",
          card: "rgb(var(--obd-card) / <alpha-value>)",
          border: "rgb(var(--obd-border) / <alpha-value>)",
          "border-light": "rgb(var(--obd-border-light) / <alpha-value>)",
          accent: "#06B6D4",
          "accent-light": "#22D3EE",
          "accent-glow": "#06B6D440",
          success: "#10B981",
          warning: "#F59E0B",
          danger: "#EF4444",
          "danger-glow": "#EF444440",
          info: "#3B82F6",
          text: "rgb(var(--obd-text) / <alpha-value>)",
          "text-secondary": "rgb(var(--obd-text-secondary) / <alpha-value>)",
          "text-muted": "rgb(var(--obd-text-muted) / <alpha-value>)",
        },
      },
      fontFamily: {
        sans: [
          "SF Pro Display",
          "Inter",
          "-apple-system",
          "BlinkMacSystemFont",
          "Segoe UI",
          "Helvetica",
          "Arial",
          "sans-serif",
        ],
        mono: [
          "SF Mono",
          "JetBrains Mono",
          "Menlo",
          "Consolas",
          "monospace",
        ],
      },
      backdropBlur: {
        xs: "2px",
      },
      animation: {
        "pulse-slow": "pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite",
        "glow": "glow 2s ease-in-out infinite alternate",
        "slide-in": "slideIn 0.3s ease-out",
      },
      keyframes: {
        glow: {
          "0%": { boxShadow: "0 0 5px #06B6D420, 0 0 20px #06B6D410" },
          "100%": { boxShadow: "0 0 10px #06B6D440, 0 0 40px #06B6D420" },
        },
        slideIn: {
          "0%": { opacity: "0", transform: "translateX(-10px)" },
          "100%": { opacity: "1", transform: "translateX(0)" },
        },
      },
    },
  },
  plugins: [],
};

export default config;
