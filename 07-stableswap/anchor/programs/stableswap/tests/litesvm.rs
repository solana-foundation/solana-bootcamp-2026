use std::{error::Error, mem::size_of, path::PathBuf};

use anchor_lang::{prelude::rent, prelude::Clock};
use anchor_litesvm::{AnchorLiteSVM, AssertionHelpers, TestHelpers};
use anchor_spl::{
    associated_token::{get_associated_token_address, ID as ASSOCIATED_TOKEN_PROGRAM_ID},
    token::ID as TOKEN_PROGRAM_ID,
};
use bytemuck::{bytes_of, Pod, Zeroable};
use solana_sdk::{
    account::Account,
    native_loader,
    signature::{Keypair, Signer},
};
use stableswap::{
    accounts,
    constants::MINIMUM_LIQUIDITY,
    instruction,
    math::{calculate_lp_mint_amount, calculate_swap_output, calculate_withdraw_amounts},
    state::Pool,
    ID,
};

const PROGRAM_SO_PATH: &str = "../../target/deploy/stableswap.so";
const AMPLIFICATION: u64 = 100;
const BASE_FEE_BPS: u16 = 4;
const MAX_DYNAMIC_FEE_BPS: u16 = 100;
const DEPEG_THRESHOLD_BPS: u16 = 500;
const MAX_PRICE_AGE_SEC: u64 = 60;
const DECIMALS: u8 = 6;
const ONE_TOKEN: u64 = 1_000_000;
const INITIAL_MINT: u64 = 2_000_000 * ONE_TOKEN;
const INITIAL_DEPOSIT: u64 = 1_000_000 * ONE_TOKEN;
const SWAP_AMOUNT: u64 = 10_000 * ONE_TOKEN;

const PYTH_MAGIC: u32 = 0xa1b2c3d4;
const PYTH_VERSION_2: u32 = 2;
const PYTH_ACCOUNT_TYPE_PRICE: u32 = 3;
const PYTH_STATUS_TRADING: u8 = 1;
const PYTH_NUM_COMPONENTS: usize = 32;
const ONE_DOLLAR_PRICE: i64 = 100_000_000;
const DEPEGGED_PRICE: i64 = 90_000_000;
const PYTH_EXPONENT: i32 = -8;
const NORMALIZED_ONE_DOLLAR: u128 = 1_000_000_000;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
struct PythPriceInfo {
    price: i64,
    conf: u64,
    status: u8,
    corp_act: u8,
    padding: [u8; 6],
    pub_slot: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
struct PythRational {
    val: i64,
    numer: i64,
    denom: i64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
struct PythPriceComp {
    publisher: [u8; 32],
    agg: PythPriceInfo,
    latest: PythPriceInfo,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct PythPriceAccount {
    magic: u32,
    ver: u32,
    atype: u32,
    size: u32,
    ptype: u32,
    expo: i32,
    num: u32,
    num_qt: u32,
    last_slot: u64,
    valid_slot: u64,
    ema_price: PythRational,
    ema_conf: PythRational,
    timestamp: i64,
    min_pub: u8,
    drv2: u8,
    drv3: u16,
    drv4: u32,
    prod: [u8; 32],
    next: [u8; 32],
    prev_slot: u64,
    prev_price: i64,
    prev_conf: u64,
    prev_timestamp: i64,
    agg: PythPriceInfo,
    comp: [PythPriceComp; PYTH_NUM_COMPONENTS],
}

struct TestFixture {
    ctx: anchor_litesvm::AnchorContext,
    user: Keypair,
    token_mint_a: Keypair,
    token_mint_b: Keypair,
    lp_mint: Keypair,
    pool: solana_sdk::pubkey::Pubkey,
    vault_a: solana_sdk::pubkey::Pubkey,
    vault_b: solana_sdk::pubkey::Pubkey,
    user_token_a: solana_sdk::pubkey::Pubkey,
    user_token_b: solana_sdk::pubkey::Pubkey,
    user_lp_token: Option<solana_sdk::pubkey::Pubkey>,
    oracle_a: solana_sdk::pubkey::Pubkey,
    oracle_b: solana_sdk::pubkey::Pubkey,
}

impl TestFixture {
    fn new() -> Result<Self, Box<dyn Error>> {
        let mut ctx = AnchorLiteSVM::build_with_program(ID, &read_program_bytes()?);
        ctx.svm.set_account(
            anchor_lang::system_program::ID,
            Account {
                lamports: 1,
                data: b"system_program".to_vec(),
                owner: native_loader::ID,
                executable: true,
                rent_epoch: 0,
            },
        )?;
        let user = ctx.create_funded_account(20_000_000_000)?;
        let token_mint_a = ctx.svm.create_token_mint(&user, DECIMALS)?;
        let token_mint_b = ctx.svm.create_token_mint(&user, DECIMALS)?;
        let user_token_a = ctx
            .svm
            .create_associated_token_account(&token_mint_a.pubkey(), &user)?;
        let user_token_b = ctx
            .svm
            .create_associated_token_account(&token_mint_b.pubkey(), &user)?;
        ctx.svm
            .mint_to(&token_mint_a.pubkey(), &user_token_a, &user, INITIAL_MINT)?;
        ctx.svm
            .mint_to(&token_mint_b.pubkey(), &user_token_b, &user, INITIAL_MINT)?;

        let oracle_a = solana_sdk::pubkey::Pubkey::new_unique();
        let oracle_b = solana_sdk::pubkey::Pubkey::new_unique();
        write_pyth_price_account(&mut ctx, oracle_a, ONE_DOLLAR_PRICE)?;
        write_pyth_price_account(&mut ctx, oracle_b, ONE_DOLLAR_PRICE)?;

        let lp_mint = Keypair::new();
        let (pool, _) = solana_sdk::pubkey::Pubkey::find_program_address(
            &[b"pool", lp_mint.pubkey().as_ref()],
            &ID,
        );
        let vault_a = get_associated_token_address(&pool, &token_mint_a.pubkey());
        let vault_b = get_associated_token_address(&pool, &token_mint_b.pubkey());

        Ok(Self {
            ctx,
            user,
            token_mint_a,
            token_mint_b,
            lp_mint,
            pool,
            vault_a,
            vault_b,
            user_token_a,
            user_token_b,
            user_lp_token: None,
            oracle_a,
            oracle_b,
        })
    }

