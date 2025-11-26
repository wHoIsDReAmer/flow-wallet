use flow_wallet::node::utils::format_units;
use flow_wallet::node::{Provider, network::prelude::*};

#[tokio::main]
async fn main() {
    // My temp address
    const ADDRESS: &str = "TT5iK8oqGEyRKJAnRwrLSZ4fM5y77F2LNT";
    let tron_provider = TronProvider::new();

    // get transactions
    match tron_provider.get_transactions(ADDRESS).await {
        Ok(result) => {
            println!("result: {:?}", result)
        }
        Err(err) => {
            println!("Error: {}", err);
        }
    };

    // get balance
    if let Ok(balance) = tron_provider.get_balance(ADDRESS).await {
        println!(
            "Balance: {} TRX",
            format_units(&balance, tron_provider.get_decimals())
        );
    }
}
