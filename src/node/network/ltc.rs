use crate::node::{NodeError, Provider, Transaction};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

const BLOCKCYPHER_LTC_MAINNET: &str = "https://api.blockcypher.com/v1/ltc/main";

pub struct LtcProvider {
    client: Client,
    base_url: String,
}

impl Default for LtcProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl LtcProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: BLOCKCYPHER_LTC_MAINNET.to_string(),
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
struct BlockcypherBalance {
    balance: u64,
    // final_balance: u64,
}

#[derive(Deserialize, Debug)]
struct BlockcypherTxRef {
    tx_hash: String,
    block_height: i64,
    value: i64,
    _confirmed: Option<String>,
}

#[derive(Deserialize, Debug)]
struct BlockcypherAddressFull {
    // address: String,
    // total_received: u64,
    // total_sent: u64,
    // balance: u64,
    txrefs: Option<Vec<BlockcypherTxRef>>,
}

#[derive(Deserialize, Debug)]
struct BlockcypherChain {
    height: u64,
}

#[async_trait]
impl Provider for LtcProvider {
    fn get_decimals(&self) -> u32 {
        8
    }

    async fn get_balance(&self, address: &str) -> Result<String, NodeError> {
        // https://api.blockcypher.com/v1/ltc/main/addrs/L.../balance
        let url = format!("{}/addrs/{}/balance", self.base_url, address);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| NodeError::Network(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(NodeError::Api(format!("Status: {}", resp.status())));
        }

        let body: BlockcypherBalance = resp
            .json()
            .await
            .map_err(|e| NodeError::Parse(e.to_string()))?;

        Ok(body.balance.to_string())
    }

    async fn get_transactions(&self, address: &str) -> Result<Vec<Transaction>, NodeError> {
        // https://api.blockcypher.com/v1/ltc/main/addrs/L...
        let url = format!("{}/addrs/{}", self.base_url, address);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| NodeError::Network(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(NodeError::Api(format!("Status: {}", resp.status())));
        }

        let body: BlockcypherAddressFull = resp
            .json()
            .await
            .map_err(|e| NodeError::Parse(e.to_string()))?;

        let txs = body.txrefs.unwrap_or_default();
        let transactions = txs
            .into_iter()
            .map(|tx| {
                Transaction {
                    hash: tx.tx_hash,
                    from: "".to_string(), // Blockcypher simplified view doesn't easily show from/to without deep dive
                    to: "".to_string(),
                    value: tx.value.to_string(),
                    block_number: tx.block_height as u64,
                    timestamp: 0, // Blockcypher doesn't provide timestamp in this view
                    status: if tx.block_height > 0 {
                        "SUCCESS"
                    } else {
                        "PENDING"
                    }
                    .to_string(),
                }
            })
            .collect();

        Ok(transactions)
    }

    async fn get_block_number(&self) -> Result<u64, NodeError> {
        // https://api.blockcypher.com/v1/ltc/main
        let url = self.base_url.clone();
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| NodeError::Network(e.to_string()))?;

        let body: BlockcypherChain = resp
            .json()
            .await
            .map_err(|e| NodeError::Parse(e.to_string()))?;

        Ok(body.height)
    }

    async fn create_transaction(
        &self,
        from: &str,
        to: &str,
        amount: u64,
    ) -> Result<String, NodeError> {
        // https://api.blockcypher.com/v1/ltc/main/txs/new
        let url = format!("{}/txs/new", self.base_url);

        #[derive(serde::Serialize)]
        struct CreateTxReq {
            inputs: Vec<Input>,
            outputs: Vec<Output>,
        }
        #[derive(serde::Serialize)]
        struct Input {
            addresses: Vec<String>,
        }
        #[derive(serde::Serialize)]
        struct Output {
            addresses: Vec<String>,
            value: u64,
        }

        let req = CreateTxReq {
            inputs: vec![Input {
                addresses: vec![from.to_string()],
            }],
            outputs: vec![Output {
                addresses: vec![to.to_string()],
                value: amount,
            }],
        };

        let resp = self
            .client
            .post(&url)
            .json(&req)
            .send()
            .await
            .map_err(|e| NodeError::Network(e.to_string()))?;

        // Blockcypher returns a JSON object with "tosign" array.
        // We return the whole JSON to be processed by the signer.
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| NodeError::Parse(e.to_string()))?;

        if let Some(err) = body.get("error") {
            return Err(NodeError::Api(err.to_string()));
        }

        Ok(body.to_string())
    }

    async fn broadcast_transaction(&self, raw_tx: &str) -> Result<String, NodeError> {
        // https://api.blockcypher.com/v1/ltc/main/txs/send
        let url = format!("{}/txs/send", self.base_url);

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

        if let Some(err) = body.get("error") {
            return Err(NodeError::Api(err.to_string()));
        }

        // Returns the full tx object, we want the hash
        if let Some(tx) = body.get("tx")
            && let Some(hash) = tx.get("hash")
        {
            return Ok(hash.as_str().unwrap_or("SUCCESS").to_string());
        }

        // Fallback if structure is different
        Ok("SUCCESS".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ltc_provider_instantiation() {
        let provider = LtcProvider::new();
        assert_eq!(provider.base_url, BLOCKCYPHER_LTC_MAINNET);
    }
}