    fn initialize_pool(&mut self) {
        let system_program_account = self
            .ctx
            .svm
            .get_account(&anchor_lang::system_program::ID)
            .expect("system program account must exist in LiteSVM");
        assert!(
            system_program_account.executable,
            "system program account must be executable"
        );

        let ix = self
            .ctx
            .program()
            .accounts(accounts::InitializePool {
                admin: self.user.pubkey(),
                token_mint_a: self.token_mint_a.pubkey(),
                token_mint_b: self.token_mint_b.pubkey(),
                pool: self.pool,
                lp_mint: self.lp_mint.pubkey(),
                vault_a: self.vault_a,
                vault_b: self.vault_b,
                oracle_price_feed_a: self.oracle_a,
                oracle_price_feed_b: self.oracle_b,
                system_program: anchor_lang::system_program::ID,
                token_program: TOKEN_PROGRAM_ID,
                associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
                rent: rent::ID,
            })
            .args(instruction::InitializePool {
                amplification: AMPLIFICATION,
                base_fee_bps: BASE_FEE_BPS,
                max_dynamic_fee_bps: MAX_DYNAMIC_FEE_BPS,
                depeg_threshold_bps: DEPEG_THRESHOLD_BPS,
                max_price_age_sec: MAX_PRICE_AGE_SEC,
            })
            .instruction()
            .unwrap();

        assert_eq!(ix.accounts[9].pubkey, anchor_lang::system_program::ID);
        assert!(!ix.accounts[9].is_signer);
        assert!(!ix.accounts[9].is_writable);
        assert_eq!(ix.accounts[10].pubkey, TOKEN_PROGRAM_ID);
        assert!(!ix.accounts[10].is_signer);
        assert!(!ix.accounts[10].is_writable);
        assert_eq!(ix.accounts[11].pubkey, ASSOCIATED_TOKEN_PROGRAM_ID);
        assert!(!ix.accounts[11].is_signer);
        assert!(!ix.accounts[11].is_writable);
        assert_eq!(ix.accounts[12].pubkey, rent::ID);
        assert!(!ix.accounts[12].is_signer);
        assert!(!ix.accounts[12].is_writable);

        self.ctx
            .execute_instruction(ix, &[&self.user, &self.lp_mint])
            .unwrap()
            .assert_success();
    }

