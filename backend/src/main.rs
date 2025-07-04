pub mod addresses;
pub mod filter_analysis;
pub mod helper;
pub mod ledger_db;
pub mod local_ledger;
pub mod network_tracer;
pub mod pattern_addresses;
pub mod pattern_detector;
pub mod transactions;

use addresses::{CEXES, DEFI, FOUNDATION, IDENTIFIED, NODE_PROVIDERS, SNSES, SPAMMERS, SUSPECTS};
use candid::Principal;
use chrono::{DateTime, Utc};
use derive_more::Display;
use filter_analysis::create_filtered_report;
use helper::principal_to_account_id;
use ic_agent::Agent;
use ledger_db::LedgerDatabase;
use local_ledger::LocalLedgerReader;
use network_tracer::NetworkTracer;
use pattern_addresses::{get_all_pattern_addresses, get_pattern_address_list, CENTRAL_HUB, OTC_DESK};
use pattern_detector::{PatternDetector, Transaction};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::env;
use transactions::fetch_with_retry;

use thiserror::Error as ThisError;

const IC_URL: &str = "https://ic0.app";

///
/// Error
///

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Principal error: {0}")]
    Principal(#[from] ic_agent::export::PrincipalError),
}

///
/// AccountData
///

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountData {
    name: String,
    principals: Vec<Principal>,
    accounts: Vec<String>,
    ty: Type,
}

impl AccountData {
    pub fn new(name: &str, addresses: &[&str], ty: Type) -> Self {
        let mut accounts = Vec::new();
        let mut principals = Vec::new();

        for address in addresses {
            if address.contains("-") {
                principals.push(Principal::from_text(address).unwrap());
            } else {
                accounts.push(address.to_string())
            };
        }

        Self { name: name.to_string(), principals, accounts, ty }
    }
}

///
/// AccountType
///

#[derive(Debug, Serialize, Deserialize, Clone, Display, PartialEq)]
pub enum Type {
    Cex,
    Defi,
    Foundation,
    Identified,
    NodeProvider,
    Spammer,
    Sns,
    Suspect,
}

