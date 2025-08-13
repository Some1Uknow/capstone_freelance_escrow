use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;
use crate::events::*;

pub fn withdraw_payment(ctx: Context<WithdrawPayment>) -> Result<()> {
    let current_status = ctx.accounts.escrow_account.status;
    require!(
        current_status == EscrowStatus::Approved,
        EscrowError::InvalidStatus
    );

    let amount = ctx.accounts.escrow_account.amount;
    let bump = ctx.accounts.escrow_account.bump;
    let client_key = ctx.accounts.escrow_account.client;
    let freelancer_key = ctx.accounts.escrow_account.freelancer;

    require!(
        ctx.accounts.freelancer.key() == freelancer_key,
        EscrowError::Unauthorized
    );

    // Prepare PDA seeds for signing
    let seeds = &[
        b"escrow",
        client_key.as_ref(),
        freelancer_key.as_ref(),
        &[bump],
    ];
    let signer_seeds = &[&seeds[..]];

    // CPI: transfer from escrow PDA → freelancer
    let from = ctx.accounts.escrow_account.to_account_info();
    let to = ctx.accounts.freelancer.to_account_info();
    let system_program_ai = ctx.accounts.system_program.to_account_info();

    let transfer_instruction = anchor_lang::system_program::Transfer { from, to };
    let cpi_ctx =
        CpiContext::new_with_signer(system_program_ai, transfer_instruction, signer_seeds);

    anchor_lang::system_program::transfer(cpi_ctx, amount)?;

    let escrow = &mut ctx.accounts.escrow_account;
    escrow.status = EscrowStatus::Complete;
    escrow.completed_at = Clock::get()?.unix_timestamp;

    emit!(PaymentWithdrawn {
        escrow_key: ctx.accounts.escrow_account.key(),
        freelancer: ctx.accounts.freelancer.key(),
        amount,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct WithdrawPayment<'info> {
    #[account(mut)]
    pub freelancer: Signer<'info>,

    #[account(
        mut,
        has_one = freelancer @ EscrowError::Unauthorized,
        seeds = [b"escrow", escrow_account.client.as_ref(), escrow_account.freelancer.as_ref()],
        bump = escrow_account.bump
    )]
    pub escrow_account: Account<'info, EscrowAccount>,

    pub system_program: Program<'info, System>,
}
