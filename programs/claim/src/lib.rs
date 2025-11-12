use anchor_lang::prelude::*;
// If later you read sysvars like Clock, uncomment the next line:
// use anchor_lang::solana_program::sysvar::Sysvar;

declare_id!("CLA1m111111111111111111111111111111111111111");

#[program]
pub mod claim {
    use super::*;

    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        // Anchor 0.29: bumps are exposed on Context via a map.
        let escrow_auth_bump = *ctx
            .bumps
            .get("escrow_auth")
            .expect("escrow_auth bump not found");

        // If you need signer seeds for CPI/signature:
        let _signer_seeds: &[&[u8]] = &[b"escrow_auth", &[escrow_auth_bump]];

        // trivial example: bump a counter on Pool
        let pool = &mut ctx.accounts.pool;
        pool.total_claims = pool
            .total_claims
            .checked_add(1)
            .ok_or(ErrorCode::MathOverflow)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Claim<'info> {
    /// The user calling the claim instruction
    #[account(mut)]
    pub claimer: Signer<'info>,

    /// CHECK: PDA authority (seed + bump constraint makes this safe)
    #[account(
        seeds = [b"escrow_auth"],
        bump
    )]
    pub escrow_auth: AccountInfo<'info>,

    /// A simple on-chain state account tied to the escrow_auth PDA
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
