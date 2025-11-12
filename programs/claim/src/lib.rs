use anchor_lang::prelude::*;

declare_id!("CLA1m111111111111111111111111111111111111111");

#[program]
pub mod claim {
    use super::*;

    pub fn init_pool(ctx: Context<InitPool>) -> Result<()> {
        ctx.accounts.pool.total_claims = 0;
        Ok(())
    }

    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        // Get the bump Anchor 0.29 style (from Context::bumps)
        let escrow_auth_bump = *ctx
            .bumps
            .get("escrow_auth")
            .ok_or(ErrorCode::MissingBump)?;

        // (kept as example for signer seeds, not used yet)
        let _signer_seeds: &[&[u8]] = &[b"escrow_auth", &[escrow_auth_bump]];

        // Update some deterministic state so we prove writes work
        let pool = &mut ctx.accounts.pool;
        pool.total_claims = pool
            .total_claims
            .checked_add(1)
            .ok_or(ErrorCode::MathOverflow)?;

        Ok(())
    }
}

/* ---------------- Accounts ---------------- */

#[derive(Accounts)]
pub struct InitPool<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: PDA authority (seed+bump checked by runtime)
    #[account(
        seeds = [b"escrow_auth"],
        bump
    )]
    pub escrow_auth: AccountInfo<'info>,

    #[account(
        init,
        payer = payer,
        seeds = [b"pool", escrow_auth.key().as_ref()],
        bump,
        space = 8 + Pool::SIZE
    )]
    pub pool: Account<'info, Pool>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Claim <'info> {
    #[account(mut)]
    pub claimer: Signer<'info>,

    /// CHECK: PDA authority (seed+bump checked by runtime)
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

/* ---------------- State ---------------- */

#[account]
pub struct Pool {
    pub total_claims: u64,
}
impl Pool {
    pub const SIZE: usize = 8; // total_claims
}

/* ---------------- Errors ---------------- */

#[error_code]
pub enum ErrorCode {
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Missing bump in context")]
    MissingBump,
}
