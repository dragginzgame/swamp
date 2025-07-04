// SQLite database for local ICP ledger data
// Provides fast, indexed queries over millions of transactions

use anyhow::Result;
use rusqlite::{Connection, Transaction, params, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Instant;
use std::collections::HashMap;
use crate::local_ledger::LocalLedgerReader;
use crate::pattern_addresses::get_pattern_address_list;

const BATCH_SIZE: usize = 10000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbTransaction {
    pub id: u64,
    pub operation_type: String,
    pub from_account: Option<String>,
    pub to_account: Option<String>,
    pub amount: Option<u64>,
    pub fee: Option<u64>,
    pub timestamp: Option<u64>,
    pub memo: Option<u64>,
    pub spender: Option<String>,
}

pub struct LedgerDatabase {
    conn: Connection,
}

impl LedgerDatabase {
    /// Create or open a ledger database
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        
        // Enable performance optimizations
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "cache_size", -64000)?; // 64MB cache
        conn.pragma_update(None, "temp_store", "MEMORY")?;
        
        let db = Self { conn };
        db.create_schema()?;
        Ok(db)
    }
    
    /// Create the database schema with indexes
    fn create_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS transactions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                operation_type TEXT NOT NULL,
                from_account TEXT,
                to_account TEXT,
                amount TEXT,
                fee TEXT,
                timestamp TEXT,
                memo TEXT,
                spender TEXT
            );
            
            -- Indexes for fast account lookups
            CREATE INDEX IF NOT EXISTS idx_from_account ON transactions(from_account) WHERE from_account IS NOT NULL;
            CREATE INDEX IF NOT EXISTS idx_to_account ON transactions(to_account) WHERE to_account IS NOT NULL;
            CREATE INDEX IF NOT EXISTS idx_spender ON transactions(spender) WHERE spender IS NOT NULL;
            CREATE INDEX IF NOT EXISTS idx_timestamp ON transactions(timestamp) WHERE timestamp IS NOT NULL;
            CREATE INDEX IF NOT EXISTS idx_operation_type ON transactions(operation_type);
            
            -- Composite indexes for common queries
            CREATE INDEX IF NOT EXISTS idx_from_timestamp ON transactions(from_account, timestamp) WHERE from_account IS NOT NULL;
            CREATE INDEX IF NOT EXISTS idx_to_timestamp ON transactions(to_account, timestamp) WHERE to_account IS NOT NULL;
            
            -- Metadata table for tracking import progress
            CREATE TABLE IF NOT EXISTS import_metadata (
                key TEXT PRIMARY KEY,
                value TEXT
            );
            "
        )?;
        Ok(())
    }
    
    /// Import transactions from JSONL files
    pub fn import_from_jsonl<P: AsRef<Path>>(&mut self, ledger_directory: P) -> Result<()> {
        let reader = LocalLedgerReader::new(ledger_directory)?;
        let start_time = Instant::now();
        
        println!("Starting ledger import...");
        
        // Get last imported transaction ID
        let last_imported_id = self.get_last_imported_id()?;
        println!("Last imported transaction ID: {:?}", last_imported_id);
        
        // Get list of already imported files
        let mut imported_files = std::collections::HashSet::new();
        {
            let mut stmt = self.conn.prepare("SELECT key FROM import_metadata WHERE key LIKE 'file_%'")?;
            let files = stmt.query_map([], |row| row.get::<_, String>(0))?;
            for file in files {
                if let Ok(f) = file {
                    imported_files.insert(f);
                }
            }
        }
        
        let mut tx = self.conn.transaction()?;
        let mut total_imported = 0;
        let mut batch = Vec::new();
        
        // Process each file
        for (file_idx, ledger_file) in reader.ledger_files.iter().enumerate() {
            // Check if this file was already imported
            let file_key = format!("file_{}", ledger_file.path.display());
            
            if imported_files.contains(&file_key) {
                println!("Skipping {}, already imported", ledger_file.path.display());
                continue;
            }
            
            println!("Processing file {}/{}: {}", 
                    file_idx + 1, 
                    reader.ledger_files.len(), 
                    ledger_file.path.display());
            
            println!("  Opening file...");
            let file = std::fs::File::open(&ledger_file.path)?;
            let reader = std::io::BufReader::new(file);
            let mut file_count = 0;
            let mut line_count = 0;
            let mut parse_errors = 0;
            
            println!("  Starting to read lines...");
            
            for line in std::io::BufRead::lines(reader) {
                let line = line?;
                line_count += 1;
                
                if line.trim().is_empty() {
                    continue;
                }
                
                match serde_json::from_str::<serde_json::Value>(&line) {
                    Ok(json) => {
                        if let Some(db_tx) = parse_transaction(&json) {
                            // For now, don't skip - we'll use IGNORE to handle duplicates
                            
                            batch.push(db_tx);
                            
                            if batch.len() >= BATCH_SIZE {
                                insert_batch(&tx, &batch)?;
                                total_imported += batch.len();
                                file_count += batch.len();
                                batch.clear();
                                
                                if total_imported % 100000 == 0 {
                                    println!("  Imported {} transactions...", total_imported);
                                }
                            }
                        } else {
                            parse_errors += 1;
                            if parse_errors <= 5 {
                                println!("  Failed to parse transaction: {}", line);
                            }
                        }
                    }
                    Err(e) => {
                        parse_errors += 1;
                        if parse_errors <= 5 {
                            println!("  JSON parse error: {} - Line: {}", e, line);
                        }
                    }
                }
            }
            
            // Insert remaining batch
            if !batch.is_empty() {
                insert_batch(&tx, &batch)?;
                total_imported += batch.len();
                file_count += batch.len();
                batch.clear();
            }
            
            println!("  File complete: {} transactions from {} lines (parse errors: {})", file_count, line_count, parse_errors);
            
            // Track imported files instead of IDs
            tx.execute(
                "INSERT OR REPLACE INTO import_metadata (key, value) VALUES (?, 'imported')",
                params![format!("file_{}", ledger_file.path.display())]
            )?;
            
            // Commit every 10 files to save progress
            if (file_idx + 1) % 10 == 0 {
                tx.commit()?;
                println!("  Committed progress at file {}", file_idx + 1);
                tx = self.conn.transaction()?;
            }
        }
        
        tx.commit()?;
        
        let duration = start_time.elapsed();
        println!("\nImport complete!");
        println!("  Total transactions: {}", total_imported);
        println!("  Time taken: {:.2}s", duration.as_secs_f64());
        println!("  Rate: {:.0} tx/sec", total_imported as f64 / duration.as_secs_f64());
        
        // Run ANALYZE to update query planner statistics
        self.conn.execute("ANALYZE", [])?;
        
        Ok(())
    }
    
    /// Get the last imported transaction ID
    fn get_last_imported_id(&self) -> Result<Option<u64>> {
        let result: Option<String> = self.conn
            .query_row(
                "SELECT value FROM import_metadata WHERE key = 'last_imported_id'",
                [],
                |row| row.get(0)
            )
            .optional()?;
        
        Ok(result.and_then(|s| s.parse().ok()))
    }
    
    /// Get all transactions for an account
    pub fn get_account_transactions(&self, account: &str) -> Result<Vec<DbTransaction>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM transactions 
             WHERE from_account = ?1 OR to_account = ?1 OR spender = ?1
             ORDER BY id"
        )?;
        
        let transactions = stmt.query_map(params![account], |row| {
            Ok(DbTransaction {
                id: row.get(0)?,
                operation_type: row.get(1)?,
                from_account: row.get(2)?,
                to_account: row.get(3)?,
                amount: row.get(4)?,
                fee: row.get(5)?,
                timestamp: row.get(6)?,
                memo: row.get(7)?,
                spender: row.get(8)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(transactions)
    }
    
    /// Get account balance at a specific timestamp
    pub fn get_balance_at_timestamp(&self, account: &str, timestamp: u64) -> Result<i64> {
        let received: i64 = self.conn.query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM transactions 
             WHERE to_account = ?1 AND timestamp <= ?2",
            params![account, timestamp],
            |row| row.get(0)
        )?;
        
        let sent: i64 = self.conn.query_row(
            "SELECT COALESCE(SUM(amount + COALESCE(fee, 0)), 0) FROM transactions 
             WHERE from_account = ?1 AND timestamp <= ?2",
            params![account, timestamp],
            |row| row.get(0)
        )?;
        
        Ok(received - sent)
    }
    
    /// Find accounts that interacted with a given account
    pub fn find_connected_accounts(&self, account: &str, min_amount: Option<u64>) -> Result<Vec<(String, u64, u64)>> {
        let min_amount = min_amount.unwrap_or(0);
        
        let query = "
            WITH connections AS (
                SELECT 
                    CASE 
                        WHEN from_account = ?1 THEN to_account
                        ELSE from_account
                    END as connected_account,
                    SUM(CASE WHEN to_account = ?1 THEN amount ELSE 0 END) as received,
                    SUM(CASE WHEN from_account = ?1 THEN amount ELSE 0 END) as sent
                FROM transactions
                WHERE (from_account = ?1 OR to_account = ?1) 
                    AND amount >= ?2
                GROUP BY connected_account
            )
            SELECT connected_account, received, sent
            FROM connections
            WHERE connected_account IS NOT NULL
            ORDER BY (received + sent) DESC
        ";
        
        let mut stmt = self.conn.prepare(query)?;
        let results = stmt.query_map(params![account, min_amount], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
        
        Ok(results)
    }
    
    /// Get transaction volume statistics
    pub fn get_account_stats(&self, account: &str) -> Result<serde_json::Value> {
        let tx_count: u64 = self.conn.query_row(
            "SELECT COUNT(*) FROM transactions WHERE from_account = ?1 OR to_account = ?1",
            params![account],
            |row| row.get(0)
        )?;
        
        let total_received: Option<u64> = self.conn.query_row(
            "SELECT SUM(amount) FROM transactions WHERE to_account = ?1",
            params![account],
            |row| row.get(0)
        )?;
        
        let total_sent: Option<u64> = self.conn.query_row(
            "SELECT SUM(amount) FROM transactions WHERE from_account = ?1",
            params![account],
            |row| row.get(0)
        )?;
        
        let first_tx: Option<u64> = self.conn.query_row(
            "SELECT MIN(timestamp) FROM transactions WHERE from_account = ?1 OR to_account = ?1",
            params![account],
            |row| row.get(0)
        )?;
        
        let last_tx: Option<u64> = self.conn.query_row(
            "SELECT MAX(timestamp) FROM transactions WHERE from_account = ?1 OR to_account = ?1",
            params![account],
            |row| row.get(0)
        )?;
        
        Ok(serde_json::json!({
            "account": account,
            "transaction_count": tx_count,
            "total_received_e8s": total_received.unwrap_or(0),
            "total_sent_e8s": total_sent.unwrap_or(0),
            "balance_e8s": total_received.unwrap_or(0) as i64 - total_sent.unwrap_or(0) as i64,
            "first_transaction_timestamp": first_tx,
            "last_transaction_timestamp": last_tx
        }))
    }
    
    /// Database statistics
    pub fn get_db_stats(&self) -> Result<serde_json::Value> {
        let total_txs: u64 = self.conn.query_row(
            "SELECT COUNT(*) FROM transactions",
            [],
            |row| row.get(0)
        )?;
        
        let unique_accounts: u64 = self.conn.query_row(
            "SELECT COUNT(DISTINCT account) FROM (
                SELECT from_account as account FROM transactions WHERE from_account IS NOT NULL
                UNION
                SELECT to_account as account FROM transactions WHERE to_account IS NOT NULL
            )",
            [],
            |row| row.get(0)
        )?;
        
        Ok(serde_json::json!({
            "total_transactions": total_txs,
            "unique_accounts": unique_accounts,
            "database_size_mb": self.get_db_size_mb()?,
        }))
    }
    
    fn get_db_size_mb(&self) -> Result<f64> {
        let page_count: u64 = self.conn.query_row("PRAGMA page_count", [], |row| row.get(0))?;
        let page_size: u64 = self.conn.query_row("PRAGMA page_size", [], |row| row.get(0))?;
        Ok((page_count * page_size) as f64 / 1_048_576.0)
    }
    
    /// Generate daily balance data for all pattern addresses
    pub fn generate_daily_balances(&self) -> Result<serde_json::Value> {
        let pattern_addresses = get_pattern_address_list();
        
        // Get the timestamp range from the database
        let (min_timestamp, max_timestamp): (Option<u64>, Option<u64>) = self.conn.query_row(
            "SELECT MIN(CAST(timestamp AS INTEGER)), MAX(CAST(timestamp AS INTEGER)) FROM transactions WHERE timestamp IS NOT NULL",
            [],
            |row| Ok((row.get(0)?, row.get(1)?))
        )?;
        
        let min_timestamp = min_timestamp.unwrap_or(0);
        let max_timestamp = max_timestamp.unwrap_or(0);
        
        // Convert nanoseconds to days for binning
        let min_day = min_timestamp / (24 * 60 * 60 * 1_000_000_000);
        let max_day = max_timestamp / (24 * 60 * 60 * 1_000_000_000);
        
        println!("Generating daily balances for {} addresses", pattern_addresses.len());
        println!("Date range: {} to {} days", min_day, max_day);
        
        let mut result = serde_json::Map::new();
        
        for (idx, address) in pattern_addresses.iter().enumerate() {
            println!("Processing address {}/{}: {}...", idx + 1, pattern_addresses.len(), &address[..8]);
            
            let daily_balances = self.get_daily_balance_for_address(address, min_day, max_day)?;
            
            // Convert to array of [day, balance] pairs
            let mut balance_data = Vec::new();
            for day in min_day..=max_day {
                let balance = daily_balances.get(&day).unwrap_or(&0);
                balance_data.push(serde_json::json!([day, balance]));
            }
            
            result.insert(address.clone(), serde_json::Value::Array(balance_data));
        }
        
        Ok(serde_json::Value::Object(result))
    }
    
    /// Get daily balance for a specific address
    fn get_daily_balance_for_address(&self, address: &str, min_day: u64, max_day: u64) -> Result<HashMap<u64, i64>> {
        let mut daily_balances = HashMap::new();
        let mut current_balance = 0i64;
        
        // Get all transactions for this address, ordered by timestamp
        let mut stmt = self.conn.prepare(
            "SELECT timestamp, amount, fee, from_account, to_account, operation_type
             FROM transactions 
             WHERE (from_account = ?1 OR to_account = ?1) AND timestamp IS NOT NULL
             ORDER BY CAST(timestamp AS INTEGER)"
        )?;
        
        let rows = stmt.query_map(params![address], |row| {
            let timestamp: String = row.get(0)?;
            let amount: Option<String> = row.get(1)?;
            let fee: Option<String> = row.get(2)?;
            let from_account: Option<String> = row.get(3)?;
            let to_account: Option<String> = row.get(4)?;
            let operation_type: String = row.get(5)?;
            
            Ok((timestamp, amount, fee, from_account, to_account, operation_type))
        })?;
        
        let mut last_day = min_day;
        
        for row in rows {
            let (timestamp_str, amount_str, fee_str, from_account, to_account, operation_type) = row?;
            
            // Parse timestamp
            let timestamp: u64 = timestamp_str.parse().unwrap_or(0);
            let day = timestamp / (24 * 60 * 60 * 1_000_000_000);
            
            // Fill in missing days with current balance
            while last_day < day {
                daily_balances.insert(last_day, current_balance);
                last_day += 1;
            }
            
            // Calculate balance change
            let amount: u64 = amount_str.and_then(|s| s.parse().ok()).unwrap_or(0);
            let fee: u64 = fee_str.and_then(|s| s.parse().ok()).unwrap_or(0);
            
            match operation_type.as_str() {
                "Transfer" => {
                    if to_account.as_deref() == Some(address) {
                        // Receiving funds
                        current_balance += amount as i64;
                    } else if from_account.as_deref() == Some(address) {
                        // Sending funds (subtract amount + fee)
                        current_balance -= (amount + fee) as i64;
                    }
                }
                "Mint" => {
                    if to_account.as_deref() == Some(address) {
                        current_balance += amount as i64;
                    }
                }
                "Burn" => {
                    if from_account.as_deref() == Some(address) {
                        current_balance -= amount as i64;
                    }
                }
                _ => {
                    // Other operations - handle as needed
                }
            }
            
            last_day = day;
        }
        
        // Fill in remaining days with final balance
        while last_day <= max_day {
            daily_balances.insert(last_day, current_balance);
            last_day += 1;
        }
        
        Ok(daily_balances)
    }
}

/// Parse a JSON transaction into DbTransaction
fn parse_transaction(json: &serde_json::Value) -> Option<DbTransaction> {
    // Generate a pseudo-id from timestamp if not present
    let timestamp = json.get("timestamp")
        .and_then(|v| v.get("timestamp_nanos"))
        .and_then(|v| v.as_u64())?;
    
    // Use timestamp as ID since these files don't have explicit IDs
    let id = timestamp / 1_000_000; // Convert nanos to millis for smaller number
    
    let transaction = json.get("transaction")?;
    let operation = transaction.get("operation")?;
    
    let operation_type = operation.get("type")?.as_str()?;
    
    let from_account = operation.get("from")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    let to_account = operation.get("to")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    let amount = operation.get("amount")
        .and_then(|v| v.get("e8s"))
        .and_then(|v| v.as_u64());
    
    let fee = operation.get("fee")
        .and_then(|v| v.get("e8s"))
        .and_then(|v| v.as_u64());
    
    let memo = transaction.get("memo").and_then(|v| v.as_u64());
    
    let spender = operation.get("spender")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    Some(DbTransaction {
        id,
        operation_type: operation_type.to_string(),
        from_account,
        to_account,
        amount,
        fee,
        timestamp: Some(timestamp),
        memo,
        spender,
    })
}

/// Insert a batch of transactions
fn insert_batch(tx: &Transaction, batch: &[DbTransaction]) -> Result<()> {
    let mut stmt = tx.prepare_cached(
        "INSERT INTO transactions 
         (operation_type, from_account, to_account, amount, fee, timestamp, memo, spender)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
    )?;
    
    for transaction in batch {
        stmt.execute(params![
            transaction.operation_type,
            transaction.from_account,
            transaction.to_account,
            transaction.amount.map(|v| v.to_string()),
            transaction.fee.map(|v| v.to_string()),
            transaction.timestamp.map(|v| v.to_string()),
            transaction.memo.map(|v| v.to_string()),
            transaction.spender,
        ])?;
    }
    
    Ok(())
}

/// Run daily balance generation
pub async fn run_daily_balance_generation(db_path: &str) -> Result<()> {
    println!("===== GENERATING DAILY BALANCES =====");
    println!("Database path: {}", db_path);
    
    let db = LedgerDatabase::new(db_path)?;
    
    println!("Generating daily balance data...");
    let daily_balances = db.generate_daily_balances()?;
    
    // Save to JSON file in frontend public directory
    let output_path = "../graph/public/daily_balances.json";
    std::fs::write(output_path, serde_json::to_string_pretty(&daily_balances)?)?;
    
    println!("Daily balance data saved to: {}", output_path);
    
    Ok(())
}