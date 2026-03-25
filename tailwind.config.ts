import type { Config } from "tailwindcss";

const config: Config = {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        obd: {
          bg: "#060913",
          surface: "#0C1220",
          card: "#111827",
          border: "#1E293B",
          "border-light": "#334155",
          accent: "#06B6D4",
          "accent-light": "#22D3EE",
          "accent-glow": "#06B6D440",
          success: "#10B981",
          warning: "#F59E0B",
          danger: "#EF4444",
          "danger-glow": "#EF444440",
          info: "#3B82F6",
          text: "#F1F5F9",
          "text-secondary": "#94A3B8",
          "text-muted": "#64748B",
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
