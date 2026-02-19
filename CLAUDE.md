# wallchemybar — Project Guide for Claude

## Project Overview
A macOS menu bar / system tray app for browsing and applying wallpapers from Wallhaven.cc. Built with Tauri 2 (Rust backend) + SvelteKit (TypeScript/Svelte 5 frontend).

## Stack
- **Frontend**: SvelteKit v2, Svelte 5 (runes), TypeScript, Vite — in `src/`
- **Backend**: Tauri 2, Rust — in `src-tauri/src/`
- **IPC**: `@tauri-apps/api` `invoke()` calls from frontend to Rust `#[tauri::command]` handlers
- **UI**: Tailwind CSS v3 + DaisyUI v4 (`dim` theme)

## Project Structure
```
src/
  app.css            # @tailwind directives + global custom scrollbar
  app.html           # HTML shell (data-theme="dim" on <html>)
  routes/
    +layout.svelte   # Imports app.css globally
    +layout.ts       # SSR disabled (SPA mode for Tauri)
    +page.svelte     # Entire UI (~900 lines), all views in one file
src-tauri/src/
  main.rs            # Entry point
  lib.rs             # Tauri setup, tray icon, window management, command registration
  wallhaven.rs       # Wallhaven API: search, collections, tags, set wallpaper
  history.rs         # History persistence (history.json)
  queue.rs           # Queue persistence (queue.json)
  settings.rs        # Settings persistence (settings.json)
  setwallpaper.rs    # Platform-specific wallpaper setting (AppleScript/dbus/Win32)
postcss.config.js    # PostCSS pipeline (tailwindcss + autoprefixer)
tailwind.config.js   # Tailwind config — DaisyUI plugin, dim theme only, content: src/**
```

## Key Patterns

### Frontend
- **Svelte 5 runes** for all state: `$state`, `$derived`, `$effect`
- All views (search, query, collections, history, queue, settings) live in `+page.svelte`
- `activeView` discriminated union determines which panel renders
- Inline SVG icons (no icon library)
- Console logs prefixed with `[wallchemybar]`
- Infinite scroll: loads next page when 200px from bottom, 24 items per page
- Pagination deduplicates by wallpaper ID

### Styling
- **Tailwind CSS v3** utility classes throughout the template
- **DaisyUI v4** components: `btn`, `join`, `input-bordered`, `select-bordered`, `loading`
- Theme: `dim` — activated via `data-theme="dim"` on `<html>`
- DaisyUI color vars in custom CSS: `oklch(var(--p))` (primary), `oklch(var(--bc))` (base-content), etc. — DaisyUI v4 uses OKLCH not HSL
- Scoped `<style>` block kept minimal (~80 lines): nav button `::before` indicator, thumb hover filter, queue drag states, queue-icon visibility
- Custom scrollbar defined in `app.css` (global scope)
- Glassmorphism overlays: `bg-black/50 backdrop-blur-[8px]` on thumbnail icon buttons
- Sidebar: `w-[46px]`, `bg-base-200`, nav buttons use `nav-btn` class (custom CSS for active `::before` pseudo-element)

### Backend
- All Tauri commands registered in `lib.rs` via `invoke_handler!`
- Logging via `env_logger`: info (main flow), debug (detail), warn/error (problems)
- Data persisted as JSON in `~/Library/Application Support/com.dave.wallchemybar/`
- Smart caching: wallpapers cached in app cache dir before being applied
- API key fallback: retries with public purity (SFW-only) on 401

### Shared Data Types
```typescript
interface Wallpaper { id, url, path, thumbs: { large, original, small }, resolution, tags? }
interface Tag { id, name }
interface Collection { id, label, views, public, count }
```
Rust equivalents use `#[derive(Serialize, Deserialize)]`.

## Tauri App Config
- Identifier: `com.dave.wallchemybar`
- Window: 500×500px, hidden by default, no decorations, always-on-top, skip taskbar
- Dev server: `http://localhost:1420`

## Design System
- DaisyUI `dim` theme: `bg-base-100` (darkest), `bg-base-200` (sidebar), `text-base-content`, `primary` accent
- Sidebar: 46px wide, icon buttons with active highlight + left accent bar (via `::before`)
- Dark theme throughout, custom scrollbars (6px, subtle white thumb), glassmorphism overlays
- Settings toggles: DaisyUI `join` grouped buttons with semantic colors (success/warning/error/info)
- Queue interval: custom number input (default 30 min, 1–1440 range), no spin arrows

## Dev Commands
```bash
npm run tauri dev    # Run app with hot reload
npm run tauri build  # Production build
npm run check        # TypeScript + Svelte type check
```

## User Preferences
- `cnp` = commit and push
