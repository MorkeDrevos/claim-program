use anchor_lang::prelude::*;

declare_id!("CLA1m111111111111111111111111111111111111111");

#[program]
pub mod claim {
    use super::*;

    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        // correct way to access a PDA bump in Anchor 0.29:
        let escrow_auth_bump = *ctx
            .bumps
            .get("escrow_auth")
            .expect("escrow_auth bump not found");

        // If you need signer seeds for CPIs:
        let signer_seeds: &[&[u8]] = &[b"escrow_auth", &[escrow_auth_bump]];

        // TODO: your logic here (transfer, record, etc.)
        // Example no-op to prove compile:
        let _ = signer_seeds;

        Ok(())
    }
}

/// Accounts for the `claim` instruction.
///
/// NOTE:
/// - `escrow_auth` is a PDA defined by the seeds below.
/// - DO NOT try to read `.bump` field from AccountInfo. Use `ctx.bumps`.
#[derive(Accounts)]
pub struct Claim<'info> {
    /// Payer/claimer wallet
    #[account(mut)]
    pub claimer: Signer<'info>,

    /// CHECK: PDA authority; safe because we constrain seeds and bump.
    #[account(
        seeds = [b"escrow_auth"],
        bump
    )]
    pub escrow_auth: AccountInfo<'info>,

    /// Optional: a pool account you might be mutating (example data type below).
    /// Remove this block if you don't need a stored account yet.
    #[account(
        mut,
        seeds = [b"pool", escrow_auth.key().as_ref()],
        bump
    )]
    pub pool: Account<'info, Pool>,

    pub system_program: Program<'info, System>,
}

/// Example state (keep or delete depending on your design)
#[account]
pub struct Pool {
    pub total_claims: u64,
}
