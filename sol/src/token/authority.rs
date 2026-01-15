#[cfg(test)]
mod tests {
    use solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::CommitmentConfig};
    use solana_sdk::transaction::Transaction;
    use solana_sdk::{program_pack::Pack, signature::Keypair, signer::Signer};
    use solana_system_interface::instruction::{self};
    use spl_token::state::Mint;
    use spl_token_2022::instruction::AuthorityType;
    use spl_token_2022::{
        id as token_2022_program_id,
        instruction::{initialize_mint2, set_authority},
    };

    use crate::account::airdrop::request_airdrop;

    #[tokio::test]
    async fn test_authority() {
        let rpc_url = "http://127.0.0.1:8899".to_string();
        let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

        let palyer = Keypair::new();
        println!("player pubkey: {}", palyer.pubkey());

        let new_authority = Keypair::new();
        println!("new authority pubkey: {}\n", new_authority.pubkey());

        request_airdrop(&client, &palyer.pubkey(), 100000000000)
            .await
            .unwrap();

        // 1. 创建和初始化Mint account
        let rent_exemption = client
            .get_minimum_balance_for_rent_exemption(Mint::LEN)
            .await
            .unwrap();

        let mut instructions = vec![];

        let mint = Keypair::new();
        let mint_pk = mint.pubkey();
        println!("mint pubkey: {}", mint_pk);

        let i_create_account = instruction::create_account(
            &palyer.pubkey(),
            &mint.pubkey(),
            rent_exemption,
            Mint::LEN as u64,
            &token_2022_program_id(),
        );
        instructions.push(i_create_account);

        let i_initialize = initialize_mint2(
            &token_2022_program_id(),
            &mint.pubkey(),
            &palyer.pubkey(),
            Some(&palyer.pubkey()),
            6,
        )
        .unwrap();
        instructions.push(i_initialize);

        let recent_blockhash = client.get_latest_blockhash().await.unwrap();
        let transaction = Transaction::new_signed_with_payer(
            &instructions,
            Some(&palyer.pubkey()),
            &[&palyer, &mint],
            recent_blockhash,
        );
        let _tx_hash = client
            .send_and_confirm_transaction(&transaction)
            .await
            .unwrap();

        let mint_account = client.get_account(&mint.pubkey()).await.unwrap();
        let mint_data = Mint::unpack(&mint_account.data).unwrap();
        println!("Mint account:{:#?}", mint_account);
        println!("Mint data:{:#?}\n", mint_data);

        // 1. Change Mint Authority (MintTokens)
        let set_mint_authority_ix = set_authority(
            &token_2022_program_id(),
            &mint.pubkey(),
            Some(&new_authority.pubkey()), // null即为放弃该权限
            AuthorityType::MintTokens,
            &palyer.pubkey(),
            &[&palyer.pubkey()],
        )
        .unwrap();

        // 2. Change Freeze Authority (MintTokens)
        let set_freeze_authority_ix = set_authority(
            &token_2022_program_id(),
            &mint.pubkey(),
            Some(&new_authority.pubkey()), // null即为放弃该权限
            AuthorityType::FreezeAccount,
            &palyer.pubkey(),
            &[&palyer.pubkey()],
        )
        .unwrap();

        let recent_blockhash = client.get_latest_blockhash().await.unwrap();
        let transaction = Transaction::new_signed_with_payer(
            &[set_mint_authority_ix, set_freeze_authority_ix],
            Some(&palyer.pubkey()),
            &[&palyer],
            recent_blockhash,
        );
        let _tx_hash = client
            .send_and_confirm_transaction(&transaction)
            .await
            .unwrap();

        let mint_account = client.get_account(&mint.pubkey()).await.unwrap();
        let mint_data = Mint::unpack(&mint_account.data).unwrap();
        println!("Mint account:{:#?}", mint_account);
        println!("Mint ata:{:#?}", mint_data);
    }
}
