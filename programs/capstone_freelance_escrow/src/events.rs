use anchor_lang::prelude::*;

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
