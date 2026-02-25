# Plan: wallhaven-tui — CLI/TUI version of wallchemybar

## Context
A terminal-only sibling app to wallchemybar. Same Wallhaven API backend, same wallpaper-setting
logic, but the Tauri/Svelte frontend is replaced with a ratatui TUI + viuer inline image display.
The user will copy the current project and strip out the frontend entirely.

The Rust backend is already ~95% decoupled from Tauri — the only coupling is `tauri::AppHandle`
used for path resolution in ~15 function signatures. Decoupling those is the main migration task.

---

## New project structure

```
wallhaven-tui/
  src/
    main.rs           # tokio entry point, ratatui event loop
    app.rs            # App state machine + event dispatch
    dirs.rs           # AppDirs struct (replaces tauri AppHandle path resolution)
    api/
      mod.rs
      wallhaven.rs    # copied + decoupled from current wallhaven.rs
      settings.rs     # copied + decoupled
      history.rs      # copied + decoupled
      queue.rs        # copied + decoupled
      setwallpaper.rs # copied AS-IS — already 0% Tauri
    ui/
      mod.rs
      browse.rs       # thumbnail grid + search bar
      preview.rs      # right-pane image + metadata
      queue_view.rs   # queue list + cycling controls
      settings_view.rs
      status_bar.rs   # bottom key-hint bar
```

---

## Step 1 — New Cargo.toml

Remove all Tauri deps. Add:

```toml
[dependencies]
# TUI
ratatui       = "0.27"
crossterm     = "0.28"   # ratatui backend (cross-platform terminal control)

# Images
viuer         = "0.9"    # inline image rendering (kitty/iterm2/sixel/blocks)
image         = "0.25"   # decode downloaded thumbs before passing to viuer

# Async
tokio         = { version = "1", features = ["rt-multi-thread", "macros"] }
reqwest       = { version = "0.12", features = ["json"] }

# Serialization / paths
serde         = { version = "1", features = ["derive"] }
serde_json    = "1"
dirs          = "5"      # cross-platform data/cache/config dirs
chrono        = { version = "0.4", features = ["serde"] }

# Logging
log           = "0.4"
env_logger    = "0.11"

# macOS wallpaper setting (keep)
[target.'cfg(target_os = "macos")'.dependencies]
objc          = "0.2"
```

---

## Step 2 — `dirs.rs`: replace AppHandle path resolution

```rust
pub struct AppDirs {
    pub config_dir: PathBuf,
    pub cache_dir: PathBuf,
}

impl AppDirs {
    pub fn new() -> Self {
        let id = "com.dave.wallchemybar";
        AppDirs {
            config_dir: dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(id),
            cache_dir: dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(id),
        }
    }
}
```

Same paths as Tauri uses on macOS (`~/Library/Application Support/` and `~/Library/Caches/`),
so existing `settings.json`, `queue.json`, `history.json` are automatically picked up.

---

## Step 3 — Decouple backend modules (~15 signatures)

Pattern: replace `app: tauri::AppHandle` with `dirs: &AppDirs`.

```rust
// Before:
pub fn load_settings(app: tauri::AppHandle) -> Settings

// After:
pub fn load_settings(dirs: &AppDirs) -> Settings
```

Remove `#[tauri::command]` from all function signatures. Remove `tauri` and `tauri::Manager`
imports. The logic inside every function is unchanged.

Files to touch: `wallhaven.rs`, `settings.rs`, `history.rs`, `queue.rs`
`setwallpaper.rs` — no changes, already standalone.

---

## Step 4 — `app.rs`: state machine

```rust
pub enum View { Browse, Search, Collections, History, Queue, Settings }

pub struct App {
    pub view: View,
    pub wallpapers: Vec<Wallpaper>,
    pub selected: usize,           // grid cursor
    pub search_input: String,
    pub search_focused: bool,
    pub collections: Vec<Collection>,
    pub selected_collection: Option<u64>,
    pub queue: Vec<Wallpaper>,
    pub queue_running: bool,
    pub loading: bool,
    pub status: String,            // bottom status message
    pub thumb_cache: HashMap<String, DynamicImage>,  // id → decoded image
    pub settings: Settings,
    pub dirs: AppDirs,
}
```

Events come from a crossterm event loop. API calls run in `tokio::spawn` tasks and send
results back to the main loop via `tokio::sync::mpsc` channels.

---

## Step 5 — `main.rs`: event loop skeleton

