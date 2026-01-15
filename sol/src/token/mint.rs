#[cfg(test)]
mod tests {
    use solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::CommitmentConfig};
    use solana_sdk::transaction::Transaction;
    use solana_sdk::{program_pack::Pack, signature::Keypair, signer::Signer};
    use solana_system_interface::instruction::{self};
    use spl_associated_token_account::get_associated_token_address_with_program_id;
    use spl_associated_token_account::instruction::create_associated_token_account;
    use spl_token::state::Mint;
    use spl_token_2022::extension::StateWithExtensions;
    use spl_token_2022::{
        id as token_2022_program_id,
        instruction::{initialize_mint2, mint_to},
        state::Account as Account2022,
    };

    use crate::account::airdrop::request_airdrop;

    #[tokio::test]
    async fn test_mint() {
        let rpc_url = "http://127.0.0.1:8899".to_string();
        let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

        let palyer = Keypair::new();
        let player_pubkey = palyer.pubkey();
        println!("player pubkey: {}", palyer.pubkey());

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

        // 2. 创建和初始化ATA
        let associated_token_account = get_associated_token_address_with_program_id(
            &palyer.pubkey(),
            &mint.pubkey(),
            &token_2022_program_id(),
        );

        let create_ata_instruction = create_associated_token_account(
            &player_pubkey, // pay for fee
            &player_pubkey,
            &mint.pubkey(),
            &token_2022_program_id(),
        );
        instructions.push(create_ata_instruction);

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

        // 3. mint
        let amount = 18446744073709551615;
        let mint_to_instruction = mint_to(
            &token_2022_program_id(),
            &mint.pubkey(),            // mint
            &associated_token_account, // destination
            &player_pubkey,            // authority
            &[&player_pubkey],         // signer
            amount,                    // amount
        )
        .unwrap();

        let transaction = Transaction::new_signed_with_payer(
            &[mint_to_instruction],
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

        let token = client.get_account(&associated_token_account).await.unwrap();
        let token_data = StateWithExtensions::<Account2022>::unpack(&token.data).unwrap();

        println!("Mint account:{:?}", mint_account);
        println!("Mint data{:?}\n", mint_data);

        println!("Token account{:?}", token);
        println!("Token data{:?}", token_data);
    }
}
