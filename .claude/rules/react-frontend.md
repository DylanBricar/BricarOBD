---
description: React/TypeScript frontend rules for BricarOBD Tauri app — performance, memory, i18n
globs: src/**/*.{tsx,ts}
---

# React Frontend Rules

## i18n — Mandatory
- ALL user-visible strings MUST use `t("key")` from `useTranslation()`
- Never hardcode French or English text in JSX
- Add keys to BOTH FR and EN sections in `src/lib/i18n.ts`
- Dates: use `i18n.language === "fr" ? "fr-FR" : "en-US"` for locale
- Module-level constants with translated strings must be inside the component (not at file top)

## State Management
- Connection state: custom store in `stores/connection.ts` with `useEffect` subscription
- Vehicle data: `useVehicleData()` hook in `stores/vehicle.ts`
- `pollingModeRef` and `manufacturerRef` track current polling mode (demo vs real)
- Never use `useState` for subscription cleanup — always `useEffect` with return cleanup

## Memory Leak Prevention — Top 15 Rules
1. Always return cleanup function from `useEffect` — especially for `setInterval`
2. Use `useRef` for interval IDs — clear on unmount
3. Never add event listeners without removing them in cleanup
4. Cancel pending `invoke()` calls on unmount when possible
5. Cap array sizes (pidHistory: 120, logs: 5000, dtcHistory: saved to localStorage)
6. Use `useMemo` for expensive computations (filtered lists, sorted data)
7. Use `useCallback` for event handlers passed as props
8. Don't create objects/arrays inline in JSX — extract to constants or useMemo
9. Clear stale data on disconnect (`stopPolling` clears `pidData`)
10. Reset `selectedPids` on reconnect (detect 0→N transition with useRef)
11. Don't store large data in React state — use refs for buffers (recordBufferRef)
12. Cleanup `FileReader` and `URL.createObjectURL` after use
13. Don't override `console.log` permanently — restore in useEffect cleanup
14. Limit toast auto-dismiss with `setTimeout` and clear in cleanup
15. Use `key` prop on list items to prevent stale DOM nodes

## Tauri IPC
- Always use `invoke()` from `@tauri-apps/api/core` — never fake backend calls
- File downloads: use `invoke("save_csv_file")` — Blob URLs don't work in Tauri webview
- External links: use anchor tag with `target="_blank"` — `window.open()` doesn't work
- Database: auto-initialized at Rust startup — frontend can call commands immediately

## Component Patterns
- Toast: `{ message: string; type: "success" | "error" }` state + auto-dismiss 5s + X button
- Glass cards: use `glass-card` CSS class for consistent glassmorphism
- Nav items: `w-full border border-transparent` for consistent sizing
- Conditional render: `{data && <Component />}` — never render with placeholder 0 values

## Performance
- Polling intervals: configurable 500ms-5s, use `setInterval` with ref
- Large lists: only render visible items (PID table is already scrollable)
- Charts: `isAnimationActive={false}` on Recharts for real-time data
- Debounce search inputs if filtering large datasets
- Don't re-create polling functions on every render — use `useCallback`

## Dev Console
- Log prefix: `console.log("[BricarOBD] message")` — captured by DevConsole component
- Backend logs: polled via `invoke("get_dev_logs")` every 500ms
- Max 5000 log entries in frontend buffer

## File Naming
- Pages: PascalCase (`Dashboard.tsx`, `LiveData.tsx`)
- Components: PascalCase (`CircularGauge.tsx`, `DevConsole.tsx`)
- Stores: camelCase (`connection.ts`, `vehicle.ts`)
- Utils: camelCase (`utils.ts`, `i18n.ts`)

## CSS
- Use Tailwind utility classes — avoid custom CSS except in `globals.css`
- Custom component classes defined in `globals.css` `@layer components`
- Theme colors: `obd-*` prefix (bg, surface, card, border, accent, success, warning, danger)
- Responsive: mobile-first, but primary target is 1280x800 desktop
