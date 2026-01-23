use anchor_litesvm::{AnchorLiteSVM, Keypair, Pubkey, Signer};
use anchor_spl::associated_token::get_associated_token_address;
use litesvm_utils::{AssertionHelpers, TestHelpers};

// Declare the program to generate client types
anchor_lang::declare_program!(stablecoin);
use self::stablecoin::{client, ID as PROGRAM_ID};

// Program IDs
const TOKEN_PROGRAM_ID: Pubkey = anchor_spl::token::ID;
const ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey = anchor_spl::associated_token::ID;
const SYSTEM_PROGRAM_ID: Pubkey = anchor_lang::system_program::ID;

// Helper to create an initialized test context
fn setup_ctx() -> anchor_litesvm::AnchorContext {
    AnchorLiteSVM::build_with_program(
        PROGRAM_ID,
        include_bytes!("../../../target/deploy/stablecoin.so"),
    )
}

// Helper to get PDAs
fn get_config_pda() -> Pubkey {
    Pubkey::find_program_address(&[b"config"], &PROGRAM_ID).0
}

fn get_mint_pda() -> Pubkey {
    Pubkey::find_program_address(&[b"mint"], &PROGRAM_ID).0
}

fn get_minter_config_pda(minter: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"minter", minter.as_ref()], &PROGRAM_ID).0
}

fn get_ata(wallet: &Pubkey, mint: &Pubkey) -> Pubkey {
    get_associated_token_address(wallet, mint)
}

// ============================================================================
// Initialize Tests
// ============================================================================

#[test]
fn test_initialize() {
    let mut ctx = setup_ctx();

    let admin = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let config_pda = get_config_pda();
    let mint_pda = get_mint_pda();

    let ix = ctx
        .program()
        .accounts(client::accounts::Initialize {
            admin: admin.pubkey(),
            config: config_pda,
            mint: mint_pda,
            token_program: TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        })
        .args(client::args::Initialize {})
        .instruction()
        .unwrap();

    ctx.execute_instruction(ix, &[&admin])
        .unwrap()
        .assert_success();

    // Verify accounts were created
    assert!(ctx.account_exists(&config_pda), "Config account should exist");
    assert!(ctx.account_exists(&mint_pda), "Mint account should exist");
}

#[test]
fn test_initialize_twice_fails() {
    let mut ctx = setup_ctx();

    let admin = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let config_pda = get_config_pda();
    let mint_pda = get_mint_pda();

    let ix = ctx
        .program()
        .accounts(client::accounts::Initialize {
            admin: admin.pubkey(),
            config: config_pda,
            mint: mint_pda,
            token_program: TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        })
        .args(client::args::Initialize {})
        .instruction()
        .unwrap();

    // First initialize should succeed
    ctx.execute_instruction(ix.clone(), &[&admin])
        .unwrap()
        .assert_success();

    // Second initialize should fail
    let ix2 = ctx
        .program()
        .accounts(client::accounts::Initialize {
            admin: admin.pubkey(),
            config: config_pda,
            mint: mint_pda,
            token_program: TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        })
        .args(client::args::Initialize {})
        .instruction()
        .unwrap();

    let result = ctx.execute_instruction(ix2, &[&admin]);
    assert!(
        result.is_err() || !result.unwrap().is_success(),
        "Second initialize should fail"
    );
}

// ============================================================================
// Configure Minter Tests
// ============================================================================

fn initialize_program(ctx: &mut anchor_litesvm::AnchorContext, admin: &Keypair) {
    let config_pda = get_config_pda();
    let mint_pda = get_mint_pda();

    let ix = ctx
        .program()
        .accounts(client::accounts::Initialize {
            admin: admin.pubkey(),
            config: config_pda,
            mint: mint_pda,
            token_program: TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        })
        .args(client::args::Initialize {})
        .instruction()
        .unwrap();

    ctx.execute_instruction(ix, &[admin])
        .expect("Initialize should succeed")
        .assert_success();
}