//
// main
//

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("graph_data");
    
    let agent = Agent::builder().with_url(IC_URL).build()?;

    // Initialize the agent (fetch root key in development)
    agent.fetch_root_key().await?;

    match mode {
        "graph_data" => run_graph_data_mode(&agent).await?,
        "analyze_patterns" => run_pattern_analysis_mode(&agent).await?,
        "analyze_account" => {
            if let Some(account_hex) = args.get(2) {
                run_single_account_analysis(&agent, account_hex).await?;
            } else {
                eprintln!("Usage: cargo run analyze_account <account_hex>");
                std::process::exit(1);
            }
        }
        "trace_network" => run_network_trace(&agent).await?,
        "analyze_seeds" => run_seed_analysis(&agent).await?,
        "trace_funds" => run_funds_trace(&agent).await?,
        "trace_225a2" => run_225a2_complete_trace(&agent).await?,
        "filter_analysis" => {
            create_filtered_report()?;
        }
        "local_ledger" => {
            if let Some(account_hex) = args.get(2) {
                run_local_ledger_analysis(account_hex).await?;
            } else {
                eprintln!("Usage: cargo run local_ledger <account_hex> [ledger_directory]");
                std::process::exit(1);
            }
        }
        "import_db" => {
            let ledger_directory = args.get(2).map(|s| s.as_str()).unwrap_or("./ledger_data");
            let db_path = args.get(3).map(|s| s.as_str()).unwrap_or("./ledger.db");
            run_import_to_db(ledger_directory, db_path).await?;
        }
        "query_db" => {
            if let Some(account_hex) = args.get(2) {
                let db_path = args.get(3).map(|s| s.as_str()).unwrap_or("./ledger.db");
                run_db_query(account_hex, db_path).await?;
            } else {
                eprintln!("Usage: cargo run query_db <account_hex> [db_path]");
                std::process::exit(1);
            }
        }
        "daily_balances" => {
            let db_path = args.get(2).map(|s| s.as_str()).unwrap_or("./ledger.db");
            run_daily_balance_generation(db_path).await?;
        }
        _ => {
            eprintln!("Unknown mode: {}. Use 'graph_data', 'analyze_patterns', 'analyze_account <hex>', 'trace_network', 'analyze_seeds', 'trace_funds', 'trace_225a2', 'filter_analysis', 'local_ledger <account_hex>', 'import_db [ledger_directory] [db_path]', 'query_db <account_hex> [db_path]', or 'daily_balances [db_path]'", mode);
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn run_graph_data_mode(agent: &Agent) -> Result<(), Box<dyn std::error::Error>> {
    let entries = get_entries();

    // Group entries by category (using the account type)
    let mut groups: HashMap<String, Vec<AccountData>> = HashMap::new();
    for entry in entries {
        // Convert account type to string (adjust as necessary)
        let category = entry.ty.to_string().to_lowercase();
        groups.entry(category).or_default().push(entry);
    }

    // For each category, fetch transactions and write a JSON file
    for (category, accounts) in groups {
        let mut results = Vec::new();
        for account in accounts {
            match fetch_with_retry(account, &agent, 3).await {
                Ok(account_tx) => results.push(account_tx),
                Err(e) => eprintln!("Error fetching account transactions for {}: {}", category, e),
            }
        }
        let json_string = serde_json::to_string_pretty(&results)?;
        let file_name = format!("./../graph/public/account_transactions_{}.json", category);
        std::fs::write(&file_name, json_string)?;

        println!("Saved {} accounts transactions to {}", category, file_name);
    }

    Ok(())
}

async fn run_pattern_analysis_mode(agent: &Agent) -> Result<(), Box<dyn std::error::Error>> {
    println!("Analyzing transaction patterns for suspicious activity...");
    
    let detector = PatternDetector::new();
    let mut all_patterns = Vec::new();
    
    // Analyze suspect accounts
    for (name, addresses) in SUSPECTS {
        for address in *addresses {
            println!("Analyzing {} ({})...", name, &address[..8]);
            
            // Fetch transactions for this account
            let account_data = AccountData::new(name, &[address], Type::Suspect);
            match fetch_with_retry(account_data, agent, 3).await {
                Ok(account_tx) => {
                    // Convert to pattern detector format
                    let transactions: Vec<Transaction> = account_tx.transactions.iter().map(|tx| {
                        Transaction {
                            from: tx.from.clone(),
                            to: tx.to.clone(),
                            amount: tx.amount,
                            timestamp: tx.timestamp,
                        }
                    }).collect();
                    
                    // Detect patterns
                    let patterns = detector.detect_patterns(address, &transactions);
                    if !patterns.is_empty() {
                        println!("  Found {} suspicious patterns!", patterns.len());
                        all_patterns.extend(patterns);
                    }
                }
                Err(e) => eprintln!("  Error fetching transactions: {}", e),
            }
        }
    }
    
    // Save results
    let json_string = serde_json::to_string_pretty(&all_patterns)?;
    let file_name = "./../graph/public/suspicious_patterns.json";
    std::fs::write(&file_name, json_string)?;
    
    println!("\nAnalysis complete! Found {} suspicious patterns.", all_patterns.len());
    println!("Results saved to {}", file_name);
    
    Ok(())
}

async fn run_single_account_analysis(agent: &Agent, account_hex: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Analyzing account: {}", account_hex);
    
    let detector = PatternDetector::new();
    
    // Fetch transactions for this account
    let account_data = AccountData::new("Target Account", &[account_hex], Type::Suspect);
    match fetch_with_retry(account_data, agent, 3).await {
        Ok(account_tx) => {
            println!("Found {} transactions", account_tx.transactions.len());
            
            // Convert to pattern detector format
            let transactions: Vec<Transaction> = account_tx.transactions.iter().map(|tx| {
                Transaction {
                    from: tx.from.clone(),
                    to: tx.to.clone(),
                    amount: tx.amount,
                    timestamp: tx.timestamp,
                }
            }).collect();
            
            // Detect patterns
            let patterns = detector.detect_patterns(account_hex, &transactions);
            
            if patterns.is_empty() {
                println!("No suspicious patterns detected.");
            } else {
                println!("\nFound {} suspicious patterns:", patterns.len());
                for pattern in &patterns {
                    println!("\n  Pattern Type: {:?}", pattern.pattern_type);
                    println!("  Total Amount: {} ICP", pattern.total_amount as f64 / 100_000_000.0);
                    println!("  Withdrawals: {}", pattern.withdrawals.len());
                    println!("  Deposits: {}", pattern.deposits.len());
                    
                    for period in &pattern.holding_periods {
                        println!("  Holding Period: {:.1} days", period.duration_days);
                    }
                }
                
                // Save detailed results
                let json_string = serde_json::to_string_pretty(&patterns)?;
                let file_name = format!("./../graph/public/analysis_{}.json", &account_hex[..8]);
                std::fs::write(&file_name, json_string)?;
                println!("\nDetailed results saved to {}", file_name);
            }
        }
        Err(e) => eprintln!("Error fetching transactions: {}", e),
    }
    
    Ok(())
}

async fn run_network_trace(agent: &Agent) -> Result<(), Box<dyn std::error::Error>> {
    println!("Tracing money flow network from seed addresses...");
    
    let tracer = NetworkTracer::new();
    
    // Parameters for network tracing
    let max_depth = 3; // How many hops to follow
    let min_amount_threshold = 100_000_000; // 1 ICP minimum to follow
    
    let network = tracer.trace_network(agent, max_depth, min_amount_threshold).await?;
    
    // Save network data
    let json_string = serde_json::to_string_pretty(&network)?;
    let file_name = "./../graph/public/network_trace.json";
    std::fs::write(&file_name, json_string)?;
    
    // Print summary
    println!("\nNetwork Trace Summary:");
    println!("====================");
    println!("Total accounts discovered: {}", network.nodes.len());
    println!("Total connections: {}", network.edges.len());
    println!("Total balance held: {} ICP", network.total_balance as f64 / 100_000_000.0);
    println!("Suspicious accounts: {}", network.suspicious_accounts.len());
    
    // List top balance holders
    let mut sorted_nodes: Vec<_> = network.nodes.values().collect();
    sorted_nodes.sort_by_key(|n| std::cmp::Reverse(n.balance));
    
    println!("\nTop 10 Balance Holders:");
    for (i, node) in sorted_nodes.iter().take(10).enumerate() {
        println!("{}. {} ({}) - {} ICP", 
            i + 1,
            node.name,
            &node.address[..8],
            node.balance as f64 / 100_000_000.0
        );
        if !node.patterns_detected.is_empty() {
            println!("   Patterns: {:?}", node.patterns_detected);
        }
    }
    
    println!("\nResults saved to {}", file_name);
    
    Ok(())
}

async fn run_seed_analysis(agent: &Agent) -> Result<(), Box<dyn std::error::Error>> {
    println!("Analyzing seed addresses only...");
    
    let tracer = NetworkTracer::new();
    
    // Just analyze the seed addresses without following connections
    let network = tracer.trace_network(agent, 0, 100_000_000).await?;
    
    // Save seed analysis
    let json_string = serde_json::to_string_pretty(&network)?;
    let file_name = "./../graph/public/seed_analysis.json";
    std::fs::write(&file_name, json_string)?;
    
    // Print detailed analysis
    println!("\nSeed Address Analysis:");
    println!("====================");
    
    let mut sorted_nodes: Vec<_> = network.nodes.values().collect();
    sorted_nodes.sort_by_key(|n| std::cmp::Reverse(n.balance));
    
    for (i, node) in sorted_nodes.iter().enumerate() {
        println!("\n{}. {} ({})", i + 1, node.name, &node.address[..8]);
        println!("   Balance: {} ICP", node.balance as f64 / 100_000_000.0);
        println!("   Received: {} ICP", node.total_received as f64 / 100_000_000.0);
        println!("   Sent: {} ICP", node.total_sent as f64 / 100_000_000.0);
        if !node.patterns_detected.is_empty() {
            println!("   Patterns: {:?}", node.patterns_detected);
        }
    }
    
    let total_balance: u64 = network.nodes.values().map(|n| n.balance).sum();
    println!("\nTotal balance across all seeds: {} ICP", total_balance as f64 / 100_000_000.0);
    println!("Results saved to {}", file_name);
    
    Ok(())
}

async fn run_funds_trace(agent: &Agent) -> Result<(), Box<dyn std::error::Error>> {
    println!("Tracing total funds across all pattern addresses...");
    println!("This will calculate the exact total ICP controlled by the suspect.");
    
    let addresses = get_all_pattern_addresses();
    let address_list = get_pattern_address_list();
    let unknown_name = "Unknown".to_string();
    
    println!("Analyzing {} addresses...", address_list.len());
    
    let mut total_balance = 0u64;
    let mut total_received = 0u64;
    let mut total_sent = 0u64;
    let mut account_details = Vec::new();
    
    for (i, address) in address_list.iter().enumerate() {
        let name = addresses.get(address).unwrap_or(&unknown_name);
        println!("{}. Analyzing {} ({})...", i + 1, name, &address[..8]);
        
        let account_data = AccountData::new(name, &[address], Type::Suspect);
        match fetch_with_retry(account_data, agent, 3).await {
            Ok(account_tx) => {
                let mut received = 0u64;
                let mut sent = 0u64;
                
                for tx in &account_tx.transactions {
                    if tx.to == *address {
                        received += tx.amount;
                    } else if tx.from == *address {
                        sent += tx.amount;
                    }
                }
                
                let balance = received.saturating_sub(sent);
                total_balance += balance;
                total_received += received;
                total_sent += sent;
                
                account_details.push((
                    name.clone(),
                    address.clone(),
                    balance,
                    received,
                    sent,
                    account_tx.transactions.len()
                ));
                
                println!("   Balance: {} ICP", balance as f64 / 100_000_000.0);
                println!("   Transactions: {}", account_tx.transactions.len());
            }
            Err(e) => {
                println!("   Error: {}", e);
                account_details.push((
                    name.clone(),
                    address.clone(),
                    0,
                    0,
                    0,
                    0
                ));
            }
        }
    }
    
    // Sort by balance descending
    account_details.sort_by_key(|(_, _, balance, _, _, _)| std::cmp::Reverse(*balance));
    
    // Create summary report
    let summary = serde_json::json!({
        "total_addresses_analyzed": address_list.len(),
        "total_balance_icp": total_balance as f64 / 100_000_000.0,
        "total_received_icp": total_received as f64 / 100_000_000.0,
        "total_sent_icp": total_sent as f64 / 100_000_000.0,
        "accounts": account_details.iter().map(|(name, addr, balance, received, sent, tx_count)| {
            serde_json::json!({
                "name": name,
                "address": addr,
                "balance_icp": *balance as f64 / 100_000_000.0,
                "received_icp": *received as f64 / 100_000_000.0,
                "sent_icp": *sent as f64 / 100_000_000.0,
                "transaction_count": tx_count
            })
        }).collect::<Vec<_>>()
    });
    
    // Save detailed results to backend folder (not public)
    let json_string = serde_json::to_string_pretty(&summary)?;
    let file_name = "./funds_trace_report.json";
    std::fs::write(&file_name, json_string)?;
    
    // Print summary
    println!("\n=== FUNDS TRACE SUMMARY ===");
    println!("Total addresses analyzed: {}", address_list.len());
    println!("Total balance controlled: {} ICP (${:.2}M USD*)", 
        total_balance as f64 / 100_000_000.0,
        (total_balance as f64 / 100_000_000.0) * 10.0 // ~$10 per ICP estimate
    );
    println!("Total ever received: {} ICP", total_received as f64 / 100_000_000.0);
    println!("Total ever sent: {} ICP", total_sent as f64 / 100_000_000.0);
    
    println!("\nTop 10 Holdings:");
    for (i, (name, addr, balance, _, _, _)) in account_details.iter().take(10).enumerate() {
        println!("{}. {} ({}) - {} ICP", 
            i + 1, 
            name, 
            &addr[..8], 
            *balance as f64 / 100_000_000.0
        );
    }
    
    println!("\n* USD estimate based on ~$10/ICP");
    println!("Detailed report saved to: {}", file_name);
    
    Ok(())
}

async fn run_225a2_complete_trace(agent: &Agent) -> Result<(), Box<dyn std::error::Error>> {
    println!("===== COMPREHENSIVE 225a2 NETWORK ANALYSIS =====");
    println!("Central Hub: {}", &CENTRAL_HUB[..8]);
    println!("OTC Desk: {}", &OTC_DESK[..8]);
    println!("Finding ALL connected accounts and calculating total holdings...\n");
    
    let mut discovered_accounts = HashSet::new();
    let mut to_analyze = Vec::new();
    let mut analyzed = HashSet::new();
    let mut all_accounts_data = Vec::new();
    
    // Start with central hub
    to_analyze.push((CENTRAL_HUB.to_string(), "Central Hub 225a2".to_string(), 0));
    discovered_accounts.insert(CENTRAL_HUB.to_string());
    
    // Add OTC desk
    to_analyze.push((OTC_DESK.to_string(), "OTC Desk".to_string(), 0));
    discovered_accounts.insert(OTC_DESK.to_string());
    
    // Add known pattern addresses
    for addr in get_pattern_address_list() {
        if discovered_accounts.insert(addr.clone()) {
            let name = get_all_pattern_addresses().get(&addr).unwrap_or(&"Pattern Account".to_string()).clone();
            to_analyze.push((addr, name, 1));
        }
    }
    
    println!("Phase 1: Discovering connected accounts...");
    let mut iteration = 0;
    
    while !to_analyze.is_empty() && iteration < 3 { // Max 3 levels deep
        iteration += 1;
        println!("\nIteration {}: Analyzing {} accounts", iteration, to_analyze.len());
        
        let current_batch = to_analyze.clone();
        to_analyze.clear();
        
        for (address, name, depth) in current_batch {
            if analyzed.contains(&address) {
                continue;
            }
            analyzed.insert(address.clone());
            
            print!("  Analyzing {} ({})... ", name, &address[..8]);
            
            let account_data = AccountData::new(&name, &[&address], Type::Suspect);
            match fetch_with_retry(account_data, agent, 3).await {
                Ok(account_tx) => {
                    let mut balance_over_time = Vec::new();
                    let mut current_balance = 0i64;
                    let mut received = 0u64;
                    let mut sent = 0u64;
                    let mut connected = HashSet::new();
                    
                    // Sort transactions by timestamp
                    let mut sorted_txs = account_tx.transactions.clone();
                    sorted_txs.sort_by_key(|tx| tx.timestamp);
                    
                    for tx in &sorted_txs {
                        if tx.to == address {
                            current_balance += tx.amount as i64;
                            received += tx.amount;
                            connected.insert(tx.from.clone());
                            
                            // Track balance over time
                            balance_over_time.push((tx.timestamp, current_balance));
                        } else if tx.from == address {
                            current_balance -= tx.amount as i64;
                            sent += tx.amount;
                            connected.insert(tx.to.clone());
                            
                            // Track balance over time
                            balance_over_time.push((tx.timestamp, current_balance));
                        }
                    }
                    
                    let final_balance = current_balance.max(0) as u64;
                    println!("{} ICP, {} connections", final_balance as f64 / 100_000_000.0, connected.len());
                    
                    // Add newly discovered accounts
                    if depth < 2 { // Only go 3 levels deep
                        for conn_addr in &connected {
                            // Skip exchanges
                            let is_exchange = CEXES.iter().any(|(_, addrs)| 
                                addrs.iter().any(|a| a == conn_addr)
                            );
                            
                            if !is_exchange && discovered_accounts.insert(conn_addr.clone()) {
                                to_analyze.push((conn_addr.clone(), format!("Connected {}", &conn_addr[..8]), depth + 1));
                            }
                        }
                    }
                    
                    all_accounts_data.push((
                        name.clone(),
                        address.clone(),
                        final_balance,
                        received,
                        sent,
                        account_tx.transactions.len(),
                        balance_over_time,
                        depth
                    ));
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
    }
    
    println!("\n\nPhase 2: Calculating totals...");
    
    // Sort by balance
    all_accounts_data.sort_by_key(|(_, _, balance, _, _, _, _, _)| std::cmp::Reverse(*balance));
    
    let total_balance: u64 = all_accounts_data.iter().map(|(_, _, b, _, _, _, _, _)| b).sum();
    let total_accounts = all_accounts_data.len();
    
    // Create detailed report
    let detailed_report = serde_json::json!({
        "central_hub": CENTRAL_HUB,
        "otc_desk": OTC_DESK,
        "total_accounts_discovered": total_accounts,
        "total_balance_icp": total_balance as f64 / 100_000_000.0,
        "total_balance_usd": (total_balance as f64 / 100_000_000.0) * 10.0,
        "accounts": all_accounts_data.iter().map(|(name, addr, balance, received, sent, tx_count, balance_history, depth)| {
            serde_json::json!({
                "name": name,
                "address": addr,
                "depth_from_hub": depth,
                "balance_icp": *balance as f64 / 100_000_000.0,
                "received_icp": *received as f64 / 100_000_000.0,
                "sent_icp": *sent as f64 / 100_000_000.0,
                "transaction_count": tx_count,
                "balance_history": balance_history.iter().map(|(ts, bal)| {
                    serde_json::json!({
                        "timestamp": ts,
                        "balance_icp": *bal as f64 / 100_000_000.0
                    })
                }).collect::<Vec<_>>()
            })
        }).collect::<Vec<_>>()
    });
    
    // Save comprehensive report
    let json_string = serde_json::to_string_pretty(&detailed_report)?;
    let file_name = "./225a2_complete_network_analysis.json";
    std::fs::write(&file_name, json_string)?;
    
    // Print summary
    println!("\n===== 225a2 NETWORK SUMMARY =====");
    println!("Total accounts discovered: {}", total_accounts);
    println!("Total ICP controlled: {} ICP", total_balance as f64 / 100_000_000.0);
    println!("Total USD value: ${:.2}M", (total_balance as f64 / 100_000_000.0) * 10.0);
    
    println!("\nTop 20 Balance Holders:");
    for (i, (name, addr, balance, _, _, _, _, depth)) in all_accounts_data.iter().take(20).enumerate() {
        println!("{}. {} ({}) [depth {}] - {} ICP", 
            i + 1, 
            name, 
            &addr[..8],
            depth,
            *balance as f64 / 100_000_000.0
        );
    }
    
    // Show balance distribution
    let over_1m_icp = all_accounts_data.iter().filter(|(_, _, b, _, _, _, _, _)| *b > 100_000_000_000_000).count();
    let over_100k_icp = all_accounts_data.iter().filter(|(_, _, b, _, _, _, _, _)| *b > 10_000_000_000_000).count();
    let over_10k_icp = all_accounts_data.iter().filter(|(_, _, b, _, _, _, _, _)| *b > 1_000_000_000_000).count();
    let over_1k_icp = all_accounts_data.iter().filter(|(_, _, b, _, _, _, _, _)| *b > 100_000_000_000).count();
    
    println!("\nBalance Distribution:");
    println!("  > 1M ICP: {} accounts", over_1m_icp);
    println!("  > 100K ICP: {} accounts", over_100k_icp);
    println!("  > 10K ICP: {} accounts", over_10k_icp);
    println!("  > 1K ICP: {} accounts", over_1k_icp);
    
    println!("\nDetailed report saved to: {}", file_name);
    println!("This report includes balance history over time for each account.");
    
    Ok(())
}

// get_entries
fn get_entries() -> Vec<AccountData> {
    let mut entries = Vec::new();

    // single
    entries.extend(DEFI.iter().map(|(name, addr)| AccountData::new(name, &[addr], Type::Defi)));
    entries.extend(SNSES.iter().map(|(name, addr)| AccountData::new(name, &[addr], Type::Sns)));

    // unnamed
    entries.extend(SPAMMERS.iter().map(|addr| AccountData::new(&addr[..5], &[addr], Type::Spammer)));

    // multiple
    entries.extend(CEXES.iter().map(|(name, addrs)| AccountData::new(name, addrs, Type::Cex)));
    entries.extend(FOUNDATION.iter().map(|(name, addrs)| AccountData::new(name, addrs, Type::Foundation)));
    entries.extend(IDENTIFIED.iter().map(|(name, addrs)| AccountData::new(name, addrs, Type::Identified)));
    entries.extend(NODE_PROVIDERS.iter().map(|(name, addrs)| AccountData::new(name, addrs, Type::NodeProvider)));
    entries.extend(SUSPECTS.iter().map(|(name, addrs)| AccountData::new(name, addrs, Type::Suspect)));

    validate_entries(&entries);

    entries
}

// validate_entries
fn validate_entries(entries: &[AccountData]) {
    // check for dupes
    let mut seen_accounts = HashSet::<String>::new();
    let mut seen_principals = HashSet::<Principal>::new();
    let mut duplicate_accounts = HashSet::<String>::new();
    let mut duplicate_principals = HashSet::<Principal>::new();

    print!("Validating {} addresses...", entries.len());

    // accounts
    for entry in entries {
        for account in &entry.accounts {
            if !seen_accounts.insert(account.clone()) {
                duplicate_accounts.insert(account.clone());
            }
        }

        for principal in &entry.principals {
            if !seen_principals.insert(*principal) {
                duplicate_principals.insert(*principal);
            }
        }
    }

    // principals
    for entry in entries {
        for principal in entry.principals.clone() {
            let account = principal_to_account_id(&principal, None);
            let hex = hex::encode(account);

            if seen_accounts.contains(&hex) {
                panic!("account {hex} already added as principal {principal}");
            }
        }
    }

    // check
    if !duplicate_accounts.is_empty() || !duplicate_principals.is_empty() {
        println!("❌ Duplicates found!");

        if !duplicate_accounts.is_empty() {
            let mut sorted_accounts: Vec<_> = duplicate_accounts.into_iter().collect();
            sorted_accounts.sort();
            println!("Duplicate accounts:");
            for acc in sorted_accounts {
                println!("  - {acc}");
            }
        }

        if !duplicate_principals.is_empty() {
            let mut sorted_principals: Vec<_> = duplicate_principals.into_iter().collect();
            sorted_principals.sort_by_key(|p| p.to_text()); // Sorting by textual representation
            println!("Duplicate principals:");
            for p in sorted_principals {
                println!("  - {}", p);
            }
        }

        println!("Validation failed due to duplicates.");
    } else {
        println!("✅ All entries are valid, no duplicates found.");
    }
}

async fn run_local_ledger_analysis(account_hex: &str) -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let ledger_directory = args.get(3).map(|s| s.as_str()).unwrap_or("./ledger_data");
    
    println!("===== LOCAL LEDGER ANALYSIS =====");
    println!("Account: {}", account_hex);
    println!("Ledger directory: {}", ledger_directory);
    
    // Initialize the local ledger reader
    let ledger_reader = LocalLedgerReader::new(ledger_directory)?;
    
    // Get summary info
    let summary = ledger_reader.get_summary();
    println!("\nLedger Summary:");
    println!("  Files: {}", summary.get("total_files").unwrap_or(&serde_json::Value::Number(0.into())));
    if let Some(first_id) = summary.get("first_transaction_id") {
        println!("  First transaction ID: {}", first_id);
    }
    if let Some(last_id) = summary.get("last_transaction_id") {
        println!("  Last transaction ID: {}", last_id);
    }
    
    println!("\nSearching for transactions involving account {}...", account_hex);
    let start_time = std::time::Instant::now();
    
    // Find all transactions for this account
    let transactions = ledger_reader.find_account_transactions(account_hex)?;
    
    let search_duration = start_time.elapsed();
    println!("Search completed in {:.2} seconds", search_duration.as_secs_f64());
    
    if transactions.is_empty() {
        println!("No transactions found for account {}", account_hex);
        return Ok(());
    }
    
    println!("\n===== ANALYSIS RESULTS =====");
    println!("Total transactions found: {}", transactions.len());
    
    // Calculate balance and statistics
    let mut balance = 0i64;
    let mut total_received = 0u64;
    let mut total_sent = 0u64;
    let mut by_operation_type = HashMap::new();
    
    for tx in &transactions {
        // Count by operation type
        *by_operation_type.entry(tx.operation_type.clone()).or_insert(0) += 1;
        
        if let Some(amount) = tx.amount {
            if tx.to.as_ref() == Some(&account_hex.to_string()) {
                balance += amount as i64;
                total_received += amount;
            } else if tx.from.as_ref() == Some(&account_hex.to_string()) {
                balance -= amount as i64;
                total_sent += amount;
            }
        }
    }
    
    let final_balance = balance.max(0) as u64;
    
    println!("\nBalance Summary:");
    println!("  Current balance: {} ICP", final_balance as f64 / 100_000_000.0);
    println!("  Total received: {} ICP", total_received as f64 / 100_000_000.0);
    println!("  Total sent: {} ICP", total_sent as f64 / 100_000_000.0);
    
    println!("\nTransaction Types:");
    for (op_type, count) in &by_operation_type {
        println!("  {}: {}", op_type, count);
    }
    
    // Show first and last transaction timestamps
    if let Some(first_tx) = transactions.first() {
        if let Some(timestamp) = first_tx.timestamp {
            let seconds = (timestamp / 1_000_000_000) as i64;
            let nanos = (timestamp % 1_000_000_000) as u32;
            if let Some(dt) = DateTime::from_timestamp(seconds, nanos) {
                println!("\nFirst transaction: ID {} at {}", first_tx.id, 
                        dt.format("%Y-%m-%d %H:%M:%S UTC"));
            }
        }
    }
    
    if let Some(last_tx) = transactions.last() {
        if let Some(timestamp) = last_tx.timestamp {
            let seconds = (timestamp / 1_000_000_000) as i64;
            let nanos = (timestamp % 1_000_000_000) as u32;
            if let Some(dt) = DateTime::from_timestamp(seconds, nanos) {
                println!("Last transaction: ID {} at {}", last_tx.id, 
                        dt.format("%Y-%m-%d %H:%M:%S UTC"));
            }
        }
    }
    
    // Save detailed results
    let report = serde_json::json!({
        "account": account_hex,
        "analysis_timestamp": Utc::now().to_rfc3339(),
        "search_duration_seconds": search_duration.as_secs_f64(),
        "total_transactions": transactions.len(),
        "balance_icp": final_balance as f64 / 100_000_000.0,
        "total_received_icp": total_received as f64 / 100_000_000.0,
        "total_sent_icp": total_sent as f64 / 100_000_000.0,
        "operation_types": by_operation_type,
        "transactions": transactions.iter().map(|tx| {
            serde_json::json!({
                "id": tx.id,
                "operation_type": tx.operation_type,
                "from": tx.from,
                "to": tx.to,
                "amount_icp": tx.amount.map(|a| a as f64 / 100_000_000.0),
                "timestamp": tx.timestamp,
                "memo": tx.memo
            })
        }).collect::<Vec<_>>()
    });
    
    let filename = format!("local_ledger_analysis_{}.json", &account_hex[..8]);
    std::fs::write(&filename, serde_json::to_string_pretty(&report)?)?;
    
    println!("\nDetailed analysis saved to: {}", filename);
    
    Ok(())
}

async fn run_import_to_db(ledger_directory: &str, db_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("===== IMPORTING LEDGER TO SQLITE =====");
    println!("Ledger directory: {}", ledger_directory);
    println!("Database path: {}", db_path);
    
    let mut db = LedgerDatabase::new(db_path)?;
    db.import_from_jsonl(ledger_directory)?;
    
    // Print database statistics
    let stats = db.get_db_stats()?;
    println!("\nDatabase Statistics:");
    println!("{}", serde_json::to_string_pretty(&stats)?);
    
    Ok(())
}

async fn run_db_query(account_hex: &str, db_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("===== SQLITE LEDGER QUERY =====");
    println!("Account: {}", account_hex);
    println!("Database: {}", db_path);
    
    let db = LedgerDatabase::new(db_path)?;
    let start_time = std::time::Instant::now();
    
    // Get account statistics
    let stats = db.get_account_stats(account_hex)?;
    let query_time = start_time.elapsed();
    
    println!("\nAccount Statistics:");
    println!("{}", serde_json::to_string_pretty(&stats)?);
    println!("\nQuery completed in {:.3} ms", query_time.as_millis());
    
    // Get connected accounts
    let connected = db.find_connected_accounts(account_hex, Some(100_000_000))?; // 1 ICP minimum
    println!("\nTop Connected Accounts (>1 ICP):");
    for (i, (account, received, sent)) in connected.iter().take(20).enumerate() {
        println!("{}. {} - Received: {} ICP, Sent: {} ICP", 
                 i + 1, 
                 &account[..8],
                 *received as f64 / 100_000_000.0,
                 *sent as f64 / 100_000_000.0);
    }
    
    Ok(())
}

async fn run_daily_balance_generation(db_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    use crate::ledger_db::run_daily_balance_generation;
    Ok(run_daily_balance_generation(db_path).await?)
}
