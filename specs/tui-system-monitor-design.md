# TUI System Monitor — Demo App Design

> Design for the `rust-tui-template` system monitoring panel demo.
> Derived from the CLI template at `rust-lib-template`.

## 1. Widget Layout Diagrams

### Overall Terminal Layout

```
┌─────────────────────────────────────────────────────────────────────┐
│ [1:Overview]  [2:Processes]  [3:About]            ← TabBar (top)   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│                     TAB CONTENT AREA                                │
│                  (takes all remaining space                         │
│                   between bars)                                     │
│                                                                     │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│ ● Idle  │  2026-06-01 14:32:05  │  Refresh: 1s         ← StatusBar │
├─────────────────────────────────────────────────────────────────────┤
│ q:Quit  1-3:Tab  ←→:Switch  j/k:Nav  r:Refresh  f:Interval  ?:Help│
└─────────────────────────────────────────────────────────────────────┘
```

**Layout constraint chain (ratatui `Layout`):**

```
Frame
 ├─ [0] TabBar        → height = 1
 ├─ [1] ContentArea   → min_height = 0, fill remaining
 ├─ [2] StatusBar     → height = 1
 └─ [3] HelpBar       → height = 1 (conditionally hidden)
```

### Tab 1: Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│ [1:Overview] [2:Processes] [3:About]                                │
├─────────────────────────────────────────────────────────────────────┤
│ ┌──────────────┐  ┌──────────────┐  ┌──────────────┐               │
│ │   CPU        │  │   Memory     │  │   Disk       │               │
│ │              │  │              │  │              │               │
│ │ ████████░░░░ │  │ █████████░░░ │  │ ████████████ │               │
│ │   45.2%      │  │   62.8%      │  │   71.3%      │               │
│ │              │  │              │  │              │               │
│ │ ▂▃▄▄▅▆▇▇██  │  │ ▅▅▆▆▇▇████  │  │ ▃▃▄▄▅▅▆▆▇▇  │               │
│ │  (sparkline) │  │  (sparkline) │  │  (sparkline) │               │
│ └──────────────┘  └──────────────┘  └──────────────┘               │
├─────────────────────────────────────────────────────────────────────┤
│  CPU: AMD Ryzen 7 5800X (16 logical cores)                          │
│  Memory: Total 32.0 GB  │  Used 20.1 GB  │  Available 11.9 GB      │
│  Disk (/): Total 512 GB  │  Used 365 GB  │  Free 147 GB            │
└─────────────────────────────────────────────────────────────────────┘
```

**Widget breakdown:**
- Top row: 3 equal `Gauge` widgets (ratatui `Gauge`) arranged horizontally.
- Each gauge has a `Sparkline` underneath showing the last 60 readings.
- Bottom row: `Paragraph` with detail text in a single block.

### Tab 2: Processes

```
┌─────────────────────────────────────────────────────────────────────┐
│ [1:Overview] [2:Processes] [3:About]                                │
├─────────────────────────────────────────────────────────────────────┤
│ Sort: CPU ▼  │  Showing 20 of 187 processes                         │
├──────┬─────────────────┬────────┬────────┬───────────┬──────────────┤
│ PID  │ Name            │ CPU%   │ Mem%   │ Mem (MB)  │ Status       │
├──────┼─────────────────┼────────┼────────┼───────────┼──────────────┤
│ 1234 │ firefox         │ 12.3   │  8.7   │  2784.3   │ Running      │
│ 5678 │ cargo           │  8.9   │  2.1   │   672.0   │ Running    ← │  selected
│ 9012 │ rust-analyzer   │  5.4   │  3.2   │  1024.0   │ Sleeping     │
│ 3456 │ node            │  3.1   │  1.8   │   576.0   │ Sleeping     │
│  ... │ ...             │  ...   │  ...   │  ...      │ ...          │
│  ... │ ...             │  ...   │  ...   │  ...      │ ...          │
│  ... │ ...             │  ...   │  ...   │  ...      │ ...          │
├──────┴─────────────────┴────────┴────────┴───────────┴──────────────┤
│  → 5678  cargo          8.9% CPU  │  2.1% Mem  │  Running           │
├─────────────────────────────────────────────────────────────────────┤
│ ● Idle  │  14:32:05  │  Refresh: 1s                                  │
├─────────────────────────────────────────────────────────────────────┤
│ q:Quit 1-3:Tab ←→:Switch j/k:Nav s:Sort Enter:Detail r:Refresh ... │
└─────────────────────────────────────────────────────────────────────┘
```

**Widget breakdown:**
- Header row: sort indicator (`Paragraph`) and count (`Paragraph`).
- Center: `Table` (ratatui `Table`) with 6 columns: PID, Name, CPU%, Mem%, Mem (MB), Status.
- `TableState` tracks `selected` (highlighted row) and offset (scroll position).
- Footer row: detail line for the selected process (`Paragraph`).

### Tab 3: About / Help

```
┌─────────────────────────────────────────────────────────────────────┐
│ [1:Overview] [2:Processes] [3:About]                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│                   System Monitor  v0.1.0                            │
│                                                                     │
│     A terminal system monitoring tool built with ratatui.           │
│                                                                     │
│  ── Global Keybindings ──────────────────────────────────────────── │
│                                                                     │
│  q / Esc                 Quit                                       │
│  1 / 2 / 3               Switch to tab                              │
│  ← / →  or  h / l       Previous / Next tab                        │
│  r                       Manual refresh                             │
│  f                       Cycle refresh interval (1s → 2s → 5s → 1s) │
│  ?                       Toggle help bar                            │
│                                                                     │
│  ── Process List ────────────────────────────────────────────────── │
│                                                                     │
│  ↑ / ↓  or  j / k       Scroll up / down                           │
│  s                       Toggle sort column (CPU → Mem → PID → CPU) │
│  Home / End              Jump to top / bottom                       │
│  Enter                   Show focused process details               │
│                                                                     │
│  ── Mouse ───────────────────────────────────────────────────────── │
│                                                                     │
│  Click tabs to switch.  Scroll wheel navigates process list.        │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

