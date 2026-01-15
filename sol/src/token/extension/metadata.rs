#[cfg(test)]
mod tests {
    use solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::CommitmentConfig};
    use solana_sdk::program_pack::Pack;
    use solana_sdk::transaction::Transaction;
    use solana_sdk::{signature::Keypair, signer::Signer};
    use solana_system_interface::instruction::create_account;
    use spl_token_2022_interface::{
        ID as token_2022_program_id,
        extension::{
            BaseStateWithExtensions, ExtensionType, StateWithExtensions,
            metadata_pointer::{
                MetadataPointer, instruction::initialize as initialize_metadata_pointer,
            },
        },
        instruction::{initialize_mint2, reallocate},
        state::Mint,
    };
    use spl_token_metadata_interface::{
        instruction::{initialize as initialize_token_metadata, update_field},
        state::{Field, TokenMetadata},
    };

    use crate::account::airdrop::request_airdrop;

    #[tokio::test]
    async fn test_metadata() {
        let rpc_url = "http://127.0.0.1:8899".to_string();
        //let rpc_url = " https://api.testnet.solana.com".to_string();
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

        // 2. MetaData
        let token_metadata  = TokenMetadata {
            update_authority: Some(palyer.pubkey()).try_into().unwrap(),
            mint: mint.pubkey(),
            name: "OPOS".to_string(),
            symbol : "OPS".to_string(),
            uri : "https://raw.githubusercontent.com/solana-developers/opos-asset/main/assets/DeveloperPortal/metadata.json".to_string(),
            additional_metadata: vec![("description".to_string(),"only possible on Solana".to_string())]
        };

        let mint_space =
            ExtensionType::try_calculate_account_len::<Mint>(&[ExtensionType::MetadataPointer])
                .unwrap();
        let metadata_len = token_metadata.tlv_size_of().unwrap();

        let total_len = mint_space + metadata_len;

        println!(
            "mint_space: {}\nmetadata_len: {}\n",
            mint_space, metadata_len
        );

        // 3. 初始化Mint account
        let rent_exemption = client
            .get_minimum_balance_for_rent_exemption(total_len)
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

        let initialize_metadata_pointer_instruction = initialize_metadata_pointer(
            &token_2022_program_id,
            &mint.pubkey(),
            Some(palyer.pubkey()),
            Some(mint.pubkey()),
        )
        .unwrap();
        instructions.push(initialize_metadata_pointer_instruction);

        let i_initialize = initialize_mint2(
            &token_2022_program_id,
            &mint.pubkey(),
            &palyer.pubkey(),
            Some(&palyer.pubkey()),
            6,
        )
        .unwrap();
        instructions.push(i_initialize);

        // let i_realocate  = reallocate(
        //     &token_2022_program_id,
        //     &mint.pubkey(),
        //     &palyer.pubkey(),
        //     &palyer.pubkey(),
        //     &[&palyer.pubkey()],
        //     &[ExtensionType::TokenMetadata]
        // ).unwrap();
        // instructions.push(i_realocate);

        let initialize_metadata_instruction = initialize_token_metadata(
            &token_2022_program_id,
            &mint.pubkey(),
            &palyer.pubkey(),
            &mint.pubkey(),
            &palyer.pubkey(),
            token_metadata.name.to_string(),   // name
            token_metadata.symbol.to_string(), // symbol
            token_metadata.uri.to_string(),    // uri
        );
        instructions.push(initialize_metadata_instruction);

        let update_field_instructions: Vec<_> = token_metadata
            .additional_metadata
            .iter()
            .map(|(key, value)| {
                update_field(
                    &token_2022_program_id,
                    &mint.pubkey(),
                    &palyer.pubkey(),
                    Field::Key(key.clone()),
                    value.clone(),
                )
            })
            .collect();
        instructions.extend(update_field_instructions);

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
        println!("\nExtensions enabled: {:.?}", extension_types);

        let metadata_pointer = mint_state.get_extension::<MetadataPointer>().unwrap();
        println!("\nmetadata_pointer:{:#.?}", metadata_pointer);

        // let token_metadata = mint_state
        //     .get_variable_len_extension::<TokenMetadata>()
        //     .unwrap();
        // println!("\ntoken_metadata:{:#.?}", token_metadata);
    }
}
