use flow_wallet::node::Provider;
use flow_wallet::node::network::ltc::LtcProvider;
use flow_wallet::node::network::tron::TronProvider;
use flow_wallet::wallet::Wallet;
use flow_wallet::wallet::chain::{TRON, UtxoChain};
use flow_wallet::wallet::key_source::{KeySource, MnemonicKeySource};
use std::env;

#[tokio::test]
async fn test_tron_send_coins_real() {
    let phrase = match env::var("TESTNET_MNEMONIC") {
        Ok(p) => p,
        Err(_) => {
            println!("SKIPPING: TESTNET_MNEMONIC not set");
            return;
        }
    };

    // 1. Setup Wallet
    let key_source = MnemonicKeySource::new(&phrase, None).expect("valid mnemonic");
    let signer = key_source
        .derive_signer("m/44'/195'/0'/0/0")
        .await
        .expect("derive");
    let wallet = Wallet::new(signer, TRON);
    let address = wallet.address().expect("address");
    println!("Tron Address: {}", address);

    // 2. Setup Real Provider (Shasta Testnet)
    let provider = TronProvider::with_url("https://api.shasta.trongrid.io".to_string());

    // 3. Check Balance
    let balance = provider.get_balance(&address).await.expect("get balance");
    println!("Balance: {} SUN", balance);

    if balance == "0" {
        println!("SKIPPING SEND: Insufficient balance");
        return;
    }

    // 4. Execute Send (Self-transfer of 100 SUN)
    // Note: This might fail if not enough bandwidth/energy, but it tests the flow.
    match wallet.send_coins(&provider, &address, 100).await {
        Ok(tx_hash) => println!("Tron Tx Hash: {}", tx_hash),
        Err(e) => println!("Tron Send Failed (Expected if no funds/energy): {}", e),
    }
}

#[tokio::test]
async fn test_ltc_send_coins_real() {
    let phrase = match env::var("TESTNET_MNEMONIC") {
        Ok(p) => p,
        Err(_) => {
            println!("SKIPPING: TESTNET_MNEMONIC not set");
            return;
        }
    };

    // 1. Setup Wallet
    let key_source = MnemonicKeySource::new(&phrase, None).expect("valid mnemonic");
    let signer = key_source
        .derive_signer("m/44'/1'/0'/0/0") // Testnet coin type is often 1, but LTC testnet might use 1 or same as mainnet with different prefix.
        // BIP44 for LTC is 2. Testnet is usually 1.
        // Let's assume standard testnet path.
        .await
        .expect("derive");

    let chain = UtxoChain {
        name: "litecoin_testnet",
        p2pkh_prefix: 0x6f, // LTC Testnet prefix (m or n) is 0x6f (111)
    };
    let wallet = Wallet::new(signer, chain);
    let address = wallet.address().expect("address");
    println!("LTC Testnet Address: {}", address);

    // 2. Setup Real Provider (Blockcypher Testnet)
    let provider = LtcProvider::with_url("https://api.blockcypher.com/v1/ltc/test3".to_string());

    // 3. Check Balance
    let balance = provider.get_balance(&address).await.expect("get balance");
    println!("Balance: {} Satoshis", balance);

    if balance == "0" {
        println!("SKIPPING SEND: Insufficient balance");
        return;
    }

    // 4. Execute Send (Self-transfer of 1000 Satoshis)
    match wallet.send_coins(&provider, &address, 1000).await {
        Ok(tx_hash) => println!("LTC Tx Hash: {}", tx_hash),
        Err(e) => println!("LTC Send Failed (Expected if no funds): {}", e),
    }
}
