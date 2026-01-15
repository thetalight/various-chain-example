#[cfg(test)]
mod test {
    use solana_program::pubkey;
    use solana_sdk::pubkey::Pubkey;

    #[test]
    fn test_pda() {
        let program_address = pubkey!("11111111111111111111111111111111");
        let seeds = [b"helloWorld".as_ref()];
        let (pda, bump) = Pubkey::find_program_address(&seeds, &program_address);
        println!("PDA: {}", pda);
        println!("Bump: {}", bump);
    }
}
