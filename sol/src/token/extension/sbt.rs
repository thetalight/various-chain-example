#[cfg(test)]
mod tests {
    use solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::CommitmentConfig};

    use solana_sdk::transaction::Transaction;
    use solana_sdk::{signature::Keypair, signer::Signer};
    use solana_system_interface::instruction::create_account;
    use spl_associated_token_account::get_associated_token_address_with_program_id;
    use spl_associated_token_account::instruction::create_associated_token_account;
    use spl_token_2022::extension::non_transferable::NonTransferable;
    use spl_token_2022::extension::StateWithExtensions;
    use spl_token_2022_interface::{
        extension::{
        BaseStateWithExtensions, ExtensionType}, ID as token_2022_program_id,
        instruction::{
            initialize_mint2, initialize_non_transferable_mint, mint_to, transfer_checked,
        },
        state::Mint,
    };

    use crate::account::airdrop::request_airdrop;

    #[tokio::test]
    async fn test_sbt() {
        let rpc_url = "http://127.0.0.1:8899".to_string();
        let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

        let palyer = Keypair::new();
        println!("player pubkey: {}", palyer.pubkey());

        request_airdrop(&client, &palyer.pubkey(), 800000000000)
            .await
            .unwrap();

        // 1. 创建Mint account
        let mint = Keypair::new();
        let mint_pk = mint.pubkey();
        println!("mint pubkey: {}\n", mint_pk);

        let mint_space =
            ExtensionType::try_calculate_account_len::<Mint>(&[ExtensionType::NonTransferable])
                .unwrap();

        let rent_exemption = client
            .get_minimum_balance_for_rent_exemption(mint_space)
            .await
            .unwrap();

        let mut instructions = vec![];

        let i_create_account = create_account(
            &palyer.pubkey(),
            &mint.pubkey(),
            rent_exemption + 100000000,
            mint_space as u64,
            &token_2022_program_id,
        );
        instructions.push(i_create_account);

        let initialize_non_transferable_instruction =
            initialize_non_transferable_mint(&token_2022_program_id, &mint.pubkey()).unwrap();
        instructions.push(initialize_non_transferable_instruction);

        let i_initialize = initialize_mint2(
            &token_2022_program_id,
            &mint.pubkey(),
            &palyer.pubkey(),
            Some(&palyer.pubkey()),
            0, // 0 decimals
        )
        .unwrap();
        instructions.push(i_initialize);

        // Instruction to create associated token account
        let create_ata_instruction = create_associated_token_account(
            &palyer.pubkey(),       // payer
            &palyer.pubkey(),       // owner
            &mint.pubkey(),         // mint
            &token_2022_program_id, // token program
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

        let mint_account = client.get_account(&mint.pubkey()).await.unwrap();
        let mint_state = StateWithExtensions::<Mint>::unpack(&mint_account.data).unwrap();

        let extension_types = mint_state.get_extension_types().unwrap();
        println!("\n Extensions enabled: {:.?}", extension_types);

        let non_transferable = mint_state.get_extension::<NonTransferable>().unwrap();
        println!("\n on_transferable:{:#.?}\n", non_transferable);

        let associated_token_address = get_associated_token_address_with_program_id(
            &palyer.pubkey(),       // owner
            &mint.pubkey(),         // minti
            &token_2022_program_id, // token program
        );

        // Mint 1 token to the associated token account
        let mint_to_instruction = mint_to(
            &token_2022_program_id,
            &mint.pubkey(),
            &associated_token_address,
            &palyer.pubkey(),
            &[],
            1,
        )
        .unwrap();

        let recent_blockhash = client.get_latest_blockhash().await.unwrap();
        let mint_transaction = Transaction::new_signed_with_payer(
            &[mint_to_instruction],
            Some(&palyer.pubkey()),
            &[&palyer],
            recent_blockhash,
        );

        client
            .send_and_confirm_transaction(&mint_transaction)
            .await
            .unwrap();

        let transfer_instruction = transfer_checked(
            &token_2022_program_id,
            &associated_token_address, // source
            &mint.pubkey(),            // mint
            &associated_token_address, // destination (same as source)
            &palyer.pubkey(),          // owner
            &[],                       // signers
            1,                         // amount
            0,                         // decimals
        )
        .unwrap();

        let recent_blockhash = client.get_latest_blockhash().await.unwrap();
        let transfer_transaction = Transaction::new_signed_with_payer(
            &[transfer_instruction],
            Some(&palyer.pubkey()),
            &[&palyer],
            recent_blockhash,
        );

        match client
            .send_and_confirm_transaction(&transfer_transaction)
            .await
        {
            Ok(sig) => println!("Transfer succeeded unexpectedly: {}", sig),
            Err(e) => println!("Transfer failed as expected:\n{:#?}", e),
        }
    }
}