#[test]
fn test_configure_minter() {
    let mut ctx = setup_ctx();

    let admin = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    initialize_program(&mut ctx, &admin);

    let config_pda = get_config_pda();
    let minter = Keypair::new();
    let minter_config_pda = get_minter_config_pda(&minter.pubkey());

    let allowance: u64 = 1_000_000_000;

    let ix = ctx
        .program()
        .accounts(client::accounts::ConfigureMinter {
            admin: admin.pubkey(),
            config: config_pda,
            minter: minter.pubkey(),
            minter_config: minter_config_pda,
            system_program: SYSTEM_PROGRAM_ID,
        })
        .args(client::args::ConfigureMinter { allowance })
        .instruction()
        .unwrap();

    ctx.execute_instruction(ix, &[&admin])
        .unwrap()
        .assert_success();

    // Verify minter config account was created
    assert!(
        ctx.account_exists(&minter_config_pda),
        "Minter config account should exist"
    );
}

#[test]
fn test_configure_minter_unauthorized() {
    let mut ctx = setup_ctx();

    let admin = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let unauthorized = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    initialize_program(&mut ctx, &admin);

    let config_pda = get_config_pda();
    let minter = Keypair::new();
    let minter_config_pda = get_minter_config_pda(&minter.pubkey());

    let allowance: u64 = 1_000_000_000;

    let ix = ctx
        .program()
        .accounts(client::accounts::ConfigureMinter {
            admin: unauthorized.pubkey(),
            config: config_pda,
            minter: minter.pubkey(),
            minter_config: minter_config_pda,
            system_program: SYSTEM_PROGRAM_ID,
        })
        .args(client::args::ConfigureMinter { allowance })
        .instruction()
        .unwrap();

    let result = ctx.execute_instruction(ix, &[&unauthorized]);
    assert!(
        result.is_err() || !result.unwrap().is_success(),
        "Unauthorized configure_minter should fail"
    );
}

#[test]
fn test_update_minter_allowance() {
    let mut ctx = setup_ctx();

    let admin = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    initialize_program(&mut ctx, &admin);

    let config_pda = get_config_pda();
    let minter = Keypair::new();
    let minter_config_pda = get_minter_config_pda(&minter.pubkey());

    // First configure with initial allowance
    let allowance1: u64 = 1_000_000_000;

    let ix1 = ctx
        .program()
        .accounts(client::accounts::ConfigureMinter {
            admin: admin.pubkey(),
            config: config_pda,
            minter: minter.pubkey(),
            minter_config: minter_config_pda,
            system_program: SYSTEM_PROGRAM_ID,
        })
        .args(client::args::ConfigureMinter {
            allowance: allowance1,
        })
        .instruction()
        .unwrap();

    ctx.execute_instruction(ix1, &[&admin])
        .expect("First configure should succeed")
        .assert_success();

    // Update with new allowance
    let allowance2: u64 = 2_000_000_000;

    let ix2 = ctx
        .program()
        .accounts(client::accounts::ConfigureMinter {
            admin: admin.pubkey(),
            config: config_pda,
            minter: minter.pubkey(),
            minter_config: minter_config_pda,
            system_program: SYSTEM_PROGRAM_ID,
        })
        .args(client::args::ConfigureMinter {
            allowance: allowance2,
        })
        .instruction()
        .unwrap();

    ctx.execute_instruction(ix2, &[&admin])
        .unwrap()
        .assert_success();
}

// ============================================================================
// Remove Minter Tests
// ============================================================================

fn configure_minter(
    ctx: &mut anchor_litesvm::AnchorContext,
    admin: &Keypair,
    minter: &Pubkey,
    allowance: u64,
) {
    let config_pda = get_config_pda();
    let minter_config_pda = get_minter_config_pda(minter);

    let ix = ctx
        .program()
        .accounts(client::accounts::ConfigureMinter {
            admin: admin.pubkey(),
            config: config_pda,
            minter: *minter,
            minter_config: minter_config_pda,
            system_program: SYSTEM_PROGRAM_ID,
        })
        .args(client::args::ConfigureMinter { allowance })
        .instruction()
        .unwrap();

    ctx.execute_instruction(ix, &[admin])
        .expect("Configure minter should succeed")
        .assert_success();
}

