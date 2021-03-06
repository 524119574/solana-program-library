use solana_client::rpc_client::RpcClient;
use solana_program::program_pack::Pack;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signer},
    system_instruction::create_account,
    transaction::Transaction,
};
use spl_token::{
    instruction::approve,
    state::{Account as Token, Mint},
};
use spl_token_lending::{
    instruction::{init_lending_market, init_reserve},
    state::{LendingMarket, Reserve, ReserveConfig, ReserveFees},
};
use std::str::FromStr;
use spl_token::instruction::{initialize_mint, initialize_account, mint_to};
use spl_token::state::Account;
use spl_token_lending::instruction::{deposit_reserve_liquidity, refresh_reserve};
use core::time;
use std::thread;
use spl_token_lending::processor::unpack_mint;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::commitment_config::CommitmentLevel::{Max, Finalized};
use solana_sdk::loader_instruction::LoaderInstruction::Finalize;

// -------- UPDATE START -------
const KEYPAIR_PATH: &str = "/Users/wangge/.config/solana";
const LOCAL_NET_URL: &str = "http://127.0.0.1:8899";
const DEV_NET_URL: &str = "https://devnet.solana.com";
const TEST_NET_URL: &str = "https://testnet.solana.com";
const TEST_NET_PROGRAM: &str = "7cFfVGp6mAtBFsjp5GtjHZjLJEDdfEnG2QfuVUUeyGrY";
const DEV_NET_PROGRAM: &str = "3VQHdbsvViEnswYSQeQNL4innSx2jeJascCnUWdxAYyA";
const LOCAL_NET_PROGRAM: &str = "6isVZdDrR7dFpCjNJWvcBCGbUH3t4YdZSWxohRN5nRPE";
// solana_program::declare_id!("8c3365TtDi9LdzNBTD5Dvj3f45NWEf18nJVDD9JmTPG5");
solana_program::declare_id!("FXwW528o7nG6aRCfmcj39jnuHLVg24aa5mVkdnSVG8yD");
// -------- UPDATE END ---------

pub fn main() {
    let mut client = RpcClient::new(TEST_NET_URL.to_owned());

    let payer = read_keypair_file(&format!("{}/id.json", KEYPAIR_PATH)).unwrap();
    init_lending_market_and_reserves(&mut client, &payer);
}

fn supply_fund_to_reserve(
    client: &mut RpcClient,
    lending_market_pubkey: Pubkey,
    liquidity_source_pubkey: Pubkey,
    destination_collateral_pubkey: Pubkey,
    reserve_pubkey: Pubkey,
    liquidity_supply_pubkey: Pubkey,
    collateral_mint_pubkey: Pubkey,
    payer: &Keypair,
    oracle_pubkey: Option<Pubkey>,
) {

    let recent_blockhash = client.get_recent_blockhash().unwrap().0;

    let user_transfer_authority_keypair = Keypair::new();

    let mut transaction = Transaction::new_with_payer(
        &[
            approve(
                &spl_token::id(),
                &liquidity_source_pubkey,
                &user_transfer_authority_keypair.pubkey(),
                &payer.pubkey(),
                &[],
                1000u64,
            )
                .unwrap(),
            refresh_reserve(
                id(),
                reserve_pubkey,
                oracle_pubkey,
            ),
            deposit_reserve_liquidity(
                id(),
                1000u64,
                liquidity_source_pubkey,
                destination_collateral_pubkey,
                reserve_pubkey,
                liquidity_supply_pubkey,
                collateral_mint_pubkey,
                lending_market_pubkey,
                user_transfer_authority_keypair.pubkey(),
            ),
        ],
        Some(&payer.pubkey()),
    );

    transaction.sign(
        &vec![payer, &user_transfer_authority_keypair],
        recent_blockhash,
    );

    client.send_and_confirm_transaction_with_spinner_and_config(
        &transaction,
        CommitmentConfig {
            commitment: Finalized
        },
        RpcSendTransactionConfig {
            skip_preflight: true,
            preflight_commitment: None,
            encoding: None,
        }
    ).unwrap();

}

