# abox GUI — Real Desktop App Plan

## Tech Stack

| Layer | Choice | Why |
|-------|--------|-----|
| **Framework** | Tauri 2.0 | Rust backend + web frontend, builds AppImage |
| **Frontend** | React 18 + Vite | Fast, modern, great DX |
| **UI Library** | shadcn/ui + Tailwind CSS | Beautiful, accessible, customizable |
| **State** | Zustand | Lightweight, works great with Tauri |
| **Backend** | Existing abox Rust code | Reuse session/sandbox/recorder logic |
| **Build** | `tauri build` | Produces .AppImage, .deb, .rpm |

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                   Tauri Window                      │
│  ┌───────────────────────────────────────────────┐  │
│  │            React Frontend                     │  │
│  │  ┌─────────┬───────────┬──────────────────┐   │  │
│  │  │ Sidebar │   Main    │   Detail Panel   │   │  │
│  │  │         │           │                  │   │  │
│  │  │ □ Home  │ Sessions  │  Session Info    │   │  │
│  │  │ □ Sess  │ Table     │  Tags            │   │  │
│  │  │ □ Time  │ Search    │  Notes           │   │  │
│  │  │ □ Stats │ Filters   │  Actions Log     │   │  │
│  │  │ □ Set   │           │                  │   │  │
│  │  └─────────┴───────────┴──────────────────┘   │  │
│  └───────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────┐  │
│  │            Status Bar                         │  │
│  └───────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
          │ IPC (commands) │
          ▼                ▼
┌─────────────────────────────────────────────────────┐
│              Tauri Backend (Rust)                   │
│  ┌───────────┬────────────┬────────────────────┐    │
│  │  Sessions │  Sandbox   │    Recorder        │    │
│  │  Manager  │  Manager   │    (existing)      │    │
│  │  (existing)│           │                    │    │
│  └───────────┴────────────┴────────────────────┘    │
└─────────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────┐
│  ~/.agent-sandbox/                                  │
│  ├── workspaces/   (session data)                   │
│  ├── meta/         (tags, notes)                    │
│  └── config        (settings)                       │
└─────────────────────────────────────────────────────┘
```

## 23-Step Implementation Plan

### Phase 1: Project Setup (Steps 1-4)

**Step 1 — Initialize Tauri project**
```
cd /home/ashil/Coding/agent-sandbox
npm create tauri-app@latest abox-gui -- --template react-ts
```
- Set up Vite + React + TypeScript
- Configure Tauri window size (1200x800 default)
- App icon, title "abox"

**Step 2 — Tailwind CSS + shadcn/ui setup**
- Install Tailwind CSS v4
- Initialize shadcn/ui
- Configure dark theme as default
- Add Lucide icons

**Step 3 — Tauri Rust backend hooks**
- Move session.rs, recorder logic into Tauri commands
- Create `src-tauri/src/commands/` module
- Expose via `#[tauri::command]`:
  - `list_sessions()`
  - `inspect_session(id)`
  - `tag_session(id, tag)`
  - `add_note(id, note)`
  - `search_sessions(query)`
  - `get_stats()`
  - `get_timeline()`
  - `launch_agent(agent)`
  - `clean_sessions(days)`

**Step 4 — IPC bridge (frontend ↔ backend)**
- Create `src/lib/tauri.ts` with invoke wrappers
- Type-safe API for all commands
- Error handling utilities

### Phase 2: Core Screens (Steps 5-10)

**Step 5 — App shell + sidebar navigation**
```
┌──────────────────────────────────────────┐
│ ┌──────┐  ┌──────────────────────────┐   │
│ │      │  │                          │   │
│ │ Home │  │     Main Content         │   │
│ │      │  │                          │   │
│ │ Sess │  │     (route outlet)       │   │
│ │      │  │                          │   │
│ │ Time │  │                          │   │
│ │      │  │                          │   │
│ │Stats │  │                          │   │
│ │      │  │                          │   │
│ │ Set  │  │                          │   │
│ │      │  └──────────────────────────┘   │
│ └──────┘  ┌──────────────────────────┐   │
│           │ Status Bar               │   │
│           └──────────────────────────┘   │
└──────────────────────────────────────────┘
```
- Collapsible sidebar
- Active state highlighting
- React Router for navigation