#[test]
fn test_remove_minter() {
    let mut ctx = setup_ctx();

    let admin = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    initialize_program(&mut ctx, &admin);

    let minter = Keypair::new();
    configure_minter(&mut ctx, &admin, &minter.pubkey(), 1_000_000_000);

    let config_pda = get_config_pda();
    let minter_config_pda = get_minter_config_pda(&minter.pubkey());

    // Verify minter config exists
    assert!(
        ctx.account_exists(&minter_config_pda),
        "Minter config should exist"
    );

    // Remove minter
    let ix = ctx
        .program()
        .accounts(client::accounts::RemoveMinter {
            admin: admin.pubkey(),
            config: config_pda,
            minter: minter.pubkey(),
            minter_config: minter_config_pda,
        })
        .args(client::args::RemoveMinter {})
        .instruction()
        .unwrap();

    ctx.execute_instruction(ix, &[&admin])
        .unwrap()
        .assert_success();

    // Verify minter config was closed
    ctx.svm.assert_account_closed(&minter_config_pda);
}

// ============================================================================
// Mint Tokens Tests
// ============================================================================

#[test]
fn test_mint_tokens() {
    let mut ctx = setup_ctx();

    let admin = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let minter = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let recipient = Keypair::new();

    initialize_program(&mut ctx, &admin);
    configure_minter(&mut ctx, &admin, &minter.pubkey(), 1_000_000_000);

    let config_pda = get_config_pda();
    let mint_pda = get_mint_pda();
    let minter_config_pda = get_minter_config_pda(&minter.pubkey());
    let destination_ata = get_ata(&recipient.pubkey(), &mint_pda);

    let mint_amount: u64 = 100_000_000;

    let ix = ctx
        .program()
        .accounts(client::accounts::MintTokens {
            minter: minter.pubkey(),
            config: config_pda,
            minter_config: minter_config_pda,
            mint: mint_pda,
            destination: destination_ata,
            destination_owner: recipient.pubkey(),
            token_program: TOKEN_PROGRAM_ID,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        })
        .args(client::args::MintTokens {
            amount: mint_amount,
        })
        .instruction()
        .unwrap();

    ctx.execute_instruction(ix, &[&minter])
        .unwrap()
        .assert_success();

    // Verify destination token account was created and has tokens
    assert!(
        ctx.account_exists(&destination_ata),
        "Destination token account should exist"
    );
    ctx.svm.assert_token_balance(&destination_ata, mint_amount);
}

#[test]
fn test_mint_exceeds_allowance() {
    let mut ctx = setup_ctx();

    let admin = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let minter = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let recipient = Keypair::new();

    initialize_program(&mut ctx, &admin);
    configure_minter(&mut ctx, &admin, &minter.pubkey(), 100_000_000); // 100 token allowance

    let config_pda = get_config_pda();
    let mint_pda = get_mint_pda();
    let minter_config_pda = get_minter_config_pda(&minter.pubkey());
    let destination_ata = get_ata(&recipient.pubkey(), &mint_pda);

    // Try to mint more than allowance
    let mint_amount: u64 = 200_000_000; // 200 tokens (exceeds 100 allowance)

    let ix = ctx
        .program()
        .accounts(client::accounts::MintTokens {
            minter: minter.pubkey(),
            config: config_pda,
            minter_config: minter_config_pda,
            mint: mint_pda,
            destination: destination_ata,
            destination_owner: recipient.pubkey(),
            token_program: TOKEN_PROGRAM_ID,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        })
        .args(client::args::MintTokens {
            amount: mint_amount,
        })
        .instruction()
        .unwrap();

    let result = ctx.execute_instruction(ix, &[&minter]);
    assert!(
        result.is_err() || !result.unwrap().is_success(),
        "Mint exceeding allowance should fail"
    );
}

#[test]
fn test_mint_unauthorized() {
    let mut ctx = setup_ctx();

    let admin = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let unauthorized = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let recipient = Keypair::new();

    initialize_program(&mut ctx, &admin);
    // Note: unauthorized is NOT configured as a minter

    let config_pda = get_config_pda();
    let mint_pda = get_mint_pda();
    let minter_config_pda = get_minter_config_pda(&unauthorized.pubkey());
    let destination_ata = get_ata(&recipient.pubkey(), &mint_pda);

    let mint_amount: u64 = 100_000_000;

    let ix = ctx
        .program()
        .accounts(client::accounts::MintTokens {
            minter: unauthorized.pubkey(),
            config: config_pda,
            minter_config: minter_config_pda,
            mint: mint_pda,
            destination: destination_ata,
            destination_owner: recipient.pubkey(),
            token_program: TOKEN_PROGRAM_ID,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        })
        .args(client::args::MintTokens {
            amount: mint_amount,
        })
        .instruction()
        .unwrap();

    let result = ctx.execute_instruction(ix, &[&unauthorized]);
    assert!(
        result.is_err() || !result.unwrap().is_success(),
        "Unauthorized mint should fail"
    );
}