fn init_lending_market_and_reserves(mut client: &mut RpcClient, payer: &Keypair) {

    let srm_oracle_pubkey = Pubkey::from_str("BsMF3TfH9wiBMA9wwB1uKUffqcQW34GFdeczqT5UMDcc").unwrap();
    let sol_oracle_pubkey = Pubkey::from_str("B28fYyBaDMLkYfZWNUok6cnFceLzPEpdTXdVngep1KuA").unwrap();

    let (fake_usdc_mint_pubkey, fake_usdc_token_account_pubkey) = create_and_mint_tokens(
        &mut client,
        6,
        &payer,
    );

    println!("Created fake USDC mint {}, token account: {}", fake_usdc_mint_pubkey, fake_usdc_token_account_pubkey);

    let (lending_market_owner, lending_market_pubkey, _lending_market) =
        create_lending_market(&mut client, fake_usdc_mint_pubkey, &payer);
    println!("Created lending market: {} ", lending_market_pubkey);

    let usdc_reserve_config = ReserveConfig {
        optimal_utilization_rate: 80,
        loan_to_value_ratio: 75,
        liquidation_bonus: 5,
        liquidation_threshold: 80,
        min_borrow_rate: 0,
        optimal_borrow_rate: 4,
        max_borrow_rate: 30,
        fees: ReserveFees {
            borrow_fee_wad: 100_000_000_000_000, // 1 bp
            flash_loan_fee_wad: 0,
            host_fee_percentage: 20,
        },
    };

    let (usdc_reserve_pubkey, _usdc_reserve) = create_reserve(
        &mut client,
        usdc_reserve_config,
        lending_market_pubkey,
        &lending_market_owner,
        None,
        fake_usdc_token_account_pubkey,
        fake_usdc_mint_pubkey,
        &payer,
    );

    println!("Created usdc reserve with pubkey: {}", usdc_reserve_pubkey);

    let (fake_sol_mint_pubkey, fake_sol_token_account_pubkey) = create_and_mint_tokens(
        &mut client,
        9,
        &payer,
    );

    println!("Created fake SOL mint {} , token account: {} ", fake_sol_mint_pubkey, fake_sol_token_account_pubkey);

    let sol_reserve_config = ReserveConfig {
        optimal_utilization_rate: 0,
        loan_to_value_ratio: 75,
        liquidation_bonus: 10,
        liquidation_threshold: 80,
        min_borrow_rate: 0,
        optimal_borrow_rate: 2,
        max_borrow_rate: 15,
        fees: ReserveFees {
            borrow_fee_wad: 1_000_000_000_000, // 0.01 bp
            flash_loan_fee_wad: 0,
            host_fee_percentage: 20,
        },
    };

    let (sol_reserve_pubkey, _sol_reserve) = create_reserve(
        &mut client,
        sol_reserve_config,
        lending_market_pubkey,
        &lending_market_owner,
        Some(sol_oracle_pubkey),
        fake_sol_token_account_pubkey,
        fake_usdc_mint_pubkey,
        &payer,
    );

    println!("Created sol reserve with pubkey: {}", sol_reserve_pubkey);

    let (fake_srm_mint_pubkey, fake_srm_token_account_pubkey) = create_and_mint_tokens(
        &mut client,
        6,
        &payer,
    );

    println!("Created fake SRM mint {}, token account: {}", fake_srm_mint_pubkey, fake_srm_token_account_pubkey);

    let srm_reserve_config = ReserveConfig {
        optimal_utilization_rate: 0,
        loan_to_value_ratio: 75,
        liquidation_bonus: 10,
        liquidation_threshold: 80,
        min_borrow_rate: 0,
        optimal_borrow_rate: 2,
        max_borrow_rate: 15,
        fees: ReserveFees {
            borrow_fee_wad: 10_000_000_000_000, // 0.1 bp
            flash_loan_fee_wad: 0,
            host_fee_percentage: 25,
        },
    };

    let (srm_reserve_pubkey, _srm_reserve) = create_reserve(
        &mut client,
        srm_reserve_config,
        lending_market_pubkey,
        &lending_market_owner,
        Some(srm_oracle_pubkey),
        fake_srm_token_account_pubkey,
        fake_usdc_mint_pubkey,
        &payer,
    );

    println!("Created srm reserve with pubkey: {}", srm_reserve_pubkey);
}

