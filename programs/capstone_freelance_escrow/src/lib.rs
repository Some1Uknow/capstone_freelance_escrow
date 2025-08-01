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
        let escrow = &mut ctx.accounts.escrow_account;
        escrow.client = ctx.accounts.client.key();
        escrow.freelancer = freelancer;
        escrow.amount = amount;
        escrow.status = EscrowStatus::Pending;
        escrow.work_link = "".to_string(); // Empty initially
        escrow.bump = *ctx.bumps.get("escrow_account").unwrap();
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
#[instruction(amount: u64, freelancer: Pubkey)]
pub struct InitializeEscrow<'info> {
    #[account(mut)]
    pub client: Signer<'info>,

    #[account(
        init,
        payer = client,
        space = 8 + 32 + 32 + 8 + 1 + 4 + 200 + 1,
        seeds = [b"escrow", client.key().as_ref(), freelancer.key().as_ref()],
        bump
    )]
    pub escrow_account: Account<'info, EscrowAccount>,

    pub system_program: Program<'info, System>,
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
