use anchor_lang::prelude::*;

declare_id!("CLA1m111111111111111111111111111111111111111");

#[program]
pub mod claim {
    use super::*;

    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        // Example: get PDA bump correctly on Anchor 0.29
        let escrow_auth_bump = *ctx
            .bumps
            .get("escrow_auth")
            .expect("escrow_auth bump not found");

        // If you need signer seeds:
        let _signer_seeds: &[&[u8]] = &[b"escrow_auth", &[escrow_auth_bump]];
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut)]
    pub claimer: Signer<'info>,

    /// CHECK: PDA authority (seeds + bump enforce correctness)
    #[account(
        seeds = [b"escrow_auth"],
        bump
    )]
    pub escrow_auth: AccountInfo<'info>,

    // Example on-chain state (delete if not needed)
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
