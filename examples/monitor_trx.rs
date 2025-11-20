use flow_wallet::monitor::TransactionMonitor;
use flow_wallet::node::tron::TronProvider;

#[tokio::main]
async fn main() {
    let provider = TronProvider::new();
    // Binance Hot Wallet Address (Very active)
    let address = "put_your_address".to_string();

    println!("Creating monitor for {}", address);
    let mut monitor = TransactionMonitor::new(provider, address, 5); // Check every 5 seconds
    let mut rx = monitor.broadcast_tx.subscribe();

    tokio::spawn(async move { monitor.run().await });

    while let Ok(txs) = rx.recv().await {
        println!("Received transactions: {:#?}", txs);
    }
}
