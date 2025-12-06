use crate::node::{NodeError, Provider, Transaction};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

const TRON_GRID_MAINNET: &str = "https://api.trongrid.io";
const TRON_GRID_NILE: &str = "https://nile.trongrid.io";

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

    pub fn nile() -> Self {
        Self {
            client: Client::new(),
            base_url: TRON_GRID_NILE.to_string(),
        }
    }

    pub fn with_url(url: String) -> Self {
        Self {
            client: Client::new(),
            base_url: url,
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
    fn get_decimals(&self) -> u32 {
        6
    }

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

    async fn create_transaction(
        &self,
        from: &str,
        to: &str,
        amount: u64,
    ) -> Result<String, NodeError> {
        // https://developers.tron.network/reference/createtransaction
        let url = format!("{}/wallet/createtransaction", self.base_url);

        #[derive(serde::Serialize)]
        struct CreateTxReq {
            to_address: String,
            owner_address: String,
            amount: u64,
        }

        let req = CreateTxReq {
            to_address: to.to_string(),
            owner_address: from.to_string(),
            amount,
        };

        let resp = self
            .client
            .post(&url)
            .json(&req)
            .send()
            .await
            .map_err(|e| NodeError::Network(e.to_string()))?;

        // Tron returns the full JSON transaction object. We just return it as string.
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| NodeError::Parse(e.to_string()))?;

        if let Some(err) = body.get("Error") {
            return Err(NodeError::Api(err.to_string()));
        }

        Ok(body.to_string())
    }

    async fn broadcast_transaction(&self, raw_tx: &str) -> Result<String, NodeError> {
        // https://developers.tron.network/reference/broadcasttransaction
        let url = format!("{}/wallet/broadcasttransaction", self.base_url);

        let tx: serde_json::Value =
            serde_json::from_str(raw_tx).map_err(|e| NodeError::Parse(e.to_string()))?;

        let resp = self
            .client
            .post(&url)
            .json(&tx)
            .send()
            .await
            .map_err(|e| NodeError::Network(e.to_string()))?;

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| NodeError::Parse(e.to_string()))?;

        if let Some(result) = body.get("result")
            && result.as_bool() == Some(true)
        {
            // Return txID if available, or just "SUCCESS"
            return Ok(body
                .get("txid")
                .and_then(|v| v.as_str())
                .unwrap_or("SUCCESS")
                .to_string());
        }

        Err(NodeError::Api(format!("Broadcast failed: {}", body)))
    }
}