pub fn create_lending_market(
    client: &mut RpcClient,
    quote_token_mint: Pubkey,
    payer: &Keypair,
) -> (Keypair, Pubkey, LendingMarket) {
    let owner = read_keypair_file(&format!("{}/id.json", KEYPAIR_PATH)).unwrap();
    let keypair = Keypair::new();
    let pubkey = keypair.pubkey();

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
            init_lending_market(id(), pubkey, owner.pubkey(), quote_token_mint),
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
    liquidity_oracle_pubkey: Option<Pubkey>,
    liquidity_source_pubkey: Pubkey,
    quote_token_mint_pubkey: Pubkey,
    payer: &Keypair,
) -> (Pubkey, Reserve) {
    let reserve_keypair = Keypair::new();
    let reserve_pubkey = reserve_keypair.pubkey();
    let collateral_mint_keypair = Keypair::new();
    let collateral_supply_keypair = Keypair::new();
    let liquidity_supply_keypair = Keypair::new();
    let liquidity_fee_receiver_keypair = Keypair::new();
    let user_collateral_token_keypair = Keypair::new();
    let user_transfer_authority_keypair = Keypair::new();

    let liquidity_source_account = client.get_account(&liquidity_source_pubkey).unwrap();
    let liquidity_source_token = Token::unpack(&liquidity_source_account.data).unwrap();
    let liquidity_mint_pubkey = liquidity_source_token.mint;

    let recent_blockhash = client.get_recent_blockhash().unwrap().0;
    let token_balance = client
        .get_minimum_balance_for_rent_exemption(Token::LEN)
        .unwrap();

    let mut transaction = Transaction::new_with_payer(
        &[
            create_account(
                &payer.pubkey(),
                &collateral_mint_keypair.pubkey(),
                client
                    .get_minimum_balance_for_rent_exemption(Mint::LEN)
                    .unwrap(),
                Mint::LEN as u64,
                &spl_token::id(),
            ),
            create_account(
                &payer.pubkey(),
                &collateral_supply_keypair.pubkey(),
                token_balance,
                Token::LEN as u64,
                &spl_token::id(),
            ),
            create_account(
                &payer.pubkey(),
                &liquidity_supply_keypair.pubkey(),
                token_balance,
                Token::LEN as u64,
                &spl_token::id(),
            ),
            create_account(
                &payer.pubkey(),
                &liquidity_fee_receiver_keypair.pubkey(),
                token_balance,
                Token::LEN as u64,
                &spl_token::id(),
            ),
            create_account(
                &payer.pubkey(),
                &user_collateral_token_keypair.pubkey(),
                token_balance,
                Token::LEN as u64,
                &spl_token::id(),
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
            &liquidity_fee_receiver_keypair,
            &liquidity_supply_keypair,
            &user_collateral_token_keypair,
        ],
        recent_blockhash,
    );


    client.send_and_confirm_transaction_with_spinner_and_config(
        &transaction,
        CommitmentConfig {
            commitment: Finalized
        },
        RpcSendTransactionConfig {
            skip_preflight: true,
            preflight_commitment: None,
            encoding: None,
        }
    ).unwrap();


    let mut transaction = Transaction::new_with_payer(
        &[
            approve(
                &spl_token::id(),
                &liquidity_source_pubkey,
                &user_transfer_authority_keypair.pubkey(),
                &payer.pubkey(),
                &[],
                10_000_000u64,
            )
                .unwrap(),
            init_reserve(
                id(),
                10_000_000u64,
                config,
                liquidity_source_pubkey,
                user_collateral_token_keypair.pubkey(),
                reserve_pubkey,
                liquidity_mint_pubkey,
                liquidity_supply_keypair.pubkey(),
                liquidity_fee_receiver_keypair.pubkey(),
                collateral_mint_keypair.pubkey(),
                collateral_supply_keypair.pubkey(),
                quote_token_mint_pubkey,
                lending_market_pubkey,
                lending_market_owner.pubkey(),
                user_transfer_authority_keypair.pubkey(),
                liquidity_oracle_pubkey,
            ),
        ],
        Some(&payer.pubkey()),
    );

    transaction.sign(
        &vec![payer, &lending_market_owner, &user_transfer_authority_keypair],
        recent_blockhash,
    );

    client.send_and_confirm_transaction_with_spinner_and_config(
        &transaction,
        CommitmentConfig {
            commitment: Finalized
        },
        RpcSendTransactionConfig {
            skip_preflight: true,
            preflight_commitment: None,
            encoding: None,
        }
    ).unwrap();

    supply_fund_to_reserve(
        client,
        lending_market_pubkey,
        liquidity_source_pubkey,
        user_collateral_token_keypair.pubkey(),
        reserve_pubkey,
        liquidity_supply_keypair.pubkey(),
        collateral_mint_keypair.pubkey(),
        payer,
        liquidity_oracle_pubkey,
    );

    let account = client.get_account(&reserve_pubkey).unwrap();
    (reserve_pubkey, Reserve::unpack(&account.data).unwrap())
}

pub fn create_and_mint_tokens(
    client: &mut RpcClient,
    decimals: u8,
    payer: &Keypair
) -> (Pubkey, Pubkey) {
    let mint_keypair = Keypair::new();
    let mint_pubkey = mint_keypair.pubkey();
    let token_account_keypair = Keypair::new();
    let token_account_pubkey = token_account_keypair.pubkey();
    let token_account_owner_keypair = Keypair::new();

    let recent_blockhash = client.get_recent_blockhash().unwrap().0;
    let mut transaction = Transaction::new_with_payer(
        &[
            create_account(
                &payer.pubkey(),
                &mint_pubkey,
                client
                    .get_minimum_balance_for_rent_exemption(Mint::LEN)
                    .unwrap(),
                Mint::LEN as u64,
                &spl_token::id(),
            ),
            create_account(
                &payer.pubkey(),
                &token_account_pubkey,
                client
                    .get_minimum_balance_for_rent_exemption(Token::LEN)
                    .unwrap(),
                Token::LEN as u64,
                &spl_token::id(),
            ),
            initialize_mint(
                &spl_token::id(),
                &mint_pubkey,
                &payer.pubkey(),
                None,
                decimals
            ).unwrap(),
            initialize_account(
                &spl_token::id(),
                &token_account_pubkey,
                &mint_pubkey,
                &payer.pubkey()
            ).unwrap(),
            mint_to(
                &spl_token::id(),
                &mint_pubkey,
                &token_account_pubkey,
                &payer.pubkey(),
                &[],
                100_000_000_000_000u64,
            ).unwrap(),
        ],
        Some(&payer.pubkey()),
    );

    transaction.sign(
        &vec![
            payer,
            &mint_keypair,
            &token_account_keypair,
        ],
        recent_blockhash,
    );

    client.send_and_confirm_transaction(&transaction).unwrap();
    (mint_pubkey, token_account_pubkey)
}