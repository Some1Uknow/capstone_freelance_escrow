use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;
use crate::events::*;

pub fn initiate_dispute(ctx: Context<InitiateDispute>) -> Result<()> {
    let current_status = ctx.accounts.escrow_account.status;
    require!(
        current_status == EscrowStatus::Funded || current_status == EscrowStatus::Submitted,
        EscrowError::InvalidStatus
    );

    let escrow = &mut ctx.accounts.escrow_account;
    escrow.status = EscrowStatus::Disputed;
    escrow.disputed_at = Clock::get()?.unix_timestamp;

    emit!(DisputeInitiated {
        escrow_key: ctx.accounts.escrow_account.key(),
        initiator: ctx.accounts.client.key(),
    });

    Ok(())
}

#[derive(Accounts)]
pub struct InitiateDispute<'info> {
    #[account(mut)]
    pub client: Signer<'info>,

    #[account(
        mut,
        has_one = client @ EscrowError::Unauthorized
    )]
    pub escrow_account: Account<'info, EscrowAccount>,
}
