use std::collections::HashSet;
use tokio::time::{sleep, Duration};

use crate::{
    helper::{is_valid_account_id, principal_to_account_id},
    AccountData,
};
use candid::{CandidType, Decode, Encode};
use ic_agent::{export::Principal, Agent};
use icp_ledger::AccountIdentifier;
use serde::{Deserialize, Serialize};

const INDEX_CANISTER_ID: &str = "qhbym-qaaaa-aaaaa-aaafq-cai";
const GOVERNANCE_CANISTER_ID: &str = "rrkah-fqaaa-aaaaa-aaaaq-cai";

#[derive(CandidType, Deserialize)]
pub struct Account {
    pub owner: Principal,
    pub subaccount: Option<serde_bytes::ByteBuf>,
}

#[derive(CandidType, Deserialize)]
pub struct GetAccountTransactionsArgs {
    pub max_results: u64,
    pub start: Option<u64>,
    pub account_identifier: String,
}

#[derive(CandidType, Deserialize, Serialize, Debug)]
pub struct Tokens {
    pub e8s: u64,
}

#[derive(CandidType, Deserialize, Serialize, Debug)]
pub struct TimeStamp {
    pub timestamp_nanos: u64,
}

#[derive(CandidType, Deserialize, Serialize, Debug)]
pub enum Operation {
    Approve {
        fee: Tokens,
        from: String,
        allowance: Tokens,
        expected_allowance: Option<Tokens>,
        expires_at: Option<TimeStamp>,
        spender: String,
    },
    Burn {
        from: String,
        amount: Tokens,
        spender: Option<String>,
    },
    Mint {
        to: String,
        amount: Tokens,
    },
    Transfer {
        to: String,
        fee: Tokens,
        from: String,
        amount: Tokens,
        spender: Option<String>,
    },
}

#[derive(CandidType, Deserialize, Serialize, Debug)]
pub struct Transaction {
    pub memo: u64,
    pub icrc1_memo: Option<serde_bytes::ByteBuf>,
    pub operation: Operation,
    pub timestamp: Option<TimeStamp>,
    pub created_at_time: Option<TimeStamp>,
}

#[derive(CandidType, Deserialize, Serialize, Debug)]
pub struct TransactionWithId {
    pub id: u64,
    pub transaction: Transaction,
}

#[derive(CandidType, Deserialize)]
pub struct GetAccountIdentifierTransactionsResponse {
    pub balance: u64,
    pub transactions: Vec<TransactionWithId>,
    pub oldest_tx_id: Option<u64>,
}

#[derive(CandidType, Deserialize)]
pub struct GetAccountIdentifierTransactionsError {
    pub message: String,
}

#[derive(CandidType, Deserialize)]
pub enum GetAccountIdentifierTransactionsResult {
    Ok(GetAccountIdentifierTransactionsResponse),
    Err(GetAccountIdentifierTransactionsError),
}

#[derive(Debug, Serialize, Deserialize, CandidType, Clone)]
struct GovAccountIdentifierentifier {
    hash: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, CandidType)]
struct NodeProviderReward {
    id: Option<Principal>,
    reward_account: Option<GovAccountIdentifierentifier>,
}

#[derive(Debug, Serialize, Deserialize, CandidType)]
struct RewardToNeuron {
    dissolve_delay_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize, CandidType)]
struct RewardToAccount {
    to_account: Option<GovAccountIdentifierentifier>,
}

#[derive(Debug, Serialize, Deserialize, CandidType)]
enum RewardMode {
    RewardToNeuron(RewardToNeuron),
    RewardToAccount(RewardToAccount),
}

#[derive(Debug, Serialize, Deserialize, CandidType)]
struct RewardNodeProvider {
    node_provider: Option<NodeProviderReward>,
    reward_mode: Option<RewardMode>,
    amount_e8s: u64,
}

#[derive(Debug, Serialize, Deserialize, CandidType)]
struct XdrConversionRate {
    xdr_permyriad_per_icp: Option<u64>,
    timestamp_seconds: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, CandidType)]
struct DateRangeFilter {
    start_timestamp_seconds: Option<u64>,
    end_timestamp_seconds: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, CandidType)]
