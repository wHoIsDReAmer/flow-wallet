use flow_wallet::wallet::Wallet;
use flow_wallet::wallet::chain::{TRON, UtxoChain};
use flow_wallet::wallet::key_source::{KeySource, MnemonicKeySource};

const LTC_TESTNET: UtxoChain = UtxoChain {
    name: "litecoin_testnet",
    p2pkh_prefix: 0x6f, // Testnet prefix
};

#[tokio::main]
async fn main() {
    // 1. Generate a random mnemonic
    let source = MnemonicKeySource::random(None);
    let phrase = source.phrase();

    println!("\n==================================================");
    println!("             NEW TESTNET WALLET GENERATED         ");
    println!("==================================================");
    println!("MNEMONIC (Save this!):");
    println!("{}", phrase);
    println!("==================================================\n");

    // 2. Derive Tron Address (Shasta)
    // Path: m/44'/195'/0'/0/0
    let signer_tron = source
        .derive_signer("m/44'/195'/0'/0/0")
        .await
        .expect("derive tron");
    let wallet_tron = Wallet::new(signer_tron, TRON);
    let addr_tron = wallet_tron.address().expect("address tron");

    println!("TRON (Shasta Testnet)");
    println!("Address: {}", addr_tron);
    println!("Faucet:  https://www.trongrid.io/shasta");
    println!("--------------------------------------------------");

    // 3. Derive Litecoin Address (Testnet)
    // Path: m/44'/1'/0'/0/0 (Coin type 1 is often used for testnets)
    let signer_ltc = source
        .derive_signer("m/44'/1'/0'/0/0")
        .await
        .expect("derive ltc");
    let wallet_ltc = Wallet::new(signer_ltc, LTC_TESTNET);
    let addr_ltc = wallet_ltc.address().expect("address ltc");

    println!("LITECOIN (Testnet)");
    println!("Address: {}", addr_ltc);
    println!("Faucet:  https://live.blockcypher.com/ltc-testnet/faucet/");
    println!("==================================================\n");

    println!("INSTRUCTIONS:");
    println!("1. Copy the addresses above.");
    println!("2. Go to the Faucet URLs and request funds.");
    println!("3. Run the integration test with the mnemonic:");
    println!(
        "   TESTNET_MNEMONIC=\"{}\" cargo test --test integration_test -- --nocapture",
        phrase
    );
    println!("\n");
}
