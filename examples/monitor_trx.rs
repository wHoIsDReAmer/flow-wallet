use flow_wallet::monitor::TransactionMonitor;
use flow_wallet::node::tron::TronProvider;

#[tokio::main]
async fn main() {
    let provider = TronProvider::new();
    // Binance Hot Wallet Address (Very active)
    let address = "TMT4gXGWmJccmduMw6KrcCgz5gDDSosqbB".to_string();

    println!("Creating monitor for {}", address);
    let mut monitor = TransactionMonitor::new(provider, address, 5); // Check every 5 seconds

    monitor.run().await;
}
