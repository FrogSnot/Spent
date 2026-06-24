//! Governed-AI bolt-on for Spent, built with kriya's Rust authoring SDK (`kriya::Registry`).
//!
//! Each action is `wrap`ped once — a schema plus a closure that calls the **same `Database`
//! method the UI calls** (`crate::database`). The registry then both **generates the MCP tool
//! schemas** (`spent --dump-tools` → `kriya-mcp/tools.json`) and **dispatches** calls
//! (`spent --exec`). No hand-written tool schemas, no second implementation of the app's logic.
//!
//! Governance is *not* here: the external `kriya-mcp` binary speaks MCP to the assistant and
//! enforces policy → human approval → budget → signed audit, then drives `spent --exec` over a
//! one-line-per-call protocol (`{"action","params"}` in, `{"success","data"}` out). By the time a
//! line reaches us the action has already cleared every gate. See `kriya-mcp/README.md`.

use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::sync::Arc;

use kriya::{json_result, Action, Param, Registry};
use serde_json::{json, Value};

use crate::database::{Database, NewTransaction};

/// `spent --exec`: serve the registry over the kriya-mcp line protocol until EOF.
pub fn run() {
    let registry = build_registry(open_db());

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let reply = handle(&registry, &line);
        if writeln!(out, "{reply}").is_err() {
            break;
        }
        let _ = out.flush(); // flush so kriya-mcp --persistent gets each reply immediately
    }
}

/// `spent --dump-tools`: print the generated `tools.json` for `kriya-mcp --tools`.
pub fn dump_tools() {
    println!("{}", build_registry(open_db()).tools_json());
}

fn handle(registry: &Registry<Arc<Database>>, line: &str) -> String {
    let request: Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(err) => return json!({"success": false, "error": format!("request was not valid JSON: {err}")}).to_string(),
    };
    let action = request.get("action").and_then(Value::as_str).unwrap_or_default();
    let params = request.get("params").cloned().unwrap_or_else(|| json!({}));
    let outcome = registry.dispatch(action, &params);
    if outcome.success {
        json!({ "success": true, "data": outcome.data }).to_string()
    } else {
        json!({ "success": false, "error": outcome.error.unwrap_or_default() }).to_string()
    }
}

