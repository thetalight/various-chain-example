pub mod account;
pub mod token;

#[cfg(test)]
mod tests {
    use solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::CommitmentConfig};
    use solana_sdk::{signature::Keypair, signer::Signer};

    #[tokio::test]
    async fn it_works() {
        let rpc_url = "http://127.0.0.1:8899".to_string();
        let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

        // Load or create a keypair for the payer
        let payer = Keypair::new();
        let balance = client.get_balance(&payer.pubkey()).await.unwrap();
        println!("Balance: {}", balance);

        let data = client.get_account(&payer.pubkey()).await.unwrap();
        println!("Data: {:?}", data);
    }
}
