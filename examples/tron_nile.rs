use flow_wallet::node::{Provider, network::prelude::*, utils::format_units};

#[tokio::main]
async fn main() {
    // Nile Testnet Address (Just a random valid-looking address or one from documentation)
    // 41... is raw address, T... is base58check.
    // Let's use a known Nile faucet address or similar if possible, otherwise a random one.
    // Nile Faucet Address (from random search): TX72Yv5X...
    // Let's just use the one from the mainnet example, it's a valid address format. 
    // It will likely have 0 balance on Nile.
    const ADDRESS: &str = "TT5iK8oqGEyRKJAnRwrLSZ4fM5y77F2LNT";
    
    println!("Connecting to Tron Nile Testnet...");
    let tron_provider = TronProvider::nile();

    // get transactions
    println!("Fetching transactions for {}...", ADDRESS);
    match tron_provider.get_transactions(ADDRESS).await {
        Ok(result) => {
            println!("Transactions: {} found", result.len());
            // println!("result: {:?}", result);
        }
        Err(err) => {
            println!("Error fetching transactions: {}", err);
        }
    };

    // get balance
    println!("Fetching balance for {}...", ADDRESS);
    if let Ok(balance) = tron_provider.get_balance(ADDRESS).await {
        println!(
            "Balance: {} TRX",
            format_units(&balance, tron_provider.get_decimals())
        );
    } else {
        println!("Failed to fetch balance");
    }
}
