use solana_client::rpc_client::RpcClient;
use solana_program::program_pack::Pack;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signer},
    system_instruction::create_account,
    transaction::Transaction,
};
use spl_token::{
    instruction::{approve, initialize_mint, initialize_account, mint_to},
    state::{Account as Token, Mint},
};
use spl_token_lending::{
    instruction::{init_lending_market, init_reserve},
    state::{LendingMarket, Reserve, ReserveConfig, ReserveFees},
};
use spl_token_lending::instruction::{flash_loan_start, flash_loan_end};
use spl_token::instruction::transfer;
use std::str::FromStr;

// -------- UPDATE START -------
const KEYPAIR_PATH: &str = "/Users/wangge/.config/solana";
solana_program::declare_id!("5vgQ3Usm8xCLMenfhA8o1dhtiCgPPv1rF7uX7XytBBnH");
// -------- UPDATE END ---------

pub struct DexMarket {
    pub name: &'static str,
    pub pubkey: Pubkey,
}

pub fn main() {
    let mut client = RpcClient::new("http://127.0.0.1:8899".to_owned());

    let token_pubkey: Pubkey = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();

    let sol_usdc_dex_market = DexMarket {
        name: "sol_usdc",
        pubkey: Pubkey::from_str("9wFFyRfZBsuAha4YcuxcXLKwMxJR43S7fPfQLusDBzvT").unwrap(),
    };

    let srm_usdc_dex_market = DexMarket {
        name: "srm_usdc",
        pubkey: Pubkey::from_str("ByRys5tuUWDgL73G8JBAEfkdFf8JWBzPBDHsBVQ5vbQA").unwrap(),
    };

    let quote_token_mint = mint_account.pubkey();
    let (lending_market_owner, lending_market_pubkey, _lending_market) =
        create_lending_market(&mut client, quote_token_mint, &payer);
    println!("created lending market: {}", lending_market_pubkey);
    let token_liquidity_source = token_account.pubkey();
    let token_reserve_config = ReserveConfig {
        optimal_utilization_rate: 80,
        loan_to_value_ratio: 75,
        liquidation_bonus: 5,
        liquidation_threshold: 80,
        min_borrow_rate: 0,
        optimal_borrow_rate: 4,
        max_borrow_rate: 30,
        fees: ReserveFees {
            borrow_fee_wad: 100_000_000_000_000, // 1 bp
            host_fee_percentage: 20,
        },
    };

    let (usdc_reserve_pubkey, liquidity_supply, _usdc_reserve) = create_reserve(
        &mut client,
        token_reserve_config,
        lending_market_pubkey,
        &lending_market_owner,
        None,
        token_liquidity_source,
        &payer,
    );

    println!("Created token reserve with pubkey: {}", usdc_reserve_pubkey);
    println!("Liquidity supply pubkey is {}", liquidity_supply);

    let balance = client.get_token_account_balance(&token_account.pubkey()).unwrap();
    println!("Token account balance is: {}", balance.amount);

    println!("Starting flash loan...");
    let mut transaction = Transaction::new_with_payer(
        &[
            flash_loan_start(
                id(), 25u64,
                2u8,
                token_account_pubkey,
                usdc_reserve_pubkey,
                liquidity_supply,
                lending_market_pubkey,
                token_pubkey
            ),
            transfer(
                &token_pubkey,
                &token_account_pubkey,
                &liquidity_supply,
                &payer.pubkey(),
                &[],
                25u64
            ).unwrap(),
            flash_loan_end(
                id(),
                usdc_reserve_pubkey,
                liquidity_supply,
                lending_market_pubkey,
            )
        ],
        Some(&payer.pubkey()),
    );
    let recent_blockhash = client.get_recent_blockhash().unwrap().0;
    transaction.sign(&[&payer], recent_blockhash);
    client.send_and_confirm_transaction(&transaction).unwrap();
    
    println!("flash loan done!");
}

pub fn create_lending_market(
    client: &mut RpcClient,
    quote_token_mint: Pubkey,
    payer: &Keypair,
) -> (Keypair, Pubkey, LendingMarket) {
    let owner = read_keypair_file(&format!("{}/id.json", KEYPAIR_PATH)).unwrap();
    let keypair = Keypair::new();
    let pubkey = keypair.pubkey();
    let token_pubkey: Pubkey = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();


    let mut transaction = Transaction::new_with_payer(
        &[
            create_account(
                &payer.pubkey(),
                &pubkey,
                client
                    .get_minimum_balance_for_rent_exemption(LendingMarket::LEN)
                    .unwrap(),
                LendingMarket::LEN as u64,
                &id(),
            ),
            init_lending_market(
                id(), pubkey,
                owner.pubkey(), quote_token_mint, token_pubkey),
        ],
        Some(&payer.pubkey()),
    );

    let recent_blockhash = client.get_recent_blockhash().unwrap().0;
    transaction.sign(&[&payer, &keypair], recent_blockhash);
    client.send_and_confirm_transaction(&transaction).unwrap();

    let account = client.get_account(&pubkey).unwrap();
    let lending_market = LendingMarket::unpack(&account.data).unwrap();

    (owner, pubkey, lending_market)
}

