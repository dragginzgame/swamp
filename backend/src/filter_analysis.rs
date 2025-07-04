use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize, Serialize)]
pub struct BalanceEntry {
    pub balance_icp: f64,
    pub timestamp: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Account {
    pub address: String,
    pub balance_history: Vec<BalanceEntry>,
    pub balance_icp: f64,
    pub depth_from_hub: u32,
    pub name: String,
    pub received_icp: f64,
    pub sent_icp: f64,
    pub transaction_count: u32,
}

#[derive(Debug, Deserialize)]
pub struct NetworkAnalysis {
    pub accounts: Vec<Account>,
}

#[derive(Debug, Serialize)]
pub struct FilteredAccount {
    pub address: String,
    pub name: String,
    pub balance_icp: f64,
    pub transaction_count: u32,
    pub suspicious: bool,
}

#[derive(Debug, Serialize)]
pub struct FilteredReport {
    pub filtered_accounts: Vec<FilteredAccount>,
    pub summary: FilterSummary,
}

#[derive(Debug, Serialize)]
pub struct FilterSummary {
    pub total_accounts_analyzed: usize,
    pub accounts_with_10k_plus: usize,
    pub suspicious_accounts: usize,
    pub total_icp_in_filtered_accounts: f64,
    pub filter_criteria: FilterCriteria,
}

#[derive(Debug, Serialize)]
pub struct FilterCriteria {
    pub minimum_balance_icp: f64,
    pub suspicious_transaction_threshold: u32,
}

pub fn create_filtered_report() -> Result<(), Box<dyn std::error::Error>> {
    println!("Reading network analysis file...");
    
    let json_content = fs::read_to_string("225a2_complete_network_analysis.json")?;
    
    println!("Parsing JSON...");
    let network_analysis: NetworkAnalysis = serde_json::from_str(&json_content)?;
    
    println!("Processing {} accounts...", network_analysis.accounts.len());
    
    const MIN_BALANCE: f64 = 10000.0;
    const SUSPICIOUS_TX_THRESHOLD: u32 = 15;
    
    // Filter accounts with balance >= 10,000 ICP
    let mut filtered_accounts: Vec<FilteredAccount> = network_analysis.accounts
        .iter()
        .filter(|account| account.balance_icp >= MIN_BALANCE)
        .map(|account| FilteredAccount {
            address: account.address.clone(),
            name: account.name.clone(),
            balance_icp: account.balance_icp,
            transaction_count: account.transaction_count,
            suspicious: account.transaction_count < SUSPICIOUS_TX_THRESHOLD,
        })
        .collect();
    
    // Sort by balance descending
    filtered_accounts.sort_by(|a, b| b.balance_icp.partial_cmp(&a.balance_icp).unwrap());
    
    // Calculate summary statistics
    let total_accounts = network_analysis.accounts.len();
    let accounts_with_10k_plus = filtered_accounts.len();
    let suspicious_accounts = filtered_accounts.iter().filter(|a| a.suspicious).count();
    let total_icp_in_filtered: f64 = filtered_accounts.iter().map(|a| a.balance_icp).sum();
    
    let report = FilteredReport {
        filtered_accounts,
        summary: FilterSummary {
            total_accounts_analyzed: total_accounts,
            accounts_with_10k_plus,
            suspicious_accounts,
            total_icp_in_filtered_accounts: total_icp_in_filtered,
            filter_criteria: FilterCriteria {
                minimum_balance_icp: MIN_BALANCE,
                suspicious_transaction_threshold: SUSPICIOUS_TX_THRESHOLD,
            },
        },
    };
    
    println!("Writing filtered report...");
    let output_json = serde_json::to_string_pretty(&report)?;
    fs::write("filtered_high_balance_report.json", output_json)?;
    
    println!("Report created successfully!");
    println!("Summary:");
    println!("  Total accounts analyzed: {}", report.summary.total_accounts_analyzed);
    println!("  Accounts with 10k+ ICP: {}", report.summary.accounts_with_10k_plus);
    println!("  Suspicious accounts (< 15 tx): {}", report.summary.suspicious_accounts);
    println!("  Total ICP in filtered accounts: {:.2}", report.summary.total_icp_in_filtered_accounts);
    
    Ok(())
}