```rust
#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<AppEvent>(64);
    let mut app = App::new();

    // enter alternate screen, hide cursor
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    execute!(stdout(), EnterAlternateScreen)?;

    // initial load
    spawn_search(tx.clone(), "hot".into(), None, None);

    loop {
        // 1. drain incoming events from background tasks
        while let Ok(ev) = rx.try_recv() { app.handle_event(ev); }

        // 2. draw frame
        terminal.draw(|f| ui::draw(f, &app))?;

        // 3. handle keyboard input (non-blocking, 16ms poll)
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if app.handle_key(key, tx.clone()) == Action::Quit { break; }
            }
        }
    }
    // restore terminal
}
```

---

## Step 6 — `ui/browse.rs`: thumbnail grid + viuer

ratatui draws the layout chrome. For each thumbnail cell, viuer renders the image
at the absolute terminal pixel position of that cell.

```rust
// ratatui gives us Rect { x, y, width, height } in terminal cells
// viuer needs pixel position — multiply by cell pixel size (approx 8x16px per cell)
let conf = viuer::Config {
    x: rect.x as u16,
    y: rect.y as u16,
    width: Some(rect.width as u32),
    height: Some(rect.height as u32),
    ..Default::default()
};
viuer::print(&image, &conf)?;
```

Thumbnails are downloaded async (tokio task) and stored in `App::thumb_cache` as decoded
`DynamicImage`. Cells show a `[loading]` placeholder until the download completes.

Grid layout: columns = terminal_width / thumb_width_cells. Typically 4–6 columns.
Selected cell is highlighted with a ratatui `Block` border overlay drawn on top.

---

## Step 7 — Key bindings (same as GUI app where possible)

| Key | Action |
|-----|--------|
| `h/j/k/l` or arrows | Move grid cursor |
| `Enter` | Apply selected wallpaper |
| `Space` | Focus preview pane |
| `a` | Toggle queue |
| `O/N/T/R` | Hot / New / Top / Random browse |
| `S` | Focus search input |
| `C` | Collections view |
| `H` | History |
| `Q` | Queue view |
| `,` | Settings |
| `q` / `Esc` | Quit |

---

## Step 8 — Preview pane (`ui/preview.rs`)

Right-side panel (30–35% of width). Shows:
- Full-quality thumbnail rendered via viuer (larger than grid thumbs)
- Resolution, wallpaper ID
- Tags (fetched lazily on selection change)
- Key hints: Enter=Apply, A=Queue

Activated by moving the grid cursor — no separate modal needed in TUI.

---

## Image display compatibility on macOS

viuer auto-detects protocol. Priority order:
1. Kitty graphics protocol (Kitty, WezTerm, Ghostty) — best quality
2. iTerm2 inline images (iTerm2, WezTerm) — very good
3. Sixel — moderate quality, supported by some terminals
4. Unicode half-blocks — works everywhere, pixelated but functional

The app will work in Terminal.app using half-blocks. It will look great in iTerm2/Kitty.

---

## What to drop from the GUI app

- Tauri window management (`lib.rs`) — gone
- System tray icon — gone
- Global shortcuts (Cmd+Shift+W) — gone (launch from terminal)
- Expanded window view — not needed (single TUI window)
- Collection cycling timer — can be added in v2

---

## Known challenge: ratatui + viuer coordinate conflict

ratatui uses "alternate screen" mode and redraws the entire frame each tick. viuer renders
images by writing escape codes at absolute cursor positions. These can conflict — viuer output
may be overwritten by ratatui's next frame redraw.

**Mitigation approaches (pick one at build time):**
- **Leave image cells blank in ratatui**, render viuer images after each frame draw call
  (works because ratatui only writes to cells it owns)
- Use **ratatui-image** crate (0.8+) which integrates image rendering directly into ratatui's
  widget system — avoids the conflict entirely and is the cleaner long-term solution
- Use **tui-image** or **ratatui_image** widget for in-band rendering

`ratatui-image` is probably the right choice — it handles protocol detection, async loading,
and frame integration. Worth evaluating before committing to raw viuer.

---

## Verification

1. `cargo build` — compiles cleanly
2. Run in iTerm2: thumbnail grid renders with images
3. Run in Terminal.app: thumbnail grid renders with half-blocks (degraded but functional)
4. Navigate with hjkl, confirm cursor moves and preview updates
5. Press Enter on a wallpaper — confirm wallpaper changes on desktop
6. Press S, type query, Enter — confirm search results load
7. Settings: API key saves to same `settings.json` as the GUI app
8. Queue cycling: verify timer applies wallpapers on schedule
