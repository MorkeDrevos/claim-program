use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, Transfer, InitializeAccount, AssociatedToken, associated_token::AssociatedToken};

declare_id!("CLA1m111111111111111111111111111111111111111");

#[program]
pub mod claim {
    use super::*;

    pub fn init_round(
        ctx: Context<InitRound>,
        id: u64,
        opens_at: i64,
        closes_at: i64
    ) -> Result<()> {
        let r = &mut ctx.accounts.round;
        r.id = id;
        r.opens_at = opens_at;
        r.closes_at = closes_at;
        r.pool_mint = ctx.accounts.pool_mint.key();
        r.pool_amount = 0;
        r.total_claimers = 0;
        r.per_share = 0;
        r.status = 0; // draft
        Ok(())
    }

    // Anyone can fund by transferring SPL to the escrow ATA (transparency!)
    pub fn record_funding(_ctx: Context<RecordFunding>) -> Result<()> {
        // No-op — pool_amount read from escrow at finalize for truth
        Ok(())
    }

    pub fn open_round(ctx: Context<ToggleRound>) -> Result<()> {
        require!(ctx.accounts.round.status == 0, ClaimError::BadStatus);
        ctx.accounts.round.status = 1; // open
        Ok(())
    }

    pub fn close_round(ctx: Context<ToggleRound>) -> Result<()> {
        require!(ctx.accounts.round.status == 1, ClaimError::BadStatus);
        ctx.accounts.round.status = 2; // closed
        Ok(())
    }

    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let r = &mut ctx.accounts.round;
        require!(r.status == 1, ClaimError::NotOpen);
        require!(now >= r.opens_at && now <= r.closes_at, ClaimError::Window);
        require!(!ctx.accounts.ticket.claimed, ClaimError::AlreadyClaimed);

        ctx.accounts.ticket.round = r.key();
        ctx.accounts.ticket.claimer = ctx.accounts.user.key();
        ctx.accounts.ticket.claimed = true;
        ctx.accounts.ticket.withdrawn = false;

        r.total_claimers = r.total_claimers.checked_add(1).ok_or(ClaimError::Math)?;
        Ok(())
    }

    pub fn finalize_round(ctx: Context<Finalize>) -> Result<()> {
        let r = &mut ctx.accounts.round;
        require!(r.status == 2, ClaimError::BadStatus);
        require!(r.total_claimers > 0, ClaimError::NoClaimers);

        // Read real balance (source of truth)
        r.pool_amount = ctx.accounts.escrow.amount;
        r.per_share = r.pool_amount / r.total_claimers;
        r.status = 3; // finalized
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        let r = &ctx.accounts.round;
        require!(r.status == 3 && r.per_share > 0, ClaimError::NotFinalized);
        require!(ctx.accounts.ticket.claimed, ClaimError::NotClaimed);
        require!(!ctx.accounts.ticket.withdrawn, ClaimError::AlreadyWithdrawn);

        // PDA signer for escrow authority
        let seeds: &[&[u8]] = &[
            b"escrow-auth",
            r.key().as_ref(),
            &[ctx.accounts.escrow_auth.bump],
        ];
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.escrow.to_account_info(),
                    to: ctx.accounts.user_ata.to_account_info(),
                    authority: ctx.accounts.escrow_auth.to_account_info(),
                },
                &[&seeds],
            ),
            r.per_share,
        )?;

        let t = &mut ctx.accounts.ticket;
        t.withdrawn = true;
        Ok(())
    }
}

/* ----------------- Accounts & PDAs ----------------- */

#[derive(Accounts)]
#[instruction(id: u64)]
pub struct InitRound<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    pub pool_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = admin,
        space = 8 + Round::SIZE,
        seeds = [b"round", &id.to_le_bytes()],
        bump
    )]
    pub round: Account<'info, Round>,

    /// Escrow ATA owned by PDA "escrow-auth"
    #[account(
        init_if_needed,
        payer = admin,
        associated_token::mint = pool_mint,
        associated_token::authority = escrow_auth
    )]
    pub escrow: Account<'info, TokenAccount>,

    /// CHECK: PDA authority for escrow
    #[account(
        seeds = [b"escrow-auth", round.key().as_ref()],
        bump
    )]
    pub escrow_auth: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct RecordFunding<'info> {
    pub pool_mint: Account<'info, Mint>,
    #[account(mut, has_one = pool_mint)]
    pub round: Account<'info, Round>,
    #[account(
        mut,
        associated_token::mint = pool_mint,
        associated_token::authority = escrow_auth
    )]
    pub escrow: Account<'info, TokenAccount>,
    /// CHECK:
    #[account(
        seeds = [b"escrow-auth", round.key().as_ref()],
        bump
    )]
    pub escrow_auth: AccountInfo<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct ToggleRound<'info> {
    pub admin: Signer<'info>,
    #[account(mut)]
    pub round: Account<'info, Round>,
}

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub round: Account<'info, Round>,
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + ClaimTicket::SIZE,
        seeds = [b"ticket", round.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub ticket: Account<'info, ClaimTicket>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Finalize<'info> {
    pub admin: Signer<'info>,
    #[account(mut)]
    pub round: Account<'info, Round>,
    #[account(
        mut,
        associated_token::mint = pool_mint,
        associated_token::authority = escrow_auth
    )]
    pub escrow: Account<'info, TokenAccount>,
    pub pool_mint: Account<'info, Mint>,
    /// CHECK:
    #[account(
        seeds = [b"escrow-auth", round.key().as_ref()],
        bump
    )]
    pub escrow_auth: AccountInfo<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    pub user: Signer<'info>,
    #[account(mut)]
    pub round: Account<'info, Round>,
    #[account(mut,
        seeds = [b"ticket", round.key().as_ref(), user.key().as_ref()],
        bump,
        constraint = ticket.claimer == user.key()
    )]
    pub ticket: Account<'info, ClaimTicket>,

    #[account(
        mut,
        associated_token::mint = pool_mint,
        associated_token::authority = escrow_auth
    )]
    pub escrow: Account<'info, TokenAccount>,
    pub pool_mint: Account<'info, Mint>,

    #[account(mut)]
    pub user_ata: Account<'info, TokenAccount>,

    /// CHECK:
    #[account(
        seeds = [b"escrow-auth", round.key().as_ref()],
        bump
    )]
    pub escrow_auth: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[account]
pub struct Round {
    pub id: u64,
    pub opens_at: i64,
    pub closes_at: i64,
    pub pool_mint: Pubkey,
    pub pool_amount: u64,
    pub total_claimers: u64,
    pub per_share: u64,
    pub status: u8,
}
impl Round { pub const SIZE: usize = 8 + 8 + 8 + 32 + 8 + 8 + 8 + 1; }

#[account]
pub struct ClaimTicket {
    pub round: Pubkey,
    pub claimer: Pubkey,
    pub claimed: bool,
    pub withdrawn: bool,
}
impl ClaimTicket { pub const SIZE: usize = 32 + 32 + 1 + 1; }

#[error_code]
pub enum ClaimError {
    #[msg("Round not open")] NotOpen,
    #[msg("Outside window")] Window,
    #[msg("Bad status")] BadStatus,
    #[msg("Already claimed")] AlreadyClaimed,
    #[msg("Not claimed")] NotClaimed,
    #[msg("Already withdrawn")] AlreadyWithdrawn,
    #[msg("No claimers")] NoClaimers,
    #[msg("Not finalized")] NotFinalized,
    #[msg("Math error")] Math,
}
