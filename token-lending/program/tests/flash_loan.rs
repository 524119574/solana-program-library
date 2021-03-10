#![cfg(feature = "test-bpf")]

mod helpers;

use helpers::*;
use solana_program_test::*;
use solana_sdk::{pubkey::Pubkey, signature::Keypair};
use spl_token_lending::{
    instruction::BorrowAmountType, math::Decimal, processor::process_instruction,
    state::INITIAL_COLLATERAL_RATIO,
};
use solana_sdk::transaction::Transaction;
use spl_token_lending::instruction::{flash_loan_start, flash_loan_end};
use spl_token::instruction::transfer;
use solana_sdk::signature::Signer;

#[tokio::test]
async fn test_flash_loan() {

    let mut test = ProgramTest::new(
        "spl_token_lending",
        spl_token_lending::id(),
        processor!(process_instruction),
    );

    // limit to track compute unit increase
    test.set_bpf_compute_max_units(118_000);

    let user_accounts_owner = Keypair::new();
    let sol_usdc_dex_market = TestDexMarket::setup(&mut test, TestDexMarketPair::SOL_USDC);
    let usdc_mint = add_usdc_mint(&mut test);
    let lending_market = add_lending_market(&mut test, usdc_mint.pubkey);

    let mut reserve_config = TEST_RESERVE_CONFIG;
    reserve_config.loan_to_value_ratio = 80;

    let usdc_reserve = add_reserve(
        &mut test,
        &user_accounts_owner,
        &lending_market,
        AddReserveArgs {
            liquidity_amount: INITIAL_USDC_RESERVE_SUPPLY_FRACTIONAL,
            liquidity_mint_pubkey: usdc_mint.pubkey,
            liquidity_mint_decimals: usdc_mint.decimals,
            config: reserve_config,
            ..AddReserveArgs::default()
        },
    );

    let (mut banks_client, payer, _recent_blockhash) = test.start().await;

    let borrow_amount =
        get_token_balance(&mut banks_client, usdc_reserve.user_liquidity_account).await;
    assert_eq!(borrow_amount, 0);

    let mut transaction = Transaction::new_with_payer(
        &[
            flash_loan_start(
                &spl_token_lending::id(),
                25u64,
                2u8,
                &usdc_reserve.user_liquidity_account,
                &usdc_reserve.pubkey,
                &usdc_reserve.liquidity_supply,
                &lending_market.pubkey,
                &spl_token::id()
            ),
            transfer(
                &spl_token::id(),
                &usdc_reserve.liquidity_supply,
                &usdc_reserve.user_liquidity_account,
                &user_accounts_owner.pubkey(),
                &[],
                25u64
            ).unwrap(),
            flash_loan_end(
                &spl_token_lending::id(),
                &usdc_reserve.pubkey,
                &usdc_reserve.liquidity_supply,
                &lending_market.pubkey,
            )
        ],
        Some(&payer.pubkey()),
    );

    transaction.sign(
        &[&payer, &user_accounts_owner, &user_transfer_authority],
        recent_blockhash,
    );
    assert!(banks_client.process_transaction(transaction).await.is_ok());
}
