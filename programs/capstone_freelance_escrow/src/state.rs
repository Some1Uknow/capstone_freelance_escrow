use anchor_lang::prelude::*;

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