/// The whole bolt-on: wrap each existing `Database` method as a governed action. Reads and routine
/// writes flow; the destructive/bulk ones are tagged `require_approval` (the policy gates them).
fn build_registry(db: Arc<Database>) -> Registry<Arc<Database>> {
    let mut r = Registry::new(db);

    // ── reads ──
    r.wrap(Action::new("get_containers", "List all containers (isolated balance buckets)."),
        |db, _p| json_result(db.get_containers()));
    r.wrap(Action::new("get_categories", "List all category names."),
        |db, _p| json_result(db.get_categories()));
    r.wrap(Action::new("get_db_path", "Return the local filesystem path of the Spent database."),
        |db, _p| json_result::<_, String>(Ok(db.get_db_path())));
    r.wrap(Action::new("get_transactions", "List a container's transactions, newest first.")
            .param("container_id", Param::int()).param("limit", Param::int().optional()),
        |db, p| json_result(db.get_transactions(p.i64("container_id")?, p.opt_i64("limit"))));
    r.wrap(Action::new("get_transactions_for_month", "List a container's transactions for a month (YYYY-MM).")
            .param("container_id", Param::int()).param("month", Param::string()).param("limit", Param::int().optional()),
        |db, p| json_result(db.get_transactions_for_month(p.i64("container_id")?, p.str("month")?, p.opt_i64("limit"))));
    r.wrap(Action::new("get_monthly_balance", "Net balance (cents) for the current month in a container.")
            .param("container_id", Param::int()),
        |db, p| json_result(db.get_monthly_balance(p.i64("container_id")?)));
    r.wrap(Action::new("get_all_time_balance", "All-time net balance (cents) for a container.")
            .param("container_id", Param::int()),
        |db, p| json_result(db.get_all_time_balance(p.i64("container_id")?)));
    r.wrap(Action::new("get_balance_for_month", "Net balance (cents) for a month (YYYY-MM) in a container.")
            .param("container_id", Param::int()).param("month", Param::string()),
        |db, p| json_result(db.get_balance_for_month(p.i64("container_id")?, p.str("month")?)));
    r.wrap(Action::new("get_category_totals", "Spending totals per category for the current month.")
            .param("container_id", Param::int()),
        |db, p| json_result(db.get_category_totals(p.i64("container_id")?)));
    r.wrap(Action::new("get_category_totals_for_month", "Spending totals per category for a month (YYYY-MM).")
            .param("container_id", Param::int()).param("month", Param::string()),
        |db, p| json_result(db.get_category_totals_for_month(p.i64("container_id")?, p.str("month")?)));
    r.wrap(Action::new("get_available_months", "List the months (YYYY-MM) that have transactions in a container.")
            .param("container_id", Param::int()),
        |db, p| json_result(db.get_available_months(p.i64("container_id")?)));
    r.wrap(Action::new("export_csv", "Export a container's transactions as a CSV string.")
            .param("container_id", Param::int()),
        |db, p| json_result(db.export_transactions_csv(p.i64("container_id")?)));

    // ── writes (audited) ──
    r.wrap(Action::new("add_transaction", "Add a transaction. amount is cents: negative = expense, positive = income.")
            .param("amount", Param::int()).param("container_id", Param::int())
            .param("description", Param::string().optional()).param("category", Param::string().optional()),
        |db, p| json_result(db.add_transaction(NewTransaction {
            amount: p.i64("amount")?,
            description: p.opt_str("description"),
            category: p.opt_str("category"),
            container_id: p.i64("container_id")?,
        })));
    r.wrap(Action::new("update_transaction", "Update a transaction's amount (cents), description and category.")
            .param("id", Param::int()).param("amount", Param::int())
            .param("description", Param::string()).param("category", Param::string()),
        |db, p| json_result(db.update_transaction(p.i64("id")?, p.i64("amount")?, p.str("description")?, p.str("category")?)));
    r.wrap(Action::new("add_category", "Create a new category.").param("name", Param::string()),
        |db, p| json_result(db.add_category(p.str("name")?)));
    r.wrap(Action::new("add_container", "Create a new container (isolated balance bucket).").param("name", Param::string()),
        |db, p| json_result(db.add_container(p.str("name")?)));
    r.wrap(Action::new("update_container", "Rename an existing container.")
            .param("id", Param::int()).param("name", Param::string()),
        |db, p| json_result(db.update_container(p.i64("id")?, p.str("name")?)));

    // ── destructive / bulk — tagged for human approval; the policy enforces it ──
    r.wrap(Action::new("delete_transaction", "Permanently delete a transaction by id.").param("id", Param::int()).require_approval(),
        |db, p| json_result(db.delete_transaction(p.i64("id")?)));
    r.wrap(Action::new("delete_category", "Delete a non-default category by name.").param("name", Param::string()).require_approval(),
        |db, p| json_result(db.delete_category(p.str("name")?)));
    r.wrap(Action::new("delete_container", "Delete a non-default container by id (cascades its transactions).").param("id", Param::int()).require_approval(),
        |db, p| json_result(db.delete_container(p.i64("id")?)));
    r.wrap(Action::new("clear_all_data", "Delete ALL transactions and non-default containers. Irreversible.").require_approval(),
        |db, _p| json_result(db.clear_all_data()));
    r.wrap(Action::new("import_csv", "Bulk-import transactions from CSV text into a container.")
            .param("csv_content", Param::string()).param("container_id", Param::int())
            .param("amount_column", Param::int()).param("description_column", Param::int())
            .param("category_column", Param::int()).param("date_column", Param::int())
            .param("skip_header", Param::boolean().optional()).require_approval(),
        |db, p| json_result(db.import_transactions_from_csv(
            p.str("csv_content")?, p.i64("container_id")?,
            p.usize("amount_column")?, p.usize("description_column")?,
            p.usize("category_column")?, p.usize("date_column")?,
            p.opt_bool("skip_header").unwrap_or(true),
        )));

    r
}

/// Open the Spent database. `$SPENT_DB_PATH` (or `--db <path>`) wins, else the same per-OS location
/// the app uses — `dirs::data_dir()/com.spent.app/spent.db`, exactly what Tauri's `app_data_dir()`
/// resolves to (both wrap the `dirs` crate), so the agent reads the file the UI writes.
fn open_db() -> Arc<Database> {
    let path = resolve_db_path();
    match Database::new(path.clone()) {
        Ok(db) => Arc::new(db),
        Err(err) => {
            eprintln!("[spent --exec] cannot open database at {path:?}: {err}");
            std::process::exit(1);
        }
    }
}

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
    dirs::data_dir().expect("could not determine the OS data directory").join("com.spent.app").join("spent.db")
}
