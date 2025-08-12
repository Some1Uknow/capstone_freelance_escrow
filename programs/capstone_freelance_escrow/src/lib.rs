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
        escrow.bump = ctx.bumps.escrow_account;
        Ok(())
    }

    pub fn deposit_funds(ctx: Context<DepositFunds>) -> Result<()> {
        // read values first (no mutable borrow yet)
        let escrow_amount = ctx.accounts.escrow_account.amount;
        let escrow_status = ctx.accounts.escrow_account.status.clone(); // small clone of enum
        let client_lamports = ctx.accounts.client.lamports();

        // checks (use previously-read values)
        require!(
            escrow_status == EscrowStatus::Pending,
            EscrowError::InvalidStatus
        );
        require!(
            client_lamports >= escrow_amount,
            EscrowError::InsufficientFunds
        );

        // collect AccountInfos for CPI BEFORE taking a mutable borrow
        let from = ctx.accounts.client.to_account_info();
        let to = ctx.accounts.escrow_account.to_account_info();
        let system_program_ai = ctx.accounts.system_program.to_account_info();

        let transfer_instruction = anchor_lang::system_program::Transfer { from, to };
        let cpi_ctx = CpiContext::new(system_program_ai, transfer_instruction);
        anchor_lang::system_program::transfer(cpi_ctx, escrow_amount)?;

        // now mutate the escrow account safely
        let escrow = &mut ctx.accounts.escrow_account;
        escrow.status = EscrowStatus::Funded;

        Ok(())
    }

    pub fn submit_work(ctx: Context<SubmitWork>, work_link: String) -> Result<()> {
        // 1. Read status before mut borrow
        let current_status = ctx.accounts.escrow_account.status.clone();
        require!(
            current_status == EscrowStatus::Funded,
            EscrowError::InvalidStatus
        );

        // 2. Mutably borrow after checks
        let escrow = &mut ctx.accounts.escrow_account;

        // 3. Store work link
        escrow.work_link = work_link;

        // 4. Update status
        escrow.status = EscrowStatus::Submitted;

        Ok(())
    }

    pub fn approve_submission(ctx: Context<ApproveSubmission>) -> Result<()> {
        // 1. Read status before mut borrow
        let current_status = ctx.accounts.escrow_account.status.clone();
        require!(
            current_status == EscrowStatus::Submitted,
            EscrowError::InvalidStatus
        );

        // 2. Mutably borrow after checks
        let escrow = &mut ctx.accounts.escrow_account;

        // 3. Update status
        escrow.status = EscrowStatus::Approved;

        Ok(())
    }

    pub fn withdraw_payment(ctx: Context<WithdrawPayment>) -> Result<()> {
        // ✅ 1. Read status before mut borrow
        let current_status = ctx.accounts.escrow_account.status.clone();
        require!(
            current_status == EscrowStatus::Approved,
            EscrowError::InvalidStatus
        );

        let amount = ctx.accounts.escrow_account.amount;
        let bump = ctx.accounts.escrow_account.bump;
        let client_key = ctx.accounts.escrow_account.client;
        let freelancer_key = ctx.accounts.escrow_account.freelancer;

        // ✅ 2. Ensure correct freelancer is withdrawing
        require!(
            ctx.accounts.freelancer.key() == freelancer_key,
            EscrowError::Unauthorized
        );

        // ✅ 3. Prepare PDA seeds for signing
        let seeds = &[
            b"escrow",
            client_key.as_ref(),
            freelancer_key.as_ref(),
            &[bump],
        ];
        let signer_seeds = &[&seeds[..]];

        // ✅ 4. CPI: transfer from escrow PDA → freelancer
        let from = ctx.accounts.escrow_account.to_account_info();
        let to = ctx.accounts.freelancer.to_account_info();
        let system_program_ai = ctx.accounts.system_program.to_account_info();

        let transfer_instruction = anchor_lang::system_program::Transfer { from, to };
        let cpi_ctx =
            CpiContext::new_with_signer(system_program_ai, transfer_instruction, signer_seeds);

        anchor_lang::system_program::transfer(cpi_ctx, amount)?;

        // ✅ 5. Update status to Complete
        let escrow = &mut ctx.accounts.escrow_account;
        escrow.status = EscrowStatus::Complete;

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

#[error_code]
pub enum EscrowError {
    #[msg("Invalid status for this action")]
    InvalidStatus,
    #[msg("You are not authorized to perform this action")]
    Unauthorized,
    #[msg("Insufficient funds to deposit")]
    InsufficientFunds,
}