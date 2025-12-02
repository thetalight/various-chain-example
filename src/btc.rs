use secp256k1::{ecdsa::Signature, Message, Secp256k1};

#[test]
fn test_btc() {
    const PRIVATE_KEY_BYTES: [u8; 32] = [
        216, 166, 206, 234, 67, 115, 17, 206, 67, 244, 2, 74, 142, 138, 59, 3, 118, 156, 69, 148,
        111, 104, 216, 47, 49, 253, 0, 104, 186, 79, 60, 224,
    ];
    let secp = Secp256k1::new();

    let sk = secp256k1::SecretKey::from_slice(&PRIVATE_KEY_BYTES).unwrap();
    let pk = sk.public_key(&secp);

    let msg = Message::from_digest([1u8; 32]);
    let sig = secp.sign_ecdsa(&msg, &sk);
    secp.verify_ecdsa(&msg, &sig, &pk).unwrap();

    // 签名长度 64 字节
    assert_eq!(
        "74f51a3690a3f50ed813a9c68be84312abc50e5b97c6b07cccc7df19c8000c232a294312e635ce893b25e8fc56d7c5593a2bbc30b9f1517b8965a91530588fa4",
        hex::encode(&sig.serialize_compact())
    );
}



#[test]
fn test_btc1() {
    let bytes1 = hex::decode(b"57979955d10883aaa7a0ccd4347211aac4044fdb441f17e767578e862945c17b").unwrap();
    let secp = Secp256k1::new();

    let sk = secp256k1::SecretKey::from_slice(&bytes1).unwrap();
    let pk = sk.public_key(&secp);
    println!("pk: {:?}",pk.serialize_uncompressed().to_vec());

    let data:[u8;32] = hex::decode(b"71d027c296147783637ed2c26544bafef53b4cef1ab8250830175b149f38c5e5").unwrap().try_into().unwrap();
    let msg = Message::from_digest(data);
    let sig = secp.sign_ecdsa(&msg, &sk);
    secp.verify_ecdsa(&msg, &sig, &pk).unwrap();

    let sig =  Signature::from_compact(&hex::decode(b"e47b07120b3251540e3277cb41faca97a8729b96c90c0a1fb6c7a36b22427d5a50791570d368278b65dd0a7f56beab4e39034d05b6383d79b155581fd313df0a").unwrap()).unwrap();
    secp.verify_ecdsa(&msg, &sig, &pk).unwrap();

    // 8f7bc66c5543b62532f8e06abf6e1c5de248a1cec00523da2308abd66f0f1cf25c85d0c7eb0895cf5c618a9da370ba3c2688f775b90e50b91615a730c409f274
    // e47b07120b3251540e3277cb41faca97a8729b96c90c0a1fb6c7a36b22427d5a50791570d368278b65dd0a7f56beab4e39034d05b6383d79b155581fd313df0a
    // 
    // 签名长度 64 字节
    // assert_eq!(
    //     "74f51a3690a3f50ed813a9c68be84312abc50e5b97c6b07cccc7df19c8000c232a294312e635ce893b25e8fc56d7c5593a2bbc30b9f1517b8965a91530588fa4",
    //     hex::encode(&sig.serialize_compact())
    // );
}