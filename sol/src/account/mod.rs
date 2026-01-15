pub mod airdrop;
pub mod pda;
pub mod sysvar;

#[cfg(test)]
mod tests {
    use solana_client::rpc_request::Address;
    use solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::CommitmentConfig};
    use solana_sdk::{program_pack::Pack, signature::Keypair, signer::Signer};
    use spl_token::state::Mint;

    use crate::account::airdrop::request_airdrop;

    #[tokio::test]
    async fn test_wallet_account() {
        let rpc_url = "http://127.0.0.1:8899".to_string();
        let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

        let payer = Keypair::new();

        request_airdrop(&client, &payer.pubkey(), 100000000000)
            .await
            .unwrap();

        let balance = client.get_balance(&payer.pubkey()).await.unwrap();
        println!("Balance: {}", balance);

        let account = client.get_account(&payer.pubkey()).await.unwrap();
        println!("Account: {:?}", account);
    }

    #[tokio::test]
    async fn test_token_account() {
        let rpc_url = "https://api.mainnet.solana.com".to_string();
        let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

        // TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA 是spl tken
        // TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb 是token-2022，支持新特性
        let token: Address = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
            .parse()
            .unwrap();
        let account = client.get_account(&token).await.unwrap();
        println!("Account: {:?}", account);
    }

    #[tokio::test]
    async fn test_mint_account() {
        let rpc_url = "https://api.mainnet.solana.com".to_string();
        let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

        let token: Address = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
            .parse()
            .unwrap();
        let account = client.get_account(&token).await.unwrap();
        println!("Account: {:?}", account);

        let data = Mint::unpack(&account.data).unwrap();
        println!("Data: {:?}", data)
    }
}
