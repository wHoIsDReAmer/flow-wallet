use crate::node::{NodeError, Provider, Transaction};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

const BLOCKCYPHER_LTC_MAINNET: &str = "https://api.blockcypher.com/v1/ltc/main";
pub const DECIMALS: u32 = 8;

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
    confirmed: Option<String>,
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
