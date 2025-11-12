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
        // Anchor 0.29: access PDA bump via ctx.bumps
        let escrow_auth_bump = *ctx
            .bumps
            .get("escrow_auth")
            .ok_or(ErrorCode::MissingBump)?;
        let _signer_seeds: &[&[u8]] = &[b"escrow_auth", &[escrow_auth_bump]];

        let pool = &mut ctx.accounts.pool;
        pool.total_claims = pool
            .total_claims
            .checked_add(1)
            .ok_or(ErrorCode::MathOverflow)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitPool<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: seeds + bump enforce the PDA
    #[account(seeds = [b"escrow_auth"], bump)]
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
pub struct Claim<'info> {
    #[account(mut)]
    pub claimer: Signer<'info>,

    /// CHECK: seeds + bump enforce the PDA
    #[account(seeds = [b"escrow_auth"], bump)]
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
impl Pool {
    pub const SIZE: usize = 8;
}

#[error_code]
pub enum ErrorCode {
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Missing bump in context")]
    MissingBump,
}
