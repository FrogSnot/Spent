//! Headless action executor for the optional Kriya governed-MCP bolt-on.
//! See `kriya-mcp/README.md` for the full picture and how to turn it on.
//!
//! `kriya-mcp` (the external governor process) spawns this binary and speaks a tiny line
//! protocol to it: one request per line on stdin —
//!     {"action": "<name>", "params": { ... }}
//! and one reply per line on stdout —
//!     {"success": true,  "data": <json>}        (on success)
//!     {"success": false, "error": "<message>"}  (on failure)
//!
//! Why a second binary instead of new code in the app? Because it lets the agent path reuse
//! Spent's EXACT data layer. This file pulls in the very same `database.rs` the Tauri UI uses
//! (via `#[path]`), so a human button-click (a `#[tauri::command]` in `main.rs`) and an AI
//! tool-call (here) run the identical `Database` method. There is no second implementation to
//! keep in sync, and nothing new to trust inside the app.
//!
//! This binary performs NO governance. Whether an action is allowed, needs human approval,
//! fits the budget, and how it is audited is decided entirely by `kriya-mcp` before it ever
//! sends a request here (see `kriya-mcp/agent-policy.yaml`). By the time a line arrives, the
//! action has already cleared every gate.
//!
//! The main app (`main.rs`) is untouched and unaware of this binary; if a user never wires it
//! into an assistant, it is inert and never runs.

#[path = "../database.rs"]
mod database;

use std::io::{BufRead, Write};
use std::path::PathBuf;

use database::{Database, NewTransaction};
use serde_json::{json, Value};

/// Unwrap a `Result<T, String>` param, or short-circuit the whole dispatch with a one-line
/// failure reply. Defined before its use sites because `macro_rules!` is textually scoped.
macro_rules! unwrap_or_fail {
    ($expr:expr) => {
        match $expr {
            Ok(value) => value,
            Err(message) => return fail(message),
        }
    };
}

fn main() {
    let db_path = resolve_db_path();
    let database = match Database::new(db_path.clone()) {
        Ok(db) => db,
        Err(err) => {
            // Without a database there is nothing to serve. Report once on stderr (kriya-mcp
            // inherits it) and exit non-zero so the operator sees the misconfiguration.
            eprintln!("[spent kriya_exec] cannot open database at {db_path:?}: {err}");
            std::process::exit(1);
        }
    };
    eprintln!("[spent kriya_exec] ready · db={db_path:?}");

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    // One request per line, one reply per line; EOF ends the session. This loop serves both
    // kriya-mcp executor modes — per-call (one line, then stdin closes) and --persistent
    // (many lines over a long-lived process).
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let reply = handle(&database, &line);
        if writeln!(out, "{reply}").is_err() {
            break;
        }
        // Flush so --persistent mode receives each reply immediately, not at EOF.
        let _ = out.flush();
    }
}

/// Parse one request line and dispatch it, always yielding exactly one JSON reply line.
fn handle(db: &Database, line: &str) -> String {
    let request: Value = match serde_json::from_str(line) {
        Ok(value) => value,
        Err(err) => return fail(format!("request was not valid JSON: {err}")),
    };
    let action = request.get("action").and_then(Value::as_str).unwrap_or_default();
    let params = request.get("params").cloned().unwrap_or_else(|| json!({}));
    dispatch(db, action, &params)
}

