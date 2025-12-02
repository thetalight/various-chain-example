use alloy::{
    consensus::{EthereumTxEnvelope, SignableTransaction, TxEip4844Variant}, network::{Ethereum, EthereumWallet, NetworkWallet, TransactionBuilder}, node_bindings::Anvil, primitives::{FixedBytes, U256}, providers::{Provider, ProviderBuilder}, rpc::types::TransactionRequest, signers::{local::PrivateKeySigner, Signature, SignerSync}
};


// cargo test --package various-chain-sign --lib -- eth::test_sign --exact --show-output 
#[test]
fn test_sign() {
    const PRIVATE_KEY_BYTES: [u8; 32] = [
        216, 166, 206, 234, 67, 115, 17, 206, 67, 244, 2, 74, 142, 138, 59, 3, 118, 156, 69, 148,
        111, 104, 216, 47, 49, 253, 0, 104, 186, 79, 60, 224,
    ];
    let signer = PrivateKeySigner::from_slice(&PRIVATE_KEY_BYTES).unwrap();

    let msg_hash = FixedBytes::from_slice(&[1u8; 32]);
    let s = signer.sign_hash_sync(&msg_hash).unwrap();

    let r = s.recover_address_from_prehash(&msg_hash).unwrap();
    assert_eq!(r, signer.address());
    
    assert_eq!(
        "74f51a3690a3f50ed813a9c68be84312abc50e5b97c6b07cccc7df19c8000c232a294312e635ce893b25e8fc56d7c5593a2bbc30b9f1517b8965a91530588fa41b",
        hex::encode(s.as_bytes())
    );
}

#[tokio::test]
async fn test_eth() {
    let anvil = Anvil::new().block_time(1).try_spawn().unwrap();

    let signer_alice: PrivateKeySigner = anvil.keys()[0].clone().into();
    let signer_bob: PrivateKeySigner = anvil.keys()[1].clone().into();

    let alice = signer_alice.address();
    let bob = signer_bob.address();

    let rpc_url = anvil.endpoint_url();
    let provider = ProviderBuilder::new()
        .wallet(signer_alice.clone())
        .connect_http(rpc_url.clone());


    let tx = TransactionRequest::default()
        .with_to(bob)
        .with_nonce(0)
        .with_chain_id(provider.get_chain_id().await.unwrap())
        .with_value(U256::from(100))
        .with_gas_limit(21_000)
        .with_max_priority_fee_per_gas(1_000_000_000)
        .with_max_fee_per_gas(20_000_000_000);

    let tx_envelope = {
        let tx_unsigned = tx.clone().build_unsigned().unwrap();
        // tx_unsigned.set_chain_id_checked(1);

        let signature_hash = tx_unsigned.signature_hash();
        let signature = signer_alice.sign_hash_sync(&signature_hash).unwrap();
        let tx_envelope_0: EthereumTxEnvelope<TxEip4844Variant> = tx_unsigned.clone().into_signed(signature).into();


        let wallet = EthereumWallet::from(signer_alice.clone());
        let tx_envelope_1 = <EthereumWallet as NetworkWallet<Ethereum>>::sign_transaction_from(
            &wallet,
            alice, // 地址只是用来索引
            tx_unsigned,
        )
        .await.unwrap();

        assert_eq!(tx_envelope_0, tx_envelope_1);

        tx_envelope_0
    };


}


#[test]
fn test_sign1() {
    // println!("{}","e47b07120b3251540e3277cb41faca97a8729b96c90c0a1fb6c7a36b22427d5a50791570d368278b65dd0a7f56beab4e39034d05b6383d79b155581fd313df0a".len());
    let bytes1 = hex::decode(b"57979955d10883aaa7a0ccd4347211aac4044fdb441f17e767578e862945c17b").unwrap();
    let signer = PrivateKeySigner::from_slice(&bytes1).unwrap();
    assert_eq!(signer.address().to_string(), "0xa652886Cbd45B63C2F3382066C7CB378E66D280b");
   // println!("pk:{:?}", signer.public_key().to_string().split_at(2).1);
    println!("pk:{:?}",hex::decode(signer.public_key().to_string().split_at(2).1).unwrap()); // [2, 235, 151, 45, 68, 211, 89, 224, 191, 74, 14, 76, 39, 177, 219, 72, 219, 86, 191, 185, 16, 197, 146, 16, 228, 135, 198, 194, 172, 39, 201, 6, 147]

    let data = hex::decode(b"71d027c296147783637ed2c26544bafef53b4cef1ab8250830175b149f38c5e5").unwrap();
    let msg_hash = FixedBytes::from_slice(&data);

    let s = signer.sign_hash_sync(&msg_hash).unwrap();
    println!("signature:{:?}", s.as_bytes());

    let sb = hex::decode(b"e47b07120b3251540e3277cb41faca97a8729b96c90c0a1fb6c7a36b22427d5a50791570d368278b65dd0a7f56beab4e39034d05b6383d79b155581fd313df0a1b").unwrap();
    let s0 : Signature = Signature::try_from(sb.as_slice()).unwrap();

    let r = s0.recover_address_from_prehash(&msg_hash).unwrap();
    assert_eq!(r, signer.address());
}