**Widget breakdown:**
- Single `Paragraph` with wrapped text. No interactivity needed.

---

## 2. State Struct Design

### `App` — Root application state

```rust
use std::time::Instant;
use ratatui::widgets::TableState;

pub struct App {
    // --- Navigation ---
    /// Currently visible tab.
    pub active_tab: Tab,
    /// Scroll position + selected row in the process table.
    /// ratatui's TableState tracks both `selected` (Option<usize>) and `offset` (usize).
    pub process_table_state: TableState,
    /// Which column the process table is sorted by.
    pub process_sort: ProcessSortColumn,

    // --- Timing ---
    /// Current refresh cadence.
    pub refresh_interval: RefreshInterval,
    /// When the last snapshot was taken (Instant::now() at each refresh).
    pub last_refresh: Instant,
    /// True while `update()` is running — status bar shows "Refreshing..." on yellow bg.
    pub is_refreshing: bool,
    /// Timestamp of the most recent successful refresh.
    pub last_snapshot_time: Option<Instant>,

    // --- Display toggles ---
    /// Help bar visibility (toggled with `?`).
    pub show_help_bar: bool,
    /// Running in demo mode with fake data (no sysinfo dependency at runtime).
    pub demo_mode: bool,

    // --- Data ---
    /// The most recent system snapshot. Updated by `App::update()`.
    pub snapshot: SystemSnapshot,
    /// sysinfo `System` handle. `None` in demo mode.
    /// Kept alive across refreshes so per-process CPU deltas accumulate correctly.
    pub sys: Option<System>,

    // --- Config ---
    /// Deserialized from `$XDG_CONFIG_HOME/<app>/config.toml` + env vars + CLI flags.
    pub config: TuiConfig,

    // --- Lifecycle ---
    /// Set to `true` by the quit keybinding; the event loop checks this at the top.
    pub should_quit: bool,
}
```

### Supporting enums

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab { Overview, Processes, About }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshInterval { Fast, Medium, Slow }
// Fast=1s, Medium=2s, Slow=5s

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessSortColumn { Pid, Cpu, Memory, Name }
```

### `SystemSnapshot` — One refresh cycle's worth of data

```rust
#[derive(Debug, Clone)]
pub struct SystemSnapshot {
    // CPU
    pub cpu_usage_pct: f64,         // 0.0–100.0
    pub cpu_brand: String,          // e.g. "AMD Ryzen 7 5800X"
    pub cpu_logical_cores: usize,   // thread count
    pub cpu_history: Vec<f64>,      // last N readings for sparkline (0.0–100.0)

