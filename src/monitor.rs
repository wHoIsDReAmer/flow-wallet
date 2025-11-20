use crate::node::ext::Provider;
use std::time::Duration;
use tokio::time::sleep;

pub struct TransactionMonitor<P: Provider> {
    provider: P,
    address: String,
    interval: Duration,
    last_checked_timestamp: u64,
}

impl<P: Provider> TransactionMonitor<P> {
    pub fn new(provider: P, address: String, interval_secs: u64) -> Self {
        Self {
            provider,
            address,
            interval: Duration::from_secs(interval_secs),
            last_checked_timestamp: 0,
        }
    }

    pub async fn run(&mut self) {
        println!("Starting monitor for address: {}", self.address);

        // Initial fetch to set baseline or just start from 0?
        // If we start from 0, we might fetch old history.
        // Let's assume we want to see *new* transactions from now on.
        // But for the demo, seeing history is fine.
        // Let's just loop.

        loop {
            match self.provider.get_transactions(&self.address).await {
                Ok(txs) => {
                    // Sort by timestamp ascending to process in order
                    let mut sorted_txs = txs;
                    sorted_txs.sort_by_key(|t| t.timestamp);

                    for tx in sorted_txs {
                        if tx.timestamp > self.last_checked_timestamp {
                            println!("New Incoming Transaction Detected!");
                            println!("  Hash: {}", tx.hash);
                            println!("  From: {}", tx.from);
                            println!("  Value: {}", tx.value);
                            println!("  Timestamp: {}", tx.timestamp);
                            println!("---------------------------------");

                            self.last_checked_timestamp = tx.timestamp;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error fetching transactions: {}", e);
                }
            }
            sleep(self.interval).await;
        }
    }
}
