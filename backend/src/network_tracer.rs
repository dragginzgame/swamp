use crate::{
    AccountData, Type,
    addresses::{CEXES, SUSPECTS},
    pattern_detector::{PatternDetector, Transaction},
    transactions::{fetch_with_retry, AccountTransactionsJson},
};
use ic_agent::Agent;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkNode {
    pub address: String,
    pub name: String,
    pub balance: u64,
    pub total_received: u64,
    pub total_sent: u64,
    pub is_exchange: bool,
    pub is_seed: bool,
    pub depth: u32,
    pub patterns_detected: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEdge {
    pub from: String,
    pub to: String,
    pub total_amount: u64,
    pub transaction_count: usize,
    pub first_timestamp: u64,
    pub last_timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkAnalysis {
    pub nodes: HashMap<String, NetworkNode>,
    pub edges: Vec<NetworkEdge>,
    pub total_balance: u64,
    pub suspicious_accounts: Vec<String>,
}

pub struct NetworkTracer {
    exchange_addresses: HashSet<String>,
    seed_addresses: HashSet<String>,
    pattern_detector: PatternDetector,
}

impl NetworkTracer {
    pub fn new() -> Self {
        let mut exchange_addresses = HashSet::new();
        for (_, addresses) in CEXES {
            for addr in *addresses {
                exchange_addresses.insert(addr.to_string());
            }
        }
        
        let mut seed_addresses = HashSet::new();
        // Add pattern seed addresses
        for (name, addresses) in SUSPECTS {
            if name.starts_with("Pattern Seed") || *name == "David the Gnome" || *name == "David Fisher WTN" {
                for addr in *addresses {
                    seed_addresses.insert(addr.to_string());
                }
            }
        }
        
        Self {
            exchange_addresses,
            seed_addresses,
            pattern_detector: PatternDetector::new(),
        }
    }
    
    pub async fn trace_network(
        &self,
        agent: &Agent,
        max_depth: u32,
        min_amount_threshold: u64,
    ) -> Result<NetworkAnalysis, Box<dyn std::error::Error>> {
        let mut nodes = HashMap::new();
        let mut edges = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        
        // Initialize queue with seed addresses
        for seed in &self.seed_addresses {
            queue.push_back((seed.clone(), 0u32));
            visited.insert(seed.clone());
        }
        
        println!("Starting network trace from {} seed addresses...", self.seed_addresses.len());
        
        while let Some((current_address, depth)) = queue.pop_front() {
            if depth > max_depth {
                continue;
            }
            
            println!("Analyzing {} at depth {}...", &current_address[..8], depth);
            
            // Fetch transactions for current address
            let account_data = AccountData::new(
                &format!("Network {}", &current_address[..8]),
                &[&current_address],
                Type::Suspect
            );
            
            match fetch_with_retry(account_data, agent, 3).await {
                Ok(account_tx) => {
                    let (node, new_addresses) = self.analyze_account(
                        &current_address,
                        &account_tx,
                        depth,
                        min_amount_threshold,
                        &mut edges,
                    );
                    
                    nodes.insert(current_address.clone(), node);
                    
                    // Add new addresses to queue if not visited and not exchanges
                    for addr in new_addresses {
                        if !visited.contains(&addr) && !self.exchange_addresses.contains(&addr) {
                            visited.insert(addr.clone());
                            queue.push_back((addr, depth + 1));
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error fetching transactions for {}: {}", current_address, e);
                }
            }
        }
        
        // Calculate total balance and identify suspicious accounts
        let total_balance: u64 = nodes.values().map(|n| n.balance).sum();
        let suspicious_accounts: Vec<String> = nodes
            .iter()
            .filter(|(_, node)| !node.patterns_detected.is_empty())
            .map(|(addr, _)| addr.clone())
            .collect();
        
        println!("\nNetwork trace complete:");
        println!("  Total nodes: {}", nodes.len());
        println!("  Total edges: {}", edges.len());
        println!("  Total balance: {} ICP", total_balance as f64 / 100_000_000.0);
        println!("  Suspicious accounts: {}", suspicious_accounts.len());
        
        Ok(NetworkAnalysis {
            nodes,
            edges,
            total_balance,
            suspicious_accounts,
        })
    }
    
    fn analyze_account(
        &self,
        address: &str,
        account_tx: &AccountTransactionsJson,
        depth: u32,
        min_amount_threshold: u64,
        edges: &mut Vec<NetworkEdge>,
    ) -> (NetworkNode, Vec<String>) {
        let mut total_received = 0u64;
        let mut total_sent = 0u64;
        let mut connected_addresses = Vec::new();
        let mut edge_map: HashMap<(String, String), NetworkEdge> = HashMap::new();
        
        // Process transactions
        for tx in &account_tx.transactions {
            if tx.amount < min_amount_threshold {
                continue;
            }
            
            if tx.to == address {
                total_received += tx.amount;
                if !self.exchange_addresses.contains(&tx.from) {
                    connected_addresses.push(tx.from.clone());
                }
            } else if tx.from == address {
                total_sent += tx.amount;
                if !self.exchange_addresses.contains(&tx.to) {
                    connected_addresses.push(tx.to.clone());
                }
            }
            
            // Build edges
            let edge_key = (tx.from.clone(), tx.to.clone());
            edge_map.entry(edge_key.clone())
                .and_modify(|e| {
                    e.total_amount += tx.amount;
                    e.transaction_count += 1;
                    e.first_timestamp = e.first_timestamp.min(tx.timestamp);
                    e.last_timestamp = e.last_timestamp.max(tx.timestamp);
                })
                .or_insert(NetworkEdge {
                    from: tx.from.clone(),
                    to: tx.to.clone(),
                    total_amount: tx.amount,
                    transaction_count: 1,
                    first_timestamp: tx.timestamp,
                    last_timestamp: tx.timestamp,
                });
        }
        
        // Add edges to the collection
        edges.extend(edge_map.into_values());
        
        // Detect patterns
        let transactions: Vec<Transaction> = account_tx.transactions.iter().map(|tx| {
            Transaction {
                from: tx.from.clone(),
                to: tx.to.clone(),
                amount: tx.amount,
                timestamp: tx.timestamp,
            }
        }).collect();
        
        let patterns = self.pattern_detector.detect_patterns(address, &transactions);
        let pattern_names: Vec<String> = patterns.iter()
            .map(|p| format!("{:?}", p.pattern_type))
            .collect();
        
        // Calculate current balance
        let balance = total_received.saturating_sub(total_sent);
        
        let node = NetworkNode {
            address: address.to_string(),
            name: account_tx.name.clone(),
            balance,
            total_received,
            total_sent,
            is_exchange: self.exchange_addresses.contains(address),
            is_seed: self.seed_addresses.contains(address),
            depth,
            patterns_detected: pattern_names,
        };
        
        (node, connected_addresses)
    }
    
    pub async fn get_account_balance(
        &self,
        agent: &Agent,
        address: &str,
    ) -> Result<u64, Box<dyn std::error::Error>> {
        // For now, we calculate balance from transaction history
        // In the future, we could query the ledger directly for current balance
        let account_data = AccountData::new("Balance Check", &[address], Type::Suspect);
        let account_tx = fetch_with_retry(account_data, agent, 3).await?;
        
        let mut balance = 0u64;
        for tx in &account_tx.transactions {
            if tx.to == address {
                balance += tx.amount;
            } else if tx.from == address {
                balance = balance.saturating_sub(tx.amount);
            }
        }
        
        Ok(balance)
    }
}