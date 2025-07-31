use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkgMQhg7Wyb5J"); //  replace  later

#[program]
pub mod capstone_freelance_escrow {
    use super::*;

    pub fn initialize_escrow(
        ctx: Context<InitializeEscrow>,
        amount: u64,
        freelancer: Pubkey,
    ) -> Result<()> {
        Ok(())
    }

    pub fn deposit_funds(ctx: Context<DepositFunds>) -> Result<()> {
        Ok(())
    }

    pub fn submit_work(ctx: Context<SubmitWork>, work_link: String) -> Result<()> {
        Ok(())
    }

    pub fn approve_submission(ctx: Context<ApproveSubmission>) -> Result<()> {
        Ok(())
    }

    pub fn withdraw_payment(ctx: Context<WithdrawPayment>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeEscrow<'info> {
    // We'll fill this later
}

#[derive(Accounts)]
pub struct DepositFunds<'info> {}

#[derive(Accounts)]
pub struct SubmitWork<'info> {}

#[derive(Accounts)]
pub struct ApproveSubmission<'info> {}

#[derive(Accounts)]
pub struct WithdrawPayment<'info> {}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum EscrowStatus {
    Pending,
    Funded,
    Submitted,
    Approved,
    Complete,
}

#[account]
pub struct EscrowAccount {
    pub client: Pubkey,
    pub freelancer: Pubkey,
    pub amount: u64,
    pub status: EscrowStatus,
    pub work_link: String,
    pub bump: u8,
}
