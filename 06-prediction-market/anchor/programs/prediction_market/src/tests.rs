use anchor_lang::{InstructionData, ToAccountMetas};
use litesvm::LiteSVM;
use solana_sdk::{
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};

use crate::{instruction, state::Market, ID};

fn get_market_pda(creator: &Pubkey, market_id: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"market", creator.as_ref(), &market_id.to_le_bytes()],
        &ID,
    )
}

fn get_position_pda(market: &Pubkey, user: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"position", market.as_ref(), user.as_ref()], &ID)
}

fn setup() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();
    svm.add_program_from_file(ID, "target/deploy/prediction_market.so")
        .expect("Failed to load program");

    let creator = Keypair::new();
    svm.airdrop(&creator.pubkey(), 10_000_000_000).unwrap();

    (svm, creator)
}

#[test]
fn test_create_market() {
    let (mut svm, creator) = setup();
    let market_id: u64 = 1;
    let (market_pda, _) = get_market_pda(&creator.pubkey(), market_id);

    let resolution_time = svm.get_sysvar::<solana_sdk::clock::Clock>().unix_timestamp + 3600;

    let ix = Instruction {
        program_id: ID,
        accounts: instruction::CreateMarket {
            creator: creator.pubkey(),
            market: market_pda,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: instruction::CreateMarket {
            market_id,
            question: "Will it rain tomorrow?".to_string(),
            resolution_time,
        }
        .data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&creator.pubkey()),
        &[&creator],
        svm.latest_blockhash(),
    );

    svm.send_transaction(tx).expect("Create market failed");

    // Verify market was created
    let market_account = svm.get_account(&market_pda).expect("Market not found");
    assert!(market_account.lamports > 0);
}