    // Memory
    pub memory_total_gb: f64,
    pub memory_used_gb: f64,
    pub memory_available_gb: f64,
    pub memory_usage_pct: f64,      // 0.0–100.0
    pub mem_history: Vec<f64>,      // last N readings for sparkline

    // Disk (single mount point, e.g. "/")
    pub disk_total_gb: f64,
    pub disk_used_gb: f64,
    pub disk_free_gb: f64,
    pub disk_usage_pct: f64,        // 0.0–100.0
    pub disk_mount_point: String,

    // Processes (sorted by the current sort column, top N)
    pub processes: Vec<ProcessInfo>,
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage_pct: f64,     // 0.0–100.0
    pub memory_usage_pct: f64,  // 0.0–100.0
    pub memory_mb: f64,
    pub status: ProcessStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessStatus { Run, Sleep, Idle, Zombie, Unknown }
```

### `TuiConfig` — Deserialized configuration

```rust
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct TuiConfig {
    /// Interval between automatic refreshes, in milliseconds.
    pub refresh_interval_ms: u64,     // default 1000
    /// Use simulated data instead of real sysinfo.
    pub demo_mode: bool,              // default false
    /// tracing log level.
    pub log_level: String,            // default "info"
    /// Number of history points for sparklines.
    pub sparkline_history: usize,     // default 60
    /// Number of processes to show in the table.
    pub process_count: usize,         // default 50
}
```

---

## 3. Data Flow: sysinfo → App → Rendering

```
                    ┌──────────────────────────────────┐
                    │         sysinfo 0.33              │
                    │  System::new_all()                │
                    │  .refresh_cpu_all()               │
                    │  .refresh_memory()                │
                    │  .refresh_processes()             │
                    │  .cpus(), .total_memory(), etc.   │
                    └──────────────┬───────────────────┘
                                   │ raw OS data
                                   ▼
┌──────────────────────────────────────────────────────────────────┐
│  App::update(&mut self)                                          │
│                                                                  │
│  if self.demo_mode {                                             │
│      self.snapshot = mock_snapshot();  // deterministic fake     │
│  } else {                                                        │
│      let sys = self.sys.as_mut().unwrap();                       │
│      sys.refresh_cpu_all();       // must refresh before read    │
│      sys.refresh_memory();                                       │
│      sys.refresh_processes(                                     │
│          ProcessRefreshKind::everything()                        │
│          .with_cpu()                                             │
│          .with_memory()                                          │
│      );                                                          │
│                                                                  │
│      // CPU: average across all logical cores                    │
│      let cpu_pct = sys.cpus().iter()                             │
│          .map(|c| c.cpu_usage())                                 │
│          .sum::<f32>() / sys.cpus().len() as f32;                │
│                                                                  │
│      // Memory                                                   │
│      let mem_total = sys.total_memory();                         │
│      let mem_used = sys.used_memory();                           │
│                                                                  │
│      // Disk (pick root mount)                                   │
│      // Use sysinfo::Disks::new_with_refreshed_list()            │
│                                                                  │
│      // Processes: collect, sort by active column, take top N    │
│      let mut procs: Vec<ProcessInfo> = sys.processes()           │
│          .iter()                                                 │
│          .map(|(pid, p)| ProcessInfo {                           │
│              pid: pid.as_u32(),                                  │
│              name: p.name().to_string_lossy().into(),            │
│              cpu_usage_pct: p.cpu_usage() as f64,                │
│              memory_usage_pct: p.memory() as f64                 │
│                  / sys.total_memory() as f64 * 100.0,            │
│              memory_mb: p.memory() as f64 / 1_048_576.0,         │
│              status: p.status().into(),                          │
│          })                                                      │
│          .collect();                                             │
│                                                                  │
│      procs.sort_by(|a, b| match self.process_sort {              │
│          ProcessSortColumn::Cpu =>                               │
│              b.cpu_usage_pct.partial_cmp(&a.cpu_usage_pct)       │
│                  .unwrap_or(Ordering::Equal),                    │
│          ProcessSortColumn::Memory =>                            │
│              b.memory_usage_pct.partial_cmp(&a.memory_usage_pct) │
│                  .unwrap_or(Ordering::Equal),                    │
│          // ... etc                                              │
│      });                                                         │
│      procs.truncate(self.config.process_count);                  │
│                                                                  │
│      self.snapshot = SystemSnapshot {                            │
│          cpu_usage_pct: cpu_pct as f64,                          │
│          cpu_history: push_and_trim(self.snapshot.cpu_history,   │
│              cpu_pct as f64, self.config.sparkline_history),     │
│          // ... etc                                              │
│      };                                                          │
│  }                                                               │
│  self.last_refresh = Instant::now();                             │
│  self.is_refreshing = false;                                     │
└──────────────┬───────────────────────────────────────────────────┘
               │ App.snapshot (immutable borrow during draw)
               ▼
┌──────────────────────────────────────────────────────────────────┐
│  ui::render(frame: &mut Frame, app: &App)                        │
│                                                                  │
│  let [tab_bar_area, content_area, status_bar_area, help_bar_area]│
│      = Layout::vertical([                                        │
│          Constraint::Length(1),   // tab bar                     │
│          Constraint::Min(0),      // content                     │
│          Constraint::Length(1),   // status bar                  │
│          Constraint::Length(1),   // help bar (conditionally)    │
│      ]).split(frame.area());                                     │
│                                                                  │
│  tab_bar::draw(frame, tab_bar_area, app.active_tab);             │
│                                                                  │
│  match app.active_tab {                                          │
│      Tab::Overview  => overview::draw(frame, content_area,       │
│                          &app.snapshot, &app.config),            │
│      Tab::Processes => processes::draw(frame, content_area,      │
│                          &app.snapshot, &app.process_table_state,│
│                          app.process_sort),                      │
│      Tab::About     => about::draw(frame, content_area),         │
│  }                                                               │
│                                                                  │
│  status_bar::draw(frame, status_bar_area, app.is_refreshing,     │
│      app.refresh_interval, app.demo_mode);                       │
│                                                                  │
│  if app.show_help_bar {                                          │
│      help_bar::draw(frame, help_bar_area, app.active_tab);       │
│  }                                                               │
└──────────────────────────────────────────────────────────────────┘
```

### Event Loop (run_app)

```
 ┌──────────────┐
 │  app.update()│ ─── Initial data load
 └──────┬───────┘
        ↓
 ┌────────────────────────────────────────────────────┐
 │  loop {                                            │
 │    terminal.draw(|f| ui::render(f, &app))?;        │  ← Always render
 │    if app.should_quit { break; }                   │
 │                                                     │
 │    if event::poll(tick_rate)? {                     │  ← tick_rate = 100ms
 │        match event::read()? {                       │     (10 Hz input poll)
 │            Event::Key(k)  => handle_key(&mut app,k),│
 │            Event::Mouse(m) => handle_mouse(&mut app,m),
 │            Event::Resize(_,_) => {} // ratatui handles
 │        }                                            │
 │    }                                                │
 │                                                     │
 │    if app.last_refresh.elapsed() >= interval {      │
 │        app.is_refreshing = true;                    │
 │        terminal.draw(|f| ui::render(f, &app))?;     │  ← Flash yellow status
 │        app.update();                                │  ← Blocking refresh
 │    }                                                │
 │  }                                                  │
 └────────────────────────────────────────────────────┘
```

**Why the double-draw on refresh?** The first draw renders "Refreshing..." with a yellow status bar before the potentially-blocking `sys.refresh_*()` calls. The second draw (on the next loop iteration) shows the updated data. This gives a visual "pulse" that reassures the user the app is alive.

---

## 4. Recommended Workspace Dependencies

Add to the workspace `Cargo.toml` under `[workspace.dependencies]`:

```toml
# TUI framework
ratatui = "0.29"
crossterm = "0.28"

# System statistics
sysinfo = "0.33"

# Date/time for the status bar clock
chrono = "0.4"

# Non-blocking file appender for tracing (write logs to disk in TUI mode)
tracing-appender = "0.2"
```

**Note:** `ratatui` and `crossterm` are the standard TUI stack. `sysinfo` is the de-facto cross-platform system info crate. `chrono` is only used for the status bar clock; `std::time` could substitute but `chrono` is cleaner for formatting. `tracing-appender` writes logs to a file (`$XDG_STATE_HOME/<app>/app.log`) so tracing output does not corrupt the TUI.

### `apps/tui/Cargo.toml`

```toml
[package]
name = "gitflow-cli-tui"
version.workspace = true
authors.workspace = true
edition.workspace = true
license.workspace = true

[[bin]]
name = "gitflow-cli"
path = "src/main.rs"

[dependencies]
# Template core lib (reuse SafePath, etc.)
gitflow-cli-core = { workspace = true }

# TUI
ratatui = { workspace = true }
crossterm = { workspace = true }

# System data
sysinfo = { workspace = true }

# Utilities
chrono = { workspace = true }
clap = { workspace = true, features = ["derive", "env"] }
dirs = { workspace = true }
toml = { workspace = true }
serde = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true, features = ["env-filter"] }
tracing-appender = { workspace = true }
anyhow = { workspace = true }

