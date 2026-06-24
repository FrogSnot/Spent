# Governed AI access for Spent (optional)

Let an AI assistant (Claude Desktop, Cursor, …) **operate** Spent safely — not by screen-scraping
or raw database access, but through a curated, governed set of Spent's *existing* actions exposed
as an [MCP](https://modelcontextprotocol.io) server.

**Off by default. Changes nothing in the app.** If you never wire it into an assistant, none of
this runs.

## What you get

- 🔒 **Permissions** — the agent can only call allow-listed actions; everything else is denied.
- ✋ **Human approval** — destructive/bulk actions (`delete_*`, `clear_all_data`, `import_csv`)
  pause for a one-click yes/no before they touch your data.
- 🧾 **Signed audit log** — every executed action is an Ed25519-signed receipt you can replay.
- 💸 **Budget cap** — at most N actions per rolling minute, so a looping agent can't run away.

## How it works

```
  Claude Desktop ──MCP/stdio──▶  kriya-mcp  ──{action,params} per line──▶  spent --exec
  (the agent)                    (governor)                                (Spent's data layer)
                                 policy ▸ approval ▸ budget ▸ signed audit
```

- **`kriya-mcp`** (from the open-source [`kriya`](https://crates.io/crates/kriya) crate) speaks MCP
  to the assistant and enforces every gate **before** forwarding a cleared action. The signing key
  stays inside it — the agent can *propose* an action but can't approve its own guarded call or
  forge a receipt. None of this governance lives in Spent.
- **`spent --exec`** is Spent's own binary in a headless handler mode (no window). It reads one
  `{"action","params"}` line and runs the **same `Database` method a button-click runs**, then
  replies `{"success","data"}`. No second binary, no duplicated logic. The wiring uses kriya's Rust
  authoring SDK ([`kriya::Registry`](https://crates.io/crates/kriya), `default-features = false` —
  no Tauri runtime, no HTTP client): each action is one `wrap(...)` over an existing `Database`
  method, and the same registry **generates `tools.json`** (`spent --dump-tools`). See
  `src-tauri/src/exec.rs`.

## Enable it

1. **Build Spent** and **install the governor**:
   ```bash
   ( cd src-tauri && cargo build --release )   # → target/release/spent-app
   cargo install kriya                          # provides the `kriya-mcp` binary
   ```
2. **Wire it into your assistant.** Copy the `mcpServers.spent` block from [`.mcp.json`](.mcp.json)
   into your assistant's MCP config (Claude Desktop on macOS:
   `~/Library/Application Support/Claude/claude_desktop_config.json`), filling in the
   `/ABSOLUTE/PATH/...` placeholders. Restart the assistant.
   - By default it opens the same database the app uses; set `SPENT_DB_PATH` to target a copy.
3. Ask a read first (*"What did I spend on Food this month?"*), then a delete — and watch the
   approval prompt appear.

## Governance model

| Tier | Actions | Policy |
|---|---|---|
| Read | `get_*`, `export_csv` | allow (no prompt) |
| Write | `add_*`, `update_*` | allow + audit |
| Destructive / bulk | `delete_*`, `clear_all_data`, `import_csv` | **human approval** + audit |
| Anything else | — | **denied** |

Edit [`agent-policy.yaml`](agent-policy.yaml) to tighten or loosen this. Approval uses
`--approval gui` (native macOS dialog). On Linux/Windows use `--approval tty` from a terminal, or
keep the default and approval-required actions are simply **denied** (the safe failure mode; reads
and routine writes are unaffected). Audit receipts append to `$TMPDIR/kriya-audit.jsonl`
(`--audit-log` to change), attributed to `--actor`.

## Adding an action

1. Add one `r.wrap(Action::new(...)..., |db, p| json_result(db.your_method(...)))` in
   [`../src-tauri/src/exec.rs`](../src-tauri/src/exec.rs).
2. Regenerate the schemas: `spent --dump-tools > kriya-mcp/tools.json`.
3. Add a rule to [`agent-policy.yaml`](agent-policy.yaml) if it needs gating.

## What this does **not** do

- No change to the running app — `--exec` is a separate startup mode, never hit on normal launch.
- The only dependency it adds to Spent is the **kriya authoring SDK** (`default-features = false`):
  no Tauri runtime, no HTTP/network client. The `kriya-mcp` governor is a separate process you
  install (`cargo install kriya`).
- No network calls (beyond the local stdio pipe), no telemetry, nothing outside `tools.json`.