    fn create_user_lp_token(&mut self) -> solana_sdk::pubkey::Pubkey {
        let user_lp_token = self
            .ctx
            .svm
            .create_associated_token_account(&self.lp_mint.pubkey(), &self.user)
            .unwrap();
        self.user_lp_token = Some(user_lp_token);
        user_lp_token
    }

    fn add_liquidity(&mut self, amount_a: u64, amount_b: u64, min_lp_out: u64) {
        let user_lp_token = self.user_lp_token.expect("LP ATA must exist");
        let ix = self
            .ctx
            .program()
            .accounts(accounts::AddLiquidity {
                token_mint_a: self.token_mint_a.pubkey(),
                token_mint_b: self.token_mint_b.pubkey(),
                pool: self.pool,
                vault_a: self.vault_a,
                vault_b: self.vault_b,
                lp_mint: self.lp_mint.pubkey(),
                user_token_a: self.user_token_a,
                user_token_b: self.user_token_b,
                user_lp_token,
                oracle_price_feed_a: self.oracle_a,
                oracle_price_feed_b: self.oracle_b,
                user: self.user.pubkey(),
                token_program: TOKEN_PROGRAM_ID,
            })
            .args(instruction::AddLiquidity {
                amount_a,
                amount_b,
                min_lp_out,
            })
            .instruction()
            .unwrap();

        self.ctx
            .execute_instruction(ix, &[&self.user])
            .unwrap()
            .assert_success();
    }

    fn remove_liquidity(&mut self, lp_amount: u64, min_a: u64, min_b: u64) {
        let user_lp_token = self.user_lp_token.expect("LP ATA must exist");
        let ix = self
            .ctx
            .program()
            .accounts(accounts::RemoveLiquidity {
                token_mint_a: self.token_mint_a.pubkey(),
                token_mint_b: self.token_mint_b.pubkey(),
                pool: self.pool,
                vault_a: self.vault_a,
                vault_b: self.vault_b,
                lp_mint: self.lp_mint.pubkey(),
                user_token_a: self.user_token_a,
                user_token_b: self.user_token_b,
                user_lp_token,
                user: self.user.pubkey(),
                token_program: TOKEN_PROGRAM_ID,
            })
            .args(instruction::RemoveLiquidity {
                lp_amount,
                min_a,
                min_b,
            })
            .instruction()
            .unwrap();

        self.ctx
            .execute_instruction(ix, &[&self.user])
            .unwrap()
            .assert_success();
    }

    fn check_depeg(&mut self) -> anchor_litesvm::TransactionResult {
        let ix = self
            .ctx
            .program()
            .accounts(accounts::CheckDepeg {
                token_mint_a: self.token_mint_a.pubkey(),
                token_mint_b: self.token_mint_b.pubkey(),
                lp_mint: self.lp_mint.pubkey(),
                pool: self.pool,
                oracle_price_feed_a: self.oracle_a,
                oracle_price_feed_b: self.oracle_b,
            })
            .args(instruction::CheckDepeg {})
            .instruction()
            .unwrap();

        self.ctx.execute_instruction(ix, &[&self.user]).unwrap()
    }

    fn swap(
        &mut self,
        amount_in: u64,
        min_amount_out: u64,
        input_index: u8,
        output_index: u8,
    ) -> anchor_litesvm::TransactionResult {
        let mut ix = self
            .ctx
            .program()
            .accounts(accounts::Swap {
                pool: self.pool,
                oracle_price_feed_a: self.oracle_a,
                oracle_price_feed_b: self.oracle_b,
                user: self.user.pubkey(),
                token_program: TOKEN_PROGRAM_ID,
            })
            .args(instruction::Swap {
                amount_in,
                min_amount_out,
                input_index,
                output_index,
            })
            .instruction()
            .unwrap();

        ix.accounts.extend([
            anchor_lang::solana_program::instruction::AccountMeta::new(self.vault_a, false),
            anchor_lang::solana_program::instruction::AccountMeta::new(self.vault_b, false),
            anchor_lang::solana_program::instruction::AccountMeta::new(self.user_token_a, false),
            anchor_lang::solana_program::instruction::AccountMeta::new(self.user_token_b, false),
        ]);

        self.ctx.execute_instruction(ix, &[&self.user]).unwrap()
    }

