pub mod approve;
pub mod authority;
pub mod burn;
pub mod close_account;
pub mod extension;
pub mod freeze_account;
pub mod mint;
pub mod sync_native;
pub mod transfer;

#[cfg(test)]
mod tests {
    use solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::CommitmentConfig};
    use solana_sdk::transaction::Transaction;
    use solana_sdk::{program_pack::Pack, signature::Keypair, signer::Signer};
    use solana_system_interface::instruction::{self, create_account};
    use spl_associated_token_account::get_associated_token_address_with_program_id;
    use spl_associated_token_account::{
        get_associated_token_address, instruction::create_associated_token_account,
    };
    use spl_token::state::{Account, Mint};
    use spl_token_2022::extension::StateWithExtensions;
    use spl_token_2022::instruction::initialize_account3;
    use spl_token_2022::{
        id as token_2022_program_id, instruction::initialize_mint2, state::Account as Account2022,
    };

    use crate::account::airdrop::request_airdrop;

    #[tokio::test]
    async fn test_create_token() {
        let rpc_url = "http://127.0.0.1:8899".to_string();
        let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

        let palyer = Keypair::new();
        let player_pubkey = palyer.pubkey();
        println!("player pubkey: {}", palyer.pubkey());

        request_airdrop(&client, &palyer.pubkey(), 100000000000)
            .await
            .unwrap();

        // 1. 创建和初始化Mint account

        // 一个账户如果要永久不被收租，最少需要存多少 SOL
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

        let account = client.get_account(&mint_pk).await.unwrap();
        let data = Mint::unpack(&account.data).unwrap();
        println!("Mint_Data: {:?}", data);

        // 2. 创建token account
        let token_account = Keypair::new();
        let token_account_rent = client
            .get_minimum_balance_for_rent_exemption(Account::LEN)
            .await
            .unwrap();

        let mut instructions = vec![];

        let i_create_token_account = create_account(
            &player_pubkey,
            &token_account.pubkey(),
            token_account_rent,
            Account::LEN as u64,
            &token_2022_program_id(),
        );
        instructions.push(i_create_token_account);

        let i_initialize_account = initialize_account3(
            &token_2022_program_id(),
            &token_account.pubkey(),
            &mint_pk,
            &&player_pubkey,
        )
        .unwrap();
        instructions.push(i_initialize_account);

        let recent_blockhash = client.get_latest_blockhash().await.unwrap();
        let transaction = Transaction::new_signed_with_payer(
            &instructions,
            Some(&player_pubkey),
            &[&palyer, &token_account],
            recent_blockhash,
        );
        let _tx_hash = client
            .send_and_confirm_transaction(&transaction)
            .await
            .unwrap();

        let token = client.get_account(&token_account.pubkey()).await.unwrap();
        let token_data = Account::unpack(&token.data).unwrap();
        println!("\nToken Account: {:?}", token);
        println!("Token Account Data:{:#?}", token_data);

        // 3. 创建ATA
        let mut instructions = vec![];

        // spl token
        // let associated_token_account =
        //     get_associated_token_address(&palyer.pubkey(), &mint.pubkey());
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
            &[&palyer],
            recent_blockhash,
        );
        let _tx_hash = client
            .send_and_confirm_transaction(&transaction)
            .await
            .unwrap();

        let token = client.get_account(&associated_token_account).await.unwrap();
        let token_data = StateWithExtensions::<Account2022>::unpack(&token.data).unwrap();

        println!("\nATA Account: {:?}", token);
        println!("ATA Account Data:{:#?}", token_data);
    }
}
