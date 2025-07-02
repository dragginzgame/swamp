use crate::addresses::CEXES;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

const SIX_WEEKS_NANOS: u64 = 6 * 7 * 24 * 60 * 60 * 1_000_000_000; // 6 weeks in nanoseconds
const TOLERANCE_NANOS: u64 = 7 * 24 * 60 * 60 * 1_000_000_000; // 1 week tolerance

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuspiciousPattern {
    pub account: String,
    pub pattern_type: PatternType,
    pub withdrawals: Vec<ExchangeTransfer>,
    pub deposits: Vec<ExchangeTransfer>,
    pub total_amount: u64,
    pub holding_periods: Vec<HoldingPeriod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    ExchangeCycle, // Withdraw from exchange -> Hold -> Deposit to exchange
    LargeHolding,  // Large amounts held for specific periods
    MixerPattern,  // Multiple small transactions to obfuscate origin
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeTransfer {
    pub exchange_name: String,
    pub exchange_account: String,
    pub amount: u64,
    pub timestamp: u64,
    pub is_withdrawal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoldingPeriod {
    pub start_timestamp: u64,
    pub end_timestamp: u64,
    pub duration_days: f64,
    pub amount_held: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub timestamp: u64,
}

pub struct PatternDetector {
    exchange_addresses: HashMap<String, String>, // address -> exchange name
}

impl PatternDetector {
    pub fn new() -> Self {
        let mut exchange_addresses = HashMap::new();
        
        // Build lookup map for exchange addresses
        for (exchange_name, addresses) in CEXES {
            for address in *addresses {
                exchange_addresses.insert(address.to_string(), exchange_name.to_string());
            }
        }
        
        Self { exchange_addresses }
    }
    
    pub fn detect_patterns(&self, account: &str, transactions: &[Transaction]) -> Vec<SuspiciousPattern> {
        let mut patterns = Vec::new();
        
        // Detect exchange cycle pattern
        if let Some(pattern) = self.detect_exchange_cycle(account, transactions) {
            patterns.push(pattern);
        }
        
        // Add more pattern detection methods here
        
        patterns
    }
    
    fn detect_exchange_cycle(&self, account: &str, transactions: &[Transaction]) -> Option<SuspiciousPattern> {
        let mut withdrawals = Vec::new();
        let mut deposits = Vec::new();
        
        // Categorize transactions
        for tx in transactions {
            // Check if it's a withdrawal from exchange to the account
            if tx.to == account && self.exchange_addresses.contains_key(&tx.from) {
                withdrawals.push(ExchangeTransfer {
                    exchange_name: self.exchange_addresses[&tx.from].clone(),
                    exchange_account: tx.from.clone(),
                    amount: tx.amount,
                    timestamp: tx.timestamp,
                    is_withdrawal: true,
                });
            }
            
            // Check if it's a deposit from account to exchange
            if tx.from == account && self.exchange_addresses.contains_key(&tx.to) {
                deposits.push(ExchangeTransfer {
                    exchange_name: self.exchange_addresses[&tx.to].clone(),
                    exchange_account: tx.to.clone(),
                    amount: tx.amount,
                    timestamp: tx.timestamp,
                    is_withdrawal: false,
                });
            }
        }
        
        // Sort by timestamp
        withdrawals.sort_by_key(|w| w.timestamp);
        deposits.sort_by_key(|d| d.timestamp);
        
        // Find matching withdrawal-deposit pairs with ~6 week holding period
        let mut holding_periods = Vec::new();
        let mut matched_deposits = HashSet::new();
        
        for withdrawal in &withdrawals {
            for (idx, deposit) in deposits.iter().enumerate() {
                if matched_deposits.contains(&idx) {
                    continue;
                }
                
                let time_diff = deposit.timestamp.saturating_sub(withdrawal.timestamp);
                
                // Check if holding period is around 6 weeks (with tolerance)
                if time_diff >= (SIX_WEEKS_NANOS - TOLERANCE_NANOS) 
                    && time_diff <= (SIX_WEEKS_NANOS + TOLERANCE_NANOS) {
                    
                    holding_periods.push(HoldingPeriod {
                        start_timestamp: withdrawal.timestamp,
                        end_timestamp: deposit.timestamp,
                        duration_days: time_diff as f64 / (24.0 * 60.0 * 60.0 * 1_000_000_000.0),
                        amount_held: withdrawal.amount.min(deposit.amount),
                    });
                    
                    matched_deposits.insert(idx);
                    break;
                }
            }
        }
        
        // Only consider it suspicious if we found the pattern
        if !holding_periods.is_empty() {
            let total_amount: u64 = holding_periods.iter().map(|hp| hp.amount_held).sum();
            
            Some(SuspiciousPattern {
                account: account.to_string(),
                pattern_type: PatternType::ExchangeCycle,
                withdrawals,
                deposits,
                total_amount,
                holding_periods,
            })
        } else {
            None
        }
    }
    
    pub fn is_large_amount(&self, amount: u64) -> bool {
        // Consider amounts over 10,000 ICP as large (1 ICP = 100_000_000 e8s)
        amount > 10_000 * 100_000_000
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pattern_detection() {
        let detector = PatternDetector::new();
        
        // Create test transactions
        let transactions = vec![
            Transaction {
                from: "449ce7ad1298e2ed2781ed379aba25efc2748d14c60ede190ad7621724b9e8b2".to_string(), // Coinbase
                to: "test_account".to_string(),
                amount: 1_000_000_000_000, // 10,000 ICP
                timestamp: 0,
            },
            Transaction {
                from: "test_account".to_string(),
                to: "609d3e1e45103a82adc97d4f88c51f78dedb25701e8e51e8c4fec53448aadc29".to_string(), // Binance
                amount: 1_000_000_000_000,
                timestamp: SIX_WEEKS_NANOS,
            },
        ];
        
        let patterns = detector.detect_patterns("test_account", &transactions);
        assert_eq!(patterns.len(), 1);
        assert!(matches!(patterns[0].pattern_type, PatternType::ExchangeCycle));
    }
}