/// Map an action id to the matching `Database` method — the same methods `main.rs` exposes to
/// the UI. Unknown actions are rejected, so an agent can only ever reach this curated surface.
fn dispatch(db: &Database, action: &str, p: &Value) -> String {
    match action {
        // ---- Reads: safe for an agent to run unattended (policy auto-allows `get_*`). ----
        "get_containers" => reply(db.get_containers()),
        "get_categories" => reply(db.get_categories()),
        "get_db_path" => ok(json!(db.get_db_path())),
        "get_transactions" => {
            let cid = unwrap_or_fail!(req_i64(p, "container_id"));
            reply(db.get_transactions(cid, opt_i64(p, "limit")))
        }
        "get_transactions_for_month" => {
            let cid = unwrap_or_fail!(req_i64(p, "container_id"));
            let month = unwrap_or_fail!(req_str(p, "month"));
            reply(db.get_transactions_for_month(cid, month, opt_i64(p, "limit")))
        }
        "get_monthly_balance" => {
            let cid = unwrap_or_fail!(req_i64(p, "container_id"));
            reply(db.get_monthly_balance(cid))
        }
        "get_all_time_balance" => {
            let cid = unwrap_or_fail!(req_i64(p, "container_id"));
            reply(db.get_all_time_balance(cid))
        }
        "get_balance_for_month" => {
            let cid = unwrap_or_fail!(req_i64(p, "container_id"));
            let month = unwrap_or_fail!(req_str(p, "month"));
            reply(db.get_balance_for_month(cid, month))
        }
        "get_category_totals" => {
            let cid = unwrap_or_fail!(req_i64(p, "container_id"));
            reply(db.get_category_totals(cid))
        }
        "get_category_totals_for_month" => {
            let cid = unwrap_or_fail!(req_i64(p, "container_id"));
            let month = unwrap_or_fail!(req_str(p, "month"));
            reply(db.get_category_totals_for_month(cid, month))
        }
        "get_available_months" => {
            let cid = unwrap_or_fail!(req_i64(p, "container_id"));
            reply(db.get_available_months(cid))
        }
        "export_csv" => {
            let cid = unwrap_or_fail!(req_i64(p, "container_id"));
            reply(db.export_transactions_csv(cid))
        }

        // ---- Writes: allowed but audited (policy `add_*` / `update_*`). ----
        "add_transaction" => {
            let amount = unwrap_or_fail!(req_i64(p, "amount"));
            let container_id = unwrap_or_fail!(req_i64(p, "container_id"));
            reply(db.add_transaction(NewTransaction {
                amount,
                description: opt_str(p, "description"),
                category: opt_str(p, "category"),
                container_id,
            }))
        }
        "update_transaction" => {
            let id = unwrap_or_fail!(req_i64(p, "id"));
            let amount = unwrap_or_fail!(req_i64(p, "amount"));
            let description = unwrap_or_fail!(req_str(p, "description"));
            let category = unwrap_or_fail!(req_str(p, "category"));
            reply(db.update_transaction(id, amount, description, category))
        }
        "add_category" => {
            let name = unwrap_or_fail!(req_str(p, "name"));
            reply(db.add_category(name))
        }
        "add_container" => {
            let name = unwrap_or_fail!(req_str(p, "name"));
            reply(db.add_container(name))
        }
        "update_container" => {
            let id = unwrap_or_fail!(req_i64(p, "id"));
            let name = unwrap_or_fail!(req_str(p, "name"));
            reply(db.update_container(id, name))
        }

        // ---- Destructive / bulk: the policy makes each of these pause for human approval. ----
        "delete_transaction" => {
            let id = unwrap_or_fail!(req_i64(p, "id"));
            reply(db.delete_transaction(id))
        }
        "delete_category" => {
            let name = unwrap_or_fail!(req_str(p, "name"));
            reply(db.delete_category(name))
        }
        "delete_container" => {
            let id = unwrap_or_fail!(req_i64(p, "id"));
            reply(db.delete_container(id))
        }
        "clear_all_data" => reply(db.clear_all_data()),
        "import_csv" => {
            let csv_content = unwrap_or_fail!(req_str(p, "csv_content"));
            let container_id = unwrap_or_fail!(req_i64(p, "container_id"));
            let amount_column = unwrap_or_fail!(req_usize(p, "amount_column"));
            let description_column = unwrap_or_fail!(req_usize(p, "description_column"));
            let category_column = unwrap_or_fail!(req_usize(p, "category_column"));
            let date_column = unwrap_or_fail!(req_usize(p, "date_column"));
            let skip_header = opt_bool(p, "skip_header").unwrap_or(true);
            reply(db.import_transactions_from_csv(
                csv_content,
                container_id,
                amount_column,
                description_column,
                category_column,
                date_column,
                skip_header,
            ))
        }

        other => fail(format!("unknown action '{other}'")),
    }
}

// ---- reply helpers ----

fn ok(data: Value) -> String {
    json!({ "success": true, "data": data }).to_string()
}

fn fail(message: impl Into<String>) -> String {
    json!({ "success": false, "error": message.into() }).to_string()
}

/// Turn a `Database` method's `Result` into the wire reply. Generic over the error type so
/// this file never needs to name `rusqlite`.
fn reply<T: serde::Serialize, E: std::fmt::Display>(result: Result<T, E>) -> String {
    match result {
        Ok(value) => match serde_json::to_value(value) {
            Ok(data) => ok(data),
            Err(err) => fail(format!("could not serialize result: {err}")),
        },
        Err(err) => fail(err.to_string()),
    }
}

// ---- param extraction (clear, uniform error messages the agent can act on) ----

fn req_i64(p: &Value, key: &str) -> Result<i64, String> {
    p.get(key).and_then(Value::as_i64).ok_or_else(|| missing(key, "an integer"))
}

fn opt_i64(p: &Value, key: &str) -> Option<i64> {
    p.get(key).and_then(Value::as_i64)
}

fn req_usize(p: &Value, key: &str) -> Result<usize, String> {
    let n = p.get(key).and_then(Value::as_u64).ok_or_else(|| missing(key, "a non-negative integer"))?;
    Ok(n as usize)
}

fn req_str(p: &Value, key: &str) -> Result<String, String> {
    p.get(key).and_then(Value::as_str).map(str::to_string).ok_or_else(|| missing(key, "a string"))
}

fn opt_str(p: &Value, key: &str) -> Option<String> {
    p.get(key).and_then(Value::as_str).map(str::to_string)
}

fn opt_bool(p: &Value, key: &str) -> Option<bool> {
    p.get(key).and_then(Value::as_bool)
}

fn missing(key: &str, expected: &str) -> String {
    format!("parameter '{key}' is required and must be {expected}")
}

// ---- database location ----

/// Resolve the Spent SQLite file. Explicit overrides win (so an operator can point the agent at
/// a copy); otherwise fall back to the same per-OS location the app itself uses.
fn resolve_db_path() -> PathBuf {
    if let Ok(path) = std::env::var("SPENT_DB_PATH") {
        if !path.trim().is_empty() {
            return PathBuf::from(path);
        }
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(pos) = args.iter().position(|a| a == "--db") {
        if let Some(value) = args.get(pos + 1) {
            return PathBuf::from(value);
        }
    }
    default_db_path()
}

/// Mirror Tauri's `app_data_dir()/spent.db` for identifier `com.spent.app` (see README → Data).
fn default_db_path() -> PathBuf {
    const IDENTIFIER: &str = "com.spent.app";
    const DB_FILE: &str = "spent.db";

    let base = if cfg!(target_os = "macos") {
        PathBuf::from(home()).join("Library/Application Support")
    } else if cfg!(target_os = "windows") {
        PathBuf::from(std::env::var("APPDATA").unwrap_or_default())
    } else {
        std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(home()).join(".local/share"))
    };
    base.join(IDENTIFIER).join(DB_FILE)
}

fn home() -> String {
    std::env::var("HOME").unwrap_or_default()
}
