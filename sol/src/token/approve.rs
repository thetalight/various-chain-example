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
    use spl_token_2022::instruction::transfer_checked;
    use spl_token_2022::{
        id as token_2022_program_id,
        instruction::{approve_checked, initialize_mint2, mint_to, revoke},
        state::Account as Account2022,
    };

    use crate::account::airdrop::request_airdrop;

    #[tokio::test]
    async fn test_approve() {
        // 同一时间只能有一个delegate，每次的approve都会覆盖之前的设置

        let rpc_url = "http://127.0.0.1:8899".to_string();
        let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

        let palyer = Keypair::new();
        let player_pubkey = palyer.pubkey();
        println!("player pubkey: {}", palyer.pubkey());

        let delagate = Keypair::new();
        println!("delegate pubkey: {}", delagate.pubkey());

        let to = Keypair::new();
        println!("to pubkey: {}", to.pubkey());

        request_airdrop(&client, &palyer.pubkey(), 100000000000)
            .await
            .unwrap();
        request_airdrop(&client, &delagate.pubkey(), 100000000000)
            .await
            .unwrap();
        request_airdrop(&client, &to.pubkey(), 100000000000)
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

        // 2. player-ATA
        let associated_token_account_player = get_associated_token_address_with_program_id(
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

        // 3. mint to player's ATA
        let amount = 10000;
        let mint_to_instruction = mint_to(
            &token_2022_program_id(),
            &mint.pubkey(),
            &associated_token_account_player,
            &player_pubkey,
            &[&player_pubkey],
            amount,
        )
        .unwrap();
        instructions.push(mint_to_instruction);

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

        // 4. approve
        let approve_amount = 50;
        let approve_instruction = approve_checked(
            &token_2022_program_id(),
            &associated_token_account_player,
            &mint.pubkey(),
            &delagate.pubkey(),
            &player_pubkey,
            &[&player_pubkey],
            approve_amount,
            6,
        )
        .unwrap();

        let recent_blockhash = client.get_latest_blockhash().await.unwrap();
        let transaction = Transaction::new_signed_with_payer(
            &[approve_instruction],
            Some(&palyer.pubkey()),
            &[&palyer],
            recent_blockhash,
        );
        let _transaction_signature = client
            .send_and_confirm_transaction(&transaction)
            .await
            .unwrap();

        let token_player = client
            .get_account(&associated_token_account_player)
            .await
            .unwrap();
        let token_data_player =
            StateWithExtensions::<Account2022>::unpack(&token_player.data).unwrap();
        println!("\nToken data:{:?}\n", token_data_player);

        // transfer
        let associated_token_account_to = get_associated_token_address_with_program_id(
            &to.pubkey(),
            &mint.pubkey(),
            &token_2022_program_id(),
        );
        let create_ata_instruction = create_associated_token_account(
            &delagate.pubkey(), // pay for fee
            &to.pubkey(),
            &mint.pubkey(),
            &token_2022_program_id(),
        );

        let transfer_amount = 10;
        let transfer_instruction = transfer_checked(
            &token_2022_program_id(),
            &associated_token_account_player, // from
            &mint.pubkey(),
            &associated_token_account_to, // to
            &delagate.pubkey(),
            &[&delagate.pubkey()],
            transfer_amount,
            6,
        )
        .unwrap();
        let recent_blockhash = client.get_latest_blockhash().await.unwrap();
        let transaction = Transaction::new_signed_with_payer(
            &[create_ata_instruction, transfer_instruction],
            Some(&delagate.pubkey()),
            &[&delagate],
            recent_blockhash,
        );
        let _transaction_signature = client
            .send_and_confirm_transaction(&transaction)
            .await
            .unwrap();

        let token_player = client
            .get_account(&associated_token_account_player)
            .await
            .unwrap();
        let token_data_player =
            StateWithExtensions::<Account2022>::unpack(&token_player.data).unwrap();
        println!("Token data:{:?}\n", token_data_player);

        // 撤销授权
        let revoke_instruction = revoke(
            &token_2022_program_id(),
            &associated_token_account_player,
            &player_pubkey,
            &[&player_pubkey],
        )
        .unwrap();

        let recent_blockhash = client.get_latest_blockhash().await.unwrap();
        let transaction = Transaction::new_signed_with_payer(
            &[revoke_instruction],
            Some(&palyer.pubkey()),
            &[&palyer],
            recent_blockhash,
        );
        let _transaction_signature = client
            .send_and_confirm_transaction(&transaction)
            .await
            .unwrap();

        let token_player = client
            .get_account(&associated_token_account_player)
            .await
            .unwrap();
        let token_data_player =
            StateWithExtensions::<Account2022>::unpack(&token_player.data).unwrap();
        println!("Token data:{:?}\n", token_data_player);
    }
}