**Step 6 — Home/Dashboard screen**
- Hero section: logo, version, tagline
- Quick stats cards (sessions, time, storage)
- Recent sessions list (click to open)
- Agent launcher grid (click to launch)
- System status indicators

**Step 7 — Sessions table**
- Data table with columns: Name, Agent, Date, Duration, Files
- Sort by any column (click header)
- Search bar with live filtering
- Tag filter chips
- Row click → detail panel
- Bulk select for batch operations

**Step 8 — Session detail panel**
- Slide-in panel from right (or full page)
- Tabs: Overview | Actions | Notes | Tags
- **Overview:** ID, agent, duration, created date
- **Actions:** Expandable tree of file operations
- **Notes:** Inline editable textarea
- **Tags:** Tag chips with add/remove

**Step 9 — Timeline view**
- Horizontal scrollable timeline
- Sessions as colored bars (length = duration)
- Color-coded by agent
- Tooltip on hover with session info
- Click to open session detail
- Zoom controls (day/week/month)

**Step 10 — Stats screen**
- Summary cards at top
- Bar chart: sessions per agent
- Line chart: sessions over time
- Pie chart: agent distribution
- Storage usage breakdown

### Phase 3: Interactions (Steps 11-15)

**Step 11 — Modal system**
- Confirm dialogs (delete, export)
- Input dialogs (tag name, note)
- Selection dialogs (agent picker)
- Keyboard accessible (Escape to close)

**Step 12 — Toast notifications**
- Top-right corner
- Success (green), Error (red), Info (blue)
- Auto-dismiss 3s with progress bar
- Stackable

**Step 13 — Search with highlights**
- Global search (Ctrl+K)
- Searches names, tags, notes
- Highlight matches
- Arrow key navigation
- Enter to open

**Step 14 — Keyboard shortcuts**
- `Ctrl+K` — Command palette / search
- `Ctrl+N` — New session / launch agent
- `Ctrl+E` — Edit note
- `Ctrl+T` — Add tag
- `Delete` — Delete selected
- `Escape` — Close panel/modal

**Step 15 — Launch agent flow**
- Agent picker modal
- Shows installed/not-installed status
- Progress indicator during launch
- Opens in new window or embedded terminal

### Phase 4: Polish (Steps 16-20)

**Step 16 — Animations**
- Page transitions (fade)
- Panel slide-in/out
- List item hover effects
- Loading skeletons
- Spinner animations

**Step 17 — Settings screen**
- Theme selector (Dark/Ocean/Neon + custom)
- Default agent picker
- Auto-cleanup toggle + schedule
- Keyboard shortcut customization
- Export/import config

**Step 18 — Session diff view**
- Select 2 sessions
- Side-by-side comparison
- Highlight added/removed/changed actions
- Like git diff but for sessions

**Step 19 — Context menus**
- Right-click on session row
  - Inspect, Tag, Note, Export, Delete
- Right-click on tag
  - Remove, Rename, Change color
- Right-click on agent
  - Launch, Configure, View sessions

**Step 20 — Drag & drop**
- Drag session to tag to apply
- Drag files to export
- Drag to reorder sidebar

### Phase 5: Packaging & Distribution (Steps 21-23)

**Step 21 — AppImage build config**
```json
// tauri.conf.json
{
  "bundle": {
    "targets": ["AppImage", "deb"],
    "icon": ["icons/32x32.png", "icons/128x128.png", "icons/512x512.png"]
  }
}
```

**Step 22 — Auto-update setup**
- Tauri updater plugin
- Check for updates on launch
- Download and install silently
- Version display in settings

**Step 3 — CI/CD pipeline**
- GitHub Actions workflow
- Build on push to main
- Create release with AppImage + deb
- Update AUR package (optional)

## File Structure

