use crate::node::{NodeError, Provider, Transaction};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

const TRON_GRID_MAINNET: &str = "https://api.trongrid.io";
pub const DECIMALS: u32 = 6;

pub struct TronProvider {
    client: Client,
    base_url: String,
}

impl Default for TronProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TronProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: TRON_GRID_MAINNET.to_string(),
        }
    }
}

#[derive(Deserialize, Debug)]
struct TronGridResponse<T> {
    data: Vec<T>,
    success: bool,
}

#[derive(Deserialize, Debug)]
struct Trc20Transfer {
    transaction_id: String,
    token_info: TokenInfo,
    block_timestamp: u64,
    from: String,
    to: String,
    value: String,
    // type: String, // "Transfer"
}

#[derive(Deserialize, Debug)]
struct TokenInfo {
    symbol: String,
    address: String,
    decimals: u8,
    name: String,
}

#[async_trait]
impl Provider for TronProvider {
    async fn get_transactions(&self, address: &str) -> Result<Vec<Transaction>, NodeError> {
        // Fetch TRC-20 transactions
        // Docs: https://developers.tron.network/reference/get-trc20-transaction-info-by-account-address
        let url = format!(
            "{}/v1/accounts/{}/transactions/trc20",
            self.base_url, address
        );

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| NodeError::Network(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(NodeError::Api(format!("Status: {}", resp.status())));
        }

        let body: TronGridResponse<Trc20Transfer> = resp
            .json()
            .await
            .map_err(|e| NodeError::Parse(e.to_string()))?;

        if !body.success {
            return Err(NodeError::Api(
                "TronGrid returned success: false".to_string(),
            ));
        }

        let transactions = body
            .data
            .into_iter()
            .map(|tx| {
                Transaction {
                    hash: tx.transaction_id,
                    from: tx.from,
                    to: tx.to,
                    value: tx.value,
                    block_number: 0, // TronGrid v1 API might not return block number in this endpoint easily, or we need to look closer. For now 0.
                    timestamp: tx.block_timestamp,
                    status: "SUCCESS".to_string(), // Assumed success if it appears here? Need verification.
                }
            })
            .collect();

        Ok(transactions)
    }

    async fn get_block_number(&self) -> Result<u64, NodeError> {
        // https://developers.tron.network/reference/get-now-block
        // But that's wallet/getnowblock (POST).
        // Let's use wallet/getnowblock
        let url = format!("{}/wallet/getnowblock", self.base_url);
        let resp = self
            .client
            .post(&url)
            .send()
            .await
            .map_err(|e| NodeError::Network(e.to_string()))?;

        #[derive(Deserialize)]
        struct BlockHeader {
            raw_data: BlockRawData,
        }
        #[derive(Deserialize)]
        struct BlockRawData {
            number: u64,
        }
        #[derive(Deserialize)]
        struct BlockResponse {
            block_header: BlockHeader,
        }

        let body: BlockResponse = resp
            .json()
            .await
            .map_err(|e| NodeError::Parse(e.to_string()))?;

        Ok(body.block_header.raw_data.number)
    }

    async fn get_balance(&self, address: &str) -> Result<String, NodeError> {
        // Docs: https://developers.tron.network/reference/account-getaccount
        let url = format!("{}/v1/accounts/{}", self.base_url, address);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| NodeError::Network(e.to_string()))?;

        #[derive(Deserialize)]
        struct AccountResponse {
            data: Vec<AccountData>,
            success: bool,
        }
        #[derive(Deserialize)]
        struct AccountData {
            balance: Option<u64>,
        }

        let body: AccountResponse = resp
            .json()
            .await
            .map_err(|e| NodeError::Parse(e.to_string()))?;

        if !body.success {
            return Err(NodeError::Api(
                "TronGrid returned success: false".to_string(),
            ));
        }

        if let Some(account) = body.data.first() {
            // Balance is in Sun (1 TRX = 1,000,000 Sun)
            Ok(account.balance.unwrap_or(0).to_string())
        } else {
            // Account not found usually means 0 balance on Tron
            Ok("0".to_string())
        }
    }
}
