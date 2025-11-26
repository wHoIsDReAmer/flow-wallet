use flow_wallet::node::{Provider, network::prelude::*};

#[tokio::main]
async fn main() {
    // My temp address
    const ADDRESS: &str = "TMT4gXGWmJccmduMw6KrcCgz5gDDSosqbB";
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
            flow_wallet::node::utils::format_units(
                &balance,
                flow_wallet::node::network::tron::DECIMALS
            )
        );
    }
}