```
abox-gui/
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       ├── commands/
│       │   ├── mod.rs
│       │   ├── sessions.rs
│       │   ├── sandbox.rs
│       │   └── config.rs
│       └── abox/          ← symlink to /home/ashil/Coding/agent-sandbox/src
│           ├── session.rs
│           ├── recorder/
│           ├── sandbox/
│           └── ...
├── src/
│   ├── App.tsx
│   ├── main.tsx
│   ├── lib/
│   │   ├── tauri.ts       ← invoke wrappers
│   │   └── utils.ts
│   ├── components/
│   │   ├── ui/            ← shadcn components
│   │   ├── layout/
│   │   │   ├── Sidebar.tsx
│   │   │   ├── StatusBar.tsx
│   │   │   └── AppShell.tsx
│   │   ├── sessions/
│   │   │   ├── SessionTable.tsx
│   │   │   ├── SessionDetail.tsx
│   │   │   └── SessionRow.tsx
│   │   ├── timeline/
│   │   │   └── Timeline.tsx
│   │   ├── stats/
│   │   │   └── Charts.tsx
│   │   └── shared/
│   │       ├── Toast.tsx
│   │       ├── Modal.tsx
│   │       └── SearchDialog.tsx
│   ├── pages/
│   │   ├── Home.tsx
│   │   ├── Sessions.tsx
│   │   ├── Timeline.tsx
│   │   ├── Stats.tsx
│   │   └── Settings.tsx
│   ├── hooks/
│   │   ├── useSessions.ts
│   │   ├── useStats.ts
│   │   └── useTheme.ts
│   └── styles/
│       └── globals.css
├── package.json
├── vite.config.ts
├── tsconfig.json
└── tailwind.config.ts
```

## Dependencies

### Frontend
```json
{
  "dependencies": {
    "react": "^18.3",
    "react-router-dom": "^6.26",
    "@tauri-apps/api": "^2.0",
    "zustand": "^5.0",
    "date-fns": "^3.0",
    "recharts": "^2.12",
    "lucide-react": "^0.400",
    "clsx": "^2.1",
    "tailwind-merge": "^2.3"
  }
}
```

### Backend (Tauri)
```toml
[dependencies]
tauri = { version = "2.0", features = ["shell-open", "updater"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
# Reuse existing abox code
```

## Visual Design

### Color Palette (Dark Theme)
```
Background:    #0a0a0f (deep dark)
Surface:       #12121a
Surface Hover: #1a1a24
Border:        #2a2a3a
Primary:       #64b4ff (blue)
Secondary:     #b482ff (purple)
Accent:        #ffb43c (amber)
Success:       #50dc8c
Warning:       #ffc83c
Error:         #ff5050
Text Primary:  #dcdcff
Text Muted:    #646478
```

### Typography
```
Font: Inter (system fallback)
Title:     24px / Bold
Subtitle:  16px / Medium
Body:      14px / Regular
Caption:   12px / Regular
Mono:      JetBrains Mono (code)
```

### Component Examples

**Session Table Row:**
```
┌─────────────────────────────────────────────────────────────┐
│ □  opencode-20260315-182651   opencode   3m 20s   12 files  │
│    [important] [work]                    Mar 15, 18:26      │
└─────────────────────────────────────────────────────────────┘
```

**Agent Card:**
```
┌─────────────────────────┐
│  ┌─┐                    │
│  │ ◆ │ OpenCode         │
│  └─┘  installed         │
│         5 sessions      │
│  [ Launch ]             │
└─────────────────────────┘
```

**Stat Card:**
```
┌─────────────────────────┐
│  Total Sessions         │
│  16                     │
│  ↑ 12% from last week   │
└─────────────────────────┘
```

## Build Commands

```bash
# Development
cd abox-gui
npm install
npm run tauri dev

# Production
npm run tauri build
# Output: src-tauri/target/release/bundle/

# Install
sudo dpkg -i src-tauri/target/release/bundle/deb/abox_0.1.0_amd64.deb
# Or
sudo install -m 755 src-tauri/target/release/bundle/appimage/abox_0.1.0_amd64.AppImage /usr/local/bin/abox-gui
```
