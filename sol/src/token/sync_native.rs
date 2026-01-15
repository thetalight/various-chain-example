#[cfg(test)]
mod tests {
    use solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::CommitmentConfig};
    use solana_sdk::transaction::Transaction;
    use solana_sdk::{signature::Keypair, signer::Signer};
    use solana_system_interface::instruction::{self, transfer};
    use spl_associated_token_account::get_associated_token_address_with_program_id;
    use spl_associated_token_account::instruction::create_associated_token_account;

    use spl_token_2022::extension::StateWithExtensions;
    use spl_token_2022::{
        id as token_2022_program_id, instruction::sync_native, native_mint::ID as NATIVE_MINT_ID,
        state::Account as Account2022,
    };

    use crate::account::airdrop::request_airdrop;

    #[tokio::test]
    async fn test_sync() {
        let rpc_url = "http://127.0.0.1:8899".to_string();
        let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

        let palyer = Keypair::new();
        let player_pubkey = palyer.pubkey();
        println!("player pubkey: {}", palyer.pubkey());

        request_airdrop(&client, &palyer.pubkey(), 100000000000)
            .await
            .unwrap();

        let associated_token_account = get_associated_token_address_with_program_id(
            &palyer.pubkey(),
            &NATIVE_MINT_ID,
            &token_2022_program_id(),
        );

        let create_ata_instruction = create_associated_token_account(
            &player_pubkey, // pay for fee
            &player_pubkey,
            &NATIVE_MINT_ID,
            &token_2022_program_id(),
        );

        let amount = 1_000_000;
        let transfer_instruction = transfer(&player_pubkey, &associated_token_account, amount);

        let sync_native_instruction =
            sync_native(&token_2022_program_id(), &associated_token_account).unwrap();

        let recent_blockhash = client.get_latest_blockhash().await.unwrap();
        let transaction = Transaction::new_signed_with_payer(
            &[
                create_ata_instruction,
                transfer_instruction,
                sync_native_instruction,
            ],
            Some(&palyer.pubkey()),
            &[&palyer],
            recent_blockhash,
        );
        let _tx_hash = client
            .send_and_confirm_transaction(&transaction)
            .await
            .unwrap();

        // let token_account = client.get_token_account(&associated_token_account).await.unwrap();
        // if let Some(token_account) = token_account {
        //     println!("{:#?}", token_account);
        // }

        let token_to = client.get_account(&associated_token_account).await.unwrap();
        let token_data_to = StateWithExtensions::<Account2022>::unpack(&token_to.data).unwrap();

        println!("Token account:{:?}", token_to);
        println!("Token data:{:?}\n", token_data_to);
    }
}