#[test]
fn test_place_bet_yes() {
    let (mut svm, creator) = setup();
    let market_id: u64 = 1;
    let (market_pda, _) = get_market_pda(&creator.pubkey(), market_id);

    let resolution_time = svm.get_sysvar::<solana_sdk::clock::Clock>().unix_timestamp + 3600;

    // Create market
    let create_ix = Instruction {
        program_id: ID,
        accounts: instruction::CreateMarket {
            creator: creator.pubkey(),
            market: market_pda,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: instruction::CreateMarket {
            market_id,
            question: "Test market".to_string(),
            resolution_time,
        }
        .data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&creator.pubkey()),
        &[&creator],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).unwrap();

    // Create bettor
    let bettor = Keypair::new();
    svm.airdrop(&bettor.pubkey(), 5_000_000_000).unwrap();

    let (position_pda, _) = get_position_pda(&market_pda, &bettor.pubkey());

    // Place YES bet
    let bet_ix = Instruction {
        program_id: ID,
        accounts: instruction::PlaceBet {
            user: bettor.pubkey(),
            market: market_pda,
            user_position: position_pda,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: instruction::PlaceBet {
            amount: 1_000_000_000, // 1 SOL
            bet_yes: true,
        }
        .data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[bet_ix],
        Some(&bettor.pubkey()),
        &[&bettor],
        svm.latest_blockhash(),
    );

    svm.send_transaction(tx).expect("Place bet failed");

    // Verify position was created
    let position_account = svm.get_account(&position_pda).expect("Position not found");
    assert!(position_account.lamports > 0);
}

#[test]
fn test_cannot_bet_after_deadline() {
    let (mut svm, creator) = setup();
    let market_id: u64 = 1;
    let (market_pda, _) = get_market_pda(&creator.pubkey(), market_id);

    // Create market with resolution time in past (1 second from now)
    let resolution_time = svm.get_sysvar::<solana_sdk::clock::Clock>().unix_timestamp + 1;

    let create_ix = Instruction {
        program_id: ID,
        accounts: instruction::CreateMarket {
            creator: creator.pubkey(),
            market: market_pda,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: instruction::CreateMarket {
            market_id,
            question: "Test market".to_string(),
            resolution_time,
        }
        .data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&creator.pubkey()),
        &[&creator],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).unwrap();

    // Warp time past resolution
    svm.warp_to_slot(100);

    let bettor = Keypair::new();
    svm.airdrop(&bettor.pubkey(), 5_000_000_000).unwrap();

    let (position_pda, _) = get_position_pda(&market_pda, &bettor.pubkey());

    let bet_ix = Instruction {
        program_id: ID,
        accounts: instruction::PlaceBet {
            user: bettor.pubkey(),
            market: market_pda,
            user_position: position_pda,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: instruction::PlaceBet {
            amount: 1_000_000_000,
            bet_yes: true,
        }
        .data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[bet_ix],
        Some(&bettor.pubkey()),
        &[&bettor],
        svm.latest_blockhash(),
    );

    // Should fail because betting is closed
    let result = svm.send_transaction(tx);
    assert!(result.is_err(), "Betting after deadline should fail");
}

#[test]
fn test_resolve_market() {
    let (mut svm, creator) = setup();
    let market_id: u64 = 1;
    let (market_pda, _) = get_market_pda(&creator.pubkey(), market_id);

    let resolution_time = svm.get_sysvar::<solana_sdk::clock::Clock>().unix_timestamp + 1;

    // Create market
    let create_ix = Instruction {
        program_id: ID,
        accounts: instruction::CreateMarket {
            creator: creator.pubkey(),
            market: market_pda,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: instruction::CreateMarket {
            market_id,
            question: "Test market".to_string(),
            resolution_time,
        }
        .data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&creator.pubkey()),
        &[&creator],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).unwrap();

    // Warp past resolution time
    svm.warp_to_slot(100);

    // Resolve market
    let resolve_ix = Instruction {
        program_id: ID,
        accounts: instruction::ResolveMarket {
            creator: creator.pubkey(),
            market: market_pda,
        }
        .to_account_metas(None),
        data: instruction::ResolveMarket { outcome: true }.data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[resolve_ix],
        Some(&creator.pubkey()),
        &[&creator],
        svm.latest_blockhash(),
    );

    svm.send_transaction(tx).expect("Resolve market failed");
}

#[test]
fn test_only_creator_can_resolve() {
    let (mut svm, creator) = setup();
    let market_id: u64 = 1;
    let (market_pda, _) = get_market_pda(&creator.pubkey(), market_id);

    let resolution_time = svm.get_sysvar::<solana_sdk::clock::Clock>().unix_timestamp + 1;

    // Create market
    let create_ix = Instruction {
        program_id: ID,
        accounts: instruction::CreateMarket {
            creator: creator.pubkey(),
            market: market_pda,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: instruction::CreateMarket {
            market_id,
            question: "Test market".to_string(),
            resolution_time,
        }
        .data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&creator.pubkey()),
        &[&creator],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).unwrap();

    // Warp past resolution time
    svm.warp_to_slot(100);

    // Try to resolve with different user
    let attacker = Keypair::new();
    svm.airdrop(&attacker.pubkey(), 1_000_000_000).unwrap();

    let resolve_ix = Instruction {
        program_id: ID,
        accounts: instruction::ResolveMarket {
            creator: attacker.pubkey(),
            market: market_pda,
        }
        .to_account_metas(None),
        data: instruction::ResolveMarket { outcome: true }.data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[resolve_ix],
        Some(&attacker.pubkey()),
        &[&attacker],
        svm.latest_blockhash(),
    );

    let result = svm.send_transaction(tx);
    assert!(result.is_err(), "Non-creator should not be able to resolve");
}

#[test]
fn test_full_flow_with_payout() {
    let (mut svm, creator) = setup();
    let market_id: u64 = 1;
    let (market_pda, _) = get_market_pda(&creator.pubkey(), market_id);

    let resolution_time = svm.get_sysvar::<solana_sdk::clock::Clock>().unix_timestamp + 1;

    // Create market
    let create_ix = Instruction {
        program_id: ID,
        accounts: instruction::CreateMarket {
            creator: creator.pubkey(),
            market: market_pda,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: instruction::CreateMarket {
            market_id,
            question: "Test market".to_string(),
            resolution_time,
        }
        .data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&creator.pubkey()),
        &[&creator],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).unwrap();

    // Bettor 1 bets YES (1 SOL)
    let bettor1 = Keypair::new();
    svm.airdrop(&bettor1.pubkey(), 5_000_000_000).unwrap();
    let (position1_pda, _) = get_position_pda(&market_pda, &bettor1.pubkey());

    let bet_ix = Instruction {
        program_id: ID,
        accounts: instruction::PlaceBet {
            user: bettor1.pubkey(),
            market: market_pda,
            user_position: position1_pda,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: instruction::PlaceBet {
            amount: 1_000_000_000,
            bet_yes: true,
        }
        .data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[bet_ix],
        Some(&bettor1.pubkey()),
        &[&bettor1],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).unwrap();

    // Bettor 2 bets NO (2 SOL)
    let bettor2 = Keypair::new();
    svm.airdrop(&bettor2.pubkey(), 5_000_000_000).unwrap();
    let (position2_pda, _) = get_position_pda(&market_pda, &bettor2.pubkey());

    let bet_ix = Instruction {
        program_id: ID,
        accounts: instruction::PlaceBet {
            user: bettor2.pubkey(),
            market: market_pda,
            user_position: position2_pda,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: instruction::PlaceBet {
            amount: 2_000_000_000,
            bet_yes: false,
        }
        .data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[bet_ix],
        Some(&bettor2.pubkey()),
        &[&bettor2],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).unwrap();

    // Warp past resolution time
    svm.warp_to_slot(100);

    // Resolve market as YES
    let resolve_ix = Instruction {
        program_id: ID,
        accounts: instruction::ResolveMarket {
            creator: creator.pubkey(),
            market: market_pda,
        }
        .to_account_metas(None),
        data: instruction::ResolveMarket { outcome: true }.data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[resolve_ix],
        Some(&creator.pubkey()),
        &[&creator],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).unwrap();

    // Bettor 1 claims winnings
    let balance_before = svm.get_account(&bettor1.pubkey()).unwrap().lamports;

    let claim_ix = Instruction {
        program_id: ID,
        accounts: instruction::ClaimWinnings {
            user: bettor1.pubkey(),
            market: market_pda,
            user_position: position1_pda,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: instruction::ClaimWinnings {}.data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[claim_ix],
        Some(&bettor1.pubkey()),
        &[&bettor1],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).expect("Claim winnings failed");

    let balance_after = svm.get_account(&bettor1.pubkey()).unwrap().lamports;

    // Bettor 1 should receive: 1 SOL (original) + 2 SOL (NO pool) = 3 SOL
    // Minus transaction fees
    assert!(
        balance_after > balance_before + 2_500_000_000,
        "Winner should receive winnings"
    );
}

#[test]
fn test_cannot_double_claim() {
    let (mut svm, creator) = setup();
    let market_id: u64 = 1;
    let (market_pda, _) = get_market_pda(&creator.pubkey(), market_id);

    let resolution_time = svm.get_sysvar::<solana_sdk::clock::Clock>().unix_timestamp + 1;

    // Create market
    let create_ix = Instruction {
        program_id: ID,
        accounts: instruction::CreateMarket {
            creator: creator.pubkey(),
            market: market_pda,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: instruction::CreateMarket {
            market_id,
            question: "Test market".to_string(),
            resolution_time,
        }
        .data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&creator.pubkey()),
        &[&creator],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).unwrap();

    // Bettor bets YES
    let bettor = Keypair::new();
    svm.airdrop(&bettor.pubkey(), 5_000_000_000).unwrap();
    let (position_pda, _) = get_position_pda(&market_pda, &bettor.pubkey());

    let bet_ix = Instruction {
        program_id: ID,
        accounts: instruction::PlaceBet {
            user: bettor.pubkey(),
            market: market_pda,
            user_position: position_pda,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: instruction::PlaceBet {
            amount: 1_000_000_000,
            bet_yes: true,
        }
        .data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[bet_ix],
        Some(&bettor.pubkey()),
        &[&bettor],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).unwrap();

    // Warp and resolve
    svm.warp_to_slot(100);

    let resolve_ix = Instruction {
        program_id: ID,
        accounts: instruction::ResolveMarket {
            creator: creator.pubkey(),
            market: market_pda,
        }
        .to_account_metas(None),
        data: instruction::ResolveMarket { outcome: true }.data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[resolve_ix],
        Some(&creator.pubkey()),
        &[&creator],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).unwrap();

    // First claim - should succeed
    let claim_ix = Instruction {
        program_id: ID,
        accounts: instruction::ClaimWinnings {
            user: bettor.pubkey(),
            market: market_pda,
            user_position: position_pda,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: instruction::ClaimWinnings {}.data(),
    };

    let tx = Transaction::new_signed_with_payer(
        &[claim_ix.clone()],
        Some(&bettor.pubkey()),
        &[&bettor],
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).unwrap();

    // Second claim - should fail
    let tx = Transaction::new_signed_with_payer(
        &[claim_ix],
        Some(&bettor.pubkey()),
        &[&bettor],
        svm.latest_blockhash(),
    );

    let result = svm.send_transaction(tx);
    assert!(result.is_err(), "Double claim should fail");
}
