use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;
use crate::events::*;

pub fn deposit_funds(ctx: Context<DepositFunds>) -> Result<()> {
    let escrow_amount = ctx.accounts.escrow_account.amount;
    let escrow_status = ctx.accounts.escrow_account.status;

    require!(
        escrow_status == EscrowStatus::Pending,
        EscrowError::InvalidStatus
    );

    require!(
        ctx.accounts.client.lamports() >= escrow_amount,
        EscrowError::InsufficientFunds
    );

    // Collect AccountInfos for CPI BEFORE taking a mutable borrow
    let from = ctx.accounts.client.to_account_info();
    let to = ctx.accounts.escrow_account.to_account_info();
    let system_program_ai = ctx.accounts.system_program.to_account_info();

    let transfer_instruction = anchor_lang::system_program::Transfer { from, to };
    let cpi_ctx = CpiContext::new(system_program_ai, transfer_instruction);
    anchor_lang::system_program::transfer(cpi_ctx, escrow_amount)?;

    let escrow = &mut ctx.accounts.escrow_account;
    escrow.status = EscrowStatus::Funded;
    escrow.funded_at = Clock::get()?.unix_timestamp;

    emit!(FundsDeposited {
        escrow_key: ctx.accounts.escrow_account.key(),
        amount: escrow_amount,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct DepositFunds<'info> {
    #[account(mut)]
    pub client: Signer<'info>,

    #[account(
        mut,
        has_one = client @ EscrowError::Unauthorized,
    )]
    pub escrow_account: Account<'info, EscrowAccount>,

    pub system_program: Program<'info, System>,
}