[lints]
workspace = true

[lints.rust]
unsafe_code = "deny"
```

---

## 5. "Wow Factor" for First-Time `make run`

| Moment | What the user sees |
|---|---|
| **0.0s** | Terminal clears, the TUI paints: dark background, three tabs, a status bar with the clock ticking. No loading screen, no splash — instant dashboard. |
| **0.5s** | CPU, Memory, Disk gauges fill with real data. The CPU gauge is green (45%), memory is yellow (72%), disk is green (33%). Colors communicate state without reading numbers. |
| **1.0s** | The sparklines start scrolling — a subtle "EKG" effect as the line shifts left and the newest point appears on the right. It looks like a professional monitoring tool. |
| **3.0s** | User presses `2` — the view snaps to a rich process table. Their own `cargo`, `rust-analyzer`, and browser processes are listed, sorted by CPU. It feels like `htop` but prettier. |
| **5.0s** | User presses `j` / `k` — the highlight bar moves up and down the process list. `s` cycles the sort column, and the table rerenders instantly. |
| **10.0s** | User presses `f` twice — the status bar shows "Refresh: 2s" then "Refresh: 5s". The refresh cadence slows visibly. Press `r` for a manual refresh that flashes the status bar yellow. |
| **15.0s** | User presses `3` for About. Clean, centered keybinding reference. They learn about `?` to toggle the help bar, `Home/End` for jumping, and mouse support for tabs. |
| **20.0s** | User presses `q` — the terminal restores instantly. No leftover characters, no cursor offset. Clean exit. |
| **Total** | The user has seen a complete, polished TUI app with zero configuration. The code they will build from is clean, idiomatic Rust following the template's strict standards. They're excited to fork it and build their own TUI tool. |

### Why this design specifically:

1. **Three tabs is the minimum for a template** — enough to demonstrate 3 different widget families (`Gauge`+`Sparkline`, `Table`, `Paragraph`) without complexity creep.
2. **System monitoring is visually rich** — gauges, sparklines, colors, tables. A TODO app or markdown viewer would not show ratatui's capabilities.
3. **Real data is impressive** — `sysinfo` showing the user's actual CPU model and processes makes it feel like a real tool, not a toy.
4. **Template is still lean** — 3 tabs, ~8 source files, no plugin system, no networking. Someone can read the entire codebase in 30 minutes.
5. **Mouse support separates it from CLI tools** — most Rust CLI templates ignore mouse input. This shows how to handle it correctly.
6. **Color theming with `NO_COLOR` respect** — demonstrates the template's commitment to accessibility and standards compliance.

---

## 6. File Structure (under `apps/tui/src/`)

```
main.rs           — entry: parse CLI args (--demo, --config, --log-level),
                    init tracing → file, setup panic hook, terminal setup
                    (crossterm::terminal::enable_raw_mode, EnterAlternateScreen),
                    construct App, call app::run_app, restore terminal on exit.

