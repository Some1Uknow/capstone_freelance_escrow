use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;
use crate::events::*;

pub fn initialize_escrow(
    ctx: Context<InitializeEscrow>,
    amount: u64,
    freelancer: Pubkey,
    dispute_timeout_days: u8,
) -> Result<()> {
    require!(amount > 0, EscrowError::InvalidAmount);
    require!(dispute_timeout_days >= 1 && dispute_timeout_days <= 90, EscrowError::InvalidTimeout);

    let escrow = &mut ctx.accounts.escrow_account;
    escrow.client = ctx.accounts.client.key();
    escrow.freelancer = freelancer;
    escrow.amount = amount;
    escrow.status = EscrowStatus::Pending;
    escrow.work_link = "".to_string();
    escrow.bump = ctx.bumps.escrow_account;
    escrow.created_at = Clock::get()?.unix_timestamp;
    escrow.dispute_timeout_days = dispute_timeout_days;
    
    emit!(EscrowInitialized {
        escrow_key: ctx.accounts.escrow_account.key(),
        client: ctx.accounts.client.key(),
        freelancer,
        amount,
    });
    
    Ok(())
}

#[derive(Accounts)]
#[instruction(amount: u64, freelancer: Pubkey, dispute_timeout_days: u8)]
pub struct InitializeEscrow<'info> {
    #[account(mut)]
    pub client: Signer<'info>,

    #[account(
        init,
        payer = client,
        space = 8 + 32 + 32 + 8 + 1 + 4 + 600 + 1 + 8 + 8 + 8 + 8 + 8 + 8 + 1, // Fixed space calculation
        seeds = [b"escrow", client.key().as_ref(), freelancer.key().as_ref()],
        bump
    )]
    pub escrow_account: Account<'info, EscrowAccount>,

    pub system_program: Program<'info, System>,
}
