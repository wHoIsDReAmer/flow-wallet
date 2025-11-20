use flow_wallet::node::{Provider, tron::TronProvider};

#[tokio::main]
async fn main() {
    // My temp address
    const ADDRESS: &str = "put_your_address_here";
    let tron_provider = TronProvider::new();

    match tron_provider.get_transactions(ADDRESS).await {
        Ok(result) => {
            println!("result: {:?}", result)
        }
        Err(err) => {
            println!("Error: {}", err);
        }
    }
}
