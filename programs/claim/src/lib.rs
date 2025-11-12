use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::{self, Sysvar}; // 👈 fixes the get() error

declare_id!("CLA1m111111111111111111111111111111111111111");

#[program]
pub mod claim {
    use super::*;

    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        // Get bump safely from Anchor 0.29 Context
        let escrow_auth_bump = *ctx
            .bumps
            .get("escrow_auth")
            .expect("escrow_auth bump not found");

        // Optional: get the current network clock
        let clock = Clock::get()?; // 👈 now compiles correctly
        msg!("Timestamp: {}", clock.unix_timestamp);

        // Example logic: increment a counter
        let pool = &mut ctx.accounts.pool;
        pool.total_claims = pool
            .total_claims
            .checked_add(1)
            .ok_or(ErrorCode::MathOverflow)?;

        msg!("Claim processed. Total claims: {}", pool.total_claims);

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut)]
    pub claimer: Signer<'info>,

    /// CHECK: PDA authority (bump ensures safety)
    #[account(
        seeds = [b"escrow_auth"],
        bump
    )]
    pub escrow_auth: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"pool", escrow_auth.key().as_ref()],
        bump
    )]
    pub pool: Account<'info, Pool>,

    pub system_program: Program<'info, System>,
}

#[account]
pub struct Pool {
    pub total_claims: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Math overflow")]
    MathOverflow,
}
