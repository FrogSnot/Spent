use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use csv::ReaderBuilder;

#[derive(Debug, Serialize, Deserialize)]
pub struct Container {
    pub id: i64,
    pub name: String,
    pub created_at: String,
    pub is_default: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub id: i64,
    pub amount: i64,
    pub description: String,
    pub category: String,
    pub date: String,
    pub container_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewTransaction {
    pub amount: i64,
    pub description: Option<String>,
    pub category: Option<String>,
    pub container_id: i64,
}

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS containers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL,
                is_default INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;

        let container_count: i64 = conn.query_row("SELECT COUNT(*) FROM containers", [], |row| row.get(0))?;
        if container_count == 0 {
            let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            conn.execute(
                "INSERT INTO containers (name, created_at, is_default) VALUES (?1, ?2, 1)",
                ["Personal", &now],
            )?;
        }

        conn.execute(
            "CREATE TABLE IF NOT EXISTS transactions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                amount INTEGER NOT NULL,
                description TEXT NOT NULL,
                category TEXT NOT NULL,
                date TEXT NOT NULL,
                container_id INTEGER NOT NULL DEFAULT 1,
                FOREIGN KEY (container_id) REFERENCES containers(id) ON DELETE CASCADE
            )",
            [],
        )?;

        let has_container_id: Result<i64, _> = conn.query_row(
            "SELECT COUNT(*) FROM pragma_table_info('transactions') WHERE name='container_id'",
            [],
            |row| row.get(0)
        );
        
        if let Ok(0) = has_container_id {
            conn.execute(
                "ALTER TABLE transactions ADD COLUMN container_id INTEGER NOT NULL DEFAULT 1",
                [],
            )?;
        }

        conn.execute(
            "CREATE TABLE IF NOT EXISTS categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                is_default INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;

        let count: i64 = conn.query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))?;
        if count == 0 {
            let defaults = vec![
                "Food & Dining",
                "Transportation",
                "Shopping",
                "Entertainment",
                "Bills & Utilities",
                "Healthcare",
                "Income",
                "Other"
            ];
            for category in defaults {
                conn.execute(
                    "INSERT INTO categories (name, is_default) VALUES (?1, 1)",
                    [category],
                )?;
            }
        }

        Ok(Database {
            conn: Mutex::new(conn),
        })
    }

    pub fn add_transaction(&self, transaction: NewTransaction) -> Result<Transaction> {
        let conn = self.conn.lock().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        let description = transaction.description.unwrap_or_else(|| "Untitled".to_string());
        let category = transaction.category.unwrap_or_else(|| "Other".to_string());
        
        conn.execute(
            "INSERT INTO transactions (amount, description, category, date, container_id) VALUES (?1, ?2, ?3, ?4, ?5)",
            [
                &transaction.amount.to_string(),
                &description,
                &category,
                &date,
                &transaction.container_id.to_string(),
            ],
        )?;

        let id = conn.last_insert_rowid();
        
        Ok(Transaction {
            id,
            amount: transaction.amount,
            description,
            category,
            date,
            container_id: transaction.container_id,
        })
    }

    pub fn get_transactions(&self, container_id: i64, limit: Option<i64>) -> Result<Vec<Transaction>> {
        let conn = self.conn.lock().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let (query, params): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = match limit {
            Some(l) => (
                "SELECT id, amount, description, category, date, container_id FROM transactions WHERE container_id = ?1 ORDER BY date DESC LIMIT ?2".to_string(),
                vec![Box::new(container_id), Box::new(l)],
            ),
            None => (
                "SELECT id, amount, description, category, date, container_id FROM transactions WHERE container_id = ?1 ORDER BY date DESC".to_string(),
                vec![Box::new(container_id)],
            ),
        };

        let mut stmt = conn.prepare(&query)?;
        let params_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let transactions = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(Transaction {
                id: row.get(0)?,
                amount: row.get(1)?,
                description: row.get(2)?,
                category: row.get(3)?,
                date: row.get(4)?,
                container_id: row.get(5)?,
            })
        })?;

        transactions.collect()
    }

    pub fn update_transaction(
        &self,
        id: i64,
        amount: i64,
        description: String,
        category: String,
    ) -> Result<Transaction> {
        let conn = self.conn.lock().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        
        conn.execute(
            "UPDATE transactions SET amount = ?1, description = ?2, category = ?3 WHERE id = ?4",
            [&amount.to_string(), &description, &category, &id.to_string()],
        )?;

        let transaction = conn.query_row(
            "SELECT id, amount, description, category, date, container_id FROM transactions WHERE id = ?1",
            [id],
            |row| {
                Ok(Transaction {
                    id: row.get(0)?,
                    amount: row.get(1)?,
                    description: row.get(2)?,
                    category: row.get(3)?,
                    date: row.get(4)?,
                    container_id: row.get(5)?,
                })
            },
        )?;

        Ok(transaction)
    }

    pub fn get_monthly_balance(&self, container_id: i64) -> Result<i64> {
        let conn = self.conn.lock().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let current_month = chrono::Local::now().format("%Y-%m").to_string();
        
        let balance: i64 = conn.query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM transactions WHERE container_id = ?1 AND date LIKE ?2",
            [&container_id.to_string(), &format!("{}%", current_month)],
            |row| row.get(0),
        )?;

        Ok(balance)
    }

    pub fn get_all_time_balance(&self, container_id: i64) -> Result<i64> {
        let conn = self.conn.lock().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        
        let balance: i64 = conn.query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM transactions WHERE container_id = ?1",
            [container_id],
            |row| row.get(0),
        )?;

        Ok(balance)
    }

    pub fn export_transactions_csv(&self, container_id: i64) -> Result<String> {
        let conn = self.conn.lock().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, amount, description, category, date FROM transactions WHERE container_id = ?1 ORDER BY date DESC"
        )?;
        
        let mut wtr = csv::Writer::from_writer(vec![]);
        wtr.write_record(&["ID", "Amount", "Description", "Category", "Date"]).map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

        let rows = stmt.query_map([container_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
            ))
        })?;

        for row in rows {
            let (id, amount, desc, cat, date) = row?;
            let dollars = (amount as f64) / 100.0;
            wtr.write_record(&[
                id.to_string(),
                format!("{:.2}", dollars),
                desc,
                cat,
                date,
            ]).map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        }

        let csv_bytes = wtr.into_inner().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        String::from_utf8(csv_bytes).map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))
    }

    pub fn delete_transaction(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        conn.execute("DELETE FROM transactions WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn get_category_totals(&self, container_id: i64) -> Result<Vec<(String, i64)>> {
        let conn = self.conn.lock().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let current_month = chrono::Local::now().format("%Y-%m").to_string();
        
        let mut stmt = conn.prepare(
            "SELECT category, SUM(amount) as total 
             FROM transactions 
             WHERE container_id = ?1 AND date LIKE ?2 AND amount < 0
             GROUP BY category 
             ORDER BY total ASC"
        )?;
        
        let results = stmt.query_map([&container_id.to_string(), &format!("{}%", current_month)], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;
        
        results.collect()
    }

    pub fn get_categories(&self) -> Result<Vec<String>> {
        let conn = self.conn.lock().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let mut stmt = conn.prepare("SELECT name FROM categories ORDER BY is_default DESC, name ASC")?;
        
        let categories = stmt.query_map([], |row| row.get(0))?;
        categories.collect()
    }

    pub fn add_category(&self, name: String) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        conn.execute(
            "INSERT INTO categories (name, is_default) VALUES (?1, 0)",
            [name],
        )?;
        Ok(())
    }

    pub fn delete_category(&self, name: String) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        conn.execute(
            "DELETE FROM categories WHERE name = ?1 AND is_default = 0",
            [name],
        )?;
        Ok(())
    }

    pub fn get_available_months(&self, container_id: i64) -> Result<Vec<String>> {
        let conn = self.conn.lock().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT DISTINCT strftime('%Y-%m', date) as month 
             FROM transactions 
             WHERE container_id = ?1
             ORDER BY month DESC"
        )?;
        
        let months = stmt.query_map([container_id], |row| row.get(0))?;
        months.collect()
    }

    pub fn get_balance_for_month(&self, container_id: i64, month: String) -> Result<i64> {
        let conn = self.conn.lock().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        
        let balance: i64 = conn.query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM transactions WHERE container_id = ?1 AND date LIKE ?2",
            [&container_id.to_string(), &format!("{}%", month)],
            |row| row.get(0),
        )?;

        Ok(balance)
    }

    pub fn get_transactions_for_month(&self, container_id: i64, month: String, limit: Option<i64>) -> Result<Vec<Transaction>> {
        let conn = self.conn.lock().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let month_pattern = format!("{}%", month);
        let (query, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = match limit {
            Some(l) => (
                "SELECT id, amount, description, category, date, container_id FROM transactions WHERE container_id = ?1 AND date LIKE ?2 ORDER BY date DESC LIMIT ?3",
                vec![Box::new(container_id), Box::new(month_pattern.clone()), Box::new(l)],
            ),
            None => (
                "SELECT id, amount, description, category, date, container_id FROM transactions WHERE container_id = ?1 AND date LIKE ?2 ORDER BY date DESC",
                vec![Box::new(container_id), Box::new(month_pattern.clone())],
            ),
        };

        let mut stmt = conn.prepare(query)?;
        let params_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let transactions = stmt.query_map(params_refs.as_slice(), |row| {
            Ok(Transaction {
                id: row.get(0)?,
                amount: row.get(1)?,
                description: row.get(2)?,
                category: row.get(3)?,
                date: row.get(4)?,
                container_id: row.get(5)?,
            })
        })?;

        transactions.collect()
    }

    pub fn get_category_totals_for_month(&self, container_id: i64, month: String) -> Result<Vec<(String, i64)>> {
        let conn = self.conn.lock().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT category, SUM(amount) as total 
             FROM transactions 
             WHERE container_id = ?1 AND amount < 0 AND date LIKE ?2
             GROUP BY category 
             ORDER BY total ASC"
        )?;

        let results = stmt.query_map([&container_id.to_string(), &format!("{}%", month)], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;
        
        results.collect()
    }

    pub fn get_containers(&self) -> Result<Vec<Container>> {
        let conn = self.conn.lock().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let mut stmt = conn.prepare("SELECT id, name, created_at, is_default FROM containers ORDER BY is_default DESC, created_at ASC")?;
        
        let containers = stmt.query_map([], |row| {
            Ok(Container {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
                is_default: row.get::<_, i64>(3)? == 1,
            })
        })?;
        
        containers.collect()
    }

    pub fn add_container(&self, name: String) -> Result<Container> {
        let conn = self.conn.lock().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        conn.execute(
            "INSERT INTO containers (name, created_at, is_default) VALUES (?1, ?2, 0)",
            [&name, &now],
        )?;

        let id = conn.last_insert_rowid();
        
        Ok(Container {
            id,
            name,
            created_at: now,
            is_default: false,
        })
    }

    pub fn delete_container(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        
        let is_default: i64 = conn.query_row(
            "SELECT is_default FROM containers WHERE id = ?1",
            [id],
            |row| row.get(0),
        )?;
        
        if is_default == 1 {
            return Err(rusqlite::Error::InvalidParameterName("Cannot delete default container".to_string()));
        }
        
        conn.execute("DELETE FROM containers WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn update_container(&self, id: i64, name: String) -> Result<Container> {
        let conn = self.conn.lock().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        
        conn.execute(
            "UPDATE containers SET name = ?1 WHERE id = ?2",
            [&name, &id.to_string()],
        )?;

        let container = conn.query_row(
            "SELECT id, name, created_at, is_default FROM containers WHERE id = ?1",
            [id],
            |row| {
                Ok(Container {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    created_at: row.get(2)?,
                    is_default: row.get::<_, i64>(3)? == 1,
                })
            },
        )?;

        Ok(container)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportResult {
    pub success_count: usize,
    pub error_count: usize,
    pub errors: Vec<String>,
}

impl Database {
    pub fn import_transactions_from_csv(
        &self,
        csv_content: String,
        container_id: i64,
        amount_column: usize,
        description_column: usize,
        category_column: usize,
        date_column: usize,
        skip_header: bool,
    ) -> Result<ImportResult> {
        let mut reader = ReaderBuilder::new()
            .has_headers(skip_header)
            .from_reader(csv_content.as_bytes());

        let mut success_count = 0;
        let mut error_count = 0;
        let mut errors = Vec::new();

        for (index, result) in reader.records().enumerate() {
            let row_num = if skip_header { index + 2 } else { index + 1 };
            
            match result {
                Ok(record) => {
                    let amount_str = record.get(amount_column).unwrap_or("").trim();
                    let description = record.get(description_column).unwrap_or("Imported").trim().to_string();
                    let category = record.get(category_column).unwrap_or("Other").trim().to_string();
                    let date_str = record.get(date_column).unwrap_or("").trim();

                    let amount_cents = match Self::parse_amount(amount_str) {
                        Ok(amt) => amt,
                        Err(e) => {
                            errors.push(format!("Row {}: Invalid amount '{}' - {}", row_num, amount_str, e));
                            error_count += 1;
                            continue;
                        }
                    };

                    let parsed_date = match Self::parse_date(date_str) {
                        Ok(date) => date,
                        Err(e) => {
                            errors.push(format!("Row {}: Invalid date '{}' - {}", row_num, date_str, e));
                            error_count += 1;
                            continue;
                        }
                    };

                    match self.insert_imported_transaction(
                        container_id,
                        amount_cents,
                        description,
                        category,
                        parsed_date,
                    ) {
                        Ok(_) => success_count += 1,
                        Err(e) => {
                            errors.push(format!("Row {}: Failed to insert - {}", row_num, e));
                            error_count += 1;
                        }
                    }
                }
                Err(e) => {
                    errors.push(format!("Row {}: Failed to parse CSV - {}", row_num, e));
                    error_count += 1;
                }
            }
        }

        Ok(ImportResult {
            success_count,
            error_count,
            errors,
        })
    }

    fn parse_amount(amount_str: &str) -> Result<i64, String> {
        let cleaned = amount_str
            .replace("$", "")
            .replace("€", "")
            .replace("£", "")
            .replace(",", "")
            .trim()
            .to_string();

        match cleaned.parse::<f64>() {
            Ok(amount) => Ok((amount * 100.0).round() as i64),
            Err(_) => Err(format!("Cannot parse as number")),
        }
    }

    fn parse_date(date_str: &str) -> Result<String, String> {
        let formats = vec![
            "%Y-%m-%d",
            "%m/%d/%Y",
            "%d/%m/%Y",
            "%Y/%m/%d",
            "%m-%d-%Y",
            "%d-%m-%Y",
            "%Y-%m-%d %H:%M:%S",
            "%m/%d/%Y %H:%M",
        ];

        for format in formats {
            if let Ok(parsed) = chrono::NaiveDateTime::parse_from_str(&format!("{} 00:00:00", date_str), "%Y-%m-%d %H:%M:%S") {
                return Ok(parsed.format("%Y-%m-%d %H:%M:%S").to_string());
            }
            if let Ok(parsed) = chrono::NaiveDateTime::parse_from_str(date_str, format) {
                return Ok(parsed.format("%Y-%m-%d %H:%M:%S").to_string());
            }
            if let Ok(parsed) = chrono::NaiveDate::parse_from_str(date_str, format) {
                let datetime = parsed.and_hms_opt(0, 0, 0).unwrap();
                return Ok(datetime.format("%Y-%m-%d %H:%M:%S").to_string());
            }
        }

        Err("Unsupported date format".to_string())
    }

    fn insert_imported_transaction(
        &self,
        container_id: i64,
        amount: i64,
        description: String,
        category: String,
        date: String,
    ) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
        
        conn.execute(
            "INSERT INTO transactions (amount, description, category, date, container_id) VALUES (?1, ?2, ?3, ?4, ?5)",
            [
                &amount.to_string(),
                &description,
                &category,
                &date,
                &container_id.to_string(),
            ],
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_db() -> Database {
        Database::new(PathBuf::from(":memory:")).expect("Failed to create in-memory database")
    }

    #[test]
    fn init_creates_default_container() {
        let db = test_db();
        let containers = db.get_containers().unwrap();
        assert_eq!(containers.len(), 1);
        assert_eq!(containers[0].name, "Personal");
        assert!(containers[0].is_default);
    }

    #[test]
    fn init_creates_default_categories() {
        let db = test_db();
        let categories = db.get_categories().unwrap();
        assert!(!categories.is_empty());
        assert!(categories.contains(&"Food & Dining".to_string()));
        assert!(categories.contains(&"Income".to_string()));
        assert!(categories.contains(&"Other".to_string()));
    }

    #[test]
    fn add_and_get_transaction() {
        let db = test_db();
        let tx = db.add_transaction(NewTransaction {
            amount: -1500,
            description: Some("Groceries".into()),
            category: Some("Food & Dining".into()),
            container_id: 1,
        }).unwrap();

        assert_eq!(tx.amount, -1500);
        assert_eq!(tx.description, "Groceries");
        assert_eq!(tx.category, "Food & Dining");
        assert_eq!(tx.container_id, 1);

        let transactions = db.get_transactions(1, None).unwrap();
        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].id, tx.id);
    }

    #[test]
    fn add_transaction_defaults() {
        let db = test_db();
        let tx = db.add_transaction(NewTransaction {
            amount: 5000,
            description: None,
            category: None,
            container_id: 1,
        }).unwrap();

        assert_eq!(tx.description, "Untitled");
        assert_eq!(tx.category, "Other");
    }

    #[test]
    fn get_transactions_respects_limit() {
        let db = test_db();
        for i in 0..5 {
            db.add_transaction(NewTransaction {
                amount: i * 100,
                description: None,
                category: None,
                container_id: 1,
            }).unwrap();
        }

        let all = db.get_transactions(1, None).unwrap();
        assert_eq!(all.len(), 5);

        let limited = db.get_transactions(1, Some(3)).unwrap();
        assert_eq!(limited.len(), 3);
    }

    #[test]
    fn update_transaction() {
        let db = test_db();
        let tx = db.add_transaction(NewTransaction {
            amount: -500,
            description: Some("Coffee".into()),
            category: Some("Food & Dining".into()),
            container_id: 1,
        }).unwrap();

        let updated = db.update_transaction(tx.id, -750, "Latte".into(), "Shopping".into()).unwrap();
        assert_eq!(updated.amount, -750);
        assert_eq!(updated.description, "Latte");
        assert_eq!(updated.category, "Shopping");
    }

    #[test]
    fn delete_transaction() {
        let db = test_db();
        let tx = db.add_transaction(NewTransaction {
            amount: -100,
            description: None,
            category: None,
            container_id: 1,
        }).unwrap();

        db.delete_transaction(tx.id).unwrap();
        let transactions = db.get_transactions(1, None).unwrap();
        assert!(transactions.is_empty());
    }

    #[test]
    fn balance_calculations() {
        let db = test_db();
        db.add_transaction(NewTransaction {
            amount: 10000,
            description: Some("Salary".into()),
            category: Some("Income".into()),
            container_id: 1,
        }).unwrap();
        db.add_transaction(NewTransaction {
            amount: -3000,
            description: Some("Groceries".into()),
            category: Some("Food & Dining".into()),
            container_id: 1,
        }).unwrap();

        let monthly = db.get_monthly_balance(1).unwrap();
        assert_eq!(monthly, 7000);

        let all_time = db.get_all_time_balance(1).unwrap();
        assert_eq!(all_time, 7000);
    }

    #[test]
    fn container_isolation() {
        let db = test_db();
        let c2 = db.add_container("Business".into()).unwrap();

        db.add_transaction(NewTransaction {
            amount: 5000,
            description: None,
            category: None,
            container_id: 1,
        }).unwrap();
        db.add_transaction(NewTransaction {
            amount: 9000,
            description: None,
            category: None,
            container_id: c2.id,
        }).unwrap();

        assert_eq!(db.get_all_time_balance(1).unwrap(), 5000);
        assert_eq!(db.get_all_time_balance(c2.id).unwrap(), 9000);

        let t1 = db.get_transactions(1, None).unwrap();
        let t2 = db.get_transactions(c2.id, None).unwrap();
        assert_eq!(t1.len(), 1);
        assert_eq!(t2.len(), 1);
    }

    #[test]
    fn add_and_delete_category() {
        let db = test_db();
        let before = db.get_categories().unwrap().len();

        db.add_category("Custom".into()).unwrap();
        let after = db.get_categories().unwrap();
        assert_eq!(after.len(), before + 1);
        assert!(after.contains(&"Custom".to_string()));

        db.delete_category("Custom".into()).unwrap();
        assert_eq!(db.get_categories().unwrap().len(), before);
    }

    #[test]
    fn cannot_delete_default_category() {
        let db = test_db();
        let before = db.get_categories().unwrap().len();
        db.delete_category("Food & Dining".into()).unwrap();
        assert_eq!(db.get_categories().unwrap().len(), before);
    }

    #[test]
    fn add_update_delete_container() {
        let db = test_db();
        let c = db.add_container("Savings".into()).unwrap();
        assert_eq!(c.name, "Savings");
        assert!(!c.is_default);

        let updated = db.update_container(c.id, "Emergency Fund".into()).unwrap();
        assert_eq!(updated.name, "Emergency Fund");

        db.delete_container(c.id).unwrap();
        let containers = db.get_containers().unwrap();
        assert_eq!(containers.len(), 1);
    }

    #[test]
    fn cannot_delete_default_container() {
        let db = test_db();
        let result = db.delete_container(1);
        assert!(result.is_err());
    }

    #[test]
    fn export_csv() {
        let db = test_db();
        db.add_transaction(NewTransaction {
            amount: -1050,
            description: Some("Coffee".into()),
            category: Some("Food & Dining".into()),
            container_id: 1,
        }).unwrap();

        let csv = db.export_transactions_csv(1).unwrap();
        assert!(csv.contains("ID,Amount,Description,Category,Date"));
        assert!(csv.contains("-10.50"));
        assert!(csv.contains("Coffee"));
    }

    #[test]
    fn parse_amount_various_formats() {
        assert_eq!(Database::parse_amount("10.50").unwrap(), 1050);
        assert_eq!(Database::parse_amount("-5.99").unwrap(), -599);
        assert_eq!(Database::parse_amount("$100.00").unwrap(), 10000);
        assert_eq!(Database::parse_amount("€25.50").unwrap(), 2550);
        assert_eq!(Database::parse_amount("1,234.56").unwrap(), 123456);
        assert!(Database::parse_amount("abc").is_err());
    }

    #[test]
    fn parse_date_various_formats() {
        let result = Database::parse_date("2025-06-15").unwrap();
        assert!(result.starts_with("2025-06-15"));

        let result = Database::parse_date("06/15/2025").unwrap();
        assert!(result.contains("2025"));

        assert!(Database::parse_date("not-a-date").is_err());
    }

    #[test]
    fn import_csv() {
        let db = test_db();
        let csv = "Amount,Description,Category,Date\n-10.50,Coffee,Food & Dining,2025-01-15\n50.00,Refund,Income,2025-01-16\n";
        let result = db.import_transactions_from_csv(
            csv.into(), 1, 0, 1, 2, 3, true,
        ).unwrap();

        assert_eq!(result.success_count, 2);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn import_csv_handles_bad_rows() {
        let db = test_db();
        let csv = "amt,desc,cat,date\nabc,Bad,Other,2025-01-01\n10.00,Good,Other,2025-01-01\n";
        let result = db.import_transactions_from_csv(
            csv.into(), 1, 0, 1, 2, 3, true,
        ).unwrap();

        assert_eq!(result.success_count, 1);
        assert_eq!(result.error_count, 1);
        assert!(result.errors[0].contains("Row 2"));
    }

    #[test]
    fn available_months() {
        let db = test_db();
        db.add_transaction(NewTransaction {
            amount: 1000,
            description: None,
            category: None,
            container_id: 1,
        }).unwrap();

        let months = db.get_available_months(1).unwrap();
        assert_eq!(months.len(), 1);

        let current_month = chrono::Local::now().format("%Y-%m").to_string();
        assert_eq!(months[0], current_month);
    }

    #[test]
    fn category_totals_only_counts_expenses() {
        let db = test_db();
        db.add_transaction(NewTransaction {
            amount: 10000,
            description: Some("Salary".into()),
            category: Some("Income".into()),
            container_id: 1,
        }).unwrap();
        db.add_transaction(NewTransaction {
            amount: -2000,
            description: Some("Lunch".into()),
            category: Some("Food & Dining".into()),
            container_id: 1,
        }).unwrap();

        let totals = db.get_category_totals(1).unwrap();
        assert_eq!(totals.len(), 1);
        assert_eq!(totals[0].0, "Food & Dining");
        assert_eq!(totals[0].1, -2000);
    }
}
