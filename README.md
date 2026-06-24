# Spent

Minimalist personal finance tracker for Linux desktop with cross-platform support.

![Spent UI](src-tauri/icons/Media/Media-1.png)

**[View More Screenshots](src-tauri/icons/Media)**

> Spent is feature-complete and stable. It has primarily been tested on Arch Linux. If you encounter issues on other distros or OSs, please open an Issue!

[![AUR votes](https://img.shields.io/aur/votes/spent?logo=arch-linux&style=flat-square&color=blue)](https://aur.archlinux.org/packages/spent)
[![GitHub stars](https://img.shields.io/github/stars/FrogSnot/Spent?style=flat-square&color=yellow)](https://github.com/FrogSnot/Spent)
![Tauri](https://img.shields.io/badge/Tauri-FFC131?style=flat-square&logo=tauri&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-000000?style=flat-square&logo=rust&logoColor=white)
![Svelte](https://img.shields.io/badge/Svelte-FF3E00?style=flat-square&logo=svelte&logoColor=white)
![SQLite](https://img.shields.io/badge/SQLite-003B57?style=flat-square&logo=sqlite&logoColor=white)

## Features

- **Containers**: Create separate balance containers to manage personal, business, or different accounts in complete isolation
- Offline-first with local SQLite storage
- Quick transaction entry (Ctrl+N)
- Monthly balance tracking with history
- Category-based organization
- Dark mode interface
- CSV export per container
- Cross-platform: Linux (.deb, .AppImage), Windows (.exe), macOS (.app)

## Stack

- Backend: Tauri v2 (Rust)
- Frontend: Svelte + TypeScript
- Database: SQLite
- UI: TailwindCSS

## Prerequisites

- Node.js
- Rust
- Linux dev dependencies:
  ```bash
  # Ubuntu/Debian
  sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev

  # Arch
  sudo pacman -S webkit2gtk-4.1 base-devel curl wget file openssl gtk3 libappindicator-gtk3 librsvg
  ```

## Development

```bash
npm install
npm run tauri dev
```

Build:
```bash
npm run tauri build
```

## Usage

- Launch: `spent-app` (binary renamed from `spent` in v1.1.5 to avoid conflict with opensp package)
- `Ctrl+N` - Add transaction
- `Ctrl+K` - Command palette (access all features)
- Hover transaction to edit/delete
- Switch between containers using the dropdown in sidebar
- Create/manage containers via Command Palette → "Manage Containers"

## Governed AI access (optional)

Spent is local-first with no API, so the only ways an AI assistant could operate it today are
screen-scraping or raw database access — both ungoverned. The optional [`kriya-mcp/`](kriya-mcp/)
integration instead exposes a curated set of Spent's existing actions to an assistant (e.g. Claude
Desktop) as a **governed MCP server**:

- **Reads** (balances, transactions, category totals) run freely.
- **Writes** (add/update transactions, categories, containers) are recorded in a signed audit log.
- **Destructive/bulk** actions (`delete_*`, `clear_all_data`, CSV import) pause for **human approval**.
- A per-minute **budget** caps a runaway agent; anything not allow-listed is **denied by default**.

It's **off by default** and changes nothing in the running app. Spent's binary gains a headless
`spent --exec` action handler (no window) — each governed action is one `wrap(...)` over an
existing `Database` method, built with kriya's lean Rust authoring SDK (no Tauri runtime, no HTTP
client added). The external [`kriya-mcp`](https://crates.io/crates/kriya) governor drives it,
enforcing policy, approval, budget, and a signed audit log around the **same `Database` methods the
UI calls**. See [`kriya-mcp/README.md`](kriya-mcp/README.md) to enable it.

## Data

The database is stored locally in platform-specific locations:

- **Linux**: `~/.local/share/com.spent.app/spent.db`
- **Windows**: `%APPDATA%\com.spent.app\spent.db` (typically `C:\Users\<username>\AppData\Roaming\com.spent.app\spent.db`)
- **macOS**: `~/Library/Application Support/com.spent.app/spent.db`

Money is stored as integers (cents) to avoid floating-point issues.

## Thank you to the amazing people that star this project!

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=FrogSnot/spent&type=date&legend=bottom-right)](https://www.star-history.com/#FrogSnot/spent&type=date&legend=bottom-right)


## License

AGPLv3
