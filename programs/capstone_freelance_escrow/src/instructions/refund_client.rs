use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;
use crate::events::*;

pub fn refund_client(ctx: Context<RefundClient>) -> Result<()> {
    let current_status = ctx.accounts.escrow_account.status;
    let current_time = Clock::get()?.unix_timestamp;
    let escrow = &ctx.accounts.escrow_account;

    // Allow refund if disputed OR if timeout period has passed after funding
    let timeout_seconds = (escrow.dispute_timeout_days as i64) * 24 * 60 * 60;
    let funding_timeout_passed = escrow.funded_at > 0 && 
        current_time >= escrow.funded_at.checked_add(timeout_seconds).ok_or(EscrowError::InvalidAmount)?;

    require!(
        current_status == EscrowStatus::Disputed || 
        (current_status == EscrowStatus::Funded && funding_timeout_passed),
        EscrowError::InvalidStatus
    );

    let amount = escrow.amount;
    let bump = escrow.bump;
    let client_key = escrow.client;
    let freelancer_key = escrow.freelancer;

    // Prepare PDA seeds for signing
    let seeds = &[
        b"escrow",
        client_key.as_ref(),
        freelancer_key.as_ref(),
        &[bump],
    ];
    let signer_seeds = &[&seeds[..]];

    // CPI: transfer from escrow PDA → client
    let from = ctx.accounts.escrow_account.to_account_info();
    let to = ctx.accounts.client.to_account_info();
    let system_program_ai = ctx.accounts.system_program.to_account_info();

    let transfer_instruction = anchor_lang::system_program::Transfer { from, to };
    let cpi_ctx =
        CpiContext::new_with_signer(system_program_ai, transfer_instruction, signer_seeds);

    anchor_lang::system_program::transfer(cpi_ctx, amount)?;

    let escrow = &mut ctx.accounts.escrow_account;
    escrow.status = EscrowStatus::Refunded;
    escrow.refunded_at = current_time;

    emit!(ClientRefunded {
        escrow_key: ctx.accounts.escrow_account.key(),
        client: ctx.accounts.client.key(),
        amount,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct RefundClient<'info> {
    #[account(mut)]
    pub client: Signer<'info>,

    #[account(
        mut,
        has_one = client @ EscrowError::Unauthorized,
        seeds = [b"escrow", escrow_account.client.as_ref(), escrow_account.freelancer.as_ref()],
        bump = escrow_account.bump
    )]
    pub escrow_account: Account<'info, EscrowAccount>,

    pub system_program: Program<'info, System>,
}
