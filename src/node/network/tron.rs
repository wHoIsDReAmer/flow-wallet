use crate::node::{NodeError, Provider, Transaction};
use crate::wallet::crypto::hash::double_sha256;
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
    #[serde(default)]
    meta: Option<TronGridMeta>,
}

#[derive(Deserialize, Debug)]
struct TronGridMeta {
    at: Option<u64>,
    page_size: Option<u64>,
}

#[derive(Deserialize, Debug)]
struct TronTransaction {
    #[serde(rename = "txID")]
    tx_id: String,
    #[serde(rename = "blockNumber")]
    block_number: Option<u64>,
    #[serde(rename = "block_timestamp")]
    block_timestamp: Option<u64>,
    #[serde(default)]
    ret: Vec<TronContractRet>,
    raw_data: Option<TronRawData>,
}

#[derive(Deserialize, Debug)]
struct TronContractRet {
    #[serde(rename = "contractRet")]
    contract_ret: Option<String>,
}

#[derive(Deserialize, Debug)]
struct TronRawData {
    contract: Vec<TronContract>,
}

#[derive(Deserialize, Debug)]
struct TronContract {
    parameter: Option<TronContractParameter>,
}

#[derive(Deserialize, Debug)]
struct TronContractParameter {
    value: Option<TronTransferValue>,
}

#[derive(Deserialize, Debug)]
struct TronTransferValue {
    amount: Option<TronAmount>,
    owner_address: Option<String>,
    to_address: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum TronAmount {
    Number(u64),
    String(String),
}

fn tron_hex_to_base58(address_hex: &str) -> Option<String> {
    let trimmed = address_hex.strip_prefix("0x").unwrap_or(address_hex);
    let bytes = hex::decode(trimmed).ok()?;
    if bytes.len() != 21 {
        return None;
    }

    let checksum_full = double_sha256(&bytes);
    let mut address_bytes = Vec::with_capacity(25);
    address_bytes.extend_from_slice(&bytes);
    address_bytes.extend_from_slice(&checksum_full[..4]);

    Some(bs58::encode(address_bytes).into_string())
}

#[async_trait]
impl Provider for TronProvider {
    fn get_decimals(&self) -> u32 {
        6
    }

    async fn get_transactions(&self, address: &str) -> Result<Vec<Transaction>, NodeError> {
        // Fetch account transactions
        // Docs: https://developers.tron.network/reference/get-account-transaction
        let url = format!("{}/v1/accounts/{}/transactions", self.base_url, address);

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| NodeError::Network(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(NodeError::Api(format!("Status: {}", resp.status())));
        }

        let body: TronGridResponse<TronTransaction> = resp
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
                let (from, to, value) = tx
                    .raw_data
                    .as_ref()
                    .and_then(|raw| raw.contract.first())
                    .and_then(|contract| contract.parameter.as_ref())
                    .and_then(|param| param.value.as_ref())
                    .map(|value| {
                        let amount = match &value.amount {
                            Some(TronAmount::Number(n)) => n.to_string(),
                            Some(TronAmount::String(s)) => s.clone(),
                            None => "0".to_string(),
                        };
                        let owner_hex = value.owner_address.clone().unwrap_or_default();
                        let to_hex = value.to_address.clone().unwrap_or_default();
                        let from = tron_hex_to_base58(&owner_hex).unwrap_or(owner_hex);
                        let to = tron_hex_to_base58(&to_hex).unwrap_or(to_hex);
                        (from, to, amount)
                    })
                    .unwrap_or_else(|| ("".to_string(), "".to_string(), "0".to_string()));

                let status = tx
                    .ret
                    .first()
                    .and_then(|ret| ret.contract_ret.as_deref())
                    .unwrap_or("UNKNOWN")
                    .to_string();

                Transaction {
                    hash: tx.tx_id,
                    from,
                    to,
                    value,
                    block_number: tx.block_number.unwrap_or(0),
                    timestamp: tx.block_timestamp.unwrap_or(0),
                    status,
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
