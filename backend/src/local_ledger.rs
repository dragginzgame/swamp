// Local ledger file processing for JSONL transaction files
// Handles streaming reads of large transaction datasets without loading into memory

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, read_dir};
use std::io::{BufRead, BufReader, Result as IoResult};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalTransaction {
    pub id: u64,
    pub from: Option<String>,
    pub to: Option<String>,
    pub amount: Option<u64>,
    pub timestamp: Option<u64>,
    pub memo: Option<u64>,
    pub operation_type: String,
}

#[derive(Debug, Clone)]
pub struct LedgerFile {
    pub path: PathBuf,
    pub start_id: u64,
    pub end_id: u64,
}

pub struct LocalLedgerReader {
    pub ledger_files: Vec<LedgerFile>,
    ledger_directory: PathBuf,
}

impl LocalLedgerReader {
    pub fn new<P: AsRef<Path>>(ledger_directory: P) -> IoResult<Self> {
        let ledger_directory = ledger_directory.as_ref().to_path_buf();
        let ledger_files = Self::discover_ledger_files(&ledger_directory)?;
        
        println!("Discovered {} ledger files", ledger_files.len());
        if !ledger_files.is_empty() {
            let first = &ledger_files[0];
            let last = &ledger_files[ledger_files.len() - 1];
            println!("Transaction range: {} to {}", first.start_id, last.end_id);
        }
        
        Ok(Self {
            ledger_files,
            ledger_directory,
        })
    }
    