    fn pool_state(&self) -> Pool {
        self.ctx.get_account::<Pool>(&self.pool).unwrap()
    }

    fn overwrite_oracle(&mut self, oracle: solana_sdk::pubkey::Pubkey, price: i64) {
        write_pyth_price_account(&mut self.ctx, oracle, price).unwrap();
    }
}

fn read_program_bytes() -> Result<Vec<u8>, Box<dyn Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(PROGRAM_SO_PATH);
    Ok(std::fs::read(&path).map_err(|err| {
        format!(
            "failed to read compiled program at {}: {}. Run `anchor build` first.",
            path.display(),
            err
        )
    })?)
}

fn write_pyth_price_account(
    ctx: &mut anchor_litesvm::AnchorContext,
    oracle: solana_sdk::pubkey::Pubkey,
    price: i64,
) -> Result<(), Box<dyn Error>> {
    let clock = ctx.ctx_svm_clock();
    let price_account = PythPriceAccount {
        magic: PYTH_MAGIC,
        ver: PYTH_VERSION_2,
        atype: PYTH_ACCOUNT_TYPE_PRICE,
        size: size_of::<PythPriceAccount>() as u32,
        ptype: 0,
        expo: PYTH_EXPONENT,
        num: 1,
        num_qt: 1,
        last_slot: clock.slot,
        valid_slot: clock.slot,
        ema_price: PythRational {
            val: price,
            numer: price,
            denom: 1,
        },
        ema_conf: PythRational {
            val: 0,
            numer: 0,
            denom: 1,
        },
        timestamp: clock.unix_timestamp,
        min_pub: 1,
        drv2: 0,
        drv3: 0,
        drv4: 0,
        prod: [0; 32],
        next: [0; 32],
        prev_slot: clock.slot,
        prev_price: price,
        prev_conf: 0,
        prev_timestamp: clock.unix_timestamp,
        agg: PythPriceInfo {
            price,
            conf: 0,
            status: PYTH_STATUS_TRADING,
            corp_act: 0,
            padding: [0; 6],
            pub_slot: clock.slot,
        },
        comp: [PythPriceComp::default(); PYTH_NUM_COMPONENTS],
    };

    ctx.svm.set_account(
        oracle,
        Account {
            lamports: ctx
                .svm
                .minimum_balance_for_rent_exemption(size_of::<PythPriceAccount>()),
            data: bytes_of(&price_account).to_vec(),
            owner: solana_sdk::pubkey::Pubkey::new_unique(),
            executable: false,
            rent_epoch: 0,
        },
    )?;

    Ok(())
}

trait ClockAccess {
    fn ctx_svm_clock(&self) -> Clock;
}

impl ClockAccess for anchor_litesvm::AnchorContext {
    fn ctx_svm_clock(&self) -> Clock {
        self.svm.get_sysvar::<Clock>()
    }
}

#[test]
fn initialize_pool_creates_pool_state_and_vaults() -> Result<(), Box<dyn Error>> {
    let mut fixture = TestFixture::new()?;
    fixture.initialize_pool();

    let pool = fixture.pool_state();
    assert_eq!(pool.admin, fixture.user.pubkey());
    assert_eq!(pool.lp_mint, fixture.lp_mint.pubkey());
    assert_eq!(pool.amplification, AMPLIFICATION);
    assert_eq!(pool.fee_bps, BASE_FEE_BPS);
    assert_eq!(pool.token_mints[0], fixture.token_mint_a.pubkey());
    assert_eq!(pool.token_mints[1], fixture.token_mint_b.pubkey());
    assert_eq!(pool.oracle_config.oracle_a, fixture.oracle_a);
    assert_eq!(pool.oracle_config.oracle_b, fixture.oracle_b);
    assert!(!pool.is_paused);

    fixture.ctx.svm.assert_account_exists(&fixture.pool);
    fixture.ctx.svm.assert_account_exists(&fixture.vault_a);
    fixture.ctx.svm.assert_account_exists(&fixture.vault_b);
    fixture
        .ctx
        .svm
        .assert_mint_supply(&fixture.lp_mint.pubkey(), 0);

    Ok(())
}

#[test]
fn add_and_remove_liquidity_use_proportional_lp_accounting() -> Result<(), Box<dyn Error>> {
    let mut fixture = TestFixture::new()?;
    fixture.initialize_pool();
    let user_lp_token = fixture.create_user_lp_token();

    let expected_lp = calculate_lp_mint_amount(
        0,
        0,
        INITIAL_DEPOSIT as u128,
        INITIAL_DEPOSIT as u128,
        0,
        AMPLIFICATION as u128,
        MINIMUM_LIQUIDITY,
    )?;

    fixture.add_liquidity(INITIAL_DEPOSIT, INITIAL_DEPOSIT, expected_lp);

    fixture
        .ctx
        .svm
        .assert_token_balance(&fixture.vault_a, INITIAL_DEPOSIT);
    fixture
        .ctx
        .svm
        .assert_token_balance(&fixture.vault_b, INITIAL_DEPOSIT);
    fixture
        .ctx
        .svm
        .assert_token_balance(&user_lp_token, expected_lp);
    fixture
        .ctx
        .svm
        .assert_mint_supply(&fixture.lp_mint.pubkey(), expected_lp);

    let burn_amount = expected_lp / 2;
    let withdraw_amounts = calculate_withdraw_amounts(
        &[INITIAL_DEPOSIT as u128, INITIAL_DEPOSIT as u128],
        burn_amount as u128,
        expected_lp as u128 + MINIMUM_LIQUIDITY as u128,
    )?;
    fixture.remove_liquidity(burn_amount, withdraw_amounts[0], withdraw_amounts[1]);

    fixture.ctx.svm.assert_token_balance(
        &fixture.user_token_a,
        INITIAL_MINT - INITIAL_DEPOSIT + withdraw_amounts[0],
    );
    fixture.ctx.svm.assert_token_balance(
        &fixture.user_token_b,
        INITIAL_MINT - INITIAL_DEPOSIT + withdraw_amounts[1],
    );
    fixture
        .ctx
        .svm
        .assert_token_balance(&user_lp_token, expected_lp - burn_amount);

    Ok(())
}

#[test]
fn swap_uses_remaining_accounts_and_matches_quote() -> Result<(), Box<dyn Error>> {
    let mut fixture = TestFixture::new()?;
    fixture.initialize_pool();
    fixture.create_user_lp_token();
    fixture.add_liquidity(INITIAL_DEPOSIT, INITIAL_DEPOSIT, 0);

    let quote = calculate_swap_output(
        INITIAL_DEPOSIT as u128,
        INITIAL_DEPOSIT as u128,
        SWAP_AMOUNT as u128,
        AMPLIFICATION as u128,
        BASE_FEE_BPS,
        MAX_DYNAMIC_FEE_BPS,
        NORMALIZED_ONE_DOLLAR,
        NORMALIZED_ONE_DOLLAR,
        DEPEG_THRESHOLD_BPS,
    )?;

    fixture
        .swap(SWAP_AMOUNT, quote.amount_out as u64, 0, 1)
        .assert_success();

    fixture.ctx.svm.assert_token_balance(
        &fixture.user_token_a,
        INITIAL_MINT - INITIAL_DEPOSIT - SWAP_AMOUNT,
    );
    fixture.ctx.svm.assert_token_balance(
        &fixture.user_token_b,
        INITIAL_MINT - INITIAL_DEPOSIT + quote.amount_out as u64,
    );

    Ok(())
}

#[test]
fn check_depeg_pauses_pool_and_blocks_swaps() -> Result<(), Box<dyn Error>> {
    let mut fixture = TestFixture::new()?;
    fixture.initialize_pool();
    fixture.create_user_lp_token();
    fixture.add_liquidity(INITIAL_DEPOSIT, INITIAL_DEPOSIT, 0);

    fixture.overwrite_oracle(fixture.oracle_b, DEPEGGED_PRICE);
    fixture.check_depeg().assert_success();

    let pool = fixture.pool_state();
    assert!(pool.is_paused);

    fixture
        .swap(SWAP_AMOUNT, 0, 0, 1)
        .assert_anchor_error("PoolPaused");

    Ok(())
}
