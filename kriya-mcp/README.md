# Governed AI access for Spent (optional)

This folder turns Spent into something an AI assistant (Claude Desktop, Cursor, …) can safely
**operate** — not by screen-scraping or handing over raw database access, but by exposing a
curated set of Spent's *existing* actions as a **governed [MCP](https://modelcontextprotocol.io)
server**.

It is **off by default** and **changes nothing in the app**. If you never wire it into an
assistant, none of this runs.

> Why this exists: Spent is local-first with no API, so until now there was no *safe* way to let
> an assistant do things in it. The only options were ungoverned (scrape the screen, or open the
> SQLite file directly). This gives the agent a narrow, permissioned, audited door instead.

---

## What you get

- 🔒 **Permissions** — the agent can only call the actions you allow-list (everything else is
  denied by default).
- ✋ **Human approval** — destructive/bulk actions (`delete_*`, `clear_all_data`, `import_csv`)
  pause for a one-click yes/no before they touch your data.
- 🧾 **Signed audit log** — every executed agent action is recorded as an Ed25519-signed receipt
  you can replay later.
- 💸 **Budget cap** — at most N actions per rolling minute, so a looping agent can't run away.

## How it works

```
  Claude Desktop ──MCP/stdio──▶  kriya-mcp  ──one JSON line per action──▶  kriya_exec
  (the agent)                    (governor)                                (this app's data layer)
                                    │                                          │
                          policy ▸ approval ▸ budget ▸ audit            reuses Spent's exact
                          (agent-policy.yaml)                            Database methods → spent.db
```

- **`kriya_exec`** (`src-tauri/src/bin/kriya_exec.rs`) is a tiny second binary in *this* crate.
  It does **no** business logic of its own — it pulls in the very same `database.rs` the Tauri UI
  uses, so an AI tool-call runs the **identical `Database` method** a human button-click does.
  There is no second implementation to keep in sync.
- **`kriya-mcp`** is the external governor (from the open-source [`kriya`](https://crates.io/crates/kriya)
  crate). It speaks MCP to the assistant and, for every call, enforces the policy → approval →
  budget → signed-audit gates **before** forwarding the cleared action to `kriya_exec`. None of
  that governance lives in Spent.

## Enable it

1. **Build the executor** (release):
   ```bash
   cd src-tauri
   cargo build --release --bin kriya_exec
   # → src-tauri/target/release/kriya_exec
   ```
2. **Install the governor**:
   ```bash
   cargo install kriya        # provides the `kriya-mcp` binary on your PATH
   ```
3. **Point it at your data.** By default `kriya_exec` opens the same database the app uses
   (see the main README → *Data*). To target a copy instead, set `SPENT_DB_PATH` or pass
   `--db <path>`.
4. **Wire it into your assistant.** Copy the `mcpServers.spent` block from
   [`.mcp.json`](.mcp.json) into your assistant's MCP config (Claude Desktop on macOS:
   `~/Library/Application Support/Claude/claude_desktop_config.json`), replacing the
   `/ABSOLUTE/PATH/...` placeholders. Restart the assistant.
5. Ask it something read-only first, e.g. *"What did I spend on Food this month in my Personal
   container?"* Then try a write and watch the approval prompt appear for a delete.

## Governance model

| Tier | Actions | Policy |
|---|---|---|
| Read | `get_*`, `export_csv` | allow (no prompt) |
| Write | `add_transaction`, `update_transaction`, `add_category`, `add_container`, `update_container` | allow + audit |
| Destructive / bulk | `delete_transaction`, `delete_category`, `delete_container`, `clear_all_data`, `import_csv` | **human approval** + audit |
| Anything else | — | **denied** |

Edit [`agent-policy.yaml`](agent-policy.yaml) to tighten or loosen this — e.g. set a destructive
action to `allow: false` to forbid it outright, or drop `require_approval` to let it run audited.

## Approval on each OS

`kriya-mcp`'s approval prompt is `--approval gui` (a native macOS dialog that works even though
Claude Desktop has no terminal). On **Linux/Windows** there is no GUI gate yet, so:

- Run `kriya-mcp` from a terminal with `--approval tty` to get an interactive prompt, **or**
- keep the default and know that approval-required actions will simply be **denied** when there's
  no way to ask a human — the safe failure mode. Reads and routine writes are unaffected.

## Audit log

Executed actions are appended as signed JSONL receipts to `$TMPDIR/kriya-audit.jsonl` (override
with `kriya-mcp --audit-log <path>`). Each receipt is attributable to the `--actor` you set.

## Adding an action

The exposed surface is intentionally small. To add one:

1. Add a `match` arm in `dispatch()` in `src-tauri/src/bin/kriya_exec.rs` that calls the relevant
   `Database` method.
2. Add a tool entry to [`tools.json`](tools.json) (name = the action id, with its input schema).
3. Add a rule to [`agent-policy.yaml`](agent-policy.yaml) deciding its tier.

## What this does **not** do

- It does not add any dependency to the app or change the app's build/runtime — `kriya_exec` is a
  separate binary and the governor is a separate process.
- It makes no network calls (beyond the local stdio MCP pipe) and adds no telemetry.
- It does not expose anything not listed in `tools.json`; the policy denies everything else.