    /// Discover all ledger files in the directory and parse their ranges
    fn discover_ledger_files(directory: &Path) -> IoResult<Vec<LedgerFile>> {
        let mut files = Vec::new();
        
        for entry in read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();
            
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.starts_with("icp_ledger_") && filename.ends_with(".jsonl") {
                    if let Some((start_id, end_id)) = Self::parse_filename_range(filename) {
                        files.push(LedgerFile {
                            path: path.clone(),
                            start_id,
                            end_id,
                        });
                    }
                }
            }
        }
        
        // Sort by start_id for efficient range queries
        files.sort_by_key(|f| f.start_id);
        Ok(files)
    }
    
    /// Parse filename to extract transaction ID range
    /// Examples: "icp_ledger_0_100000.jsonl" -> (0, 100000)
    ///          "icp_ledger_1099000_1199000.jsonl" -> (1099000, 1199000)
    fn parse_filename_range(filename: &str) -> Option<(u64, u64)> {
        let without_prefix = filename.strip_prefix("icp_ledger_")?;
        let without_suffix = without_prefix.strip_suffix(".jsonl")?;
        
        let parts: Vec<&str> = without_suffix.split('_').collect();
        if parts.len() == 2 {
            let start_id = parts[0].parse::<u64>().ok()?;
            let end_id = parts[1].parse::<u64>().ok()?;
            Some((start_id, end_id))
        } else {
            None
        }
    }
    
    /// Find all transactions for a specific account across all ledger files
    pub fn find_account_transactions(&self, account_id: &str) -> IoResult<Vec<LocalTransaction>> {
        let mut transactions = Vec::new();
        
        for ledger_file in &self.ledger_files {
            let file_transactions = self.search_file_for_account(&ledger_file.path, account_id)?;
            transactions.extend(file_transactions);
        }
        
        // Sort by transaction ID
        transactions.sort_by_key(|t| t.id);
        Ok(transactions)
    }
    
    /// Search a specific file for transactions involving an account
    fn search_file_for_account(&self, file_path: &Path, account_id: &str) -> IoResult<Vec<LocalTransaction>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let mut transactions = Vec::new();
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line?;
            
            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }
            
            match serde_json::from_str::<serde_json::Value>(&line) {
                Ok(json) => {
                    // Check if this transaction involves our account
                    if self.transaction_involves_account(&json, account_id) {
                        if let Some(transaction) = self.parse_transaction(&json) {
                            transactions.push(transaction);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error parsing JSON at {}:{}: {}", 
                             file_path.display(), line_num + 1, e);
                }
            }
        }
        
        Ok(transactions)
    }
    
    /// Check if a transaction JSON involves a specific account
    fn transaction_involves_account(&self, json: &serde_json::Value, account_id: &str) -> bool {
        // Check various possible locations for account IDs in the transaction
        if let Some(operation) = json.get("operation") {
            // Check 'from' field
            if let Some(from) = operation.get("from").and_then(|v| v.as_str()) {
                if from == account_id {
                    return true;
                }
            }
            
            // Check 'to' field
            if let Some(to) = operation.get("to").and_then(|v| v.as_str()) {
                if to == account_id {
                    return true;
                }
            }
            
            // Check 'spender' field (for approve operations)
            if let Some(spender) = operation.get("spender").and_then(|v| v.as_str()) {
                if spender == account_id {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// Parse a JSON transaction into our LocalTransaction struct
    fn parse_transaction(&self, json: &serde_json::Value) -> Option<LocalTransaction> {
        let id = json.get("id")?.as_u64()?;
        let operation = json.get("operation")?;
        
        let operation_type = if operation.get("Transfer").is_some() {
            "Transfer".to_string()
        } else if operation.get("Mint").is_some() {
            "Mint".to_string()
        } else if operation.get("Burn").is_some() {
            "Burn".to_string()
        } else if operation.get("Approve").is_some() {
            "Approve".to_string()
        } else {
            "Unknown".to_string()
        };
        
        let from = operation.get("from").and_then(|v| v.as_str()).map(|s| s.to_string());
        let to = operation.get("to").and_then(|v| v.as_str()).map(|s| s.to_string());
        let amount = operation.get("amount")
            .and_then(|v| v.get("e8s"))
            .and_then(|v| v.as_u64());
        
        let timestamp = json.get("timestamp")
            .and_then(|v| v.get("timestamp_nanos"))
            .and_then(|v| v.as_u64());
        
        let memo = json.get("memo").and_then(|v| v.as_u64());
        
        Some(LocalTransaction {
            id,
            from,
            to,
            amount,
            timestamp,
            memo,
            operation_type,
        })
    }
    
    /// Process transactions in batches to avoid memory issues
    pub fn process_account_in_batches<F>(&self, account_id: &str, batch_size: usize, mut processor: F) -> IoResult<()>
    where
        F: FnMut(&[LocalTransaction]) -> IoResult<()>,
    {
        let mut batch = Vec::with_capacity(batch_size);
        
        for ledger_file in &self.ledger_files {
            let file = File::open(&ledger_file.path)?;
            let reader = BufReader::new(file);
            
            for line in reader.lines() {
                let line = line?;
                if line.trim().is_empty() {
                    continue;
                }
                
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
                    if self.transaction_involves_account(&json, account_id) {
                        if let Some(transaction) = self.parse_transaction(&json) {
                            batch.push(transaction);
                            
                            if batch.len() >= batch_size {
                                processor(&batch)?;
                                batch.clear();
                            }
                        }
                    }
                }
            }
        }
        
        // Process remaining transactions
        if !batch.is_empty() {
            processor(&batch)?;
        }
        
        Ok(())
    }
    
    /// Get summary statistics about the ledger files
    pub fn get_summary(&self) -> HashMap<String, serde_json::Value> {
        let mut summary = HashMap::new();
        
        summary.insert("total_files".to_string(), 
                       serde_json::Value::Number(self.ledger_files.len().into()));
        
        if !self.ledger_files.is_empty() {
            summary.insert("first_transaction_id".to_string(), 
                          serde_json::Value::Number(self.ledger_files[0].start_id.into()));
            summary.insert("last_transaction_id".to_string(), 
                          serde_json::Value::Number(self.ledger_files.last().unwrap().end_id.into()));
        }
        
        let file_list: Vec<String> = self.ledger_files.iter()
            .map(|f| f.path.file_name().unwrap().to_str().unwrap().to_string())
            .collect();
        summary.insert("files".to_string(), 
                      serde_json::Value::Array(file_list.into_iter().map(serde_json::Value::String).collect()));
        
        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_filename_range() {
        assert_eq!(
            LocalLedgerReader::parse_filename_range("icp_ledger_0_100000.jsonl"),
            Some((0, 100000))
        );
        assert_eq!(
            LocalLedgerReader::parse_filename_range("icp_ledger_1099000_1199000.jsonl"),
            Some((1099000, 1199000))
        );
        assert_eq!(
            LocalLedgerReader::parse_filename_range("invalid_file.jsonl"),
            None
        );
    }
}