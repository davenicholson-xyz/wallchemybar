# wallchemybar

A lightweight desktop wallpaper manager that lives in your menu bar. Browse, search, and apply wallpapers from [Wallhaven](https://wallhaven.cc) without ever opening a browser.

## Features

- **Browse** wallpapers by Latest, Hot, Top, or Random
- **Search** with full-text queries or find similar wallpapers
- **Collections** from your Wallhaven account
- **Preview** wallpapers with tags, resolution info, and one-click apply
- **History** of applied wallpapers with individual delete
- **Undo** to revert to your previous wallpaper instantly
- **Smart caching** so wallpapers are never downloaded twice
- **Content filtering** by category (General, Anime, People) and purity (SFW, Sketchy, NSFW)
- **Minimum resolution** filtering
- **Cross-platform** support for macOS, Linux, and Windows

## Screenshots

<!-- Add screenshots here -->

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/)
- [Rust](https://www.rust-lang.org/tools/install)
- [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

### Development

```bash
npm install
npm run tauri dev
```

### Build

```bash
npm run tauri build
```

## Configuration

Click the gear icon in the sidebar or use the tray menu to open Settings.

- **Wallhaven API Key** — required for collections and NSFW content. Get one at [wallhaven.cc/settings/account](https://wallhaven.cc/settings/account).
- **Categories & Purity** — filter the types of wallpapers shown.
- **Minimum Resolution** — only show wallpapers at or above a chosen resolution.

## Tech Stack

- [Tauri 2](https://v2.tauri.app/) (Rust backend)
- [SvelteKit](https://svelte.dev/) + TypeScript (frontend)
- [Wallhaven API](https://wallhaven.cc/help/api)

## Platform Support

| Platform | Wallpaper Setting Method |
|----------|--------------------------|
| macOS    | AppleScript              |
| Linux    | KDE, GNOME, Cinnamon, MATE, Budgie, XFCE |
| Windows  | Win32 API                |

## License

<!-- Add license here -->
