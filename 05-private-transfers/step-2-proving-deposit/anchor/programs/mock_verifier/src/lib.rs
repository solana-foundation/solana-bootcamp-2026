use anchor_lang::prelude::*;

declare_id!("4T8nxeiE9c4x55cPhXbqr1rzwXvggLTZkUfDGoSb9yrJ");

/// Mock verifier program for local testing
/// In production, replace with actual Sunspot-generated verifier
///
/// NOTE: This Anchor-based mock cannot handle raw CPI calls with arbitrary data.
/// For withdrawal testing, you need the real Sunspot verifier.
/// The deposit flow and account management can be tested without this.
#[program]
pub mod mock_verifier {
    use super::*;

    /// Verify a proof (mock - always succeeds)
    ///
    /// In the real Sunspot verifier:
    /// - Instruction data format: proof_bytes || public_inputs_bytes
    /// - Proof is 256 bytes (Groth16: A=64, B=128, C=64)
    /// - Public inputs are 32 bytes each (field elements)
    pub fn verify(_ctx: Context<Verify>) -> Result<()> {
        msg!("Mock verifier: Proof accepted (TESTING ONLY)");
        msg!("WARNING: This is a mock verifier. Do not use in production!");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Verify {}
