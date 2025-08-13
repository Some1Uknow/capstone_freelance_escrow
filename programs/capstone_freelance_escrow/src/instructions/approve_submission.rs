use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;
use crate::events::*;

pub fn approve_submission(ctx: Context<ApproveSubmission>) -> Result<()> {
    let current_status = ctx.accounts.escrow_account.status;
    require!(
        current_status == EscrowStatus::Submitted,
        EscrowError::InvalidStatus
    );
    require!(
        current_status != EscrowStatus::Complete,
        EscrowError::EscrowAlreadyComplete
    );

    let escrow = &mut ctx.accounts.escrow_account;
    escrow.status = EscrowStatus::Approved;
    escrow.approved_at = Clock::get()?.unix_timestamp;

    emit!(SubmissionApproved {
        escrow_key: ctx.accounts.escrow_account.key(),
        client: ctx.accounts.client.key(),
    });

    Ok(())
}

#[derive(Accounts)]
pub struct ApproveSubmission<'info> {
    #[account(mut)]
    pub client: Signer<'info>,

    #[account(
        mut,
        has_one = client @ EscrowError::Unauthorized
    )]
    pub escrow_account: Account<'info, EscrowAccount>,
}
