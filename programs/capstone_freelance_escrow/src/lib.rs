use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkgMQhg7Wyb5J");

#[program]
pub mod capstone_freelance_escrow {
    use super::*;

    pub fn initialize_escrow(
        ctx: Context<InitializeEscrow>,
        amount: u64,
        freelancer: Pubkey,
        dispute_timeout_days: u8,
    ) -> Result<()> {
        require!(amount > 0, EscrowError::InvalidAmount);
        require!(dispute_timeout_days >= 1 && dispute_timeout_days <= 90, EscrowError::InvalidTimeout);

        let escrow = &mut ctx.accounts.escrow_account;
        escrow.client = ctx.accounts.client.key();
        escrow.freelancer = freelancer;
        escrow.amount = amount;
        escrow.status = EscrowStatus::Pending;
        escrow.work_link = "".to_string();
        escrow.bump = ctx.bumps.escrow_account;
        escrow.created_at = Clock::get()?.unix_timestamp;
        escrow.dispute_timeout_days = dispute_timeout_days;
        
        emit!(EscrowInitialized {
            escrow_key: ctx.accounts.escrow_account.key(),
            client: ctx.accounts.client.key(),
            freelancer,
            amount,
        });
        
        Ok(())
    }

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
}

#[derive(Accounts)]
#[instruction(amount: u64, freelancer: Pubkey, dispute_timeout_days: u8)]
pub struct InitializeEscrow<'info> {
    #[account(mut)]
    pub client: Signer<'info>,

    #[account(
        init,
        payer = client,
        space = 8 + 32 + 32 + 8 + 1 + 4 + 600 + 1 + 8 + 8 + 8 + 8 + 8 + 8 + 1, // Fixed space calculation
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

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Copy)]
pub enum EscrowStatus {
    Pending,
    Funded,
    Submitted,
    Approved,
    Complete,
    Disputed,
    Refunded,
}

#[account]
pub struct EscrowAccount {
    pub client: Pubkey,          // 32 bytes
    pub freelancer: Pubkey,      // 32 bytes
    pub amount: u64,             // 8 bytes
    pub status: EscrowStatus,    // 1 byte
    pub work_link: String,       // 4 + up to 600 bytes
    pub bump: u8,                // 1 byte
    pub created_at: i64,         // 8 bytes
    pub funded_at: i64,          // 8 bytes
    pub submitted_at: i64,       // 8 bytes
    pub approved_at: i64,        // 8 bytes
    pub completed_at: i64,       // 8 bytes
    pub disputed_at: i64,        // 8 bytes
    pub refunded_at: i64,        // 8 bytes
    pub dispute_timeout_days: u8, // 1 byte
}

#[error_code]
pub enum EscrowError {
    #[msg("Invalid status for this action")]
    InvalidStatus,
    #[msg("You are not authorized to perform this action")]
    Unauthorized,
    #[msg("Insufficient funds to deposit")]
    InsufficientFunds,
    #[msg("Invalid amount specified")]
    InvalidAmount,
    #[msg("Work link cannot be empty")]
    InvalidWorkLink,
    #[msg("Work link is too long")]
    WorkLinkTooLong,
    #[msg("Escrow is already complete")]
    EscrowAlreadyComplete,
    #[msg("Invalid timeout period (must be 1-90 days)")]
    InvalidTimeout,
}

// Events for logging state changes
#[event]
pub struct EscrowInitialized {
    pub escrow_key: Pubkey,
    pub client: Pubkey,
    pub freelancer: Pubkey,
    pub amount: u64,
}

#[event]
pub struct FundsDeposited {
    pub escrow_key: Pubkey,
    pub amount: u64,
}

#[event]
pub struct WorkSubmitted {
    pub escrow_key: Pubkey,
    pub freelancer: Pubkey,
    pub work_link: String,
}

#[event]
pub struct SubmissionApproved {
    pub escrow_key: Pubkey,
    pub client: Pubkey,
}

#[event]
pub struct PaymentWithdrawn {
    pub escrow_key: Pubkey,
    pub freelancer: Pubkey,
    pub amount: u64,
}

#[event]
pub struct DisputeInitiated {
    pub escrow_key: Pubkey,
    pub initiator: Pubkey,
}

#[event]
pub struct ClientRefunded {
    pub escrow_key: Pubkey,
    pub client: Pubkey,
    pub amount: u64,
}