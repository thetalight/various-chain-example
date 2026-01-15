#[cfg(test)]
mod test {
    use bincode::deserialize;
    use solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::CommitmentConfig};
    use solana_sdk::sysvar::{self, clock::Clock};

    #[tokio::test]
    async fn test_sysvar() {
        let rpc_url = "https://api.mainnet.solana.com".to_string();
        let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

        let account = client.get_account(&sysvar::clock::ID).await.unwrap();
        // Deserialize the account data
        let clock: Clock = deserialize(&account.data).unwrap();

        println!("{:#?}", account);
        println!("{:#?}", clock);
    }
}