// ============================================================================
// Burn Tokens Tests
// ============================================================================

fn mint_tokens(
    ctx: &mut anchor_litesvm::AnchorContext,
    minter: &Keypair,
    recipient: &Pubkey,
    amount: u64,
) {
    let config_pda = get_config_pda();
    let mint_pda = get_mint_pda();
    let minter_config_pda = get_minter_config_pda(&minter.pubkey());
    let destination_ata = get_ata(recipient, &mint_pda);

    let ix = ctx
        .program()
        .accounts(client::accounts::MintTokens {
            minter: minter.pubkey(),
            config: config_pda,
            minter_config: minter_config_pda,
            mint: mint_pda,
            destination: destination_ata,
            destination_owner: *recipient,
            token_program: TOKEN_PROGRAM_ID,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        })
        .args(client::args::MintTokens { amount })
        .instruction()
        .unwrap();

    ctx.execute_instruction(ix, &[minter])
        .expect("Mint should succeed")
        .assert_success();
}

#[test]
fn test_burn_tokens() {
    let mut ctx = setup_ctx();

    let admin = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let minter = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let user = ctx.svm.create_funded_account(10_000_000_000).unwrap();

    initialize_program(&mut ctx, &admin);
    configure_minter(&mut ctx, &admin, &minter.pubkey(), 1_000_000_000);
    mint_tokens(&mut ctx, &minter, &user.pubkey(), 100_000_000);

    let config_pda = get_config_pda();
    let mint_pda = get_mint_pda();
    let user_ata = get_ata(&user.pubkey(), &mint_pda);

    // Burn tokens
    let burn_amount: u64 = 50_000_000;

    let ix = ctx
        .program()
        .accounts(client::accounts::BurnTokens {
            owner: user.pubkey(),
            config: config_pda,
            mint: mint_pda,
            token_account: user_ata,
            token_program: TOKEN_PROGRAM_ID,
        })
        .args(client::args::BurnTokens {
            amount: burn_amount,
        })
        .instruction()
        .unwrap();

    ctx.execute_instruction(ix, &[&user])
        .unwrap()
        .assert_success();

    // Verify remaining balance
    ctx.svm.assert_token_balance(&user_ata, 50_000_000);
}

#[test]
fn test_burn_more_than_balance() {
    let mut ctx = setup_ctx();

    let admin = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let minter = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let user = ctx.svm.create_funded_account(10_000_000_000).unwrap();

    initialize_program(&mut ctx, &admin);
    configure_minter(&mut ctx, &admin, &minter.pubkey(), 1_000_000_000);
    mint_tokens(&mut ctx, &minter, &user.pubkey(), 100_000_000); // 100 tokens

    let config_pda = get_config_pda();
    let mint_pda = get_mint_pda();
    let user_ata = get_ata(&user.pubkey(), &mint_pda);

    // Try to burn more than balance
    let burn_amount: u64 = 200_000_000; // 200 tokens (only have 100)

    let ix = ctx
        .program()
        .accounts(client::accounts::BurnTokens {
            owner: user.pubkey(),
            config: config_pda,
            mint: mint_pda,
            token_account: user_ata,
            token_program: TOKEN_PROGRAM_ID,
        })
        .args(client::args::BurnTokens {
            amount: burn_amount,
        })
        .instruction()
        .unwrap();

    let result = ctx.execute_instruction(ix, &[&user]);
    assert!(
        result.is_err() || !result.unwrap().is_success(),
        "Burn more than balance should fail"
    );
}

// ============================================================================
// Pause/Unpause Tests
// ============================================================================