struct ListNodeProviderRewardsRequest {
    date_filter: Option<DateRangeFilter>,
}

#[derive(Debug, Serialize, Deserialize, CandidType)]
pub struct MonthlyNodeProviderRewards {
    timestamp: u64,
    rewards: Vec<RewardNodeProvider>,
    xdr_conversion_rate: Option<XdrConversionRate>,
    #[serde(default)]
    node_providers: Vec<NodeProviderReward>,
    #[serde(default)]
    pub registry_version: Option<u64>,
    #[serde(default)]
    pub minimum_xdr_permyriad_per_icp: Option<u64>,
    #[serde(default)]
    pub maximum_node_provider_rewards_e8s: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, CandidType)]
pub struct ListNodeProviderRewardsResponse {
    pub rewards: Vec<MonthlyNodeProviderRewards>,
}

#[derive(Eq, PartialEq, Debug)]
pub struct ChecksumError {
    input: [u8; 32],
    expected_checksum: [u8; 4],
    found_checksum: [u8; 4],
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderRewardInfo {
    reward_account_hex: Option<String>,
    pub reward_account_formatted: Option<String>,
    reward_account_dashboard_link: Option<String>,
    most_recent_reward_e8s: Option<u64>,
    most_recent_reward_xdr: Option<f64>,
    most_recent_timestamp: Option<u64>,
    total_mint_rewards_e8s: Option<u64>,
    total_mint_rewards_icp: Option<f64>,
    mint_transaction_count: Option<u32>,
    first_mint_timestamp: Option<u64>,
    last_mint_timestamp: Option<u64>,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct SimplifiedTransfer {
    pub op_type: String,
    pub from: String,
    pub to: String,
    pub id: u64,
    pub timestamp: u64,
    pub amount: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountTransactionsJson {
    pub name: String,
    pub principal: Option<String>,
    pub account: Option<(String, u64)>,
    pub ty: String,
    extra_accounts: Vec<(String, u64)>,
    pub transactions: Vec<SimplifiedTransfer>,
    pub oldest_tx_id: Option<u64>,
}

pub fn process_account_hex(hex: &str) -> (Option<String>, Option<String>, Option<String>) {
    // Original hex
    let orig_hex = Some(hex.to_string());

    // Try to convert to proper AccountIdentifier format
    if let Ok(account) = AccountIdentifier::from_hex(hex) {
        let formatted = account.to_hex();
        let dashboard_link = format!("https://dashboard.internetcomputer.org/account/{}", formatted);
        return (orig_hex, Some(formatted), Some(dashboard_link));
    }

    // If conversion fails, return only the original hex
    (orig_hex, None, None)
}

pub async fn fetch_nodes_rewards(agent: &Agent) -> Result<ListNodeProviderRewardsResponse, Box<dyn std::error::Error>> {
    let request = ListNodeProviderRewardsRequest { date_filter: None };

    // Encode the request using Candid
    let args = Encode!(&request)?;

    // Call the governance canister
    let principal = Principal::from_text(GOVERNANCE_CANISTER_ID)?;
    let response = agent.query(&principal, "list_node_provider_rewards").with_arg(args).call().await?;

    // Decode the response
    let result = Decode!(response.as_slice(), ListNodeProviderRewardsResponse)?;

    Ok(result)
}

pub async fn get_accounts_from_rewards(principal: Principal, rewards: ListNodeProviderRewardsResponse) -> Vec<String> {
    // Compute the default account identifier for the given principal (with default subaccount)
    let default_account: [u8; 32] = principal_to_account_id(&principal, None);
    let default_vec = default_account.to_vec();

    let mut extra_accounts: HashSet<String> = HashSet::new();

    for monthly in &rewards.rewards {
        for reward in &monthly.rewards {
            // Check if the reward mode is RewardToAccount.
            if let Some(RewardMode::RewardToAccount(ref reward_to_account)) = reward.reward_mode {
                if let Some(ref account) = reward_to_account.to_account {
                    // If the reward account's hash is different from the default, record it.
                    if account.hash != default_vec {
                        let hex = hex::encode(&account.hash);
                        extra_accounts.insert(hex);
                    }
                }
            }
        }
    }

    // Convert the HashSet into a Vec. Order is not guaranteed; use a BTreeSet if you need sorting.
    extra_accounts.into_iter().collect()
}

fn get_operation_type(op: &Operation) -> &str {
    match op {
        Operation::Approve { .. } => "Approve",
        Operation::Burn { .. } => "Burn",
        Operation::Mint { .. } => "Mint",
        Operation::Transfer { .. } => "Transfer",
    }
}

pub async fn fetch_with_retry(
    account: AccountData,
    agent: &Agent,
    max_retries: usize,
) -> Result<AccountTransactionsJson, Box<dyn std::error::Error>> {
    let mut attempts = 0;
    loop {
        match fetch_transactions(&account, agent).await {
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

pub async fn fetch_transactions(
    account_data: &AccountData,
    agent: &Agent,
) -> Result<AccountTransactionsJson, Box<dyn std::error::Error>> {
    let principal = Principal::from_text(INDEX_CANISTER_ID)?;
    let mut all_transactions = Vec::new();
    let mut extra_accounts = Vec::new();
    let mut oldest_tx_id = None;

    // Gather all account identifiers: from principals and from accounts field
    let mut identifiers: Vec<String> = account_data
        .principals
        .iter()
        .map(|p| {
            let acc_id = principal_to_account_id(p, None);
            hex::encode(acc_id)
        })
        .collect();

    identifiers.extend(account_data.accounts.iter().cloned());

    identifiers.sort();
    identifiers.dedup();

    // Use the first one as "main", others go to extra_accounts
    extra_accounts.extend(identifiers.iter().skip(1).cloned());

    let mut account_balances: Vec<(String, u64)> = Vec::new();

    for account_identifier in &identifiers {
        if !is_valid_account_id(account_identifier)? {
            println!("Skipping invalid account ID: {}", account_identifier);
            continue;
        }

        println!("Fetching txs for account {}", account_identifier);

        let request = GetAccountTransactionsArgs {
            max_results: 10000,
            start: None,
            account_identifier: account_identifier.clone(),
        };

        let args = Encode!(&request)?;
        let response_bytes =
            agent.query(&principal, "get_account_identifier_transactions").with_arg(args).call().await?;

        let result = Decode!(response_bytes.as_slice(), GetAccountIdentifierTransactionsResult)?;
        match result {
            GetAccountIdentifierTransactionsResult::Ok(resp) => {
                if oldest_tx_id.is_none() || resp.oldest_tx_id < oldest_tx_id {
                    oldest_tx_id = resp.oldest_tx_id;
                }
                account_balances.push((account_identifier.clone(), resp.balance));
                all_transactions.extend(resp.transactions);
            }
            GetAccountIdentifierTransactionsResult::Err(err) => {
                println!("Error from canister for {}: {}", account_identifier, err.message);
                continue;
            }
        }
    }

    // Sort the account_balances by the account identifier
    account_balances.sort_by(|a, b| a.0.cmp(&b.0));

    // Use the first one as "main", others go to extra_accounts
    let main_account = account_balances.first().cloned();
    let extra_accounts =
        if account_balances.len() > 1 { account_balances.iter().skip(1).cloned().collect() } else { Vec::new() };

    let simplified_transactions: Vec<SimplifiedTransfer> = all_transactions
        .into_iter()
        .filter_map(|tx_with_id| {
            if let Operation::Transfer { to, from, amount, .. } = &tx_with_id.transaction.operation {
                Some(SimplifiedTransfer {
                    op_type: get_operation_type(&tx_with_id.transaction.operation).to_string(),
                    from: from.clone(),
                    to: to.clone(),
                    id: tx_with_id.id,
                    timestamp: tx_with_id.transaction.timestamp.map(|ts| ts.timestamp_nanos).unwrap_or(0),
                    amount: amount.e8s,
                })
            } else {
                None
            }
        })
        .collect();

    Ok(AccountTransactionsJson {
        name: account_data.name.clone(),
        principal: account_data.principals.first().map(|p| p.to_text()),
        account: main_account,
        ty: format!("{:?}", account_data.ty),
        transactions: simplified_transactions,
        extra_accounts,
        oldest_tx_id,
    })
}
