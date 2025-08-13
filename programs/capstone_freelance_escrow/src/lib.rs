use anchor_lang::prelude::*;

// Module declarations
pub mod instructions;
pub mod state;
pub mod errors;
pub mod events;

// Re-exports
pub use instructions::*;
pub use state::*;
pub use errors::*;
pub use events::*;

declare_id!("3PWoWeSxThLbCFGxMAmSpsCqrFJJhbdrvZg4DVrPdu1y");

#[program]
pub mod capstone_freelance_escrow {
    use super::*;

    pub fn initialize_escrow(
        ctx: Context<InitializeEscrow>,
        amount: u64,
        freelancer: Pubkey,
        dispute_timeout_days: u8,
    ) -> Result<()> {
        instructions::initialize_escrow(ctx, amount, freelancer, dispute_timeout_days)
    }

    pub fn deposit_funds(ctx: Context<DepositFunds>) -> Result<()> {
        instructions::deposit_funds(ctx)
    }

    pub fn submit_work(ctx: Context<SubmitWork>, work_link: String) -> Result<()> {
        instructions::submit_work(ctx, work_link)
    }

    pub fn approve_submission(ctx: Context<ApproveSubmission>) -> Result<()> {
        instructions::approve_submission(ctx)
    }

    pub fn withdraw_payment(ctx: Context<WithdrawPayment>) -> Result<()> {
        instructions::withdraw_payment(ctx)
    }

    pub fn initiate_dispute(ctx: Context<InitiateDispute>) -> Result<()> {
        instructions::initiate_dispute(ctx)
    }

    pub fn refund_client(ctx: Context<RefundClient>) -> Result<()> {
        instructions::refund_client(ctx)
    }
}