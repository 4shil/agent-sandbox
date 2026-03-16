# abox Desktop-App UX Analysis & 23-Step Plan

## Current UI/UX Audit

### What Works ✅
- Clear ASCII box structure
- Consistent session formatting
- Basic dashboard with session listing
- Setup wizard for onboarding
- Status command for quick overview

### What Feels "CLI" (Not Desktop) ❌
| Problem | Desktop Expectation |
|---------|---------------------|
| Static screen clears (`\x1B[2J`) | Smooth scrolling, no flicker |
| Text-only menu (`[L]ist [C]lean`) | Visual buttons, hover states |
| No scroll | Scrollable lists with scrollbar |
| No keyboard nav (arrows) | Arrow keys, Enter, Tab, Escape |
| No progress bars | Animated progress bars |
| No mouse support | Click, scroll, drag |
| No status bar | Bottom status/info bar |
| No color theming | Consistent theme system |
| No modals | Popup confirmations |
| No search-as-you-type | Live filtering |
| No animations | Cursor blink, transitions |
| No tabs/panels | Split-pane views |
| No notification system | Toast notifications |
| No resizable output | Responsive layout |

---

## 23-Step Implementation Plan

### PHASE 1: Foundation (Steps 1-5)

#### Step 1 — Add `ratatui` TUI framework dependency
- Add `ratatui`, `crossterm` to Cargo.toml
- Create `src/tui/mod.rs` — initialize terminal, alternate screen, raw mode
- Create `src/tui/event.rs` — unified event loop (Key, Mouse, Resize)
- **Result:** Proper terminal enters/exits cleanly, no flicker

#### Step 2 — Theme system (`src/tui/theme.rs`)
- Central `Theme` struct: primary, secondary, accent, bg, fg, border, error, success
- Presets: `Default`, `Dark`, `Ocean`, `Forest`, `Neon`
- Read from `~/.aboxrc` config or env vars
- Apply via `ratatui::style::Style` everywhere
- **Result:** Consistent look, user-customizable

#### Step 3 — Reusable widgets (`src/tui/widgets/`)
- `block.rs` — Styled block with title, border style, padding
- `list.rs` — Scrollable list with selection, scrollbar, item builder
- `statusbar.rs` — Bottom bar with left/right sections, mode indicator
- `modal.rs` — Centered popup dialog (confirm, input, info)
- `progress.rs` — Animated progress bar (determinate/indeterminate)
- `toast.rs` — Corner notification (success/error/info) with auto-dismiss
- **Result:** Desktop-grade UI components

#### Step 4 — App state machine (`src/tui/app.rs`)
- `App` struct: current screen, previous screen, loading state
- Screens enum: `Home`, `Sessions`, `SessionDetail`, `Timeline`, `Stats`, `Settings`, `Wizard`
- Navigation stack for Back/Escape behavior
- Global keybindings registry
- **Result:** Proper screen management like a desktop app

#### Step 5 — Event loop with tick rate (`src/tui/loop.rs`)
- `crossterm::event::poll` with 250ms tick for animations
- Event channel: separate UI events from IO events
- Handle window resize → re-layout
- Double-buffered rendering (no flicker)
- **Result:** Smooth 4fps animations, responsive to resize

---

### PHASE 2: Home & Navigation (Steps 6-10)

#### Step 6 — Home screen (dashboard)
- **Top:** Logo + version, system uptime
- **Left panel:** Agent list with [installed] badges and click-to-launch
- **Right panel:** Recent sessions with duration bars
- **Bottom:** Status bar showing sessions count, disk usage, shortcuts
- **Footer:** Key hints (`↑↓ Navigate`, `Enter Launch`, `Esc Quit`)
- **Result:** Feels like a launcher app

#### Step 7 — Session list screen
- Full-width scrollable table
- Columns: Name, Agent, Date, Duration, Files, Tags
- Sort by any column (click or `S` + column key)
- Filter bar at top (live search-as-you-type)
- Multi-select with `Space` for batch operations
- Scrollbar on right with position indicator
- **Result:** Like a file manager

#### Step 8 — Session detail screen (inspect)
- Split layout: left = metadata, right = action log
- **Left:** ID, Agent, Duration, Tags (editable), Notes (editable)
- **Right:** Scrollable action list with expand/collapse per action
- Tab key switches between panels
- **e** to edit notes inline
- **t** to add/edit tags
- **Result:** Like an IDE inspector panel

#### Step 9 — Timeline screen
- Horizontal timeline view
- Sessions as bars (length = duration)
- Color-coded by agent type
- Hover/focus shows tooltip with details
- Scroll left/right through time
- Zoom in/out with `+`/`-`
- **Result:** Like a Gantt chart in project management apps

#### Step 10 — Stats screen with charts
- **Top:** Summary cards (sessions, time, files, storage)
- **Middle:** Bar chart of sessions per agent
- **Bottom:** Line chart of sessions over time (last 30 days)
- All charts drawn with `ratatui::widgets::BarChart`, `LineChart`
- **Result:** Feels like a monitoring dashboard

---

### PHASE 3: Interactions (Steps 11-15)

