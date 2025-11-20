use tokio::sync::broadcast;

use crate::node::{Provider, Transaction};
use std::time::Duration;

pub struct TransactionMonitor<P: Provider> {
    pub broadcast_tx: broadcast::Sender<Vec<Transaction>>,

    provider: P,
    address: String,
    interval: Duration,
    last_checked_timestamp: u64,
}

impl<P: Provider> TransactionMonitor<P> {
    pub fn new(provider: P, address: String, interval_secs: u64) -> Self {
        let (tx, _) = broadcast::channel(16);

        Self {
            broadcast_tx: tx,

            provider,
            address,
            interval: Duration::from_secs(interval_secs),
            last_checked_timestamp: 0,
        }
    }

    pub async fn run(&mut self) {
        let mut is_first = true;

        println!("Starting monitor for address: {}", self.address);

        loop {
            match self.provider.get_transactions(&self.address).await {
                Ok(txs) => {
                    let transactions = Vec::new();

                    // Sort by timestamp ascending to process in order
                    let mut sorted_txs = txs;
                    sorted_txs.sort_by_key(|t| t.timestamp);

                    sorted_txs.iter().for_each(|tx| {
                        if tx.timestamp > self.last_checked_timestamp {
                            println!("New Incoming Transaction Detected!");
                            println!("  Hash: {}", tx.hash);
                            println!("  From: {}", tx.from);
                            println!("  Value: {}", tx.value);
                            println!("  Timestamp: {}", tx.timestamp);
                            println!("---------------------------------");

                            self.last_checked_timestamp = tx.timestamp;
                        }
                    });

                    if !is_first {
                        continue;
                    }

                    if let Err(err) = self.broadcast_tx.send(transactions) {
                        eprintln!("Error broadcasting transactions: {}", err);
                    }
                }

                Err(e) => {
                    eprintln!("Error fetching transactions: {}", e);
                }
            }

            tokio::time::sleep(self.interval).await;
            is_first = false;
        }
    }
}
