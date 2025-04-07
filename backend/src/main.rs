pub mod addresses;
pub mod helper;
pub mod transactions;

use addresses::{CEXES, DEFI, FOUNDATION, IDENTIFIED, NODE_PROVIDERS, SNSES, SNS_PARTICIPANTS, SPAMMERS, SUSPECTS};
use candid::Principal;
use helper::principal_to_account_id;
use ic_agent::Agent;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt,
};

use thiserror::Error as ThisError;
use tokio::time::{sleep, Duration};
use transactions::{
    fetch_nodes_rewards, fetch_transactions, process_rewards_data, AccountTransactionsJson, ProviderRewardInfo,
};

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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Type {
    Cex,
    Defi,
    Foundation,
    Identified,
    NodeProvider,
    Spammer,
    Sns,
    SnsParticipant,
    Suspect,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Type::Cex => "Cex",
            Type::Defi => "Defi",
            Type::Foundation => "Foundation",
            Type::Identified => "Identified",
            Type::NodeProvider => "NodeProvider",
            Type::Spammer => "Spammer",
            Type::Sns => "Sns",
            Type::SnsParticipant => "SnsParticipant",
            Type::Suspect => "Suspect",
        };
        write!(f, "{}", s)
    }
}

async fn fetch_with_retry(
    account: AccountData,
    rewards_by_principal: HashMap<String, ProviderRewardInfo>,
    agent: &Agent,
    max_retries: usize,
) -> Result<AccountTransactionsJson, Box<dyn std::error::Error>> {
    let mut attempts = 0;
    loop {
        match fetch_transactions(&account, &rewards_by_principal, agent).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempts += 1;
                if attempts >= max_retries {
                    return Err(e);
                }
                println!(
                    "Error fetching account transactions for {}: {}. Retrying {}/{}...",
                    account.name, e, attempts, max_retries
                );
                sleep(Duration::from_secs(10)).await;
            }
        }
    }
}

//
// main
//

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let agent = Agent::builder().with_url(IC_URL).build()?;

    // Initialize the agent (fetch root key in development)
    agent.fetch_root_key().await?;

    let entries = get_entries();

    // Group entries by category (using the account type)
    let mut groups: HashMap<String, Vec<AccountData>> = HashMap::new();
    for entry in entries {
        // Convert account type to string (adjust as necessary)
        let category = entry.ty.to_string().to_lowercase();
        groups.entry(category).or_default().push(entry);
    }

    let rewards = fetch_nodes_rewards(&agent).await?;
    let rewards_by_principal = process_rewards_data(rewards);

    // For each category, fetch transactions and write a JSON file
    for (category, accounts) in groups {
        let mut results = Vec::new();
        for account in accounts {
            match fetch_with_retry(account, rewards_by_principal.clone(), &agent, 3).await {
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

// get_entries
fn get_entries() -> Vec<AccountData> {
    let mut entries = Vec::new();

    // single
    entries.extend(DEFI.iter().map(|(name, addr)| AccountData::new(name, &[addr], Type::Defi)));
    entries.extend(IDENTIFIED.iter().map(|(name, addr)| AccountData::new(name, &[addr], Type::Identified)));
    entries.extend(NODE_PROVIDERS.iter().map(|(name, addr)| AccountData::new(name, &[addr], Type::NodeProvider)));
    entries.extend(SNSES.iter().map(|(name, addr)| AccountData::new(name, &[addr], Type::Sns)));
    entries.extend(SNS_PARTICIPANTS.iter().map(|(name, addr)| AccountData::new(name, &[addr], Type::SnsParticipant)));
    entries.extend(SUSPECTS.iter().map(|(name, addr)| AccountData::new(name, &[addr], Type::Suspect)));
    entries.extend(SPAMMERS.iter().map(|addr| AccountData::new(&addr[..5], &[addr], Type::Spammer)));

    // multiple
    entries.extend(CEXES.iter().map(|(name, addrs)| AccountData::new(name, addrs, Type::Cex)));
    entries.extend(FOUNDATION.iter().map(|(name, addrs)| AccountData::new(name, addrs, Type::Foundation)));

    validate_entries(&entries);

    entries
}

// validate_entries
fn validate_entries(entries: &[AccountData]) {
    // check for dupes
    let mut seen_accounts = HashSet::<String>::new();
    let mut seen_principals = HashSet::<Principal>::new();
    print!("Validating {} addresses...", entries.len());

    for entry in entries {
        for account in entry.accounts.clone() {
            if !seen_accounts.insert(account.clone()) {
                panic!("duplicate account found: {account}");
            }
        }

        for principal in entry.principals.clone() {
            if !seen_principals.insert(principal) {
                panic!("duplicate principal found: {principal}");
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

    println!(" ok");
}
