use flow_wallet::node::{Provider, network::prelude::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = LtcProvider::new();

    let balance: String = provider
        .get_balance("ltc1qp7cnlxmz8wgc93g0m020ckru2s55t25y3wunf6")
        .await?;

    println!(
        "Balance: {} LTC",
        flow_wallet::node::utils::format_units(&balance, flow_wallet::node::network::ltc::DECIMALS)
    );

    Ok(())
}