app.rs            — App struct definition, App::new(config) constructor,
                    App::update() (real sysinfo path or mock path),
                    handle_key_event(), handle_mouse_event(),
                    run_app() event loop.

ui.rs             — Frame rendering dispatch:
                    fn render(frame: &mut Frame, app: &App)
                    → layout calculation, tab dispatch, component calls.

tabs/
  mod.rs          — Re-exports: pub use overview::draw as draw_overview; etc.
  overview.rs     — draw(frame, area, &SystemSnapshot) → 3-column gauge+sparkline
                    layout, detail text.
  processes.rs   — draw(frame, area, &SystemSnapshot, &mut TableState,
                    ProcessSortColumn) → Table widget with 6 columns,
                    sort indicator header, detail footer for selected row.
  about.rs        — draw(frame, area) → centered Paragraph with keybinding text.

components/
  mod.rs          — Re-exports.
  tab_bar.rs      — draw(frame, area, active_tab: Tab) → 3 Tabs rendered with
                    ratatui::widgets::Tabs, active one highlighted.
  status_bar.rs   — draw(frame, area, is_refreshing, interval, demo_mode)
                    → "● Refreshing... | 14:32:05 | Refresh: 1s [DEMO]"
  help_bar.rs     — draw(frame, area, active_tab: Tab) → context-sensitive
                    keybinding hints.