#### Step 11 — Modal system
- `Modal::confirm("Delete 5 sessions?")` → Yes/No
- `Modal::input("Tag name:")` → text input
- `Modal::info("Session exported")` → auto-dismiss 3s
- Dim background with overlay
- Escape/Enter to dismiss
- **Result:** Desktop-grade confirmation flows

#### Step 12 — Toast notifications
- Corner placement (top-right or bottom-right)
- Types: Success (green), Error (red), Info (blue), Warning (yellow)
- Auto-dismiss after 3s with fade-out animation
- Stack multiple toasts
- **Result:** Modern notification UX

#### Step 13 — Keyboard navigation
- `↑`/`↓` navigate lists
- `Enter` select/launch
- `Tab` switch panels
- `Escape` back/dismiss
- `/` focus search bar
- `?` show keybinding help overlay
- `g` then `h`/`s`/`t`/`d` for screen jumps (vim-style)
- `q` quit (from home)
- **Result:** Power-user keyboard control

#### Step 14 — Mouse support
- Click to select list items
- Scroll wheel in scrollable areas
- Click on buttons/tabs
- Resize-aware (re-layout on terminal resize)
- Cursor position detection for hover states
- **Result:** Clickable, scrollable, mouse-friendly

#### Step 15 — Loading states & spinners
- Animated spinner during agent launch (`⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏`)
- Progress bar for batch clean/export
- Skeleton loading for session detail (gray placeholders)
- **Result:** No "did it freeze?" moments

---

### PHASE 4: Polish (Steps 16-20)

#### Step 16 — Session tags UI
- Inline tag editing in session detail
- Tag chips with colored backgrounds
- Tag filter in session list (`tag:important`)
- Tag color assignment in settings
- **Result:** Visual organization like labels in desktop apps

#### Step 17 — Inline note editor
- Press `e` in session detail → focus note area
- Multi-line text input with cursor movement
- Save with `Ctrl+S`, cancel with `Escape`
- Auto-save indicator (dot in corner)
- **Result:** Feels like a text editor

#### Step 18 — Search with highlights
- Global search (`/`) with live filtering
- Highlight matching text in results
- Search across names, tags, notes, agent names
- Arrow keys to navigate results, Enter to open
- **Result:** Spotlight-like search

#### Step 19 — Confirmation & undo
- Delete/export → modal confirm
- After destructive action, show "Undo" toast (5s)
- `Ctrl+Z` to undo last action (if available)
- **Result:** Safe, recoverable operations

#### Step 20 — Help overlay
- Press `?` anywhere → overlay with all keybindings
- Grouped by context: Global, List, Detail, Edit
- Dismiss with `?` or `Escape`
- Search within help for specific bindings
- **Result:** Built-in documentation like desktop apps

---

### PHASE 5: Advanced (Steps 21-23)

#### Step 21 — Split-pane / tabs
- `Tab 1: Home`, `Tab 2: Sessions`, `Tab 3: Timeline`
- `Ctrl+Tab` to switch tabs
- Split view: session list (left) + detail (right) side by side
- Resize split with `Ctrl+←/→`
- **Result:** IDE-like workspace

#### Step 22 — Session diff view
- Select 2 sessions → `D` for diff
- Side-by-side comparison of actions
- Highlight added/removed/changed
- Like `git diff` but for sessions
- **Result:** Debugging power tool

#### Step 23 — Settings screen
- Visual settings editor (not just .aboxrc)
- Sections: General, Agents, Appearance, Storage
- Toggle switches for booleans
- Dropdowns for choices
- Color pickers for theme
- Save with `Ctrl+S`, applies immediately
- **Result:** No manual config file editing

---

## Dependency Summary

```toml
[dependencies]
ratatui = "0.28"       # TUI framework
crossterm = "0.28"     # Terminal input/output
tui-textarea = "0.7"   # Multi-line text editing
tui-tree-widget = "0.3" # Tree view (for action details)
```

## File Structure

```
src/
├── tui/
│   ├── mod.rs          # pub use, init/close terminal
│   ├── app.rs          # App state, screen enum
│   ├── event.rs        # Event loop, key mapping
│   ├── theme.rs        # Color theme system
│   ├── loop.rs         # Render loop, tick rate
│   └── widgets/
│       ├── mod.rs      # Re-exports
│       ├── block.rs    # Styled panels
│       ├── list.rs     # Scrollable lists
│       ├── statusbar.rs
│       ├── modal.rs
│       ├── progress.rs
│       └── toast.rs
├── screens/
│   ├── mod.rs
│   ├── home.rs         # Dashboard/launcher
│   ├── sessions.rs     # Session list + search
│   ├── detail.rs       # Session inspector
│   ├── timeline.rs     # Gantt-like timeline
│   ├── stats.rs        # Charts & analytics
│   └── settings.rs     # Visual settings
├── main.rs             # Entry point (CLI or TUI)
├── session.rs          # Session logic
├── recorder/
└── ui.rs               # Legacy (keep for --no-tui flag)
```

## Migration Strategy
- `abox dashboard` → launches full TUI
- `abox` (no args) → opens Home screen
- Add `--no-tui` flag for legacy text output
- Existing commands (`list`, `stats`, `timeline`) still work in CLI mode
- TUI is opt-in per command, not a breaking change
