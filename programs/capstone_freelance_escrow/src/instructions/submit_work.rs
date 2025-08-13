use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;
use crate::events::*;

pub fn submit_work(ctx: Context<SubmitWork>, work_link: String) -> Result<()> {
    // Validate and sanitize work link
    let work_link = work_link.trim().to_string();
    require!(!work_link.is_empty(), EscrowError::InvalidWorkLink);
    
    // Check UTF-8 character count (not byte count)
    let char_count = work_link.chars().count();
    require!(char_count <= 200, EscrowError::WorkLinkTooLong);
    
    // Check byte size for serialization
    require!(work_link.len() <= 600, EscrowError::WorkLinkTooLong); // UTF-8 can be up to 3 bytes per char

    let current_status = ctx.accounts.escrow_account.status;
    require!(
        current_status == EscrowStatus::Funded,
        EscrowError::InvalidStatus
    );
    require!(
        current_status != EscrowStatus::Complete,
        EscrowError::EscrowAlreadyComplete
    );

    let escrow = &mut ctx.accounts.escrow_account;
    escrow.work_link = work_link.clone();
    escrow.status = EscrowStatus::Submitted;
    escrow.submitted_at = Clock::get()?.unix_timestamp;

    emit!(WorkSubmitted {
        escrow_key: ctx.accounts.escrow_account.key(),
        freelancer: ctx.accounts.freelancer.key(),
        work_link,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct SubmitWork<'info> {
    #[account(mut)]
    pub freelancer: Signer<'info>,

    #[account(
        mut,
        has_one = freelancer @ EscrowError::Unauthorized
    )]
    pub escrow_account: Account<'info, EscrowAccount>,
}