theme.rs          — Color constants as functions (so NO_COLOR is evaluated
                    once at startup): fn cpu_color(pct: f64) -> Color,
                    fn status_bg(is_refreshing: bool) -> Color,
                    COLOR_TEXT, COLOR_HIGHLIGHT, COLOR_DIM, etc.
                    fn no_color_mode() -> bool { std::env::var("NO_COLOR").is_ok() }

config.rs         — TuiConfig with serde::Deserialize, Default impl,
                    TuiConfig::load() (reads $XDG_CONFIG_HOME/<app>/config.toml,
                    then TUI_* env vars, then CLI flags).

mock.rs            — mock_snapshot() → returns a deterministic SystemSnapshot
                    with CPU oscillating in 30-60%, processes named after
                    common dev tools. Used when --demo or config.demo_mode is set.
```

### File tree:

```
apps/
├── cli/           (existing, unchanged)
└── tui/
    ├── Cargo.toml
    └── src/
        ├── main.rs
        ├── app.rs
        ├── ui.rs
        ├── config.rs
        ├── theme.rs
        ├── mock.rs
        ├── tabs/
        │   ├── mod.rs
        │   ├── overview.rs
        │   ├── processes.rs
        │   └── about.rs
        └── components/
            ├── mod.rs
            ├── tab_bar.rs
            ├── status_bar.rs
            └── help_bar.rs
```

### Makefile additions (in workspace Makefile):

```makefile
# New target for running the TUI app
run-tui: build
	@cargo run --bin gitflow-cli -- --demo

# Or if the TUI is the default binary:
run: build
	@cargo run
```

---

## 7. Keybinding Reference

### Global (processed before tab dispatch)

| Key | Action |
|---|---|
| `q` / `Esc` | Set `app.should_quit = true` |
| `1` | `app.active_tab = Tab::Overview` |
| `2` | `app.active_tab = Tab::Processes` |
| `3` | `app.active_tab = Tab::About` |
| `←` / `h` | `app.active_tab = app.active_tab.prev()` |
| `→` / `l` | `app.active_tab = app.active_tab.next()` |
| `r` | Force `app.update()` immediately |
| `f` | `app.refresh_interval = app.refresh_interval.next()` |
| `?` | Toggle `app.show_help_bar` |

### Tab-specific (processed only when that tab is active)

**Processes tab:**

| Key | Action |
|---|---|
| `↑` / `k` | `process_table_state.selected = ...prev` (scroll up) |
| `↓` / `j` | `process_table_state.selected = ...next` (scroll down) |
| `s` | Cycle `app.process_sort` (CPU → Memory → PID → Name → CPU) |
| `Home` | `process_table_state.select(Some(0))` |
| `End` | `process_table_state.select(Some(processes.len() - 1))` |
| `Enter` | (future: show process detail popup; for template: no-op with status message) |
| `Ctrl+d` / `PageDown` | Scroll page down |
| `Ctrl+u` / `PageUp` | Scroll page up |

### Mouse

| Event | Action |
|---|---|
| `MouseEventKind::Down(Left)` on tab area (row 0) | Compute column from x-coordinate, switch to that tab |
| `MouseEventKind::ScrollDown` in content area + Processes tab | `scroll_down(5)` |
| `MouseEventKind::ScrollUp` in content area + Processes tab | `scroll_up(5)` |

---

## 8. What is NOT in Scope

Kept out intentionally to keep the template lean:

- **No dynamic theme switching.** One dark theme. `NO_COLOR` handles accessibility.
- **No plugin system.**
- **No network monitoring** — adds `reqwest`/`pcap` deps and platform complexity.
- **No GPU monitoring** — highly platform-specific, adds heavy deps.
- **No save/export** — no CSV, JSON export of process lists or snapshots.
- **No process killing** — no `kill` or signal sending. Read-only safety.
- **No search/filter in process list** — keeps the table component simple.
- **No popup/modals** — keeps event handling simple (no stacked focus).
- **No async runtime** — `crossterm` event loop is synchronous. Tokio not needed.
- **No custom widgets** — stick to built-in ratatui widgets (`Gauge`, `Sparkline`, `Table`, `Paragraph`, `Tabs`, `Block`, `List`).