#[test]
fn test_pause() {
    let mut ctx = setup_ctx();

    let admin = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    initialize_program(&mut ctx, &admin);

    let config_pda = get_config_pda();

    let ix = ctx
        .program()
        .accounts(client::accounts::Pause {
            admin: admin.pubkey(),
            config: config_pda,
        })
        .args(client::args::Pause {})
        .instruction()
        .unwrap();

    ctx.execute_instruction(ix, &[&admin])
        .unwrap()
        .assert_success();
}

#[test]
fn test_pause_unauthorized() {
    let mut ctx = setup_ctx();

    let admin = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let unauthorized = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    initialize_program(&mut ctx, &admin);

    let config_pda = get_config_pda();

    let ix = ctx
        .program()
        .accounts(client::accounts::Pause {
            admin: unauthorized.pubkey(),
            config: config_pda,
        })
        .args(client::args::Pause {})
        .instruction()
        .unwrap();

    let result = ctx.execute_instruction(ix, &[&unauthorized]);
    assert!(
        result.is_err() || !result.unwrap().is_success(),
        "Unauthorized pause should fail"
    );
}

fn pause_program(ctx: &mut anchor_litesvm::AnchorContext, admin: &Keypair) {
    let config_pda = get_config_pda();

    let ix = ctx
        .program()
        .accounts(client::accounts::Pause {
            admin: admin.pubkey(),
            config: config_pda,
        })
        .args(client::args::Pause {})
        .instruction()
        .unwrap();

    ctx.execute_instruction(ix, &[admin])
        .expect("Pause should succeed")
        .assert_success();
}

#[test]
fn test_mint_when_paused() {
    let mut ctx = setup_ctx();

    let admin = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let minter = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let recipient = Keypair::new();

    initialize_program(&mut ctx, &admin);
    configure_minter(&mut ctx, &admin, &minter.pubkey(), 1_000_000_000);
    pause_program(&mut ctx, &admin);

    let config_pda = get_config_pda();
    let mint_pda = get_mint_pda();
    let minter_config_pda = get_minter_config_pda(&minter.pubkey());
    let destination_ata = get_ata(&recipient.pubkey(), &mint_pda);

    // Try to mint when paused
    let mint_amount: u64 = 100_000_000;

    let ix = ctx
        .program()
        .accounts(client::accounts::MintTokens {
            minter: minter.pubkey(),
            config: config_pda,
            minter_config: minter_config_pda,
            mint: mint_pda,
            destination: destination_ata,
            destination_owner: recipient.pubkey(),
            token_program: TOKEN_PROGRAM_ID,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        })
        .args(client::args::MintTokens {
            amount: mint_amount,
        })
        .instruction()
        .unwrap();

    let result = ctx.execute_instruction(ix, &[&minter]);
    assert!(
        result.is_err() || !result.unwrap().is_success(),
        "Mint when paused should fail"
    );
}

#[test]
fn test_unpause() {
    let mut ctx = setup_ctx();

    let admin = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    initialize_program(&mut ctx, &admin);
    pause_program(&mut ctx, &admin);

    let config_pda = get_config_pda();

    let ix = ctx
        .program()
        .accounts(client::accounts::Unpause {
            admin: admin.pubkey(),
            config: config_pda,
        })
        .args(client::args::Unpause {})
        .instruction()
        .unwrap();

    ctx.execute_instruction(ix, &[&admin])
        .unwrap()
        .assert_success();
}

