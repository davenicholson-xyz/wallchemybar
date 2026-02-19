# wallchemybar — Project Guide for Claude

## Project Overview
A macOS menu bar / system tray app for browsing and applying wallpapers from Wallhaven.cc. Built with Tauri 2 (Rust backend) + SvelteKit (TypeScript/Svelte 5 frontend).

## Stack
- **Frontend**: SvelteKit v2, Svelte 5 (runes), TypeScript, Vite — in `src/`
- **Backend**: Tauri 2, Rust — in `src-tauri/src/`
- **IPC**: `@tauri-apps/api` `invoke()` calls from frontend to Rust `#[tauri::command]` handlers
- **No UI library** — vanilla CSS with dark theme

## Project Structure
```
src/routes/
  +page.svelte       # Entire UI (~1800 lines), all views in one file
  +layout.ts         # SSR disabled (SPA mode for Tauri)
src-tauri/src/
  main.rs            # Entry point
  lib.rs             # Tauri setup, tray icon, window management, command registration
  wallhaven.rs       # Wallhaven API: search, collections, tags, set wallpaper
  history.rs         # History persistence (history.json)
  queue.rs           # Queue persistence (queue.json)
  settings.rs        # Settings persistence (settings.json)
  setwallpaper.rs    # Platform-specific wallpaper setting (AppleScript/dbus/Win32)
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
- Background: `#161618`, text: `#f0f0f0`, accent: `#646cff`
- Sidebar: 46px wide, icon buttons with active highlight + left accent bar
- Dark theme throughout, custom scrollbars, glassmorphism icon overlays

## Dev Commands
```bash
npm run tauri dev    # Run app with hot reload
npm run tauri build  # Production build
npm run check        # TypeScript + Svelte type check
```

## User Preferences
- `cnp` = commit and push
