pub mod addresses;
pub mod helper;
pub mod network_tracer;
pub mod pattern_detector;
pub mod transactions;

use addresses::{CEXES, DEFI, FOUNDATION, IDENTIFIED, NODE_PROVIDERS, SNSES, SPAMMERS, SUSPECTS};
use candid::Principal;
use derive_more::Display;
use helper::principal_to_account_id;
use ic_agent::Agent;
use network_tracer::NetworkTracer;
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
        _ => {
            eprintln!("Unknown mode: {}. Use 'graph_data', 'analyze_patterns', 'analyze_account <hex>', or 'trace_network'", mode);
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