#[test]
fn test_mint_after_unpause() {
    let mut ctx = setup_ctx();

    let admin = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let minter = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let recipient = Keypair::new();

    initialize_program(&mut ctx, &admin);
    configure_minter(&mut ctx, &admin, &minter.pubkey(), 1_000_000_000);
    pause_program(&mut ctx, &admin);

    let config_pda = get_config_pda();

    // Unpause
    let unpause_ix = ctx
        .program()
        .accounts(client::accounts::Unpause {
            admin: admin.pubkey(),
            config: config_pda,
        })
        .args(client::args::Unpause {})
        .instruction()
        .unwrap();

    ctx.execute_instruction(unpause_ix, &[&admin])
        .expect("Unpause should succeed")
        .assert_success();

    // Now mint should work
    let mint_pda = get_mint_pda();
    let minter_config_pda = get_minter_config_pda(&minter.pubkey());
    let destination_ata = get_ata(&recipient.pubkey(), &mint_pda);

    let mint_amount: u64 = 100_000_000;

    let mint_ix = ctx
        .program()
        .accounts(client::accounts::MintTokens {
            minter: minter.pubkey(),
            config: config_pda,
            minter_config: minter_config_pda,
            mint: mint_pda,
            destination: destination_ata,
            destination_owner: recipient.pubkey(),
            token_program: TOKEN_PROGRAM_ID,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        })
        .args(client::args::MintTokens {
            amount: mint_amount,
        })
        .instruction()
        .unwrap();

    ctx.execute_instruction(mint_ix, &[&minter])
        .unwrap()
        .assert_success();

    ctx.svm.assert_token_balance(&destination_ata, mint_amount);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_full_stablecoin_flow() {
    let mut ctx = setup_ctx();

    let admin = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let minter = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let user1 = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let user2 = ctx.svm.create_funded_account(10_000_000_000).unwrap();

    // 1. Initialize
    initialize_program(&mut ctx, &admin);

    // 2. Configure minter
    configure_minter(&mut ctx, &admin, &minter.pubkey(), 1_000_000_000);

    // 3. Mint to user1
    mint_tokens(&mut ctx, &minter, &user1.pubkey(), 100_000_000);

    // 4. Mint to user2
    mint_tokens(&mut ctx, &minter, &user2.pubkey(), 200_000_000);

    // 5. User1 burns some tokens
    let config_pda = get_config_pda();
    let mint_pda = get_mint_pda();
    let user1_ata = get_ata(&user1.pubkey(), &mint_pda);

    let burn_amount: u64 = 50_000_000;

    let burn_ix = ctx
        .program()
        .accounts(client::accounts::BurnTokens {
            owner: user1.pubkey(),
            config: config_pda,
            mint: mint_pda,
            token_account: user1_ata,
            token_program: TOKEN_PROGRAM_ID,
        })
        .args(client::args::BurnTokens {
            amount: burn_amount,
        })
        .instruction()
        .unwrap();

    ctx.execute_instruction(burn_ix, &[&user1])
        .unwrap()
        .assert_success();

    // Verify user1 balance after burn
    ctx.svm.assert_token_balance(&user1_ata, 50_000_000);

    // 6. Pause and unpause
    pause_program(&mut ctx, &admin);

    let unpause_ix = ctx
        .program()
        .accounts(client::accounts::Unpause {
            admin: admin.pubkey(),
            config: config_pda,
        })
        .args(client::args::Unpause {})
        .instruction()
        .unwrap();

    ctx.execute_instruction(unpause_ix, &[&admin])
        .unwrap()
        .assert_success();

    // 7. Remove minter
    let minter_config_pda = get_minter_config_pda(&minter.pubkey());

    let remove_minter_ix = ctx
        .program()
        .accounts(client::accounts::RemoveMinter {
            admin: admin.pubkey(),
            config: config_pda,
            minter: minter.pubkey(),
            minter_config: minter_config_pda,
        })
        .args(client::args::RemoveMinter {})
        .instruction()
        .unwrap();

    ctx.execute_instruction(remove_minter_ix, &[&admin])
        .unwrap()
        .assert_success();

    // Verify minter was removed
    ctx.svm.assert_account_closed(&minter_config_pda);
}

#[test]
fn test_multiple_minters() {
    let mut ctx = setup_ctx();

    let admin = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let minter1 = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let minter2 = ctx.svm.create_funded_account(10_000_000_000).unwrap();
    let user = Keypair::new();

    initialize_program(&mut ctx, &admin);

    // Configure two minters with different allowances
    configure_minter(&mut ctx, &admin, &minter1.pubkey(), 500_000_000);
    configure_minter(&mut ctx, &admin, &minter2.pubkey(), 1_000_000_000);

    // Both minters mint to the same user
    mint_tokens(&mut ctx, &minter1, &user.pubkey(), 100_000_000);
    mint_tokens(&mut ctx, &minter2, &user.pubkey(), 200_000_000);

    // Verify user received tokens from both minters
    let mint_pda = get_mint_pda();
    let user_ata = get_ata(&user.pubkey(), &mint_pda);

    assert!(
        ctx.account_exists(&user_ata),
        "User should have token account"
    );
    ctx.svm.assert_token_balance(&user_ata, 300_000_000);
}