pub fn create_reserve(
    client: &mut RpcClient,
    config: ReserveConfig,
    lending_market_pubkey: Pubkey,
    lending_market_owner: &Keypair,
    dex_market_pubkey: Option<Pubkey>,
    liquidity_source_pubkey: Pubkey,
    payer: &Keypair,
) -> (Pubkey, Pubkey,Reserve) {
    let reserve_keypair = Keypair::new();
    let reserve_pubkey = reserve_keypair.pubkey();
    let collateral_mint_keypair = Keypair::new();
    let collateral_supply_keypair = Keypair::new();
    let collateral_fees_receiver_keypair = Keypair::new();
    let liquidity_supply_keypair = Keypair::new();
    let user_collateral_token_keypair = Keypair::new();
    let user_transfer_authority = Keypair::new();

    let liquidity_source_account = client.get_account(&liquidity_source_pubkey).unwrap();
    let liquidity_source_token = Token::unpack(&liquidity_source_account.data).unwrap();
    let liquidity_mint_pubkey = liquidity_source_token.mint;

    let recent_blockhash = client.get_recent_blockhash().unwrap().0;
    let token_balance = client
        .get_minimum_balance_for_rent_exemption(Token::LEN)
        .unwrap();
    let token_pubkey: Pubkey = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();
    let mut transaction = Transaction::new_with_payer(
        &[
            create_account(
                &payer.pubkey(),
                &collateral_mint_keypair.pubkey(),
                client
                    .get_minimum_balance_for_rent_exemption(Mint::LEN)
                    .unwrap(),
                Mint::LEN as u64,
                &token_pubkey,
            ),
            create_account(
                &payer.pubkey(),
                &collateral_supply_keypair.pubkey(),
                token_balance,
                Token::LEN as u64,
                &token_pubkey,
            ),
            create_account(
                &payer.pubkey(),
                &collateral_fees_receiver_keypair.pubkey(),
                token_balance,
                Token::LEN as u64,
                &token_pubkey,
            ),
            create_account(
                &payer.pubkey(),
                &liquidity_supply_keypair.pubkey(),
                token_balance,
                Token::LEN as u64,
                &token_pubkey,
            ),
            create_account(
                &payer.pubkey(),
                &user_collateral_token_keypair.pubkey(),
                token_balance,
                Token::LEN as u64,
                &token_pubkey,
            ),
            create_account(
                &payer.pubkey(),
                &reserve_pubkey,
                client
                    .get_minimum_balance_for_rent_exemption(Reserve::LEN)
                    .unwrap(),
                Reserve::LEN as u64,
                &id(),
            ),
        ],
        Some(&payer.pubkey()),
    );

    transaction.sign(
        &vec![
            payer,
            &reserve_keypair,
            &collateral_mint_keypair,
            &collateral_supply_keypair,
            &liquidity_supply_keypair,
            &user_collateral_token_keypair,
            &collateral_fees_receiver_keypair,
        ],
        recent_blockhash,
    );

    client.send_and_confirm_transaction(&transaction).unwrap();

    let mut transaction = Transaction::new_with_payer(
        &[
            approve(
                &token_pubkey,
                &liquidity_source_pubkey,
                &user_transfer_authority.pubkey(),
                &payer.pubkey(),
                &[],
                liquidity_source_token.amount,
            )
            .unwrap(),
            init_reserve(
                id(),
                liquidity_source_token.amount,
                config,
                liquidity_source_pubkey,
                user_collateral_token_keypair.pubkey(),
                reserve_pubkey,
                liquidity_mint_pubkey,
                liquidity_supply_keypair.pubkey(),
                collateral_mint_keypair.pubkey(),
                collateral_supply_keypair.pubkey(),
                collateral_fees_receiver_keypair.pubkey(),
                lending_market_pubkey,
                lending_market_owner.pubkey(),
                user_transfer_authority.pubkey(),
                dex_market_pubkey,
                token_pubkey,
            ),
        ],
        Some(&payer.pubkey()),
    );

    transaction.sign(
        &vec![payer, &lending_market_owner, &user_transfer_authority],
        recent_blockhash,
    );

    client.send_and_confirm_transaction(&transaction).unwrap();

    let account = client.get_account(&reserve_pubkey).unwrap();
    (reserve_pubkey, liquidity_supply_keypair.pubkey(), Reserve::unpack(&account.data).unwrap())
}
